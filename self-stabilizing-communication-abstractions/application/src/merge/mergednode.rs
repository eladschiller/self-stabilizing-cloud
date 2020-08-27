use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use std::collections::{HashMap, HashSet, VecDeque, BTreeSet};
use commons::types::{NodeId, Int, Tag};
use crate::settings::SETTINGS;
use std::sync::{Mutex, Condvar, MutexGuard, mpsc, Arc};
use ring_channel::{RingSender, RingReceiver};
use commons::{constants, arguments};
use commons::run_result::RunResult;
use crate::urb::thetafd::{ThetafdMessage, json_is_ThetafdMessage};
use crate::urb::hbfd::{HbfdMessage, json_is_HbfdMessage};
use nix::sys::socket::send;
use std::cmp::max;
use std::{cmp, thread, fs};
use crate::terminal_output::printlnu;
use commons::constants::WINDOW_SIZE;
use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::time::{Duration, Instant, SystemTime};
use crate::urb::types::BufferRecord;
use crate::urb::messages::{MSG, GOSSIP, json_is_MSG_message, json_is_MSGAck_message, json_is_GOSSIP_message, MSGAck, Message};
use std::borrow::{Cow, Borrow};
use ring_channel::*;
use std::num::NonZeroUsize;
use std::str;
use crate::scd::types::{VectorClock, Entry};
use crate::scd::messages::{SCDGOSSIP, SCDMETA, json_is_SCDGOSSIP_message, FORWARD, json_is_FORWARD_message};
use std::rc::Rc;
use std::ops::Deref;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hasher, Hash};
use crate::scd::scd::SCD;
use crate::merge::mergednode::StatusCode::ErrNoSpace;
use std::process::id;
use std::iter::FromIterator;
use rand::{Rng, thread_rng};
use rand::prelude::ThreadRng;
use std::path::Prefix::Verbatim;
use commons::arguments::record_evaluation_info;
use crate::scd::algorithm6::messages::{PLUS, MINUS};
use commons::variant::Variant;
use crate::merge::snapshot::Timestamp;
use bit_vec::BitVec;

#[derive(Hash, Eq, PartialEq)]
pub enum MessageType {
    MSG,
    MSGAck,
    GOSSIP,
    FORWARD,
    SCDGOSSIP,
}

pub struct MergedNode {
    pub node_id: NodeId,
    pub node_ids: HashSet<NodeId>,

    //Communicator
    pub socket: UdpSocket,
    pub socket_addrs: HashMap<NodeId, SocketAddr>,

    // Theta
    pub theta_vector: HashMap<NodeId, Int>,
    pub current_trusted: BitVec,

    //Hbfd
    pub hb: HashMap<NodeId, Int>,

    //Urb
    pub seq: Int,
    pub buffer: Vec<BufferRecord<String>>,
    pub urb_rxObsS: Vec<Int>,
    pub urb_txObsS: Vec<Int>,

    pub msgs_buffer_txs: Option<HashMap<MessageType, HashMap<NodeId, RingSender<String>>>>,
    pub msgs_buffer_rxs: Option<HashMap<MessageType, HashMap<NodeId, RingReceiver<String>>>>,

    pub next_to_deliver: Vec<Int>,

    //Scd
    pub sn: Int,
    pub scd_rxObsS: Vec<Int>,
    pub scd_txObsS: Vec<Int>,
    pub scd_rxSpace: Vec<Option<Int>>,
    pub scd_txSpace: Vec<Option<Int>>,

    // Gossip
    pub gossip_sent: Vec<bool>,

    // Application
    pub counter: Int,
    pub is_reading: bool,
    pub reg: HashMap<Int, Int>,
    pub tsa: HashMap<Int,Timestamp>,
    last_r: Int,
    last_v: Int,

    // Operations
    from_application: Option<Receiver<String>>,

    pub bcast_status: Option<Sender<StatusCode>>,

    to_application: Mutex<Sender<String>>,
    application_recv: Mutex<Receiver<String>>,

