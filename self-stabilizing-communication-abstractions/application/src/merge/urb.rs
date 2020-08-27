use super::mergednode::MergedNode;
use crate::urb::types::BufferRecord;
use commons::types::{Int, NodeId, Tag};
use std::cmp;
use crate::terminal_output::printlnu;
use std::collections::{HashSet, HashMap, VecDeque, BTreeSet};
use crate::urb::messages::{MSG, GOSSIP, MSGAck, json_is_MSG_message, json_is_MSGAck_message, json_is_GOSSIP_message, CombinedGossip};
use std::borrow::Cow;
use crate::scd::messages::{json_is_SCDGOSSIP_message, SCDGOSSIP, json_is_FORWARD_message};
use crate::merge::mergednode::MessageType;
use crate::settings::SETTINGS;
use commons::variant::Variant;
use crate::merge::mergednode::StatusCode;
use bit_vec::BitVec;
use std::cmp::{min, Ordering};
use std::time::Instant;

impl MergedNode {
    //Urb
    pub fn urb_loop_iter(&mut self, should_execute_self_stab_statement: bool) {
        if should_execute_self_stab_statement {
            self.self_stabilizing_recovery();
        }
        // printlnu(format!("Buffer is: {:?}", self.buffer));
        // self.advance_urb_rxObsS_based_on_obs_record();
        // printlnu(format!("Inside do_forever, 5"));
        // printlnu(format!("Inside do_forever, 6"));

        // Actual URB handling logic.
        self.handle_records_in_buffer(should_execute_self_stab_statement);
        // printlnu(format!("Inside do_forever, 7"));
        self.remove_unreasonable_urb_records_from_buffer();
        self.advance_urb_rxObsS_based_on_obs_record();

        if should_execute_self_stab_statement {
            self.gossip();
        }
    }

    fn self_stabilizing_recovery(&mut self) {
        self.clean_buffer_if_corrupted();
        self.reset_urb_txObsS_if_corrupted();
        self.adjust_urb_rxObsS_and_next_to_deliver_if_corrupted();
    }

    fn urb_obsolete(&mut self, record: &BufferRecord<String>, urb_rxObsS: &Vec<Int>) -> bool {
        let tag = &record.urb_tag.as_ref().unwrap();
        let mut trusted = self.trusted();
        let recBy_is_subset = MergedNode::is_subset(&trusted, &record.recBy);


        let obs = urb_rxObsS[tag.id as usize - 1] + 1 == tag.seq &&
            record.delivered &&
            recBy_is_subset;

        obs
    }

    pub(crate) fn urb_maxSeq(&self, node_id: NodeId) -> Int {
        let mut max_seq = 0;
        for record in self.buffer.iter() {
            if let Some(urb_tag) = &record.urb_tag {
                if urb_tag.id == node_id {
                    max_seq = cmp::max(max_seq, urb_tag.seq);
                }
            }
        }
        max_seq = cmp::max(max_seq, self.next_to_deliver[node_id as usize - 1] - 1);
        max_seq
    }

    pub(crate) fn min_urb_TxObsS(&mut self) -> Int {
        let trusted = self.trusted();
        let mut min_s = std::i32::MAX;
//        let urb_txObsS = &self.urb_txObsS;
        let mut index = 0;
        for node_id in trusted.iter() {
            if node_id {
                min_s = cmp::min(min_s, self.urb_txObsS[index])
            } else {
                if index == (self.node_id - 1) as usize {
                    printlnu(format!("Not trusted for myself??"));
                }
            }
            index += 1;
        }
        min_s
    }

    pub fn urb_broadcast(&mut self, msg: String) {
        //self.wait_until_receivers_have_enough_space();
        if !self.urb_available_space() {
            let _ = self.bcast_status.as_ref().unwrap().send(StatusCode::ErrNoSpace);
            return;
        }
        match SETTINGS.variant() {
            Variant::URB => {
                let _ = self.bcast_status.as_ref().unwrap().send(StatusCode::Ok);
            },
            _ => {}
        }

        self.seq += 1;
        let tag = Tag { id: self.node_id, seq: self.seq };
        if SETTINGS.print_client_operations() {
            printlnu(format!("urbBroadcasting: {:?} | {:?}", msg.clone(), tag.clone()));
        }
        self.log(format!("urbBroadcasting: {:?} | {:?}", msg.clone(), tag.clone()));
        self.update(Some(msg), tag, self.node_id, None, None);
    }

