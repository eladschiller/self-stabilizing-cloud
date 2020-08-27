use super::mergednode::MergedNode;
use commons::types::{Tag, Int, NodeId};
use crate::terminal_output::printlnu;
use std::cmp;
use crate::merge::mergednode::StatusCode;
use crate::scd::types::{VectorClock, Entry};
use crate::scd::messages::{SCDMETA, FORWARD, SCDGOSSIP, json_is_FORWARD_message};
use std::borrow::Cow;
use std::collections::{HashSet, BTreeSet, HashMap, VecDeque};
use crate::urb::types::BufferRecord;
use std::rc::Rc;
use std::cmp::max;
use crate::settings::SETTINGS;
use commons::variant::Variant;
use bit_vec::BitVec;
use std::time::Instant;
use commons::constants::WINDOW_SIZE;

impl MergedNode {
    // SCD
    pub fn scd_loop_iter(&mut self, should_execute_self_stab_statement: bool) {
        if should_execute_self_stab_statement {
            //printlnu(format!("Going to try to remove before check mSp: {:?}", self.scd_msp()));
            //self.remove_unreasonable_scd_records_from_buffer();
            self.scd_clean_buffer_if_corrupted();
            self.reset_scd_txObsS_if_corrupted();
            //self.adjust_scd_rxObsS_if_corrupted();
        }
        self.handle_scd_records(should_execute_self_stab_statement);

        self.try_deliver();

        self.advance_scd_rxObsS_based_on_obs_record();
        self.remove_unreasonable_scd_records_from_buffer();

        if should_execute_self_stab_statement {
            self.scd_gossip();
        }
    }

    pub fn scd_buffer_unit_size(&self) -> Int {
        SETTINGS.window_size()
    }


    pub fn scd_available_space(&mut self) -> bool {
        let saved = self.saved(self.node_id);
        let ms_i = self.scd_ms(self.node_id);
        let r = (saved.len() as Int) < self.scd_buffer_unit_size();
//        printlnu(format!("avail space return {} saved = {:?} buffersize = {} buffer {:?} ms_i {}, ms_k {} ,msp {}, rx space {:?} txspace {:?} scdrx {:?} scdtx {:?}", r, saved, self.scd_buffer_unit_size(), self.buffer, ms_i, ms_k, ms_p, self.scd_rxSpace, self.scd_txSpace, self.scd_rxObsS, self.scd_txObsS));
        r
    }

    pub fn scd_available_space_for(&mut self, msgs: i32) -> bool {
        let saved = self.saved(self.node_id);
        let ms_i = self.scd_ms(self.node_id);
        let r = (saved.len() as Int) + msgs <= self.scd_buffer_unit_size();
        r
    }

    pub fn scd_broadcast(&mut self, msg: String) -> Option<Tag> {
        if SETTINGS.print_client_operations() {
            printlnu(format!("Trying to scdBroadcast, sn: {:?}, rxObsS: {:?}, txObsS: {:?} txSpace {:?}", self.sn, self.scd_rxObsS, self.scd_txObsS, self.scd_txSpace));
        }
        self.log(format!("Trying to scdBroadcast, sn: {:?}, rxObsS: {:?}, txObsS: {:?}", self.sn, self.scd_rxObsS, self.scd_txObsS));

        if !self.scd_available_space() {
            if SETTINGS.variant() == Variant::SCD {
                let _ = self.bcast_status.as_ref().unwrap().send(StatusCode::ErrNoSpace);
                return None;
            }
        }
        let sn = &self.sn;
        let msg_tag = Tag {id: self.node_id, seq: *sn};

        if SETTINGS.print_client_operations() {
        printlnu(format!("scdBroadcasting: {:?}, sn: {:?}, rxObsS: {:?}, txObsS: {:?} txSpace {:?}", msg_tag, self.sn, self.scd_rxObsS, self.scd_txObsS, self.scd_txSpace));
        }
        self.log(format!("scdBroadcasting: {:?}, sn: {:?}, rxObsS: {:?}, txObsS: {:?}", msg_tag, self.sn, self.scd_rxObsS, self.scd_txObsS));

        let r = self.forward(msg, msg_tag.clone(), msg_tag, None);
        if SETTINGS.variant() == Variant::SCD || SETTINGS.variant() == Variant::COUNTER {
            let _ = self.bcast_status.as_ref().unwrap().send(StatusCode::Ok);
        }
        r
    }

    pub fn scd_broadcast_eventually(&mut self, msg: String) -> Option<Tag> {
        if SETTINGS.print_client_operations() {
            printlnu(format!("Trying to scdBroadcast, sn: {:?}, rxObsS: {:?}, txObsS: {:?}", self.sn, self.scd_rxObsS, self.scd_txObsS));
        }
        self.log(format!("Trying to scdBroadcast eventually, sn: {:?}, rxObsS: {:?}, txObsS: {:?}", self.sn, self.scd_rxObsS, self.scd_txObsS));

        if !self.scd_available_space() {
            if SETTINGS.variant() == Variant::SCD {
                let _ = self.bcast_status.as_ref().unwrap().send(StatusCode::ErrNoSpace);
                return None;
            } else {
                let mut iter = 0;
                while !self.scd_available_space() {
                    self.bare_bone_loop_iter(iter % SETTINGS.delta() == 0);
                    iter += 1;
                }
            }
        } else {
            if SETTINGS.variant() == Variant::SCD {
                let _ = self.bcast_status.as_ref().unwrap().send(StatusCode::Ok);
            }
        }
        let sn = &self.sn;
        let msg_tag = Tag {id: self.node_id, seq: *sn};

        if SETTINGS.print_client_operations() {
            printlnu(format!("scdBroadcasting: {:?}, sn: {:?}, rxObsS: {:?}, txObsS: {:?}", msg_tag, self.sn, self.scd_rxObsS, self.scd_txObsS));
        }
        self.log(format!("scdBroadcasting: {:?}, sn: {:?}, rxObsS: {:?}, txObsS: {:?}", msg_tag, self.sn, self.scd_rxObsS, self.scd_txObsS));

        self.forward(msg, msg_tag.clone(), msg_tag, None)
    }