    // Self-stabilization test
    pub(crate) has_failed: bool,

    // Evaluation
    pub run_result: RunResult,
    pub delivered_tags: BTreeSet<Tag>,
    pub start_time: SystemTime,
    pub fail_time: Option<Instant>,

    pub has_seen_bot: bool,
    pub sn_seen: HashSet<Int>,
    pub rxObsS_seen: HashMap<NodeId, HashSet<Int>>,
    pub txObsS_seen: HashMap<NodeId, HashSet<Int>>,

    pub throughput_msgs: Option<Vec<Tag>>,
    pub throughput_instant: Option<Instant>,

    // victory round
    pub victory_round: bool,
    is_every_node_ready: bool,
    nodes_ready: Vec<bool>,
}

const ITERATIONS_UNTIL_FAIL: Int = 100;

pub enum StatusCode {
    Ok,
    ResultReady,
    ResultNotReady,
    ErrNoSpace,
    Finished,
}


impl MergedNode {
    pub fn new(link_latencies: HashMap<i32, f64>) -> MergedNode {
        // Setup sockets
        let node_id = SETTINGS.node_id();
        let socket_addrs = SETTINGS.socket_addrs().clone();
        let node_ids : HashSet<NodeId> = socket_addrs.keys().map(|node_id| *node_id).collect();
        printlnu(format!("socket addresses: {:?}", socket_addrs));
        let port = socket_addrs.get(&node_id).expect("Could not found own socket address.").port();
        let own_socket_addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0,0,0,0)),
            port,
        );
        let socket = UdpSocket::bind(own_socket_addr).expect("Could not create socket.");
        let _ = socket.set_read_timeout(Some(Duration::from_millis(50)));
        let _ = socket.set_write_timeout(Some(Duration::from_millis(200)));
        let _ = socket.set_nonblocking(true);
        let mut hashmap = HashMap::new();
        let (app_send,app_recv) = mpsc::channel();
        for node_id in node_ids.clone() {
            hashmap.insert(node_id, 0);
        }
        let vector = hashmap.clone();
        let hb = hashmap;
        let current_trusted = BitVec::from_elem(node_ids.len(), true);
        let mut run_result = RunResult::new();
        run_result.link_latency = link_latencies;
        let number_of_nodes = node_ids.len();
        let mut node = MergedNode{
            node_id,
            node_ids,
            socket,
            socket_addrs,
            seq: 0,
            buffer: Vec::new(),
            urb_rxObsS: vec![0; number_of_nodes],
            urb_txObsS: vec![0; number_of_nodes],
            msgs_buffer_txs: None,
            msgs_buffer_rxs: None,
            next_to_deliver: vec![1; number_of_nodes],
            theta_vector: vector,
            current_trusted,
            hb,
            sn: 1,
            scd_rxObsS: vec![0; number_of_nodes],
            scd_txObsS: vec![0; number_of_nodes],
            scd_rxSpace: vec![None; number_of_nodes],
            scd_txSpace: vec![None; number_of_nodes],
            gossip_sent: vec![false; number_of_nodes],
            counter: 0,
            is_reading: false,
            reg: HashMap::new(),
            tsa: HashMap::new(),
            last_r: 0,
            last_v: 0,
            from_application: None,
            bcast_status: None,
            to_application: Mutex::new(app_send),
            application_recv: Mutex::new(app_recv), // should probably just be passed along not a field
            run_result: run_result,
            delivered_tags: BTreeSet::new(),
            start_time: SystemTime::now(),
            has_failed: false,
            has_seen_bot: false,
            sn_seen: HashSet::new(),
            rxObsS_seen: HashMap::new(),
            txObsS_seen: HashMap::new(),
            throughput_msgs: None,
            throughput_instant: None,
            victory_round: true,
            is_every_node_ready: false,
            nodes_ready: vec![false; number_of_nodes],
            fail_time: None
        };
        for i in 0..node.scd_rxObsS.len() {
            let mut set = HashSet::new();
            set.insert(node.scd_rxObsS[i]);
            node.rxObsS_seen.insert((i + 1) as i32, set);
        }
        for i in 0..node.scd_txObsS.len() {
            let mut set = HashSet::new();
            set.insert(node.scd_txObsS[i]);
            node.txObsS_seen.insert((i + 1) as i32, set);
        }

        node
    }

    pub fn start_the_do_forever_loop(mut node: Self) -> (Sender<()>, Sender<String>, Receiver<StatusCode>) {
        let (stop_thread_tx, stop_thread_rx) = mpsc::channel();

        let mut msgs_buffer_txs = HashMap::new();
        let mut msgs_buffer_rxs = HashMap::new();

        let mut MSG_txs = HashMap::new();
        let mut MSGAck_txs = HashMap::new();
        let mut Forward_txs = HashMap::new();
        let mut GOSSIP_txs = HashMap::new();
        let mut SCDGOSSIP_txs = HashMap::new();

        let mut MSG_rxs = HashMap::new();
        let mut MSGAck_rxs = HashMap::new();
        let mut Forward_rxs = HashMap::new();
        let mut GOSSIP_rxs = HashMap::new();
        let mut SCDGOSSIP_rxs = HashMap::new();

        for node_id in node.node_ids.clone() {
            let (mut MSG_tx, mut MSG_rx) = ring_channel(NonZeroUsize::new(2 * node.urb_buffer_unit_size() as usize + 1).unwrap());
            MSG_txs.insert(node_id, MSG_tx);
            MSG_rxs.insert(node_id, MSG_rx);
            let (mut MSGAck_tx, mut MSGAck_rx) = ring_channel(NonZeroUsize::new(2 * node.urb_buffer_unit_size() as usize + 1).unwrap());
            MSGAck_txs.insert(node_id, MSGAck_tx);
            MSGAck_rxs.insert(node_id, MSGAck_rx);
            let (mut Forward_tx, mut Forward_rx) = ring_channel(NonZeroUsize::new(2 * node.scd_buffer_unit_size() as usize + 1).unwrap());
            Forward_txs.insert(node_id, Forward_tx);
            Forward_rxs.insert(node_id, Forward_rx);
            let (mut GOSSIP_tx, mut GOSSIP_rx) = ring_channel(NonZeroUsize::new(1).unwrap());
            GOSSIP_txs.insert(node_id, GOSSIP_tx);
            GOSSIP_rxs.insert(node_id, GOSSIP_rx);
            let (mut SCDGOSSIP_tx, mut SCDGOSSIP_rx) = ring_channel(NonZeroUsize::new(1).unwrap());
            SCDGOSSIP_txs.insert(node_id, SCDGOSSIP_tx);
            SCDGOSSIP_rxs.insert(node_id, SCDGOSSIP_rx);
        }

        msgs_buffer_txs.insert(MessageType::MSG, MSG_txs);
        msgs_buffer_txs.insert(MessageType::MSGAck, MSGAck_txs);
        msgs_buffer_txs.insert(MessageType::FORWARD, Forward_txs);
        msgs_buffer_txs.insert(MessageType::GOSSIP, GOSSIP_txs);
        msgs_buffer_txs.insert(MessageType::SCDGOSSIP, SCDGOSSIP_txs);

        msgs_buffer_rxs.insert(MessageType::MSG, MSG_rxs);
        msgs_buffer_rxs.insert(MessageType::MSGAck, MSGAck_rxs);
        msgs_buffer_rxs.insert(MessageType::FORWARD, Forward_rxs);
        msgs_buffer_rxs.insert(MessageType::GOSSIP, GOSSIP_rxs);
        msgs_buffer_rxs.insert(MessageType::SCDGOSSIP, SCDGOSSIP_rxs);

        node.msgs_buffer_txs = Some(msgs_buffer_txs);
        node.msgs_buffer_rxs = Some(msgs_buffer_rxs);

        let (status_send, status_recv) = mpsc::channel();
        let (msg_send, msg_recv) = mpsc::channel();

        node.from_application = Some(msg_recv);
        node.bcast_status = Some(status_send);
        node.start_time = SystemTime::now();
        thread::spawn(move || {
            node.do_forever_loop(stop_thread_rx);
        });

        (stop_thread_tx, msg_send, status_recv)
    }

    // forever loop
    fn do_forever_loop(&mut self, rx: Receiver<()>) {
//        self.nodes_ready[self.node_id as usize - 1] = true;
//        while !self.is_every_node_ready {
//            let mut check = true;
//            for is_node_ready in self.nodes_ready.clone() {
//                if !is_node_ready { check = false; }
//            }
//            if check {
//                self.is_every_node_ready = true;
//            }
//            for node_id in self.node_ids.clone() {
//                self.send_json_message_to(format!("{}",self.node_id).as_str(), node_id);
//            }
//
//            let mut buf = [0;100000];
//            loop {
//                match self.socket.recv(&mut buf) {
//                    Ok(amt) => {
//                        let json_string = str::from_utf8(&buf[0..amt]).expect("Error converting bytes to utf8");
////                        printlnu(format!("{}",json_string));
//                        if let Ok(recv_node_id) = json_string.parse::<Int>(){
//                            if recv_node_id <= self.node_ids.len() as i32 {
//                                self.nodes_ready[recv_node_id as usize - 1] = true;
//                            }
//                        }
//
//                        if let Ok(hbfd_message) = serde_json::from_str::<HbfdMessage>(&json_string) {
//                            self.nodes_ready[hbfd_message.sender as usize - 1] = true;
//                        }
//
//                        if let Ok(thetafd_message) = serde_json::from_str::<ThetafdMessage>(&json_string) {
//                            self.nodes_ready[thetafd_message.sender as usize - 1] = true;
//                        }
//                    }
//                    Err(e)=> {
//                        break;
//                    }
//                }
//            }
//        }
        printlnu(format!("start"));

        let mut iterations = 0;
        let mut should_execute_self_stab_statement;
        loop {
            if iterations > ITERATIONS_UNTIL_FAIL * self.node_id
                && SETTINGS.is_failing_node() && !self.has_failed {
                self.has_failed = true;
                self.corrupt_variables();
                self.fail_time = Some(Instant::now());
//                self.duplicate_records();
//                self.modify_records();
//                self.modify_clocks();
            }

            should_execute_self_stab_statement = iterations % SETTINGS.delta() == 0;
            self.recv_operations();
            self.bare_bone_loop_iter(should_execute_self_stab_statement);
//            self.handle_gossip_messages();
            iterations += 1;
            match rx.try_recv() {
                Err(TryRecvError::Empty) => {}
                _ => {
                    let now = SystemTime::now();
                    let mut iter=0;
                    let mut init_num_of_tags = self.delivered_tags.len();
                    let min_iter = 200;
                    let len_size = min_iter;
                    let mut buffer_lens = vec![0;len_size as usize];
                    let is_zero = |lens: Vec<usize> | {
                        let mut ret = true;
                        for i in 0..(lens.len()) {
                           ret = ret && lens[i] == 0;
                        }
                        ret
                    };
                    printlnu(format!("Delivered messages: {}, buffer length: {}", self.delivered_tags.len(), self.buffer.len()));
                    'inner: loop {
                        let mut num_of_tags = cmp::max(self.delivered_tags.len(), 1);
                        init_num_of_tags = cmp::max(init_num_of_tags, num_of_tags);
                        if SETTINGS.print_client_operations() || iter % 10 == 0 {
                            printlnu(format!("Iter: {}, bufferlen: {}", iter, self.buffer.len()));
                        }
                        self.log(format!("Iter: {}, bufferlen: {}", iter, self.buffer.len()));
                        should_execute_self_stab_statement = iter % SETTINGS.delta() == 0;
                        self.bare_bone_loop_iter(should_execute_self_stab_statement);
                        buffer_lens[(iter % len_size) as usize] = self.buffer.len();
                        iter +=1;
                        if is_zero(buffer_lens.clone()) && iter > min_iter || iter > cmp::max(5000, SETTINGS.delta() * 4) || now.elapsed().unwrap().as_secs() as Int >= 10 * 60 {
                            printlnu(format!("Stopping ...  iter={}", iter));
                            break 'inner;
                        }
                        match rx.try_recv() {
                            Err(TryRecvError::Empty) => {}
                            _ => {
                                break 'inner;
                            }
                        }
                    }
                    self.run_result.metadata.run_length = self.start_time.elapsed().unwrap().as_secs() as Int;//SETTINGS.run_length().as_secs() as Int;
                    self.run_result.metadata.node_id = SETTINGS.node_id();
                    let mut hasher = DefaultHasher::new();
                    self.delivered_tags.hash(&mut hasher);

                    let mut run_result = self.run_result.clone();
                    let json = serde_json::to_string(&run_result).unwrap();
                    let latency = format!("{:?}", self.link_latency_pretty(run_result.link_latency));
                    self.log(format!("Stopping, delivered msgs length: {:?}, hash: {:?}, buffer len: {}, run time: {} secs",self.delivered_tags.len(), hasher.finish(), self.buffer.len(), self.run_result.metadata.run_length.clone()));
                    printlnu(format!("Stopping, delivered msgs length: {:?}, hash: {:?}, buffer len: {}, run time: {} secs counter {}",self.delivered_tags.len(), hasher.finish(), self.buffer.len(), run_result.metadata.run_length.clone(), self.counter));
                    self.log(format!("link latency {}",latency));
                    printlnu(format!("link latency {}",latency));
                    if let Some(latencies) = self.run_result.msg_latencies.clone() {
                        if latencies.len() > 0 {
                            let average_lat = {
                                let mut sum = 0;
                                for lat in latencies.clone() {
                                        sum += lat;
                                }
                                sum / latencies.len() as u128
                            };
                            self.log(format!("Average msg latency for sender: {} microseconds", average_lat ));
                            printlnu(format!("Average msg latency for sender: {} microsonds", average_lat ));
                        }
                    }

                    if let Some(tputs) = self.run_result.throughputs.clone() {
                        printlnu(format!("Throughputs: {:?}", tputs));
                        self.log(format!("Throughputs: {:?}", tputs));
                    }

                    //printlnu(format!("Terminating do_forever_loop. buffer: {:?}", self.buffer));
                    if self.has_failed {
                        let rt = self.run_result.recovery_time;
                        printlnu(format!("Recovery time {:?} micros", rt));
                        self.log(format!("Recovery time {:?} micros", rt));
                    }
                    if SETTINGS.variant() == Variant::SNAPSHOT {
                        if let Some(r_lats) = self.run_result.read_latencies.clone() {
                            let s = self.reg_pretty();
                            printlnu(format!("Number of reads: {}, Snapshot final registry: {}", r_lats.len(), s));
                            self.log(format!("Number of reads: {}, Snapshot final registry: {}", r_lats.len(), s));
                        }
                    }
                    if SETTINGS.variant() != Variant::URB {
                        let msp = self.scd_msp();
                        printlnu(format!("(scd) sn: {}, txObsS: {:?}, rxObsS: {:?}, rxSpace: {:?}, txSpace: {:?}, ms_p(i):{:?}", self.sn, self.scd_txObsS, self.scd_rxObsS, self.scd_rxSpace, self.scd_txSpace, msp));
                        self.log(format!("(scd) sn: {}, txObsS: {:?}, rxObsS: {:?}, rxSpace: {:?}, txSpace: {:?}", self.sn, self.scd_txObsS, self.scd_rxObsS, self.scd_rxSpace, self.scd_txSpace));
                    }
                    let trusted = self.trusted();
                    printlnu(format!("trusted: {:?}", trusted));
                    self.log(format!("trusted: {:?}", trusted));
                    printlnu(format!("theta: {:?}", self.theta_vector));
                    self.log(format!("theta: {:?}", self.theta_vector));
                    printlnu(format!("(urb) seq: {}, txObsS: {:?}, rxObsS: {:?}", self.seq, self.urb_txObsS, self.urb_rxObsS));
                    self.log(format!("(urb) seq: {}, txObsS: {:?}, rxObsS: {:?}", self.seq, self.urb_txObsS, self.urb_rxObsS));
                    printlnu(format!("has seen bot {}", self.has_seen_bot));
                    self.log(format!("has seen bot {}", self.has_seen_bot));
                        printlnu(format!("\n\n Buffer: {:?} \n\n", self.buffer));
                    if self.buffer.len() <= 20 {
                    }
                    self.log(format!("\n\n Buffer: {:?} \n\n", self.buffer));
                    let socket_addrs = SETTINGS.socket_addrs().clone();
                    printlnu(format!("socket addresses: {:?}", socket_addrs));
                    self.log(format!("socket addresses: {:?}", socket_addrs));
                    fs::write(
                        arguments::run_result_file_name_from_node_id(SETTINGS.node_id()),
                        json,
                    )
                        .expect("Could not write the json result file");
                    let _ = self.bcast_status.as_ref().unwrap().send(StatusCode::Finished);
                    break;
                }
            }
        }
    }

    pub(crate) fn fd_iter(&mut self) {
        self.theta_iter();
        self.hbfd_iter();
    }
    pub(crate) fn bare_bone_loop_iter(&mut self, should_exec_ss: bool) {
        self.gossip_sent = vec![false; self.node_ids.len()];
        self.recv_messages();

        self.fd_iter();

        self.handle_gossip_messages();
        self.handle_received_msgs();

        self.urb_loop_iter(should_exec_ss);

        match SETTINGS.variant() {
            Variant::URB => {},
            _ => {
                self.scd_loop_iter(should_exec_ss);
            }
        }
    }


    pub(crate) fn recv_operations(&mut self) {
        let mut to_recv = VecDeque::new();
        let mut num_of_msgs = 1;
        loop {
            match SETTINGS.variant() {
                Variant::COUNTER | Variant::SCD | Variant::SNAPSHOT => {
                    if !self.scd_available_space_for(num_of_msgs) {
                        break;
                    }
                },
                Variant::URB => {
                    if !self.urb_available_space_for(num_of_msgs) {
                        break;
                    }
                }
                _ => {}
            }
            match self.from_application.as_ref().unwrap().try_recv() {
                Err(TryRecvError::Empty) => {
                    break;
                }
                Ok(msg) => {
                    if msg == "COUNTER_INCREASE" {
                        self.increase();
                    } else if msg == "COUNTER_DECREASE" {
                        self.decrease();
                    } else if msg == "COUNTER_READ" {
                        //read operation
                        if let Some(result) = self.read() {
                            let _ = self.bcast_status.as_ref().unwrap().send(StatusCode::ResultReady);
                            printlnu(format!("counter: {}", result));
                            self.is_reading = false;
                        }
                    } else if msg == "SCD_BROADCAST" || msg == "URB_BROADCAST" {
                        to_recv.push_back(msg);
                        num_of_msgs += 1;
                    } else if msg == "SNAPSHOT" {
                        self.snapshot();

                    } else if msg == "SNAPSHOT_WRITE" {
                        self.snapshot_write(self.node_id + self.last_r, self.last_v+1);
                        self.last_r += 1;
                        self.last_v += 1;
                    }
                }
                Err(e) => {
                    if !SETTINGS.record_evaluation_info() {
                        panic!(format!("Error receiving from app: {:?}", e));
                    } else {
                        printlnu(format!("Error receiving from app: {:?}", e));
                    }
                }
            }
        }
        while let Some(msg) = to_recv.pop_front() {
            if SETTINGS.variant() == Variant::URB {
                self.urb_broadcast(msg);
            } else if SETTINGS.variant() == Variant::SCD {
                self.scd_broadcast(msg);
            }
        }
    }
}