    pub fn non_blocking_urb_broadcast(&mut self, msg: String, index: Int) -> Option<Tag> {
        if self.urb_available_space() {
            let record = self.buffer.get_mut(index as usize).unwrap();
            let urb_tag = if record.urb_tag.is_none() {
                self.seq += 1;
                record.urb_tag = Some( Tag { id: self.node_id, seq: self.seq });
                Tag { id: self.node_id, seq: self.seq }
            } else {
                self.seq += 1;
                Tag { id: self.node_id, seq: self.seq }
            };
            if SETTINGS.print_client_operations() {
                printlnu(format!("urbBroadcasting: {:?}, sn: {}, scdRxObsS: {:?}, scdTxObsS: {:?}, urb_rxObsS: {:?}", urb_tag, self.sn, self.scd_rxObsS, self.scd_txObsS, self.urb_rxObsS));
            }
            self.log(format!("urbBroadcasting: {:?}, sn: {}, scdRxObsS: {:?}, scdTxObsS: {:?}, urb_rxObsS: {:?}", urb_tag, self.sn, self.scd_rxObsS, self.scd_txObsS, self.urb_rxObsS));
            self.update(Some(msg), urb_tag.clone(), self.node_id, None, None);
            return Some(urb_tag);
        }
        return None;

    }

    pub fn urb_deliver(&mut self, tag: &Tag, msg: String) {
        if SETTINGS.print_client_operations(){
//            printlnu(format!("New message delivered: {:?} {:?}", msg, self.buffer.get(&tag).unwrap()));
            printlnu(format!("New message delivered: {:?} {:?}", msg, tag));
        }
        match SETTINGS.variant() {
            Variant::URB => {
                self.run_result.urb_delivered_msgs.insert(tag.clone());
                self.delivered_tags.insert(tag.clone());
                if self.has_failed && tag.id == self.node_id {
                    if let Some(fail_t) = self.fail_time {
                        if self.run_result.recovery_time.is_none() {
                            printlnu(format!("Recovered in {} ms", fail_t.elapsed().as_millis()));
                            self.run_result.recovery_time = Some(fail_t.elapsed().as_micros());
                        }
                    }
                }
            },
            _ => {
                self.scd_msg_received(msg);
            }
        }
    }

    pub fn urb_buffer_unit_size(&self) -> Int {
        match SETTINGS.variant() {
            Variant::URB => {SETTINGS.window_size()}
            _ => {self.scd_buffer_unit_size() * self.node_ids.len() as i32}
        }
    }

    pub(crate) fn urb_available_space(&mut self) -> bool {
        let mut seq = self.seq;

//        cmp::max((self.min_urb_TxObsS() + self.urb_buffer_unit_size()) - seq, 0)
        seq < self.min_urb_TxObsS() + self.urb_buffer_unit_size()
    }
    pub(crate) fn urb_available_space_for(&mut self, msgs: i32) -> bool {
        let mut seq = self.seq;

//        cmp::max((self.min_urb_TxObsS() + self.urb_buffer_unit_size()) - seq, 0)
        seq + msgs <= self.min_urb_TxObsS() + self.urb_buffer_unit_size()
    }

    fn wait_until_receivers_have_enough_space(&mut self) {
        let mut seq = self.seq;
        while seq >= self.min_urb_TxObsS() + self.urb_buffer_unit_size() {
            if !SETTINGS.record_evaluation_info() {
                panic!("Receiver does not have enough space, going to block. min_urb_TxObsS = {}, seq = {}", self.min_urb_TxObsS(), seq);
            } else {
                printlnu(format!("Receiver does not have enough space, going to block. min_urb_TxObsS = {}, seq = {}", self.min_urb_TxObsS(), seq));
            }
        }
    }

    fn update(&mut self, msg: Option<String>, tag: Tag, forwarder: NodeId, recv_by_bitmap: Option<BitVec>, recv_by_trusted_bitmap: Option<BitVec>) -> bool {
        let sn = self.sn;
        let scd_unit = self.scd_buffer_unit_size();
        let urb_unit = self.urb_buffer_unit_size();
        let trusted = self.trusted();
        let urb_rxObsS = &self.urb_rxObsS;
        if tag.seq <= urb_rxObsS[tag.id as usize - 1] {
            //printlnu(format!("return from update with ready {:?} {}", tag, urb_rxObsS[tag.id as usize - 1]));
            return true;
        }

        let index = self.get_index_by_urb_tag(&tag);
        let buffer_contains_key = index.is_some();
        let mut buffer = &mut self.buffer;
        if !buffer_contains_key && !msg.is_none() {
            let mut recBy = BitVec::from_elem(self.node_ids.len(), false);
            recBy.set(self.node_id as usize - 1, true);
            recBy.set(tag.id as usize - 1, true);
            // Optimization
            if let Some(bitmap) = recv_by_bitmap {
                recBy.or(&bitmap);
            }
            let number_of_nodes = self.node_ids.len();

            let record = BufferRecord {
                urb_tag: Some(tag.clone()),
                msg: msg,
                delivered: false,
                recBy: recBy,
                recBy_trusted: BitVec::from_elem(self.node_ids.len(), false),
                prevHB: vec![-1; number_of_nodes],
                urb_needed: true,
                scd_needed: true,
                meta: None,
                creation_instant:
                    if SETTINGS.variant() == Variant::URB && tag.id == self.node_id {
                        Some(Instant::now())
                    } else {
                        None
                    }
            };
            buffer.push( record);
            return false;
        } else {
//            printlnu(format!("index: {:?} msg {:?} buffer {:?}", index, msg, buffer));
            if index.is_some() {
                if let Some(record) = buffer.get_mut(index.unwrap()) {
                    record.recBy.set(tag.id as usize - 1, true);
                    record.recBy.set(forwarder as usize - 1, true);
                    // Optimization
                    if let Some(mut recv_by) = recv_by_bitmap {
                        record.recBy.or(&recv_by);
                        let recBy_is_subset = MergedNode::is_subset(&trusted, &recv_by);
                        if recBy_is_subset {
                            record.recBy_trusted.set(tag.id as usize - 1, true);
                            record.recBy_trusted.set(forwarder as usize - 1, true);
                            record.recBy_trusted.set(self.node_id as usize - 1, true);
                        }
                        if let Some(recv_by_trusted) = recv_by_trusted_bitmap {
                            record.recBy_trusted.or(&recv_by_trusted);
                        }
                    }
                    if MergedNode::is_subset(&trusted, &record.recBy) {
                        record.recBy_trusted.set(self.node_id as usize - 1, true);
                    }
                    return true;
                }
            }
//            printlnu(format!("after"));
        }
        return false;
    }

