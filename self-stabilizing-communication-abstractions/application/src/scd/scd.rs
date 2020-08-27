use std::collections::{HashMap, HashSet, BTreeSet};
use std::borrow::Cow;
use std::sync::mpsc::{self, Receiver, SyncSender, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::fmt::Debug;
use std::thread;
use std::time::Duration;
use std::hash::Hash;
use std::rc::Rc;
use std::cmp;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use commons::types::{Int, NodeId, Tag};
use crate::mediator::Mediator;
use crate::urb::NodeDelegate;
use super::messages::*;
use super::types::*;
use crate::settings::SETTINGS;
use crate::terminal_output::printlnu;
use crate::urb::messages::Message;

// TODO: Need to think about using generic type. Currently not used because of type issues in
// channels.
pub struct SCD {
    id: NodeId,
    node_ids: HashSet<NodeId>,
    buffer: Mutex<HashMap<Tag, Entry>>,
    sn: Mutex<Int>,
    clock: Mutex<VectorClock>,

    send_end: Mutex<Sender<String>>,
    send_pattern_end: Mutex<Sender<HashSet<String>>>,

    pub delivered_msgs: Mutex<BTreeSet<Tag>>,
    pub broadcasted_msg_number: Mutex<Int>,
}

impl SCD {
    //
    // Initilization
    //
    
    pub fn new(send_end: Sender<String>, recv_end: Receiver<String>, self_id: NodeId, node_ids: HashSet<NodeId>, send_pattern_end: Sender<HashSet<String>>, recv_pattern_end: Receiver<String>) -> Arc<Self> {
        let number_of_nodes = node_ids.len();
        let scd = SCD {
            id: self_id,
            node_ids: node_ids,
            buffer: Mutex::new(HashMap::new()),
            sn: Mutex::new(1),
            clock: Mutex::new(VectorClock::new(number_of_nodes, 0)),
            send_end: Mutex::new(send_end),
            delivered_msgs: Mutex::new(BTreeSet::new()),
            broadcasted_msg_number: Mutex::new(0),
            send_pattern_end: Mutex::new(send_pattern_end),

        };
        let scd = Arc::new(scd);
        let scd_arc = Arc::clone(&scd);
        thread::spawn(move || {
            scd_arc.listen_for_fifo_delivery_thread(recv_end);
        });
        let scd_arc = Arc::clone(&scd);
        thread::spawn(move || {
            scd_arc.start_do_forever_loop_thread();
        });
        scd
    }

    fn listen_for_fifo_delivery_thread(&self, recv_end: Receiver<String>) {
        loop {
            match recv_end.recv() {
                Ok(msg) => {
                    // printlnu(format!("Msg fifo delivered: {}", &msg));
                    self.msg_received(msg);
                }
                Err(e) => {
                    printlnu(format!("Error when trying to receive msg from fifo pipe: {:?}", e));
                }
            }
        }
    }

    fn start_do_forever_loop_thread(&self) {
        loop {

            // printlnu(format!("do_forever: 1"));
            // A merged iteration for line 120 and 121.
            let mut buffer = self.buffer.lock().unwrap();
            // printlnu(format!("do_forever: 1.1"));
            let mut clock = self.clock.lock().unwrap();
            // printlnu(format!("do_forever: 1.2"));
            for (tag, entry) in buffer.iter_mut() {
                if tag.seq <= clock.get(tag.id) {
                    entry.delivered = true;
                } else if entry.delivered {
                    clock.set(tag.id, tag.seq);
                }

            }

            // printlnu(format!("do_forever: 2"));
            // This is not in the paper. I added this.
            buffer.retain(|_, entry| {
                !entry.delivered
            });

            // printlnu(format!("do_forever: 3"));
            // Gossip
            for node_id in &self.node_ids {
                let gossip = GOSSIP {
                    clock: clock.get(*node_id),
                };
                let json_msg = self.jsonify_message(&gossip);
                self.fifoBroadcast(json_msg);
            }

            // printlnu(format!("do_forever: 4"));
            drop(clock);

            // printlnu(format!("Current buffer: {:?}", buffer));
            drop(buffer);
            thread::sleep(Duration::from_secs(1));
            // printlnu(format!("do_forever: 5"));

        }
    }

    //
    // SCD broadcast API 
    //

    pub fn scdBroadcast(&self, msg: String)  -> Option<Tag> {
        let mut broadcasted_msg_number = self.broadcasted_msg_number.lock().unwrap();
        *broadcasted_msg_number += 1;
        // printlnu(format!("call scdBroadcast: {:?}", msg));
        let mut sn = self.sn.lock().unwrap();
        let sn_copy = *sn;
        drop(sn);
        let msg_tag = Tag {id: self.id, seq: sn_copy};
        self.forward(msg, msg_tag.clone(), msg_tag)
    }

    pub fn scdDeliver(&self, msgs: HashSet<String>) {
        if SETTINGS.print_client_operations() {
            printlnu(format!("scdDelivered: {:?}", msgs));
        }
        let send_to_pattern_end = self.send_pattern_end.lock().unwrap();
        let _ = send_to_pattern_end.send(msgs);
    }

    //
    // Interfaces 
    //

    pub fn hasTerminated(&self, tag: &Tag) -> bool {
        let buffer = self.buffer.lock().unwrap();
        if let Some(entry) = buffer.get(tag) {
            if !entry.delivered {
                return false;
            }
        }
        true
    } 

    pub fn allHaveTerminated(&self) -> bool {
        let buffer = self.buffer.lock().unwrap();
        for (_, entry) in &*buffer {
            if !entry.delivered {
                return false;
            }
        }
        true
    } 


    //
    // Main logic 
    //
    
    fn forward(&self, msg: String, msg_tag: Tag, forward_tag: Tag) -> Option<Tag> {
        // printlnu(format!("Inside forward 1"));
        let clock = self.clock.lock().unwrap();
        // printlnu(format!("Inside forward 2"));
        let clock_for_id = clock.get(msg_tag.id); 
        drop(clock);
        if msg_tag.seq > clock_for_id {
            // printlnu(format!("Inside forward 3"));
            let mut buffer = self.buffer.lock().unwrap();
            // printlnu(format!("Inside forward 4"));
            match buffer.get_mut(&msg_tag) {
                Some(entry) => 
                    entry.cl.set(forward_tag.id, forward_tag.seq),
                None => {
                    // printlnu(format!("Inside forward 5"));
                    let mut threshold = VectorClock::new(self.node_ids.len(), Int::max_value());
                    threshold.set(forward_tag.id, forward_tag.seq);

                    let mut sn = self.sn.lock().unwrap();
                    // printlnu(format!("Inside forward 6"));
                    let forward_msg = FORWARD {
                        msg: Cow::Borrowed(&msg),
                        msg_tag: msg_tag.clone(),
                        forward_tag: Tag {id: self.id, seq: *sn},
                        cl: threshold.clone()
                    };
                    let json_msg = self.jsonify_message(&forward_msg);

                    let entry = Entry {
                        msg: msg,
                        tag: msg_tag.clone(),
                        cl: threshold,
                        delivered: false,
                        urb_tag: Tag { id: 0, seq: 0 }
                    };
                    // printlnu(format!("Inside forward(), buffer inserted with an entry"));
                    buffer.insert(msg_tag.clone(), entry);
                    self.fifoBroadcast(json_msg);
                    // printlnu(format!("Inside forward(), buffer: {:?}", buffer));
                    *sn += 1;
                    drop(sn);
                    drop(buffer);
                    self.tryDeliver();
                    let mut return_tag = msg_tag.clone();
                    return_tag.seq = return_tag.seq - 1;
                    return Some(return_tag);
                }
                
            }
        } else {
            // printlnu(format!("msg_tag: {:?} <= clock_for_id: {}", msg_tag, clock_for_id));
        }
 
        self.tryDeliver();
        None
    }

    fn fifoBroadcast(&self, json_msg: String) {
        let mut send_end = self.send_end.lock().unwrap();
        if let Err(e) = send_end.send(json_msg) {
            printlnu(format!("Error when sending in forward(): {:?}", e));
        }
    }

    // TODO: Better refactor and write some tests to test this function.
    fn tryDeliver(&self) {
        // printlnu(format!("try_deliver 1"));
        let mut to_deliver = HashSet::new();
        let mut buffer_exclude_to_deliver = HashSet::new();

        let mut buffer = self.buffer.lock().unwrap();
//        printlnu(format!("In tryDeliver, buffer: {:?}", buffer));
        // printlnu(format!("try_deliver 2"));
        for (_, entry) in &*buffer {
            if self.majority_aware(&entry.cl) && !entry.delivered {
                to_deliver.insert(Rc::new(entry));
            } else if !entry.delivered {
                buffer_exclude_to_deliver.insert(Rc::new(entry));
            }
        }

        let mut changed = true;
        while changed {
            changed = false;

            let mut cannot_deliver = None;
            'outer: for entry in to_deliver.iter().cloned() {
                for entry_p in buffer_exclude_to_deliver.iter() {
                    if self.cannot_deliver(&entry.cl, &entry_p.cl) {
                        cannot_deliver = Some(entry);
                        break 'outer ;
                    }
                } 
            }
            if let Some(entry) = cannot_deliver.take() {
                buffer_exclude_to_deliver.insert(entry.clone());
                to_deliver.remove(&entry);
                changed = true;
            }

        }
        // printlnu(format!("to_deliver: {:?}", to_deliver));
        // printlnu(format!("buffer_exclude_to_deliver: {:?}", buffer_exclude_to_deliver));
        
        
        // printlnu(format!("try_deliver 3"));
        let mut tags_to_deliver = HashSet::new();
        for entry in to_deliver {
            let tag = &entry.tag; 
            tags_to_deliver.insert(tag.clone());
        }

        // printlnu(format!("try_deliver 4"));
        let mut clock = self.clock.lock().unwrap();
        // printlnu(format!("try_deliver 5"));
        let mut msgs_to_deliver = HashSet::new();
        for tag in &tags_to_deliver {
            let mut entry = buffer.get_mut(tag).unwrap();
            let max_sn = cmp::max(clock.get(tag.id), tag.seq);
            clock.set(tag.id, max_sn);
            entry.delivered = true;
             msgs_to_deliver.insert(entry.msg.clone());
//            msgs_to_deliver.insert(format!("{:?}", entry.tag));
//            msgs_to_deliver.insert(format!("{:?}", entry.msg));
            let mut delivered_msgs = self.delivered_msgs.lock().unwrap();
            delivered_msgs.insert(tag.clone());
        }
        // printlnu(format!("msgs_to_deliver: {:?}", msgs_to_deliver));
        if !msgs_to_deliver.is_empty() {
            self.scdDeliver(msgs_to_deliver);

            let key_set: HashSet<Tag> = buffer.keys().cloned().collect(); 
            // printlnu(format!("After deliver, buffer: {:?}", buffer));
        }
    }

    fn cannot_deliver(&self, cl_in_question: &VectorClock, cl_reference: &VectorClock) -> bool {
        let mut counter = 0;
        for &id in &self.node_ids {
            if cl_in_question.get(id) < cl_reference.get(id) {
                counter += 1;
            }
        }
        if counter <= self.node_ids.len() / 2 {
            true
        } else {
            false
        }
    }

    fn majority_aware(&self, cl: &VectorClock) -> bool {
        let mut counter = 0;
        for &val in cl.inner() {
            if val < Int::max_value() {
                counter += 1;
            }
        }
        if counter > self.node_ids.len() / 2 {
            true
        } else {
            false
        }
    }


    //
    //  Protocol messages handling
    //

    fn jsonify_message<Me: Message>(&self, message: &Me) -> String {
        serde_json::to_string(message).expect("Could not serialize a message")
    }

    fn msg_received(&self, msg: String) {
        // printlnu(format!("msg_received: {:?}", msg));
        if json_is_FORWARD_message(&msg) {
            if let Ok(forward_message) = serde_json::from_str(&msg) {
                return self.FORWARD_received(forward_message);
            }

        }
        if json_is_GOSSIP_message(&msg) {
            if let Ok(gossip_message) = serde_json::from_str(&msg) {
                return self.GOSSIP_received(gossip_message);
            }

        }
        printlnu(format!("Cannot handle received msg: {}", &msg));

    }

    fn FORWARD_received(&self, forward_msg: FORWARD) {
        let m = forward_msg.msg;
        let msg_tag = forward_msg.msg_tag;
        let forward_tag = forward_msg.forward_tag;
        let _ = self.forward(m.into_owned(), msg_tag, forward_tag);

    }

    fn GOSSIP_received(&self, gossip_msg: GOSSIP) {
        let cl = gossip_msg.clock;

        let mut clock = self.clock.lock().unwrap();
        if cl > clock.get(self.id) {
            clock.set(self.id, cl);
        }
        
        let mut sn = self.sn.lock().unwrap();
        *sn = cmp::max(*sn, clock.get(self.id));
        drop(sn);
        drop(clock);
        // self.tryDeliver();
    }
}

