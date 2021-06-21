#![allow(non_snake_case)]
use std::collections::{HashMap, HashSet, VecDeque};
use std::borrow::Cow;
use std::thread;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, Weak};
use std::sync::mpsc::{self, Receiver, SyncSender, Sender, TryRecvError};
use std::cmp;
use std::time::Duration;
use std::fmt::Debug;
use std::num::NonZeroUsize;

use serde::de::DeserializeOwned;
use serde::Serialize;
use ring_channel::*;

use commons::types::{Int, NodeId};
use commons::constants::WINDOW_SIZE;

use super::types::{Tag, BufferRecord};
use super::messages::*;
use super::hbfd::*;
use super::thetafd::*;
use crate::communicator::CommunicatorDelegate;
use crate::urb::NodeDelegate;
use crate::terminal_output::printlnu;

#[derive(Hash, Eq, PartialEq)]
enum MessageType {
    MSG,
    MSGAck,
    GOSSIP,
}

pub struct UrbNode<D,V> {
    delegate: Weak<D>,
    thetafd: Arc<Thetafd<D>>,
    hbfd: Arc<Hbfd<D>>,

    seq: Mutex<Int>,
    buffer: Mutex<HashMap<Tag, BufferRecord<V>>>,
    rxObsS: Mutex<Vec<Int>>,
    txObsS: Mutex<Vec<Int>>,
    txObsS_changed: Condvar,

    msgs_buffer_txs: Mutex<Option<HashMap<MessageType, HashMap<NodeId, RingSender<String>>>>>, 

    next_to_deliver: Mutex<Vec<Int>>,
    buffer_updated: Condvar,
    dummy_mutex: Mutex<bool>,


    /* Global reset */
    echo: Mutex<VecDeque<String>>,
    /* Global reset */
    /* Omega failure detector */
    alive: Mutex<VecDeque<String>>,
    response: Mutex<VecDeque<String>>,
    /* Omega failure detector */
    /* Binary Consensus */
    phase: Mutex<VecDeque<String>>,
    decide: Mutex<VecDeque<String>>,
    /* Binary Consensus */
    /* Multivalued Consensus */
    proposal: Mutex<VecDeque<String>>,
    /* Multivalued Consensus */
    /* TO-URB */
    to_urb: Mutex<VecDeque<String>>,
    sync: Mutex<VecDeque<String>>,
    syncack: Mutex<VecDeque<String>>,
    /* TO-URB */
}