    fn clean_buffer_if_corrupted(&mut self) {
        let mut tags_seen = HashSet::new();
        let mut buffer_corrupted = false;
        for record in self.buffer.iter() {
            if let Some(urb_tag) = &record.urb_tag {
                if record.msg.is_none() || tags_seen.contains(urb_tag) {
                    buffer_corrupted = true;
                } else {
                    tags_seen.insert(urb_tag.clone());
                }
            }
        }
        if buffer_corrupted {
            if SETTINGS.print_client_operations() {
                printlnu(format!("Buffer corrupted! Emptying the buffer."));
            }
            self.buffer = Vec::new();
            if !SETTINGS.record_evaluation_info() {
                panic!("Buffer was corrupted and then emptied");
            } else {
                printlnu(format!("Buffer was corrupted and then emptied"));
            }
            self.run_result.illegally_triggered_ss = !SETTINGS.is_failing_node();
            self.log(format!("Buffer was corrupted and then emptied"));
        }
    }

    fn reset_urb_txObsS_if_corrupted(&mut self) {
        let ms = self.min_urb_TxObsS().clone();
        let seq_reasonable = self.seq >= ms && self.seq <= ms + self.urb_buffer_unit_size() * self.node_ids.len() as i32;
        if !seq_reasonable {
            if SETTINGS.print_client_operations() {
                printlnu(format!("Seq not resonable! seq: {}, ms: {}", self.seq, ms));
            }
            self.uniform_urb_txObsS(self.seq);
            if !SETTINGS.record_evaluation_info() {
                panic!("panic uniform");
            } else {
                printlnu(format!("panic: Seq not resonable! seq: {}, ms: {}", self.seq, ms));
            }
            self.run_result.illegally_triggered_ss = !SETTINGS.is_failing_node();
            self.log(format!("Seq not resonable! seq: {}, ms: {}", self.seq, ms));
            return;
        }

        let mut seqs_should_be_in_buffer: HashSet<Int> = (ms + 1  .. self.seq + 1).collect();
        // printlnu(format!("Expected seqs in buffer: {:?}, seq = {}", seqs_should_be_in_buffer, self.seq));

        let self_id = self.node_id;
        let mut tags_seen = HashSet::new();
        for record in self.buffer.iter() {
            if let Some(urb_tag) = &record.urb_tag {
                if urb_tag.id == self_id {
                    seqs_should_be_in_buffer.remove(&urb_tag.seq);
                    tags_seen.insert(&urb_tag.seq);
                }
            }
        }
        let all_seqs_in_process_are_present = seqs_should_be_in_buffer.is_empty();

        if !all_seqs_in_process_are_present {
            if SETTINGS.print_client_operations() {
                printlnu(format!("Not all urb seqs are present. missing: {:?} ms: {}", seqs_should_be_in_buffer, ms + 1));
            }
            if !SETTINGS.record_evaluation_info() {
                printlnu(format!("Not all urb seqs are present. missing: {:?} ms: {} tags seen {:?}", seqs_should_be_in_buffer, ms + 1, tags_seen));
                panic!("Node {} urb panic seq missing Not all urb seqs are present. missing: {:?} ms: {} , current seq {} {:?}", self.node_id, seqs_should_be_in_buffer, ms + 1, self.seq + 1, self.urb_txObsS);
            } else {
                printlnu(format!("Node {} urb panic seq missing Not all urb seqs are present. missing: {:?} ms: {} , current seq {} tx {:?} tags seen {:?}", self.node_id, seqs_should_be_in_buffer, ms + 1, self.seq + 1, self.urb_txObsS, tags_seen));
            }
            self.log(format!("urb seq missing Not all urb seqs are present. missing: {:?} ms: {} , current seq {} {:?}", seqs_should_be_in_buffer, ms + 1, self.seq + 1, self.urb_txObsS));
            self.run_result.illegally_triggered_ss = !SETTINGS.is_failing_node();
            self.uniform_urb_txObsS(self.seq);
        }
    }

