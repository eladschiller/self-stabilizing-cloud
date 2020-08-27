use super::mergednode::MergedNode;
use commons::types::{Tag, Int};
use crate::scd::messages::{SCDMETA, FORWARD};
use crate::settings::SETTINGS;
use std::borrow::Borrow;
use std::iter::FromIterator;
use rand::{thread_rng, Rng};
use crate::terminal_output::printlnu;
use std::time::SystemTime;
use crate::scd::types::VectorClock;
use bit_vec::BitVec;
use core::cmp;
use std::collections::HashMap;
use commons::variant::Variant;

impl MergedNode {
    pub fn get_index_by_urb_tag(&mut self, urb_tag: &Tag) -> Option<usize> {
        self.buffer.iter().position(|record|
            record.urb_tag == Some(urb_tag.clone())
        )
    }

    pub fn set_scd_meta(&mut self, index: i32, meta: SCDMETA) {
        if let Some(record) = self.buffer.get_mut(index as usize) {
            let meta_s = serde_json::to_string(&meta);
            record.meta = Some(meta_s.unwrap());
        }
    }

    fn get_scd_tag(&self, tag: &Tag) -> Option<Tag> {
        for record in self.buffer.iter() {
            if let Some(urb_tag) = &record.urb_tag {
                if urb_tag == tag {
                    if let Some(meta) = MergedNode::parse_meta(&record.meta) {
                        return Some(meta.tag);
                    }
                }
            }
        }
        None
    }

    pub fn get_urb_tag(&self, scd_tag: &Tag) -> Option<Tag> {
        for record in self.buffer.iter() {
            if let Some(meta) = MergedNode::parse_meta(&record.meta) {
                if meta.tag.id == scd_tag.id && meta.cl.get(scd_tag.id) == scd_tag.seq && record.urb_tag.is_some() {
                    return Some(record.urb_tag.as_ref().unwrap().clone());
                }
            }
        }
        None
    }

    pub fn get_urb_index(&self, scd_tag: &Tag) -> Option<Int> {
        let mut index = 0;
        for record in self.buffer.iter() {
            if let Some(meta) = MergedNode::parse_meta(&record.meta) {
                if meta.tag.id == scd_tag.id && meta.cl.get(scd_tag.id) == scd_tag.seq {
                    return Some(index)
                }
            }
            index += 1;
        }
        None
    }

    pub fn parse_meta(meta: &Option<String>) -> Option<SCDMETA> {
        if let Some(meta_s) = meta {
            if let Ok(scd_meta) = serde_json::from_str::<SCDMETA>(&meta_s) {
                return Some(scd_meta);
            }
        }
        None
    }

    pub fn parse_forward_msg(msg: &Option<String>) -> Option<FORWARD> {
        if let Ok(parsed_msg) = serde_json::from_str::<FORWARD>(msg.as_ref().unwrap()) {
            return Some(parsed_msg);
        }
        None
    }

    pub fn log(&mut self, message: String) {
        if SETTINGS.record_evaluation_info() {
            return;
            self.run_result.log.push((self.start_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(), format!("[Node {}]: {}", self.node_id, message)));
        }
    }

    pub fn update_seen(&mut self) {
        for i in 0..(self.scd_rxObsS.len()) {
            self.rxObsS_seen.get_mut(((i+1) as i32).borrow()).unwrap().insert(self.scd_rxObsS[i]);
        }
        for i in 0..(self.scd_txObsS.len()) {
            self.txObsS_seen.get_mut(((i+1) as i32).borrow()).unwrap().insert(self.scd_txObsS[i]);
        }
    }

    pub fn seen_values_with_skip(&self) -> String {
        let mut sn = 1;
        let mut skip_seen = false;
        let mut nr_of_skips = 0;
        let mut str = "".to_string();
        let sn_seen = self.sn_seen.clone();
        let mut sn_vec = Vec::from_iter(sn_seen.iter());
        sn_vec.sort();
        for seen_sn in sn_vec {
            if sn.borrow() != seen_sn {
                skip_seen = true;
            }
            sn += 1;
        }
        if skip_seen {
            let mut vec = Vec::from_iter(self.sn_seen.iter());
            vec.sort();
            str = format!("sn_seen: {:?}\n", vec);
        }
        for i in 0..self.scd_rxObsS.len() {
            sn = 0;
            skip_seen = false;
            let mut rx_skips = Vec::new();
            let rxObsS = self.rxObsS_seen.get(((i+1) as i32).borrow() ).unwrap();
            let mut rxObsS_vec = Vec::from_iter(rxObsS.iter());
            rxObsS_vec.sort();
            for seen_rx in rxObsS_vec {
                if sn != *seen_rx {
                    skip_seen = true;
                    rx_skips.push(*seen_rx);
                }
                sn +=1;
            }
            if skip_seen {
                str = format!("{} id: {} rxObsS: {:?}\n", str, i+1, rx_skips);
            }
        }

        for i in 0..self.scd_txObsS.len() {
            sn = 0;
            skip_seen = false;
            let mut rx_skips = Vec::new();
            let txObsS = self.txObsS_seen.get(((i+1) as i32).borrow() ).unwrap();
            let mut txObsS_vec = Vec::from_iter(txObsS.iter());
            txObsS_vec.sort();
            for seen_rx in txObsS_vec {
                if sn != *seen_rx {
                    skip_seen = true;
                    rx_skips.push(*seen_rx);
                }
                sn +=1;
            }
            if skip_seen {
                str = format!("{} id: {} txObsS: {:?}\n", str, i+1, rx_skips);
            }
        }

        str
    }