    pub fn scd_deliver(&mut self, msgs: Vec<String>) {

        if SETTINGS.print_client_operations() {
            printlnu(format!("scdDelivered: {:?}, sn: {}, rxObsS: {:?}, txObsS: {:?}", msgs, self.sn, self.scd_rxObsS, self.scd_txObsS));
        }
        self.log(format!("scdDelivered: {:?}, sn: {}, rxObsS: {:?}, txObsS: {:?}", msgs, self.sn, self.scd_rxObsS, self.scd_txObsS));

        match SETTINGS.variant() {
            Variant::URB => {
                panic!("this should not happen")
            },
            Variant::SCD => {},
            Variant::COUNTER => {
                self.counter_received(msgs);
            },
            Variant::SNAPSHOT => {
                self.snapshot_msg_received(msgs);
            }
        }

    }

    pub fn forward(&mut self, msg: String, msg_tag: Tag, forward_tag: Tag, cl: Option<VectorClock>) -> Option<Tag> {
        match self.get_urb_index(&msg_tag) {
            Some(index) => {
                if SETTINGS.print_client_operations() {
                        printlnu(format!("debug: Some clause msg_tag: {:?} cl {:?}", msg_tag, cl));
                }
//                printlnu(format!("index: {} urb_tag {:?} buffer {:?}", index, urb_tag, self.buffer));
                let mut entry = self.buffer.get_mut(index as usize).unwrap();
                if let Some(mut scdMeta) = MergedNode::parse_meta(&entry.meta) {
                    let mut parsed_msg = MergedNode::parse_forward_msg(&entry.msg).unwrap();
                    scdMeta.cl.set(forward_tag.id, forward_tag.seq);
                    if cl.is_some() {
                        let vc = cl.unwrap();
                        for node_id in self.node_ids.clone() {
                            let clock = vc.get(node_id);
                            if clock != Int::max_value() && scdMeta.cl.get(node_id) == Int::max_value() {
                                scdMeta.cl.set(node_id, clock);
                            }
                        }
                    }
//                    self.set_scd_meta(&urb_tag, scdMeta);
                    parsed_msg.cl = scdMeta.cl.clone();
                    let new_parsed_msg = serde_json::to_string(&parsed_msg).unwrap();
                    let meta_s = serde_json::to_string(&scdMeta).unwrap();
                    entry.meta = Some(meta_s);
                    entry.msg = Some(new_parsed_msg);
                }
            }
            None => {
                if SETTINGS.print_client_operations() {
                }
                if msg_tag.seq > self.scd_rxObsS[msg_tag.id as usize - 1] {
                    let mut threshold = VectorClock::new(self.node_ids.len(), Int::max_value());
                    threshold.set(forward_tag.id, forward_tag.seq);
                    threshold.set(self.node_id, self.sn);
                    threshold.set(msg_tag.id, msg_tag.seq);
                    let meta = SCDMETA {
                        tag: msg_tag.clone(),
                        cl: threshold.clone(),
                        delivered: false,
                        txDes: None,
                        transmission_counter: 0
                    };

                    let mut forward_msg = FORWARD {
                        msg: Cow::Borrowed(&msg),
                        msg_tag: msg_tag.clone(),
                        forward_tag: Tag { id: self.node_id, seq: self.sn },
                        cl: threshold
                    };
                    //include to buffer
//                    self.seq += 1;
//                    printlnu(format!("updated seq {}", self.seq));
                    let mut recBy = BitVec::from_elem(self.node_ids.len(), false);
                    let mut recBy_trusted = BitVec::from_elem(self.node_ids.len(), false);
                    recBy.set(self.node_id as usize - 1, true);
                    let number_of_nodes = self.node_ids.len();
                    let buffer_record = BufferRecord {
                        urb_tag: None,
                        msg: Some(self.jsonify_message(&forward_msg)),
                        delivered: false,
                        recBy: recBy,
                        recBy_trusted: recBy_trusted,
                        prevHB: vec![-1; number_of_nodes],
                        urb_needed: true,
                        scd_needed: true,
                        meta: Some(serde_json::to_string(&meta).unwrap()),
                        creation_instant:
                            if msg_tag.id == self.node_id {
                                Some(Instant::now())
                            } else {
                                None
                            }
                    };
                    self.buffer.push(buffer_record.clone());
//                    self.update_with_ready(Some(self.jsonify_message(&forward_msg)), None, self.node_id, false);
//                    self.set_scd_meta(&urb_tag, meta);
//                    printlnu(format!("inserted msg to buffer entry: {:?}", self.buffer.get(&urb_tag).unwrap()));
                    self.log(format!("Creating scd record: {:?}", buffer_record));

                    if SETTINGS.print_client_operations() {
                    printlnu(format!("Creating scd record: {:?}, rx: {:?} tx: {:?}", msg_tag.clone(), self.scd_rxObsS, self.scd_txObsS));
                    }
                    self.sn_seen.insert(self.sn.clone());
                    self.sn += 1;
                    return Some(Tag { id: self.node_id, seq: self.sn - 1 });
                } else {
                    if SETTINGS.print_client_operations() {
                        printlnu(format!("Ignoring msg: {:?}, rx: {:?}, tx: {:?}, rxObsS[f]={}, sF={}, sn={}", msg_tag, self.scd_rxObsS, self.scd_txObsS, self.scd_rxObsS[forward_tag.id as usize - 1], forward_tag.seq, self.sn));
                    }
                    self.log(format!("Ignoring msg: {:?}, rx: {:?}, tx: {:?}", msg_tag, self.scd_rxObsS, self.scd_txObsS));
                    //if self.scd_rxObsS[forward_tag.id  as usize - 1] + 1 == forward_tag.seq {
                    self.scd_rxObsS[forward_tag.id as usize - 1] = cmp::max(self.scd_rxObsS[forward_tag.id as usize - 1], forward_tag.seq);
                    //}
                }
            }
        }
        None
    }