    fn uniform_urb_txObsS(&mut self, seq: Int) {
        self.urb_txObsS = vec![seq; self.urb_txObsS.len()];
    }

    fn adjust_urb_rxObsS_and_next_to_deliver_if_corrupted(&mut self) {

        for node_id in self.node_ids.clone() {
            let at_least = self.urb_maxSeq(node_id) - self.urb_buffer_unit_size();
            if self.urb_rxObsS[(node_id - 1) as usize] < at_least {
                if SETTINGS.print_client_operations() {
                    printlnu(format!("urb_rxObsS corrupted. current value: {}, at_least: {}", self.urb_rxObsS[(node_id - 1) as usize], at_least));
                }
                self.run_result.illegally_triggered_ss = !SETTINGS.is_failing_node();
                self.log(format!("urb_rxObsS corrupted. current value: {}, at_least: {}", self.urb_rxObsS[(node_id - 1) as usize], at_least));
            }
            self.urb_rxObsS[(node_id - 1) as usize] = cmp::max(at_least, self.urb_rxObsS[(node_id - 1) as usize]);
            self.next_to_deliver[(node_id - 1) as usize] = cmp::max(self.next_to_deliver[(node_id - 1) as usize], self.urb_rxObsS[(node_id - 1) as usize] + 1);
        }
    }


    fn advance_urb_rxObsS_based_on_obs_record(&mut self) {
        let mut has_obsolete = true;
        let rx_clone = self.urb_rxObsS.clone();
        while has_obsolete {
            has_obsolete = false;
//            printlnu(format!("urb_hasobsolete {:?}", self.urb_rxObsS));
            let mut obsvec = Vec::new();
            for record in self.buffer.iter() {
                if record.urb_tag.is_some() {
                    let record_clone = record.clone();
                    obsvec.push(record_clone);
                }
            }
            for record in obsvec {
                let urb_rxObsS = self.urb_rxObsS.clone();
                if self.urb_obsolete(&record, &urb_rxObsS) {
                    self.urb_rxObsS[record.urb_tag.unwrap().id as usize - 1] += 1;
                    // printlnu(format!("Inside advance_rxObs, rxObs[{}] = {}", record.tag.id, rxObsS[record.tag.id as usize - 1]));
                    has_obsolete = true;
                }
            }
        }
        if rx_clone != self.urb_rxObsS {
            if SETTINGS.print_client_operations() {
                printlnu(format!("(urb) rxObsS updated from: {:?} to {:?}", rx_clone, self.urb_rxObsS));
            }
            self.log(format!("(urb) rxObsS updated from: {:?} to {:?}", rx_clone, self.urb_rxObsS));
        }
    }


    fn remove_unreasonable_urb_records_from_buffer(&mut self) {
        let mut maxSeqs = HashMap::new();
        for id in self.node_ids.clone() {
            if id != self.node_id.clone() {
                maxSeqs.insert(id, self.urb_maxSeq(id));
            }
        }

        let urb_rxObsS = &self.urb_rxObsS.clone();
        // printlnu(format!("Before cleaning, buffer size: {}", buffer.len()));
        let node_id = self.node_id.clone();
        let node_ids = self.node_ids.clone();
        let min_urb_TxObsS = self.min_urb_TxObsS().clone();
        let urb_buffer_unit_size = self.urb_buffer_unit_size();
//        printlnu(format!("before cleaning, buffer size: {}", self.buffer.len()));

        //self.buffer.retain(|record| {
//            return true;
        for record in self.buffer.iter_mut() {
            if let Some(urb_tag) = &record.urb_tag {
                let id = urb_tag.id;
                let seq = urb_tag.seq;
                let record_ok;

                if id == node_id {
                    record_ok = min_urb_TxObsS < seq;
                } else {
                    record_ok = node_ids.contains(&id) &&
                        urb_rxObsS[id as usize - 1] < seq &&
                        maxSeqs.get(&id).unwrap() - urb_buffer_unit_size <= seq;
                }

                record.urb_needed = record_ok
            }
        }
        let mut logvec = VecDeque::new();
        self.buffer.retain(| r|{
            if !r.urb_needed && !r.meta.is_some() {
                if SETTINGS.print_client_operations() {
                    printlnu(format!("(urb) Removing record, min_tx: {} record: {:?}", min_urb_TxObsS, r));
                }
                logvec.push_back(format!("(urb) Removing min_tx: {} record: {:?}", min_urb_TxObsS, r));
            }
            r.urb_needed || ( r.meta.is_some())
        });

        while let Some(msg) = logvec.pop_front() {
            self.log(msg);
        }

    }