impl<D,V> UrbNode<D,V>
    where D: NodeDelegate + CommunicatorDelegate + Send + Sync + 'static,
          V: Default + Serialize + DeserializeOwned + Debug + Clone + Send + Sync + 'static,
{
    //
    // Initilization
    //

    pub fn new(delegate: Weak<D>) -> Arc<UrbNode<D,V>> {
        let delegate_for_thetafd = delegate.clone();
        let thetafd = Thetafd::new(delegate_for_thetafd);
        let delegate_for_hbfd = delegate.clone();
        let hbfd = Hbfd::new(delegate_for_hbfd);

        let number_of_nodes = delegate.upgrade().unwrap().node_ids().len();

        let node = UrbNode {
            delegate: delegate,
            thetafd: thetafd,
            hbfd: hbfd,

            seq: Mutex::new(0),
            buffer: Mutex::new(HashMap::new()),
            rxObsS: Mutex::new(vec![0; number_of_nodes]),
            txObsS: Mutex::new(vec![0; number_of_nodes]),
            txObsS_changed: Condvar::new(),

            msgs_buffer_txs: Mutex::new(None),

            next_to_deliver: Mutex::new(vec![1; number_of_nodes]),
            buffer_updated: Condvar::new(),
            dummy_mutex: Mutex::new(false),

            echo: Mutex::new(VecDeque::new()),

            alive: Mutex::new(VecDeque::new()),
            response: Mutex::new(VecDeque::new()),

            phase: Mutex::new(VecDeque::new()),
            decide: Mutex::new(VecDeque::new()),

            proposal: Mutex::new(VecDeque::new()),

            to_urb: Mutex::new(VecDeque::new()),
            sync: Mutex::new(VecDeque::new()),
            syncack: Mutex::new(VecDeque::new()),
        };
        Arc::new(node)
    }

    fn delegate(&self) -> Arc<D> {
        self.delegate
            .upgrade()
            .expect("Error upgrading delegate in UrbNode")
    }

    fn id(&self) -> NodeId {
        self.delegate().node_id()
    }

    //
    // Macros
    //
    pub fn has_terminated(&self, tag: &Tag) -> bool {
        let buffer = self.buffer.lock().unwrap();
        if let Some(entry) = buffer.get(tag) {
            if !entry.delivered {
                return false;
            }
        }
        true
    }

    pub fn trusted(&self) -> HashSet<NodeId> {
        self.thetafd.trusted()
    }

    fn obsolete(&self, record: &BufferRecord<V>, rxObsS: &MutexGuard<Vec<Int>>) -> bool {
        let tag = &record.tag;

        let obs = rxObsS[tag.id as usize - 1] + 1 == tag.seq &&
            record.delivered &&
            self.trusted().is_subset(&record.recBy);
        // printlnu(format!("checking obsolete, record: {:?}, result:{:?}, rxObsS: {:?}", &record, obs, *rxObsS));
        obs
    }

    fn maxSeq(&self, node_id: NodeId) -> Int {
        let mut max_seq = 0;
        let buffer = self.buffer.lock().unwrap();
        for (tag, _) in buffer.iter() {
            if tag.id == node_id {
                max_seq = cmp::max(max_seq, tag.seq);
            }
        }
        drop(buffer);
        let next_to_deliver = self.next_to_deliver.lock().unwrap();
        max_seq = cmp::max(max_seq, next_to_deliver[node_id as usize - 1] - 1);
        max_seq
    }

    fn minTxObsS(&self) -> Int {
        let trusted = self.trusted();
        let mut min_s = std::i32::MAX;
        let txObsS = self.txObsS.lock().unwrap();
        for node_id in trusted {
            min_s = cmp::min(min_s, txObsS[node_id as usize - 1]);
        }
        min_s
    }

    //
    // URB APIs 
    //

    pub fn urbBroadcast(&self, msg: V) -> Tag{
        // printlnu(format!("Inside urbBroadcast, 1."));
        self.wait_until_receivers_have_enough_space();
        // printlnu(format!("Inside urbBroadcast, 2."));
        let mut seq = self.seq.lock().unwrap();
        // printlnu(format!("Inside urbBroadcast, 3."));
        *seq += 1;
        let tag = Tag { id: self.id(), seq: *seq };
        let tx_des = tag.clone();
        self.update(Some(msg), tag, self.id());
        // printlnu(format!("Inside urbBroadcast, 4."));
        self.buffer_updated.notify_one();
        tx_des
    } 

    pub fn urbDeliver(&self,tag: &Tag, msg: V) {
        let mut my_string_msg: String = format!("{:?}",msg.clone());
        if my_string_msg.contains("DECIDE"){
            let mut decide = self.decide.lock().unwrap().push_back(my_string_msg);
        }
        else if my_string_msg.contains("PROPOSAL"){
            let mut proposal = self.proposal.lock().unwrap().push_back(my_string_msg);
        }
        else if my_string_msg.contains("TOURB"){
            let mut to_urb = self.to_urb.lock().unwrap().push_back(my_string_msg);
        }
        self.delegate().run_result().delivered_msgs.insert((tag.id, tag.seq));
        //printlnu(format!("New message delivered: {:?}", msg));
    }

    fn wait_until_receivers_have_enough_space(&self) {
        let mut seq = self.seq.lock().unwrap();
        while *seq >= self.minTxObsS() + WINDOW_SIZE {
            // printlnu(format!("Receiver does not have enough space, going to block. minTxObsS = {}, seq = {}", self.minTxObsS(), *seq));
            seq = self.txObsS_changed.wait(seq).unwrap();
        }
    }

    fn update(&self, msg: Option<V>, tag: Tag, forwarder: NodeId) {
        // printlnu(format!("Inside update, msg: {:?}, tag:{:?}, forwarder: {:?}", &msg, &tag, &forwarder));
        // printlnu(format!("Inside update, 1"));
        let rxObsS = self.rxObsS.lock().unwrap();
        // printlnu(format!("Inside update, rxObsS: {:?}", *rxObsS));
        if tag.seq <= rxObsS[tag.id as usize - 1] {
            // printlnu(format!("Record not added to the buffer.{:?}", tag));
            return;
        }
        drop(rxObsS);
        // printlnu(format!("Inside update, 3"));
        let mut buffer = self.buffer.lock().unwrap();
        // printlnu(format!("Inside update, 4"));
        // printlnu(format!("Buffer size: {}", buffer.records.len()));
        if !buffer.contains_key(&tag) && !msg.is_none() {
            let mut recBy = HashSet::new();
            recBy.insert(self.id());
            recBy.insert(tag.id);
            let number_of_nodes = self.delegate().node_ids().len();
            let record = BufferRecord {
                tag: tag.clone(),
                msg: msg,
                delivered: false,
                recBy: recBy,
                prevHB: vec![-1; number_of_nodes],
            };
            buffer.insert(tag.clone(), record);
        } else {
            if let Some(record) = buffer.get_mut(&tag) {
                record.recBy.insert(tag.id);
                record.recBy.insert(forwarder);
            }
        }
    }

    //
    // The do forever loop
    //

    pub fn start_the_do_forever_loop(node: &Arc<Self>) -> Sender<()> {
       let (stop_thread_tx, stop_thread_rx) = mpsc::channel();
        let do_forever_loop_node = Arc::clone(&node);

        let mut msgs_buffer_txs = HashMap::new();
        let mut msgs_buffer_rxs = HashMap::new();
        
        let mut MSG_txs = HashMap::new();
        let mut MSGAck_txs = HashMap::new();
        let mut GOSSIP_txs = HashMap::new();

        let mut MSG_rxs = HashMap::new();
        let mut MSGAck_rxs = HashMap::new();
        let mut GOSSIP_rxs = HashMap::new();

        for &node_id in node.delegate().node_ids() {
            let (mut MSG_tx, mut MSG_rx) = ring_channel(NonZeroUsize::new(2 * WINDOW_SIZE as usize).unwrap());
            MSG_txs.insert(node_id, MSG_tx);
            MSG_rxs.insert(node_id, MSG_rx);
            let (mut MSGAck_tx, mut MSGAck_rx) = ring_channel(NonZeroUsize::new(2 * WINDOW_SIZE as usize).unwrap());
            MSGAck_txs.insert(node_id, MSGAck_tx);
            MSGAck_rxs.insert(node_id, MSGAck_rx);
            let (mut GOSSIP_tx, mut GOSSIP_rx) = ring_channel(NonZeroUsize::new(1).unwrap());
            GOSSIP_txs.insert(node_id, GOSSIP_tx);
            GOSSIP_rxs.insert(node_id, GOSSIP_rx);
        }

        msgs_buffer_txs.insert(MessageType::MSG, MSG_txs);
        msgs_buffer_txs.insert(MessageType::MSGAck, MSGAck_txs);
        msgs_buffer_txs.insert(MessageType::GOSSIP, GOSSIP_txs);

        msgs_buffer_rxs.insert(MessageType::MSG, MSG_rxs);
        msgs_buffer_rxs.insert(MessageType::MSGAck, MSGAck_rxs);
        msgs_buffer_rxs.insert(MessageType::GOSSIP, GOSSIP_rxs);

        let mut buffer_txs = node.msgs_buffer_txs.lock().unwrap();
        *buffer_txs = Some(msgs_buffer_txs);
        thread::spawn(move || {
            do_forever_loop_node.do_forever_loop(stop_thread_rx, msgs_buffer_rxs);
        });
        stop_thread_tx        
    }

    fn do_forever_loop(&self, rx: Receiver<()>, mut msgs_buffer_rxs: HashMap<MessageType, HashMap<NodeId, RingReceiver<String>>>) {
        let mut iteration = 0;
        loop {
            iteration += 1;

            // Self-stabilization recovery, which is carried out every N iteration.
            if iteration % 100 == 0 {
                self.self_stabilizing_recovery();
            }

            self.advance_rxObsS_based_on_obs_record();
            // printlnu(format!("Inside do_forever, 5"));
            self.remove_unreasonable_records_from_buffer();
            // printlnu(format!("Inside do_forever, 6"));
            
            // Actual URB handling logic.
            self.handle_records_in_buffer();
            // printlnu(format!("Inside do_forever, 7"));
            self.gossip();

            self.handle_received_msgs(&mut msgs_buffer_rxs);

            let dummy_mutex = self.dummy_mutex.lock().unwrap();
            let _ = self.buffer_updated.wait_timeout(dummy_mutex, Duration::from_millis(100));

            match rx.try_recv() {
                Err(TryRecvError::Empty) => {}
                _ => {
                    self.thetafd.stop_thread();
                    self.hbfd.stop_thread();
                    // let buffer = self.buffer.lock().unwrap();
                    // printlnu(format!("Terminating do_forever_loop. buffer: {:?}", buffer.records));
                    break;
                }
            }
        }
    }

    fn self_stabilizing_recovery(&self) {
        self.clean_buffer_if_corrupted();
        self.reset_txObsS_if_corrupted();
        self.adjust_rxObsS_and_next_to_deliver_if_corrupted();
    }

    fn clean_buffer_if_corrupted(&self) {
        let mut buffer = self.buffer.lock().unwrap();
        let mut tags_seen = HashSet::new();
        let mut buffer_corrupted = false;
        for (tag, record) in buffer.iter() {
            if record.msg.is_none() || tags_seen.contains(&tag) {
                buffer_corrupted = true;
            } else {
                tags_seen.insert((&tag).clone());
            }
        }
        if buffer_corrupted {
            printlnu(format!("Buffer corrupted! Emptying the buffer."));
            *buffer = HashMap::new();
            panic!("Shit Happens!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        }
    } 

    fn reset_txObsS_if_corrupted(&self) {
        let ms = self.minTxObsS();
        let seq = self.seq.lock().unwrap();
        let seq_reasonable = *seq >= ms && *seq <= ms + WINDOW_SIZE;
        if !seq_reasonable {
            printlnu(format!("Seq not resonable! seq: {}, ms: {}", *seq, ms));
            self.uniform_txObsS(*seq);
            return;
        }

        let mut seqs_should_be_in_buffer: HashSet<Int> = (ms + 1  .. *seq + 1).collect();
        // printlnu(format!("Expected seqs in buffer: {:?}, seq = {}", seqs_should_be_in_buffer, *seq));

        let buffer = self.buffer.lock().unwrap();
        let self_id = self.id();
        for (tag, _) in buffer.iter() {
            if tag.id == self_id {
                seqs_should_be_in_buffer.remove(&tag.seq);
            } 
        }
        let all_seqs_in_process_are_present = seqs_should_be_in_buffer.is_empty(); 

        if !all_seqs_in_process_are_present {
            printlnu(format!("Not all seqs are present. missing: {:?}", seqs_should_be_in_buffer));
            self.uniform_txObsS(*seq);
        } else {

            // printlnu(format!("All seqs are present."));
        }
    }

    fn uniform_txObsS(&self, seq: Int) {
        let mut txObsS = self.txObsS.lock().unwrap();
        *txObsS = vec![seq; txObsS.len()]; 
        self.txObsS_changed.notify_one();
    }

    fn adjust_rxObsS_and_next_to_deliver_if_corrupted(&self) {
        let mut rxObsS = self.rxObsS.lock().unwrap();

        for node_id in self.delegate().node_ids() {
            let at_least = self.maxSeq(*node_id) - WINDOW_SIZE;
            if rxObsS[(node_id - 1) as usize] < at_least {
                printlnu(format!("rxObsS corrupted. current value: {}, at_least: {}", rxObsS[(node_id - 1) as usize], at_least));

            }
            rxObsS[(node_id - 1) as usize] = cmp::max(at_least, rxObsS[(node_id - 1) as usize]);
            let mut next_to_deliver = self.next_to_deliver.lock().unwrap();
            next_to_deliver[(node_id - 1) as usize] = cmp::max(next_to_deliver[(node_id - 1) as usize], rxObsS[(node_id - 1) as usize]);
        }
    }

    fn advance_rxObsS_based_on_obs_record(&self) {
        // printlnu(format!("Inside advance_rxObs, 1"));
        let buffer = self.buffer.lock().unwrap();
        // printlnu(format!("Inside advance_rxObs, 2"));
        let mut rxObsS = self.rxObsS.lock().unwrap();
        // printlnu(format!("Inside advance_rxObs, 3"));
        let mut has_obsolete = true;
        while has_obsolete {
            has_obsolete = false;
            for (_, record) in buffer.iter() {
                if self.obsolete(&record, &rxObsS) {
                    rxObsS[record.tag.id as usize - 1] += 1;
                    // printlnu(format!("Inside advance_rxObs, rxObs[{}] = {}", record.tag.id, rxObsS[record.tag.id as usize - 1]));
                    has_obsolete = true;
                }
            }
        }
        // printlnu(format!("Inside advance_rxObs, 4"));
    }

    fn remove_unreasonable_records_from_buffer(&self) {
        let mut maxSeqs = HashMap::new();
        for id in self.delegate().node_ids() {
            if *id != self.id() {
                maxSeqs.insert(*id, self.maxSeq(*id));
            }
        }

        let mut buffer = self.buffer.lock().unwrap();
        let rxObsS = self.rxObsS.lock().unwrap();
        // printlnu(format!("Before cleaning, buffer size: {}", buffer.len()));
        
        buffer.retain(|tag, record| {
            let id = tag.id;
            let seq = tag.seq;
            let record_ok;
            if id == self.id() {
                record_ok = self.minTxObsS() < seq; 
            } else {
                record_ok = self.delegate().node_ids().contains(&id) &&
                    rxObsS[id as usize - 1] < seq &&
                    maxSeqs.get(&id).unwrap() - WINDOW_SIZE <= seq;
            }
            if !record_ok {
                let txObsS = self.txObsS.lock().unwrap();
                // printlnu(format!("record {} not ok!!!! TxObsS: {:?}, rxObsS: {:?} , record: {:?}, maxSeq: {:?}", id, txObsS, rxObsS, &record, maxSeqs.get(&id)))
            } else {
                // printlnu(format!("Record ok, record: {:?}", &record));
            }
            record_ok
        });
        // printlnu(format!("After cleaning, buffer size: {}", buffer.len()));
    }

    fn handle_records_in_buffer(&self) {
        let mut buffer = self.buffer.lock().unwrap();
        let trusted = self.trusted();
        if trusted.len() != self.delegate().node_ids().len() {
            // panic!("Some nodes are not trusted.");
            printlnu(format!("Trusted: {:?}", trusted));
        }

        let mut next_to_deliver = self.next_to_deliver.lock().unwrap();
        for (tag, record) in buffer.iter_mut() {
            let ack_by_trusted = trusted.is_subset(&record.recBy);
            if ack_by_trusted && !record.delivered && tag.seq == next_to_deliver[tag.id as usize - 1] {
                let msg = record.msg.clone();
                self.urbDeliver(&tag, msg.unwrap());
                record.delivered = true;
                next_to_deliver[tag.id as usize - 1] += 1;
            }else {
                // printlnu(format!("record being handled but not delivered: {:?}", record));
            }
            // printlnu(format!("record being handled : {:?}", record));
            let hb = self.hbfd.get_hb();
            let recBy = &record.recBy;
            let prevHB = &mut record.prevHB;
            let txObsS = self.txObsS.lock().unwrap();
            for node_id in self.delegate().node_ids() {
                if (!recBy.contains(&node_id) ||
                    (tag.id == self.id() && tag.seq == txObsS[*node_id as usize - 1] + 1)) &&
                    prevHB[*node_id as usize - 1] < hb[*node_id as usize - 1] {
                        let msg = MSG {sender: self.id(), msg: Cow::Borrowed(&record.msg), tag: record.tag.clone()};
                        let json_msg = self.jsonify_message(&msg);
                        self.send_json_message_to(&json_msg, *node_id);
                        // printlnu(format!("Sending buffer record msg to {}: {:?}", *node_id, &tag));
                    }
            }
            record.prevHB = hb;
        }
    }

    fn gossip(&self) {
        // printlnu(format!("Inside gossip, 1"));
        
        let rxObsS = self.rxObsS.lock().unwrap();
        // printlnu(format!("Inside gossip, 2"));
        let txObsS = self.txObsS.lock().unwrap();
        // printlnu(format!("Inside gossip, 3"));

        for &node_id in self.delegate().node_ids() {
            let maxSeq = self.maxSeq(node_id);
            let rxObsS_for_id = rxObsS[node_id as usize - 1];
            let txObsS_for_id = txObsS[node_id as usize - 1];
            let gossip_msg = GOSSIP { sender: self.id(), maxSeq: maxSeq, rxObsS: rxObsS_for_id, txObsS: txObsS_for_id };
            let json_msg = self.jsonify_message(&gossip_msg);
            self.send_json_message_to(&json_msg, node_id);
        }

    }

    fn handle_received_msgs(&self, msgs_buffer_rxs: &mut HashMap<MessageType, HashMap<NodeId, RingReceiver<String>>>) {
        // let rx = self.recv_msg_channel_rx.lock().unwrap();
        for (msg_type, rxs) in msgs_buffer_rxs {
            match msg_type {
                MessageType::MSG | MessageType::MSGAck => {
                    for (node_id, rx) in rxs {
                        // Handle at most 100 message for each channel to avoid starving.
                        for counter in 0..10000 {
                            match rx.try_recv() {
                                Ok(json) => {
                                    if json_is_MSG_message(&json) {
                                        if let Ok(MSG_message) = serde_json::from_str(&json) {
                                            self.MSG_received(MSG_message);
                                        }
                                        continue;
                                    }
                                    if json_is_MSGAck_message(&json) {
                                        if let Ok(MSGAck_message) = serde_json::from_str(&json) {
                                            self.MSGAck_received(MSGAck_message);
                                        }
                                        continue;
                                    }
                                }
                                Err(_) => break, 
                            }
                        }
                    }

                },
                MessageType::GOSSIP => {
                    for (node_id, rx) in rxs {
                        // There is only one slot for GOSSIP messages.
                        match rx.try_recv() {
                            Ok(json) => {
                                if json_is_GOSSIP_message(&json) {
                                    if let Ok(GOSSIP_message) = serde_json::from_str(&json) {
                                        self.GOSSIP_received(GOSSIP_message);
                                    }
                                }
                            }, 
                            Err(_) => continue,
                        }
                    }

                },
            }
        }
        
    }

    //
    // Message reception triggered events.
    //

    fn MSG_received(&self, msg: MSG<V>) {
        // printlnu(format!("MSG_received: {:?}", &msg));
        let ack = MSGAck {sender: self.id(), tag: msg.tag.clone() };
        // printlnu(format!("Sending ack to {} about tag: {:?} ", msg.sender, msg.tag));
        self.update(msg.msg.into_owned(), msg.tag, msg.sender);
        // printlnu(format!("Current buffer: {:?}", self.buffer.lock().unwrap()));

        // self.buffer_updated.notify_one();
        let json_ack = self.jsonify_message(&ack);
        self.send_json_message_to(&json_ack, msg.sender);

    }

    fn MSGAck_received(&self, msg: MSGAck) {
        // printlnu(format!("MSGAck_received: {:?}", &msg));
        self.update(None, msg.tag, msg.sender);
        // self.buffer_updated.notify_one();
    }

    fn GOSSIP_received(&self, msg: GOSSIP) {
        let mut seq = self.seq.lock().unwrap();
        *seq = cmp::max(*seq, msg.maxSeq);
        drop(seq);

        let mut rxObsS = self.rxObsS.lock().unwrap();
        if rxObsS[(msg.sender - 1) as usize] < msg.txObsS {
                // printlnu(format!("rxObsS changed because GOSSIP. current value: {}, gossip: {}", rxObsS[(msg.sender - 1) as usize], msg.txObsS));

        }
        rxObsS[msg.sender as usize - 1] = cmp::max(rxObsS[msg.sender as usize - 1], msg.txObsS);
        drop(rxObsS);

        let mut txObsS = self.txObsS.lock().unwrap();
        txObsS[msg.sender as usize - 1] = cmp::max(txObsS[msg.sender as usize - 1], msg.rxObsS);
        drop(txObsS);

        self.txObsS_changed.notify_one();
    }

    //
    // Message sending, reception and serialization
    //
    pub fn get_echo_msg(&self) -> String {
        let echo_msg = self.echo.lock().unwrap();
        return format!("{:?}", echo_msg.get(0));
    }
    pub fn rm_echo_msg(&self){
        let rm_echo_msg = self.echo.lock().unwrap().pop_front();
        //printlnu(format!("POPED echo VEC.len = {}", self.echo.lock().unwrap().len()));
    }
    pub fn get_alive_msg(&self) -> String {
        let alive_msg = self.alive.lock().unwrap();
        return format!("{:?}", alive_msg.get(0));
    }
    pub fn rm_alive_msg(&self){
        let rm_alive_msg = self.alive.lock().unwrap().pop_front();
        //printlnu(format!("POPED ALIVE VEC.len = {}", self.alive.lock().unwrap().len()));
    }
    pub fn get_response_msg(&self) -> String {
        let response_msg = self.response.lock().unwrap();
        return format!("{:?}", response_msg.get(0));
    }
    pub fn rm_response_msg(&self){
        let rm_response_msg = self.response.lock().unwrap().pop_front();
        //printlnu(format!("POPED RESPONSE VEC.len = {}", self.response.lock().unwrap().len()));
    }

    pub fn get_phase_msg(&self) -> String {
        let phase_msg = self.phase.lock().unwrap();
        return format!("{:?}", phase_msg.get(0));
    }
    pub fn rm_phase_msg(&self){
        let rm_phase_msg = self.phase.lock().unwrap().pop_front();
        //printlnu(format!("POPED phase VEC.len = {}", self.phase.lock().unwrap().len()));
    }

    pub fn get_decide_msg(&self) -> String {
        let decide_msg = self.decide.lock().unwrap();
        return format!("{:?}", decide_msg.get(0));
    }
    pub fn rm_decide_msg(&self){
        let rm_decide_msg = self.decide.lock().unwrap().pop_front();
        //printlnu(format!("POPED decide VEC.len = {}", self.decide.lock().unwrap().len()));
    }

    pub fn get_proposal_msg(&self) -> String {
        let proposal_msg = self.proposal.lock().unwrap();
        return format!("{:?}", proposal_msg.get(0));
    }
    pub fn rm_proposal_msg(&self){
        let rm_proposal_msg = self.proposal.lock().unwrap().pop_front();
        //printlnu(format!("POPED proposal VEC.len = {}", self.proposal.lock().unwrap().len()));
    }

    pub fn get_to_urb_msg(&self) -> String {
        let to_urb_msg = self.to_urb.lock().unwrap();
        return format!("{:?}", to_urb_msg.get(0));
    }
    pub fn rm_to_urb_msg(&self){
        let rm_to_urb_msg = self.to_urb.lock().unwrap().pop_front();
        //printlnu(format!("POPED to_urb VEC.len = {}", self.to_urb.lock().unwrap().len()));
    }

    pub fn get_sync_msg(&self) -> String {
        let sync_msg = self.sync.lock().unwrap();
        return format!("{:?}", sync_msg.get(0));
    }
    pub fn rm_sync_msg(&self){
        let rm_sync_msg = self.sync.lock().unwrap().pop_front();
        //printlnu(format!("POPED sync VEC.len = {}", self.sync.lock().unwrap().len()));
    }

    pub fn get_syncack_msg(&self) -> String {
        let syncack_msg = self.syncack.lock().unwrap();
        return format!("{:?}", syncack_msg.get(0));
    }
    pub fn rm_syncack_msg(&self){
        let rm_syncack_msg = self.syncack.lock().unwrap().pop_front();
        //printlnu(format!("POPED syncack VEC.len = {}", self.syncack.lock().unwrap().len()));
    }

    fn jsonify_message<Me: Message>(&self, message: &Me) -> String {
        serde_json::to_string(message).expect("Could not serialize a message")
    }

    fn send_json_message_to(&self, json: &str, receiver_id: NodeId) {
        self.delegate().send_json_to(json, receiver_id);
    }

    pub fn json_received(&self, json: &str) {
        if json.starts_with("\"ECHO"){
            let echo = self.echo.lock().unwrap().push_back(json.to_string());
            return;
        }

        if json.starts_with("\"ALIVE"){
            let alive = self.alive.lock().unwrap().push_back(json.to_string());
            return;
        }

        if json.starts_with("\"RESPONSE"){
            let response = self.response.lock().unwrap().push_back(json.to_string());
            return;
        }

        if json.starts_with("\"PHASE"){
            let phase = self.phase.lock().unwrap().push_back(json.to_string());
            return;
        }

        if json.starts_with("\"SYNC"){
            let sync = self.sync.lock().unwrap().push_back(json.to_string());
            return;
        }

        if json.starts_with("\"S_ACK"){
            let syncack = self.syncack.lock().unwrap().push_back(json.to_string());
            return;
        }

        if json_is_HbfdMessage(&json) {
            if let Ok(hbfd_message) = serde_json::from_str(&json) {
                return self.hbfd.on_heartbeat(hbfd_message);
            }
        }

        if json_is_ThetafdMessage(&json) {
            if let Ok(thetafd_message) = serde_json::from_str(&json) {
                return self.thetafd.on_heartbeat(thetafd_message);
            }
        }

        let mut msgs_buffer_txs = self.msgs_buffer_txs.lock().unwrap();
        match &mut *msgs_buffer_txs {
            Some(buffer_txs) => {
                if json_is_MSG_message(&json) {
                    if let Ok(MSG_message) = serde_json::from_str::<MSG<V>>(&json) {
                        let _ = buffer_txs.get_mut(&MessageType::MSG).unwrap().get_mut(&MSG_message.sender).unwrap().send(json.to_owned());
                        return;
                    }
                }
                if json_is_MSGAck_message(&json) {
                    if let Ok(MSGAck_message) = serde_json::from_str::<MSGAck>(&json) {
                        let _ = buffer_txs.get_mut(&MessageType::MSGAck).unwrap().get_mut(&MSGAck_message.sender).unwrap().send(json.to_owned());
                        return;
                    }
                }
                if json_is_GOSSIP_message(&json) {
                    if let Ok(GOSSIP_message) = serde_json::from_str::<GOSSIP>(&json) {
                        let _ = buffer_txs.get_mut(&MessageType::GOSSIP).unwrap().get_mut(&GOSSIP_message.sender).unwrap().send(json.to_owned());
                        return;
                    }
                }
            },
            None => return,
        }

        // match tx.send(json.to_owned()) {
        //     Ok(_) => {},
        //     Err(e) => {
        //         printlnu(format!("Cannot send the received msg to the buffer.{:?}", e));
        //     }
        // };
    }

    pub fn transition_to_arbitrary_state(&self) {
        
    }
}