    fn try_deliver(&mut self) {
        let mut to_deliver = Vec::new();
        let mut exclude_to_deliver = Vec::new();

        for record in self.buffer.iter() {
            if record.delivered || record.urb_tag.is_none() {
                if let Some(scd_meta) = MergedNode::parse_meta(&record.meta) {
                    //printlnu(format!("Testing entry: {:?}", scd_entry));
                    if !scd_meta.delivered {
                        let msg = record.msg.as_ref().unwrap().clone();

                        let (scd_tag, cl, delivered) = (scd_meta.tag, scd_meta.cl, scd_meta.delivered);
                        let (cl_clone, delivered_clone) = (cl.clone(), delivered.clone());
                        let urb_clone;
                        if record.urb_tag.is_some() {
                            urb_clone = record.urb_tag.as_ref().unwrap().clone();
                        } else {
                            urb_clone = Tag{ id: 0, seq: 0 };
                        }
                        let scd_entry = Entry {
                            msg,
                            tag: scd_tag,
                            cl,
                            delivered,
                            urb_tag: urb_clone
                        };
                        if self.majority_aware(&cl_clone) && !delivered_clone {
                            to_deliver.push(Rc::new(scd_entry));
                        } else if !delivered {
                            exclude_to_deliver.push(Rc::new(scd_entry));
                        } else {
                            printlnu(format!("Record not included: {:?}", record));
                        }
                    }
                }
            }
        }

        let mut changed = true;
        while changed {
            changed = false;

            let mut cannot_deliver = None;
            'outer: for entry in to_deliver.iter().cloned() {
                for excluded in exclude_to_deliver.iter() {
                    if self.cannot_deliver(&entry.cl, &excluded.cl) {
                        cannot_deliver = Some(entry);
                        break 'outer;
                    }
                }
            }
            if let Some(entry) = cannot_deliver.take() {
                exclude_to_deliver.push(entry.clone());
                to_deliver.retain(|x| {
                    entry != *x
                });
                changed = true;
            }
        }
        let mut index_to_deliver = BTreeSet::new();
        for entry in to_deliver {
            if let Some(urb_index) = self.get_urb_index(&entry.tag) {
               index_to_deliver.insert(urb_index.clone());
            }
        }
        let mut msgs_to_deliver = Vec::new();
        let mut msgs_to_deliver_info = Vec::new();
        let buf_size = self.buffer.len() as i32;
        let low_size = cmp::max((self.scd_buffer_unit_size() as f32 * 0.3) as i32, 0);
        let high_size = cmp::max((self.scd_buffer_unit_size() as f32 * 0.5) as i32, 1);

        if buf_size >= high_size && self.throughput_instant.is_none() {
            self.throughput_instant = Some(Instant::now());
            self.throughput_msgs = Some(Vec::new());
        }

        for index in &index_to_deliver {
//            let index = self.get_index_by_urb_tag(urb_tag).unwrap();
            let mut entry = self.buffer.get_mut(*index as usize).unwrap();
            let mut meta = MergedNode::parse_meta(&entry.meta).unwrap();

            let msg = MergedNode::parse_forward_msg(&entry.msg).unwrap();
            msgs_to_deliver.push(msg.msg.to_string());
            msgs_to_deliver_info.push(format!("{}|scd_tag {:?}", msg.msg, msg.msg_tag));

            match SETTINGS.variant() {
                Variant::URB => { panic!("this should not happen") },
                Variant::SCD | Variant::COUNTER | Variant::SNAPSHOT => {
                    if let Some(bset) = self.run_result.scd_delivered_msgs.get_mut(&msg.msg_tag.id) {
                        bset.insert(msg.msg_tag.clone());
                    } else {
                        let mut bset = BTreeSet::new();
                        bset.insert(msg.msg_tag.clone());
                        self.run_result.scd_delivered_msgs.insert(msg.msg_tag.id, bset);
                    }
                    self.delivered_tags.insert(msg.msg_tag.clone());
                    if let Some(instant) = entry.creation_instant {
                        if SETTINGS.variant() != Variant::SNAPSHOT {
                            self.run_result.msg_latencies.as_mut().unwrap().push(instant.elapsed().as_micros());
                        }
                    }
                },
            }
            if self.throughput_msgs.is_some() {
                self.throughput_msgs.as_mut().unwrap().push(msg.msg_tag.clone());
            }


            meta.delivered = true;
            if meta.txDes.is_none() && meta.tag.id == self.node_id {
                printlnu(format!("WTF delivered but none txDEs: {:?}", meta.clone()));
            }
            self.set_scd_meta(*index, meta);
        }

        if !msgs_to_deliver.is_empty() {
            self.scd_deliver(msgs_to_deliver);
        }