    pub fn corrupt_variables(&mut self) {
        let mut rng = thread_rng();
        match SETTINGS.variant() {

            Variant::URB => {
                let seq_copy = self.seq.clone();
                while self.seq == seq_copy {
                    self.seq = rng.gen_range(0, 10000);
                }
                let ms = self.min_urb_TxObsS().clone();
                printlnu(format!("Corrupted seq was:{} into: {}, ms: {}", seq_copy, self.seq, ms));
                self.log(format!("Corrupted seq was:{} into: {}, ms: {}", seq_copy, self.seq, ms));
            },
            _ => {
                let sn_copy = self.sn.clone();
                while self.sn == sn_copy {
                    self.sn = rng.gen_range(0, 1000);
                }
                printlnu(format!("Corrupted sn was:{} into: {}", sn_copy, self.sn));
                self.log(format!("Corrupted sn was:{} into: {}", sn_copy, self.sn));
            }
        }
    }

    pub fn duplicate_records(&mut self) {
        let buffer_len = self.buffer.len();
        let mut rng = thread_rng();
        let random_index = rng.gen_range(0,  buffer_len);
        let record = self.buffer[random_index].clone();
        self.buffer.push(record);
    }

    pub fn modify_records(&mut self) {
        let buffer_len = self.buffer.len();
        let mut rng = thread_rng();
        let random_index = rng.gen_range(0,  buffer_len);
        if let Some(record) = self.buffer.get_mut(random_index) {
            record.msg = None;
        }
    }

    pub fn modify_clocks(&mut self) {
        let mut index = 0;
        let mut meta_vec = Vec::new();
        for record in self.buffer.iter() {
            if record.meta.is_some() {
                if let Some(mut scd_meta) = MergedNode::parse_meta(&record.meta) {
                    scd_meta.cl = VectorClock::new(self.node_ids.len(), Int::max_value());
                    meta_vec.push((index, scd_meta.clone()));
                    break;
                }
            }
            index += 1;
        }

        for (index, meta) in meta_vec  {
            self.set_scd_meta(index, meta);
        }
    }

    pub fn is_subset(bitvec1: &BitVec, bitvec2: &BitVec) -> bool {
        let mut bitvec1_clone = bitvec1.clone();

        bitvec1_clone.difference(bitvec2);

        bitvec1_clone.none()
    }

    pub fn urb_is_ack_by_majority(trusted: &BitVec, recv_by: &BitVec) -> bool {
        let mut current_trusted_num = trusted.iter().filter(|x| *x).count();
//        let mut index = 0;
//        let mut count = 0;
//
//        for node_id in recv_by.iter() {
//            if node_id && trusted.get(index).unwrap() {
//                count += 1;
//            }
//            index += 1;
//        }
//
//        count >= current_trusted_num / 2

        let mut trusted_clone = trusted.clone();
        trusted_clone.difference(recv_by);
        trusted_clone.iter().filter(|x| *x).count() < cmp::max(current_trusted_num / 2, 1)
    }

    pub(crate) fn reg_pretty(&mut self) -> String {
        let mut reg_vec = Vec::new();
        for (i, val) in self.reg.iter() {
            reg_vec.push((*i , *val));
        }
        reg_vec.sort_by(|(i,v),(i2,v2)| {
            i.cmp(i2)
        });
        let mut s = "[".to_string();
        for (i, v) in reg_vec {
            s = format!("{} {},", s,v);
        }
        let (sstr,_) = s.split_at(s.len()-1);
        s = format!("{}]", sstr);
        format!("{}",s)
    }

    pub fn link_latency_pretty(&mut self, link_latency: HashMap<Int, f64>) -> String {
        let mut link_vec = Vec::new();
        for (node_id, sec) in link_latency.iter() {
            link_vec.push((*node_id, *sec));
        }

        link_vec.sort_by(|(i,v),(i2,v2)| {
            i.cmp(i2)
        });
        let mut s = "[".to_string();
        for (node_id, sec) in link_vec {
            s.push_str(format!("Node {}: {} sec, ", node_id,sec).as_str());
        }
        let (sstr,_) = s.split_at(s.len()-1);
        s = format!("{}]", sstr);
        format!("{}",s)
    }
}