    fn handle_records_in_buffer(&mut self, should_retransmit: bool) {
        let trusted = self.trusted();
        //let mut buffer = &mut self.buffer;
//        if trusted.len() != self.node_ids.len() {
////            printlnu(format!("Trusted: {:?}", trusted));
////            panic!("Some nodes are not trusted.");
//        }
        let mut send_vec = VecDeque::new();
        let mut deliver_vec = VecDeque::new();
        let hb = &self.get_hb().clone();
        let mut saved_k = HashMap::new();
        for id in self.node_ids.clone() {
            saved_k.insert(id, self.saved(id));
        }

        let urb_txObsS = &self.urb_txObsS;
        let urb_rxObsS = &self.urb_rxObsS;

        let scd_txObsS = &self.scd_txObsS;
        let scd_rxObsS = &self.scd_rxObsS;

        let mut scd_maxSeqs = HashMap::new();
        for id in self.node_ids.clone() {
            scd_maxSeqs.insert(id, self.scd_maxSeq(id));
        }

        let mut urb_maxSeqs = HashMap::new();
        for id in self.node_ids.clone() {
            urb_maxSeqs.insert(id, self.urb_maxSeq(id));
        }
        let len = self.buffer.len();

        let buf_size = self.buffer.len() as i32;
        let low_size = cmp::max((self.urb_buffer_unit_size() as f32 * 0.3) as i32, 0);
        let high_size = cmp::max((self.urb_buffer_unit_size() as f32 * 0.7) as i32, 1);

        if buf_size >= high_size && self.throughput_instant.is_none() && SETTINGS.variant() == Variant::URB {
            self.throughput_instant = Some(Instant::now());
            self.throughput_msgs = Some(Vec::new());
        }
        for mut record in self.buffer.iter_mut() {
            if record.urb_tag.is_some() {
                let urb_tag = record.urb_tag.as_ref().unwrap();

//                let ack_by_trusted = MergedNode::is_subset(&trusted, &record.recBy);
                let ack_by_majority_trusted = MergedNode::urb_is_ack_by_majority(&trusted, &record.recBy);
                if ack_by_majority_trusted && !record.delivered && urb_tag.seq == self.next_to_deliver[urb_tag.id as usize - 1] {
                    let msg = record.msg.clone();
                    //self.urbDeliver(&tag, msg.unwrap());
                    if SETTINGS.print_client_operations(){
                        printlnu(format!("urbDelivering: {:?}, sn: {}, scdRxObsS: {:?}, scdTxObsS: {:?}, buffelen: {}", record.urb_tag, self.sn, self.scd_rxObsS, self.scd_txObsS, len));
                    }
                    if self.throughput_msgs.is_some() && SETTINGS.variant() == Variant::URB {
                        self.throughput_msgs.as_mut().unwrap().push(urb_tag.clone());
                    }
                    record.delivered = true;
                    deliver_vec.push_back((urb_tag.clone(), msg.unwrap().clone(), record.clone()));
                    self.next_to_deliver[urb_tag.id as usize - 1] += 1;
                } else {
                    // printlnu(format!("record being handled but not delivered: {:?}", record));
                }

                // printlnu(format!("record being handled : {:?}", record));
                let recBy = &record.recBy;
                let recBy_trusted = &record.recBy_trusted;
                let prevHB = &mut record.prevHB;
                let urb_txObsS = &self.urb_txObsS;
//                let mut bitmap = BitVec::from_elem(self.node_ids.len(), false);
                for node_id in self.node_ids.clone() {
                    if (!recBy.get(node_id as usize - 1).unwrap() || !recBy_trusted.get(node_id as usize - 1).unwrap() ||
                        (urb_tag.id == self.node_id && urb_tag.seq == urb_txObsS[node_id as usize - 1] + 1) ||
                        should_retransmit ) &&
                        prevHB[node_id as usize - 1] < hb.clone()[node_id as usize - 1] &&
                        (urb_tag.id == self.node_id || (urb_tag.id != self.node_id && !trusted.get(node_id as usize - 1).unwrap())) {

//                        bitmap.or(recBy);

                        let urb_maxSeq = urb_maxSeqs.get(&node_id).unwrap();
                        let urb_rxObsS_for_id = urb_rxObsS[node_id as usize - 1];
                        let urb_txObsS_for_id = urb_txObsS[node_id as usize - 1];
                        let scd_rxObsS_for_id = scd_rxObsS[node_id as usize - 1];
                        let scd_txObsS_for_id = scd_txObsS[node_id as usize - 1];

                        let scd_maxSeq = scd_maxSeqs.get(&node_id).unwrap();

                        let saved_for_k = saved_k.get(&node_id).unwrap();

                        let mut scd_rxSpace_for_id;
                        if saved_for_k.is_empty() {
                            scd_rxSpace_for_id = None;
                        } else {
                            let mut min = Int::max_value();
                            for elem in saved_for_k {
                                if *elem < min {
                                    min = *elem;
                                }
                            }
                            scd_rxSpace_for_id = Some(min);
                        }
                        let scd_txSpace_for_id = self.scd_txSpace[node_id as usize - 1];

                        let gossip = CombinedGossip {
                            urb_gossip: GOSSIP {
                                sender: self.node_id,
                                urb_maxSeq: *urb_maxSeq,
                                urb_rxObsS: urb_rxObsS_for_id,
                                urb_txObsS: urb_txObsS_for_id,
                            },
                            scd_gossip: SCDGOSSIP {
                                sender: self.node_id,
                                scd_maxSeq: *scd_maxSeq,
                                scd_rxObsS: scd_rxObsS_for_id,
                                scd_txObsS: scd_txObsS_for_id,

                                scd_rxSpace: scd_rxSpace_for_id,
                                scd_txSpace: scd_txSpace_for_id,
                            },
                        };
                        let msg = MSG { sender: self.node_id, msg: Cow::Borrowed(&record.msg), tag: urb_tag.clone(), recv_by: record.recBy.to_bytes(), recv_by_trusted: record.recBy_trusted.to_bytes(), gossip };
                        send_vec.push_back((serde_json::to_string(&msg).expect(""), node_id.clone()));
                        // printlnu(format!("Sending buffer record msg to {}: {:?}", *node_id, &tag));
                        prevHB[node_id as usize -1] = hb.clone()[node_id as usize - 1];
                    }
                }
            }
        }

        if buf_size <= low_size && self.throughput_instant.is_some() && SETTINGS.variant() == Variant::URB {
            if self.run_result.throughputs.is_none() {
                self.run_result.throughputs = Some(Vec::new());
            }
            let msgs = self.throughput_msgs.as_ref().unwrap().len() as f64;
            let time = self.throughput_instant.as_ref().unwrap().elapsed().as_micros() as f64;
            self.run_result.throughputs.as_mut().unwrap().push((msgs / time) * 1000000.0);
            self.throughput_instant = None;
            self.throughput_msgs = None;
        }

        while let Some((tag, msg, record)) = deliver_vec.pop_front() {
            self.log(format!("urbDelivering: {:?}, sn: {}, scdRxObsS: {:?}, scdTxObsS: {:?}", record, self.sn, self.scd_rxObsS, self.scd_txObsS));
            if self.victory_round {
                self.victory_round(&tag, &record);
            }
            self.urb_deliver(&tag, msg);
            if let Some(instant) = record.creation_instant {
                self.run_result.msg_latencies.as_mut().unwrap().push(instant.elapsed().as_micros());
            }
        }

        while let Some((json_msg, node_id)) = send_vec.pop_front() {
            self.send_json_message_to(&json_msg, node_id);
            //self.gossip_sent[node_id as usize - 1] = true;
        }
    }

