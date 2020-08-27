use super::mergednode::MergedNode;
use crate::scd::algorithm5;
use crate::scd::algorithm5::messages::{SYNC, WRITE};
use commons::types::{Int, NodeId, Tag};
use serde::{Deserialize, Serialize};
use crate::terminal_output::printlnu;
use std::collections::HashMap;
use std::cmp::max;
use crate::settings::SETTINGS;
use crate::merge::mergednode::StatusCode;
use std::time::Instant;


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct Timestamp {
    date: Int,
    proc: Option<Int>,
}

impl Timestamp {
    pub fn new() -> Self {
        Timestamp {
            date: 0,
            proc: None
        }
    }
}

impl MergedNode {
    pub fn snapshot(&mut self) -> HashMap<Int,Int> {
        let now = Instant::now();
        printlnu(format!("-------------    Initiating snapshot read"));
        let message = SYNC { sender_id: self.node_id };

        let json_message = self.jsonify_message(&message);

        let mut txDes = self.scd_broadcast_eventually(json_message.clone());

        if txDes.is_some() {
            self.wait_until_txDex_terminate(txDes.as_ref().unwrap());
        }
        let s = format!("-------------    snapshot returns {}", self.reg_pretty());
        printlnu(s.clone());
        if SETTINGS.print_client_operations() {
        }
        self.log(s);
        let _ = self.bcast_status.as_ref().unwrap().send(StatusCode::ResultReady);
        self.run_result.read_latencies.as_mut().unwrap().push(now.elapsed().as_micros());
        return self.reg.clone()

    }

    pub fn snapshot_write(&mut self, r: Int, v: Int) {
        let now = Instant::now();
        if SETTINGS.print_client_operations() {
            printlnu(format!{"r {} v {}", r, v});
        }
        self.log(format!{"r {} v {}", r, v});
        let message = SYNC { sender_id: self.node_id };

        let json_message = self.jsonify_message(&message);
        let mut txDes = self.scd_broadcast_eventually(json_message.clone());
        while txDes.is_none() {
            txDes = self.scd_broadcast_eventually(json_message.clone());
        }

//        wait(txDes = ⊥ ∨ hasTerminated(txDes));
        if txDes.is_some() {
            self.wait_until_all_terminate();
        } else {
            panic!("txDes is non in snapshot write");
        }

        let mut tsa_date = if let Some(ts) = self.tsa.get_mut(&r) {
            ts.clone()
        } else {
            Timestamp { date: 0, proc: None }
        };

        tsa_date.date += 1;
        tsa_date.proc = Some(self.node_id);

        let message = WRITE {
            r,
            v,
            timestamp: tsa_date
        };

        let json_message = self.jsonify_message(&message);

        let mut txDes = self.scd_broadcast_eventually(json_message.clone());

        while txDes.is_none() {
            txDes = self.scd_broadcast_eventually(json_message.clone());
        }

//        wait(txDes = ⊥ ∨ hasTerminated(txDes));
        if txDes.is_some() {
            self.wait_until_all_terminate();
        } else {
            panic!("txDes is non in snapshot write 2");
        }
        let _ = self.bcast_status.as_ref().unwrap().send(StatusCode::Ok);
        self.run_result.msg_latencies.as_mut().unwrap().push(now.elapsed().as_micros());
    }

    fn compare_timestamp(&self, ts1: &Timestamp, ts2: &Timestamp) -> bool {
        //ts1 < ts2
        //printlnu(format!("ts1 {:?} ts2 {:?} return {:?}", ts1, ts2,  ts1.date < ts2.date || ((ts1.date == ts2.date) && (ts1.proc < ts2.proc))));
        ts1.date < ts2.date || ((ts1.date == ts2.date) && (ts1.proc < ts2.proc))

    }

    pub fn snapshot_msg_received(&mut self, msgs: Vec<String>) {
        let mut write_vec: Vec<WRITE> = Vec::new();

        for msg in msgs {
            if let Ok(write_msg) = serde_json::from_str::<WRITE>(msg.as_ref()) {
                write_vec.push(write_msg);
            } else {
            }
        }

//        printlnu(format!(" write_vec {:?} ", write_vec));

        if write_vec.len() == 0 { return; }

        let mut max_ts = write_vec[0].timestamp.clone();
        for write_msg in write_vec.iter() {
            if !self.compare_timestamp(&max_ts, &write_msg.timestamp) {
                max_ts = write_msg.timestamp.clone();
            }
        }

        for write_msg in write_vec.iter() {
            let r = write_msg.r;
            let v = write_msg.v;
            let ts = &write_msg.timestamp;

            let ts_to_compare = if let Some(ts) = self.tsa.get(&r) {
                ts.clone()
            } else {
                Timestamp { date: 0, proc: Some(self.node_id) }
            };
            if self.compare_timestamp(&ts_to_compare, ts) {
                if let Some(reg) = self.reg.get_mut(&r) {
                    *reg = v.clone();
                } else {
                    self.reg.insert(r, v);
                }
                if let Some(tsa) = self.tsa.get_mut(&r) {
                    *tsa = max_ts;
                } else {
                    self.tsa.insert(r, max_ts);
                }
            }
        }

    }

    fn wait_until_txDex_terminate(&mut self, txDes: &Tag) {
        let mut has_terminated = false;
        let mut iter = 0;
        while !has_terminated {
            self.bare_bone_loop_iter(iter % SETTINGS.delta() == 0);
            has_terminated = self.scd_has_terminated(txDes);
            iter += 1;
        }
    }
}