        if buf_size <= low_size && self.throughput_instant.is_some() {
            if self.run_result.throughputs.is_none() {
                self.run_result.throughputs = Some(Vec::new());
            }
            let msgs = self.throughput_msgs.as_ref().unwrap().len() as f64;
            let time = self.throughput_instant.as_ref().unwrap().elapsed().as_micros() as f64;
            self.run_result.throughputs.as_mut().unwrap().push((msgs / time) * 1000000.0);
            self.throughput_instant = None;
            self.throughput_msgs = None;
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

    fn handle_scd_records(&mut self, should_retransmit: bool) {
        let mut broadcast_vector = Vec::new();
        let mut index = 0;
        let trusted = self.trusted();
        for record in self.buffer.iter() {
            if let Some(mut meta) = MergedNode::parse_meta(&record.meta) {
                if meta.txDes.is_none() || self.urb_has_terminated(meta.txDes.clone().unwrap()) {
                    let parsed_msg = MergedNode::parse_forward_msg(&record.msg).unwrap();
                    if meta.txDes.is_some()
                        && self.urb_has_terminated(meta.txDes.clone().unwrap())
                        && meta.transmission_counter >= 2 {
                        if !should_retransmit {
                            index += 1;
                            continue;
                        }
                    }
                    if meta.tag.id == self.node_id || !trusted.get(meta.tag.id as usize - 1).unwrap() {
                        meta.transmission_counter = cmp::min(2, meta.transmission_counter + 1);
                        broadcast_vector.push((meta.tag.seq.clone(), record.msg.clone().unwrap(), meta.clone(), index));
                    } else {
                        let forward_ack_msg = FORWARD {
                            msg: parsed_msg.msg,
                            msg_tag: parsed_msg.msg_tag,
                            forward_tag: parsed_msg.forward_tag,
                            cl: meta.clone().cl,
                        };
                        let json = self.jsonify_message(&forward_ack_msg);
                        self.send_json_message_to(&json, meta.tag.id);
                    }
                }
            }
            index += 1;
        }
//        broadcast_vector.sort_by(|(_, tag1, _), (_, tag2, _)| {
//            tag2.seq.cmp(&tag1.seq)
//        });
        broadcast_vector.sort_by(|(a,_,_,_), (b,_,_,_)| {
            (*b).cmp(a)
        });
        while let Some((_, msg, mut meta, index)) = broadcast_vector.pop() {
//            let index = self.get_index_by_urb_tag(&urb_tag).unwrap();
//            let rec = self.buffer.get(index).unwrap();
            if SETTINGS.print_client_operations() {
                printlnu(format!("(in handle_scd_records) Trying to urbBroadcast: {:?}", msg));
            }

            self.log(format!("(in handle_scd_records) Trying to urbBroadcast: {:?}", msg));
            let d = self.non_blocking_urb_broadcast(msg, index);
            if d.is_none() {
                self.has_seen_bot = true;
            } else {
                meta.txDes = d.clone();
            }
            self.set_scd_meta(index, meta.clone());

        }

    }

    fn scd_gossip(&mut self) {
        // Gossip line 113
        for node_id in self.node_ids.clone() {
            if node_id != self.node_id && !self.gossip_sent[node_id as usize - 1] {
                let scd_maxSeq = self.scd_maxSeq(node_id);
                let saved_k = self.saved(node_id);
                let saved_clone = saved_k.clone();
                let scd_rxObsS_for_id = self.scd_rxObsS[node_id as usize - 1];
                let scd_txObsS_for_id = self.scd_txObsS[node_id as usize - 1];
                let mut scd_rxSpace_for_id;
                if saved_k.is_empty() {
                    scd_rxSpace_for_id = None;
                } else {
                    let mut min = Int::max_value();
                    for elem in saved_k {
                        if elem < min {
                            min = elem;
                        }
                    }
                    scd_rxSpace_for_id = Some(cmp::min(min, self.scd_rxObsS[node_id as usize - 1] + 1));
                }
                self.scd_rxSpace[node_id as usize - 1] = scd_rxSpace_for_id.clone();
                let scd_txSpace_for_id = self.scd_txSpace[node_id as usize - 1];
                let gossip = SCDGOSSIP {
                    sender: self.node_id,
                    scd_maxSeq: scd_maxSeq,
                    scd_rxObsS: scd_rxObsS_for_id,
                    scd_txObsS: scd_txObsS_for_id,
                    scd_rxSpace: scd_rxSpace_for_id,
                    scd_txSpace: scd_txSpace_for_id,
                };
                let json_msg = self.jsonify_message(&gossip);
                self.send_json_to(&json_msg, node_id);
            }
        }

        let rxS = self.saved(self.node_id);
        if rxS.is_empty() {
            self.scd_rxSpace[self.node_id as usize - 1] = None;
        } else {
            let mut min = Int::max_value();
            for elem in rxS {
                if elem < min {
                    min = elem;
                }
            }
            self.scd_rxSpace[self.node_id as usize - 1] = Some(cmp::min(min, self.scd_rxObsS[self.node_id as usize - 1] + 1));
        }
        let self_gossip = SCDGOSSIP {
            sender: self.node_id,
            scd_maxSeq: self.scd_maxSeq(self.node_id),
            scd_rxObsS: self.scd_rxObsS[self.node_id as usize - 1],
            scd_txObsS: self.scd_txObsS[self.node_id as usize - 1],
            scd_rxSpace: self.scd_rxSpace[self.node_id as usize - 1],
            scd_txSpace: self.scd_txSpace[self.node_id as usize - 1],

        };
        self.SCDGOSSIP_received(self_gossip);
    }

    fn scd_clean_buffer_if_corrupted(&mut self) {
        let mut clock_seen = HashSet::new();
        let mut buffer_corrupted_dup = false;
        let mut buffer_corrupted_max= false;
        let mut buffer_corrupted_bound = false;
        let mut faulty_records = Vec::new();
        for record in self.buffer.iter() {
            if let Some(meta) = MergedNode::parse_meta(&record.meta) {
                let sd = meta.tag.id;
                let cl_sd = meta.cl.get(sd);
                if clock_seen.contains(&(sd, cl_sd)) {
                    faulty_records.push(format!(" clock seen for node {}:{} in record: {:?} ", sd.clone(), cl_sd.clone(), record.clone()));
                    buffer_corrupted_dup = true;
                } else {
                    clock_seen.insert((sd, cl_sd));
                }
                if meta.cl.get(self.node_id) == Int::max_value() {
                    faulty_records.push(format!("record has maxvalue: {:?}", record.clone()));
                    buffer_corrupted_max = true;
                }
            }
        }

        for node_id in self.node_ids.clone() {
            let saved  = self.saved(node_id);
            if saved.len() as Int > self.scd_buffer_unit_size() {
                faulty_records.push(format!("node {} saved {:?}", node_id, saved));
                buffer_corrupted_bound = true;
            }
        }

        if buffer_corrupted_max || buffer_corrupted_dup || buffer_corrupted_bound {
            let s = if buffer_corrupted_max {
                format!("cause self value in cl was max: {:?}", faulty_records)
            } else if buffer_corrupted_dup {
                format!("cause duplicate values: {:?} and buffer is not bounded", faulty_records)
            } else if buffer_corrupted_bound {
                format!("cause bound fails {:?}, sn: {}, tx: {:?}, rx: {:?}, buffer: {:?}", faulty_records, self.sn, self.scd_txObsS, self.scd_rxObsS, self.buffer)
            } else {
                format!("")
            };

            self.buffer = Vec::new();
            if !SETTINGS.record_evaluation_info() {
                panic!("Node {} SCD Buffer was corrupted and then emptied, {}", self.node_id, s);
            } else {
                printlnu(format!("SCD Buffer was corrupted and then emptied, {}", s));
            }
            self.run_result.illegally_triggered_ss = !SETTINGS.is_failing_node();
            self.log(format!("SCD Buffer was corrupted and then emptied, {}", s))
        }
    }

    fn reset_scd_txObsS_if_corrupted(&mut self) {
        let ms_i = self.scd_ms(self.node_id);
        let mut temp_sn = 0;

        let sn_reasonable = ms_i < (self.sn); //&& (self.sn - 1) <= (ms + self.scd_buffer_unit_size()*self.node_ids.len() as i32);
        let msp = self.scd_msp();


        // Extra stored sequence numbers
        let low = cmp::max(1, if self.scd_msp().is_none() {
            self.scd_rxObsS[self.node_id as usize - 1]
        } else {
            self.scd_msp().unwrap()
        });
        let high = self.scd_ms(self.node_id);

        let mut extra_sns: HashSet<Int> = (cmp::min(low, high) .. cmp::max(low, high) + 1).collect();


        let mut req_sns : HashSet<Int> = (self.scd_ms(self.node_id) + 1 .. self.sn).collect();
        let req_clone = req_sns.clone();
        let extra_clone = extra_sns.clone();
        let mut all_sn_in_process_are_present = false;
        let mut extra_is_subset = true;
        let mut sn_found = Vec::new();
        let mut extra_found = Vec::new();
        for r in self.buffer.iter() {
            if let Some(meta) = MergedNode::parse_meta(&r.meta) {
                if meta.cl.get(self.node_id) <= ms_i {
                    if !extra_sns.remove(&meta.cl.get(self.node_id)) {
                        extra_is_subset = false;
                        extra_found.push(meta.cl.get(self.node_id));
                    }
                } else {
                    if !req_sns.remove(&meta.cl.get(self.node_id)) {
                        sn_found.push(meta.cl.get(self.node_id));
                    }
                }
            }
        }
        if req_sns.is_empty() && sn_found.is_empty()  {
            all_sn_in_process_are_present = true;
        }
        if !(sn_reasonable && all_sn_in_process_are_present) {
            if SETTINGS.print_client_operations() {
                let self_id = self.node_id;
                let mut str = "sn: ".to_string();
                for record in self.buffer.iter() {
                    if let Some(meta) = MergedNode::parse_meta(&record.meta) {
                        str.push_str(format!(", {}", meta.cl.get(self.node_id)).as_ref());
                    }
                }
                printlnu(str);
                printlnu(format!("Seq not resonable! sn: {}, ms: {}, maxBufferSize: {}", self.sn, ms_i, self.scd_buffer_unit_size()*self.node_ids.len() as i32));
            }
            if !sn_reasonable {
                printlnu(format!("Seq not resonable! sn: {}, ms: {}, maxBufferSize: {}, rx: {:?} tx: {:?}", self.sn, ms_i, self.scd_buffer_unit_size()*self.node_ids.len() as i32, self.scd_rxObsS, self.scd_txObsS));
            }
            if !all_sn_in_process_are_present {
                printlnu(format!("All sns are not present, mSp: {:?}, ms: {}, rx[i]: {} req_sns: {:?}, didnt find: {:?} sn: {}, txObsS: {:?}, rxObsS: {:?}", self.scd_msp(), self.scd_ms(self.node_id), self.scd_rxObsS[self.node_id as usize - 1], req_clone, req_sns, self.sn, self.scd_txObsS, self.scd_rxObsS));
            }
            if !extra_is_subset {
                printlnu(format!("Extra sns were not a subset, looking for: {:?} also found: {:?}, mSp: {:?}, ms: {}, rxObsS: {:?}, txObsS: {:?}, txSpace: {:?}", extra_sns, extra_found, self.scd_msp(), self.scd_ms(self.node_id), self.scd_rxObsS, self.scd_txObsS, self.scd_txSpace));
            }
//            printlnu(format!("bound(i,1): {}", self.bound(self.node_id, 1)));
            if !SETTINGS.record_evaluation_info() {
                panic!("Node {} panic scd uniform {:?}",self.node_id, self.buffer);
            } else {
                printlnu(format!("Node {} panic scd uniform\n buffer: {:?}",self.node_id, self.buffer));
            }
            self.log(format!("Seq not resonable! sn: {}, ms: {}, scdRx: {:?}, scdTx: {:?}, maxBufferSize: {}", self.sn, ms_i, self.scd_rxObsS, self.scd_txObsS, self.scd_buffer_unit_size()*self.node_ids.len() as i32));
            self.log(format!("Node {} panic scd uniform, buffer: {:?}",self.node_id, self.buffer));
            self.run_result.illegally_triggered_ss = !SETTINGS.is_failing_node();
            self.uniform_scd_obsS(self.sn);
            self.update_seen();
        }
    }

    fn uniform_scd_obsS(&mut self, sn: Int) {
        let sn =  max(0, sn-1);
        self.scd_txObsS = vec![sn; self.scd_txObsS.len()];
        self.scd_rxObsS[self.node_id as usize - 1] = sn;
    }

    fn adjust_scd_rxObsS_if_corrupted(&mut self) {
        for node_id in self.node_ids.clone() {
            let at_least = self.scd_maxSeq(node_id) - self.scd_buffer_unit_size() * self.node_ids.len() as i32;
//            printlnu(format!("node _id {} atleast {} scd_maxseq = {} buffer unit size {}", node_id,at_least, self.scd_maxSeq(node_id), self.scd_buffer_unit_size()));
            if self.scd_rxObsS[(node_id - 1) as usize] < at_least  {
                printlnu(format!("scd_rxObsS corrupted. current rxObsS for node {} = {} maxseq = {}, n*bUS = {}, at_least: {} buffer {:?}", node_id, self.scd_rxObsS[(node_id - 1) as usize], self.scd_maxSeq(node_id ), self.scd_buffer_unit_size() * self.node_ids.len() as i32, at_least, self.buffer));

                if !SETTINGS.record_evaluation_info() {
                    panic!("Node {} panic in adjust_scd_rxObsS_if_corrupted", self.node_id);
                } else {
                    printlnu(format!("scd_rxObsS corrupted. current value: {}, at_least: {}", self.scd_rxObsS[(node_id - 1) as usize], at_least));
                }
                self.log(format!("scd_rxObsS corrupted. current value: {}, at_least: {}", self.scd_rxObsS[(node_id - 1) as usize], at_least));
                self.run_result.illegally_triggered_ss = !SETTINGS.is_failing_node();

                self.scd_rxObsS[(node_id - 1) as usize] = cmp::max(at_least, self.scd_rxObsS[(node_id - 1) as usize]);
            }
            self.update_seen();
        }
    }

    fn advance_scd_rxObsS_based_on_obs_record(&mut self) {
        let mut has_obsolete = true;
        while has_obsolete {
            has_obsolete = false;
            let mut obsvec = Vec::new();
            for record in self.buffer.iter() {
                if record.meta.is_some() {
                    let record_clone = record.clone();
                    obsvec.push(record_clone);
                }
            }
            for record in obsvec {
                let mut scd_rxObsS = self.scd_rxObsS.clone();

                for node_id in self.node_ids.clone() {
                    if self.scd_obsolete(&record, node_id, &scd_rxObsS) {
                        self.scd_rxObsS[node_id as usize - 1] += 1;
                        self.update_seen();
                        has_obsolete = true;
                        scd_rxObsS = self.scd_rxObsS.clone();
                    }
                }

            }
        }

    }

    pub fn scd_safe_rm(ms: Vec<i32>, clocks: Vec<Int>) -> bool {
        let mut ret = true;
        for k in 0..clocks.len() {
            if clocks.get(k) > ms.get(k) {
                ret = false;
            }
        }
        ret
    }

    fn remove_unreasonable_scd_records_from_buffer(&mut self) {
        let mut maxSeqs = HashMap::new();
        let mut clock_i = HashSet::new();
        let trusted = self.trusted();

        let mut index = 0;
        for node_id in trusted.iter() {
            if node_id {
                maxSeqs.insert(index + 1, self.scd_maxSeq(index + 1));
            }
            index += 1;
        }

        let node_id = self.node_id.clone();
        let node_ids = self.node_ids.clone();
        let scd_txObsS = self.scd_txObsS.clone();
        let scd_rxObsS = &self.scd_rxObsS.clone();
        let scd_txSpace = &self.scd_txSpace.clone();
        let mut scd_ms = vec![0;self.node_ids.len()];
        let mut index = 0;
        for node_id in trusted.iter() {
            if node_id {
                scd_ms[index] = self.scd_ms((index + 1) as i32);
            }
            index += 1;
        }
        let scd_msp = self.scd_msp();
        let scd_buffer_unit_size = self.scd_buffer_unit_size();
        let sn = self.sn;


        let mut logvec = VecDeque::new();

        for record in self.buffer.iter_mut() {
            if let Some(meta) = MergedNode::parse_meta(&record.meta) {
                let record_ok;

                record_ok = {
                    let mut check = false;
                    if scd_msp.is_some() && meta.tag.id != self.node_id
                        || scd_msp.is_none() && meta.tag.id == self.node_id {
                        if meta.cl.get(self.node_id) != Int::max_value() {
                            clock_i.insert(meta.cl.get(self.node_id));
                        }
                    }
                    let mut index = 0;
                    for node_trusted in trusted.iter() {
                        if node_trusted {
                            if self.node_id != meta.tag.id {
                                if scd_ms[index] < meta.cl.get(index as i32 + 1) {
                                    check = true;
                                }
                            } else {
                                if scd_msp.is_some() && scd_msp.unwrap() <= meta.cl.get(self.node_id)
                                    || scd_msp.is_none() && scd_ms[index] < meta.cl.get(index as i32 + 1) {
                                    check = true;
                                }
                            }
                        }
                        index += 1;
                    }
                    check
                };

                record.scd_needed = record_ok;
            }
        }

        for record in self.buffer.iter_mut() {
            if let Some(meta) = MergedNode::parse_meta(&record.meta) {
                let record_ok;
                let mut min_ci =  if clock_i.is_empty() { break; } else { Int::max_value() };
                for c_i in clock_i.clone() {
                    min_ci = cmp::min(min_ci, c_i);
                }


                record_ok = {
                    let mut check = true;
                        if min_ci == meta.cl.get(self.node_id) {
                            let mut all = true;
                            let mut index = 0;
                            for node_trusted in trusted.iter() {
                                if node_trusted {
                                    if scd_ms[index] >= meta.cl.get(index as i32 + 1) {

                                    } else {
                                        all = false;
                                    }
                                }
                                index +=1;
                            }
                            if all {
                                check = false;
                                clock_i.remove(&min_ci);
                            }
                        }
                    check
                };
                if record.scd_needed {
                    record.scd_needed = record_ok;
                }
            }

        }


        self.buffer.retain(|r| {
            if let Some(meta) = MergedNode::parse_meta(&r.meta.clone()) {
                if !r.scd_needed && !r.urb_needed {
                        if SETTINGS.print_client_operations() {
                            printlnu(format!(" (scd) Removing cl[i] = {}: {:?} rx {:?} tx {:?} sn: {}, {} < {} tx_space {:?}, msp: {:?} ", meta.cl.get(node_id), r, scd_rxObsS, scd_txObsS, sn, scd_ms[node_id.clone() as usize - 1], meta.cl.get(node_id), scd_txSpace, scd_msp));
                        }
                    logvec.push_back(format!(" (scd) Removing: {:?} rx {:?} tx {:?} sn {}, scd_ms: {:?}", r, scd_rxObsS, scd_txObsS, sn, scd_ms));
                }
            }
            if r.urb_tag.is_some() {
                r.scd_needed || r.urb_needed
            } else {
                r.scd_needed
            }
        });
        while let Some(msg) = logvec.pop_front() {
            self.log(msg);
        }
    }


    fn is_needed_record(&self, seq: Int, node_id: NodeId) -> bool{
        self.scd_rxObsS[node_id as usize - 1] < seq && self.scd_maxSeq(node_id) - self.scd_buffer_unit_size() <= seq
    }

    // SCD macro

    fn scd_obsolete(&mut self, record: &BufferRecord<String>, forwarder: NodeId, scd_rxObsS: &Vec<Int>) -> bool {
        let scd_meta = MergedNode::parse_meta(&record.meta).unwrap();
        let trusted = self.trusted();

        let obs = ((scd_meta.txDes.is_some() && (scd_meta.tag.id == self.node_id || !trusted.get(scd_meta.tag.id as usize - 1).unwrap())) || (scd_meta.tag.id != self.node_id))
            && scd_rxObsS[forwarder as usize - 1] + 1 == scd_meta.cl.get(forwarder)
            && scd_meta.delivered;

        if scd_meta.delivered && !obs {
            //printlnu(format!("message delivered but not obsolete: {:?}, rxObs: {:?} for forwarder: {}", scd_meta, scd_rxObsS, forwarder));
        }

        obs
    }

    pub(crate) fn scd_maxSeq(&self, node_id: NodeId) -> Int {
        let mut max_seq = 0;
        for record in self.buffer.iter() {
            if let Some(meta) = MergedNode::parse_meta(&record.meta) {
                let mut forwarder_max_seq = 0;

                if meta.cl.get(node_id) < Int::max_value() {
                    forwarder_max_seq = meta.cl.get(node_id);
                }
                max_seq = cmp::max(max_seq, forwarder_max_seq);
            }
        }
        cmp::max(max_seq, self.scd_rxObsS[node_id as usize - 1])
    }

    pub fn scd_msp(&mut self) -> Option<Int> {
        let trusted = self.trusted();
        let mut min_s = Int::max_value();
        let scd_txSpace = &self.scd_txSpace;
        let mut some_seen = false;
        let mut index = 0;
        for node_id in trusted.iter() {
            if node_id && index + 1 != self.node_id {
                let txSpace = scd_txSpace[index as usize];
                if txSpace.is_some() {
                    min_s = cmp::min(min_s, txSpace.unwrap());
                    some_seen = true;
                }
            }
            index += 1;
        }
        if !some_seen {
            return None;
        }
        Some(min_s)
    }

    fn min_scd_TxObsS(&mut self) -> Int {
        let trusted = self.trusted();
        let mut min_s = Int::max_value();
        let scd_txObsS = &self.scd_txObsS;

        let mut index = 0;
        for node_id in trusted.iter() {
            if node_id {
                min_s = cmp::min(min_s, scd_txObsS[index])
            }
            index += 1;
        }
        min_s
    }

    fn min_scd_RxObsS(&mut self) -> Int {
        let trusted = self.trusted();
        let mut min_s = Int::max_value();
        let scd_rxObsS = &self.scd_rxObsS;

        if SETTINGS.print_client_operations() {
            printlnu(format!("scd_rxObsS {:?}", scd_rxObsS));
        }

        let mut index = 0;
        for node_id in trusted {
            if node_id {
                min_s = cmp::min(min_s, scd_rxObsS[index])
            }
            index += 1;
        }
        min_s
    }

    pub fn scd_ms(&mut self, node_id: Int) -> Int {
        if node_id == self.node_id {
            return self.min_scd_TxObsS()
        } else {
            if self.trusted().get(node_id as usize - 1).unwrap() {
                return self.scd_rxObsS[node_id as usize - 1];//self.min_scd_RxObsS();
            }
        }
        0
    }

    // SCD msg reception

    pub fn scd_msg_received(&mut self, msg: String) {
        if json_is_FORWARD_message(&msg) {
            if let Ok(forward_msg) = serde_json::from_str(&msg) {
                if SETTINGS.print_client_operations(){
                    printlnu(format!("FORWARD recv: {:?}", forward_msg));
                }
                self.SCD_forward_recieved(forward_msg);
            }
        }
    }

    pub fn SCD_forward_recieved(&mut self, msg: FORWARD) {
        let m = msg.msg;
        let msg_tag = msg.msg_tag;
        let forward_tag = msg.forward_tag;

        let _ = self.forward(m.into_owned(), msg_tag.clone(), forward_tag.clone(), Some(msg.cl));
    }

    pub fn SCDGOSSIP_received(&mut self, gossip: SCDGOSSIP) {
        let scd_maxSeq = gossip.scd_maxSeq;
        let txS_clone = self.scd_txSpace.clone();
        let mut scd_rxObsS = &mut self.scd_rxObsS;
        let mut scd_txObsS = &mut self.scd_txObsS;
        let mut scd_rxSpace = &mut self.scd_rxSpace;
        let mut scd_txSpace = &mut self.scd_txSpace;
        if SETTINGS.print_client_operations() {
            if self.sn < gossip.scd_maxSeq {
                printlnu(format!("updating sn from {} to {}", self.sn, gossip.scd_maxSeq));
            }
            if scd_rxObsS[gossip.sender as usize - 1] < gossip.scd_txObsS {
                printlnu(format!("Updating rx[{}] from  {} to {}", gossip.sender, scd_rxObsS[gossip.sender as usize - 1], gossip.scd_txObsS));
            }
        }

        self.sn = cmp::max(self.sn, gossip.scd_maxSeq);

        scd_rxObsS[gossip.sender as usize - 1] = cmp::max(scd_rxObsS[gossip.sender as usize - 1], gossip.scd_txObsS);
        scd_txObsS[gossip.sender as usize - 1] = cmp::max(scd_txObsS[gossip.sender as usize - 1], gossip.scd_rxObsS);

        if gossip.scd_rxSpace.is_some() && scd_txSpace[gossip.sender as usize - 1].is_some() {
            scd_txSpace[gossip.sender as usize - 1] = Some(cmp::max(scd_txSpace[gossip.sender as usize - 1].unwrap(), gossip.scd_rxSpace.unwrap()));

        } else {
            if scd_txSpace[gossip.sender as usize - 1].is_none() || gossip.scd_rxObsS + 1 > scd_txSpace[gossip.sender as usize - 1].unwrap() {
                scd_txSpace[gossip.sender as usize - 1] = gossip.scd_rxSpace;
            }
        }

        if gossip.scd_txSpace.is_some() && scd_rxSpace[gossip.sender as usize - 1].is_some() {
            scd_rxSpace[gossip.sender as usize - 1] = Some(cmp::max(gossip.scd_txSpace.unwrap(), scd_rxSpace[gossip.sender as usize - 1].unwrap()));
        } else {
            if scd_rxSpace[gossip.sender as usize - 1].is_none() || gossip.scd_txObsS + 1 > scd_rxSpace[gossip.sender as usize - 1].unwrap() {
                scd_rxSpace[gossip.sender as usize - 1] = gossip.scd_txSpace;
            }
        }

        self.update_seen();
    }

    //TODO: implement hasTerminated and allHaveTerminated
    pub fn scd_all_have_terminated(&self) -> bool {
        let mut not_delivered_found = false;
        for record in self.buffer.iter() {
            if !record.delivered {
                not_delivered_found = true;
            }
        }
        !not_delivered_found
    }

    pub fn scd_has_terminated(&self, txDes: &Tag) -> bool {
        for record in self.buffer.iter() {
            if record.meta.is_some() {
                if let Some(scd_meta) = MergedNode::parse_meta(&record.meta) {
                    if scd_meta.tag.id == txDes.id && scd_meta.cl.get(txDes.id) == txDes.seq {
//                        printlnu(format!("txDes {:?} record {:?} return {}",txDes, record, scd_meta.delivered));
                        return scd_meta.delivered;
                    }
                }
            }
        }
        true
    }

    pub(crate) fn saved(&mut self, node_id: NodeId) -> HashSet<Int> {
        let mut saved_clock = HashSet::new();

        for record in self.buffer.iter() {
            if record.meta.is_some() {
                if let Some(scd_meta) = MergedNode::parse_meta(&record.meta) {
                    if scd_meta.tag.id == node_id {
                        saved_clock.insert(scd_meta.cl.get(node_id));
                    }
                }
            }
        }
        saved_clock
    }
}