    fn victory_round(&mut self, tag: &Tag, record: &BufferRecord<String>) {
        let trusted = self.trusted();
        if !MergedNode::is_subset(&trusted, &record.recBy) {
            for node_id in self.node_ids.clone() {
                let urb_maxSeq = self.urb_maxSeq(node_id);
                let urb_rxObsS_for_id = self.urb_rxObsS[node_id as usize - 1];
                let urb_txObsS_for_id = self.urb_txObsS[node_id as usize - 1];

                let gossip = match SETTINGS.variant() {
                    Variant::URB => {
                        CombinedGossip {
                            urb_gossip: GOSSIP {
                                sender: self.node_id,
                                urb_maxSeq: urb_maxSeq,
                                urb_rxObsS: urb_rxObsS_for_id,
                                urb_txObsS: urb_txObsS_for_id,
                            },
                            scd_gossip: SCDGOSSIP {
                                sender: self.node_id,
                                scd_maxSeq: 0,
                                scd_rxObsS: 0,
                                scd_txObsS: 0,

                                scd_rxSpace: Some(0),
                                scd_txSpace: Some(0),
                            },
                        }
                    },
                    _ => {
                        let scd_rxObsS_for_id = self.scd_rxObsS[node_id as usize - 1];
                        let scd_txObsS_for_id = self.scd_txObsS[node_id as usize - 1];

                        let scd_maxSeq = self.scd_maxSeq(node_id);

                        let saved_for_k = self.saved(node_id);

                        let mut scd_rxSpace_for_id;
                        if saved_for_k.is_empty() {
                            scd_rxSpace_for_id = None;
                        } else {
                            let mut min = Int::max_value();
                            for elem in saved_for_k {
                                if elem < min {
                                    min = elem;
                                }
                            }
                            scd_rxSpace_for_id = Some(min);
                        }
                        let scd_txSpace_for_id = self.scd_txSpace[node_id as usize - 1];

                        CombinedGossip {
                            urb_gossip: GOSSIP {
                                sender: self.node_id,
                                urb_maxSeq: urb_maxSeq,
                                urb_rxObsS: urb_rxObsS_for_id,
                                urb_txObsS: urb_txObsS_for_id,
                            },
                            scd_gossip: SCDGOSSIP {
                                sender: self.node_id,
                                scd_maxSeq: scd_maxSeq,
                                scd_rxObsS: scd_rxObsS_for_id,
                                scd_txObsS: scd_txObsS_for_id,

                                scd_rxSpace: scd_rxSpace_for_id,
                                scd_txSpace: scd_txSpace_for_id,
                            },
                        }
                    }
                };

                let msg = MSG {
                    sender: self.node_id,
                    msg: Cow::Borrowed(&record.msg),
                    tag: tag.clone(),
                    recv_by: record.recBy.to_bytes(),
                    recv_by_trusted: record.recBy_trusted.to_bytes(),
                    gossip: gossip
                };
                let json_msg = serde_json::to_string(&msg).unwrap();
                self.send_json_message_to(&json_msg, node_id);
            }
        }
    }

    fn gossip(&mut self) {
        let urb_rxObsS = &self.urb_rxObsS;
        let urb_txObsS = &self.urb_txObsS;

        for node_id in self.node_ids.clone() {
            if node_id != self.node_id
                && !self.gossip_sent[node_id as usize - 1] {
                let urb_maxSeq = self.urb_maxSeq(node_id);
                let urb_rxObsS_for_id = urb_rxObsS[node_id as usize - 1];
                let urb_txObsS_for_id = urb_txObsS[node_id as usize - 1];
                let gossip_msg = GOSSIP { sender: self.node_id, urb_maxSeq: urb_maxSeq, urb_rxObsS: urb_rxObsS_for_id, urb_txObsS: urb_txObsS_for_id };
    //            printlnu(format!("urb_sending gossip {:?}", gossip_msg));
                let json_msg = self.jsonify_message(&gossip_msg);
                self.send_json_message_to(&json_msg, node_id);
            }
        }
        let urb_maxSeq = self.urb_maxSeq(self.node_id);
        let urb_rxObsS_for_id = urb_rxObsS[self.node_id as usize - 1];
        let urb_txObsS_for_id = urb_txObsS[self.node_id as usize - 1];
        let gossip_msg = GOSSIP { sender: self.node_id, urb_maxSeq: urb_maxSeq, urb_rxObsS: urb_rxObsS_for_id, urb_txObsS: urb_txObsS_for_id };
        self.GOSSIP_received(gossip_msg);
    }

    fn update_gossip(&mut self, gossip: CombinedGossip) {
        // urb
        self.GOSSIP_received(gossip.urb_gossip);
        // scd
        if SETTINGS.variant() != Variant::URB {
            self.SCDGOSSIP_received(gossip.scd_gossip);
        }
    }

    //
    // Message reception triggered events.
    //

    fn MSG_received(&mut self, msg: MSG<String>) {
        // printlnu(format!("Sending ack to {} about tag: {:?} ", msg.sender, msg.tag));
        let mut recv_by_bitvec = BitVec::from_bytes(&msg.recv_by);
        recv_by_bitvec.truncate(self.node_ids.len());
        let mut recv_by_trusted_bitvec = BitVec::from_bytes(&msg.recv_by_trusted);
        recv_by_trusted_bitvec.truncate(self.node_ids.len());
        self.update_gossip(msg.gossip.clone());
        let tag_clone = msg.tag.clone();
        let update = self.update(msg.msg.into_owned(), msg.tag, msg.sender, Some(recv_by_bitvec), Some(recv_by_trusted_bitvec));
        // printlnu(format!("Current buffer: {:?}", self.buffer.lock().unwrap()));
        let mut ack_recv = BitVec::from_elem(self.node_ids.len(), false);
        let index = self.get_index_by_urb_tag(&tag_clone);
        if update {
            if index.is_none() {
                ack_recv = BitVec::from_elem(self.node_ids.len(), true);
            }
        } else if index.is_some() {
            if let Some(record) = self.buffer.get(index.unwrap()) {
                ack_recv.or(&record.recBy);
            }
        }
        let ack = MSGAck {sender: self.node_id, tag: tag_clone, recv_by: ack_recv.to_bytes() };
        // self.buffer_updated.notify_one();
        let json_ack = self.jsonify_message(&ack);
        self.send_json_message_to(&json_ack, msg.sender);

    }

    fn MSGAck_received(&mut self, msg: MSGAck) {
//         printlnu(format!("MSGAck_received: {:?}", &msg));
        let mut recv_bitmap = BitVec::from_bytes(&msg.recv_by);
        recv_bitmap.truncate(self.node_ids.len());
        self.update(None, msg.tag, msg.sender, Some(recv_bitmap), None);
        // self.buffer_updated.notify_one();
    }

    fn GOSSIP_received(&mut self, msg: GOSSIP) {
        let copy_seq = &self.seq.clone();
        self.seq = cmp::max(self.seq, msg.urb_maxSeq);

        let copy_tx = self.urb_txObsS.clone();
        let copy_rx = self.urb_rxObsS.clone();

        if self.seq != *copy_seq {
            if SETTINGS.print_client_operations() {
                printlnu(format!("updated seq in GOSSIP before: {} after {}", copy_seq, self.seq));
            }
        }


        let mut urb_rxObsS = &mut self.urb_rxObsS;
        if urb_rxObsS[(msg.sender - 1) as usize] < msg.urb_txObsS {
            // printlnu(format!("rxObsS changed because GOSSIP. current value: {}, gossip: {}", rxObsS[(msg.sender - 1) as usize], msg.txObsS));
        }

        urb_rxObsS[msg.sender as usize - 1] = cmp::max(urb_rxObsS[msg.sender as usize - 1], msg.urb_txObsS);


        let mut urb_txObsS = &mut self.urb_txObsS;
        urb_txObsS[msg.sender as usize - 1] = cmp::max(urb_txObsS[msg.sender as usize - 1], msg.urb_rxObsS);

        let rx_clone = urb_rxObsS.clone();
        let tx_clone = urb_txObsS.clone();
        let rx_changed = copy_rx.cmp(&rx_clone) != Ordering::Equal;
        if  rx_changed {
            if SETTINGS.print_client_operations() {
                printlnu(format!("before: rxObsS for {} = {}", msg.sender - 1, urb_rxObsS[msg.sender as usize - 1]));
            }
        }
        if copy_tx.cmp(&tx_clone) != Ordering::Equal {
            if SETTINGS.print_client_operations() {
                printlnu(format!("after: rxObsS for {} = {}", msg.sender - 1, urb_rxObsS[msg.sender as usize - 1]));
            }
            self.log(format!("(urb) txObsS updated from:{:?} to:{:?}", copy_tx, tx_clone));
        }
        if rx_changed {
            self.log(format!("(urb) rxObsS updated from:{:?} to:{:?}", copy_rx, rx_clone));
        }

    }

    pub(crate) fn handle_received_msgs(&mut self) {
        let mut msg_ack_vector = VecDeque::new();
        let mut msg_vector = VecDeque::new();
        let mut forward_vector = VecDeque::new();
        for (msg_type, rxs) in self.msgs_buffer_rxs.as_mut().unwrap().iter_mut() {
            match msg_type {
                MessageType::MSG | MessageType::MSGAck | MessageType::FORWARD => {
                    for (node_id, rx) in rxs {
                        // Handle at most 100 message for each channel to avoid starving.
                        for counter in 0..10000 {
                            match rx.try_recv() {
                                Ok(json) => {
                                    if json_is_MSG_message(&json) {
                                        if let Ok(MSG_message) = serde_json::from_str(&json) {
                                            msg_vector.push_back(MSG_message);
                                        }
                                        continue;
                                    }
                                    if json_is_MSGAck_message(&json) {
                                        if let Ok(MSGAck_message) = serde_json::from_str(&json) {
                                            msg_ack_vector.push_back(MSGAck_message);
                                        }
                                        continue;
                                    }
                                    if json_is_FORWARD_message(&json) {
                                        if let Ok(Forward_message) = serde_json::from_str(&json) {
                                            forward_vector.push_back(Forward_message);
                                        }
                                        continue;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    }

                },
                _ => {}
            }
        }
        while let Some(msg) = msg_vector.pop_front() {
            self.MSG_received(msg);
        }
        while let Some(msg) = msg_ack_vector.pop_front() {
            self.MSGAck_received(msg);
        }
        while let Some(msg) = forward_vector.pop_front() {
            self.SCD_forward_recieved(msg);
        }
    }

    pub(crate) fn handle_gossip_messages(&mut self) {
        let mut scd_gossip = VecDeque::new();
        let mut gossip = VecDeque::new();
        for (msg_type, rxs) in self.msgs_buffer_rxs.as_mut().unwrap().iter_mut() {
            match msg_type {
                MessageType::GOSSIP | MessageType::SCDGOSSIP => {
                    for (node_id, rx) in rxs {
                        // There is only one slot for GOSSIP messages.
                        match rx.try_recv() {
                            Ok(json) => {
                                if json_is_GOSSIP_message(&json) {
                                    if let Ok(GOSSIP_message) = serde_json::from_str(&json) {
//                                        printlnu(format!("urb_gossip recv {:?}", GOSSIP_message));
                                        gossip.push_back(GOSSIP_message);
                                    }
                                    continue;
                                }
                                if json_is_SCDGOSSIP_message(&json) {
                                    if let Ok(GOSSIP_message) = serde_json::from_str(&json) {
                                        scd_gossip.push_back(GOSSIP_message);
                                    }
                                }
                                continue;
                            },
                            Err(_) => continue,
                        }
                    }
                },
                _ => {}
            }
        }
        while let Some(msg) =  gossip.pop_front() {
            self.GOSSIP_received(msg);
        }

        while let Some(msg) =  scd_gossip.pop_front() {
            self.SCDGOSSIP_received(msg);
        }
    }

    pub fn urb_has_terminated(&self, urb_tag: Tag) -> bool {
        for record in self.buffer.iter() {
            if record.urb_tag.is_some() {
                let tag = record.urb_tag.clone().unwrap();
                if tag == urb_tag {
                    if !record.delivered {
                        return false;
                    }
                    if let Some(meta) = MergedNode::parse_meta(&record.meta) {
                        if !meta.delivered {
                            //return false;
                        }
                    }
                }
            }
        }

        true
    }
}