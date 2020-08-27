pub mod messages;
//
//use commons::types::{Int, NodeId, Tag};
//use std::sync::{Mutex, Arc, Condvar};
//use std::collections::HashSet;
//use std::sync::mpsc::{Receiver, Sender};
//
//use crate::scd::scd::SCD;
//use crate::settings::SETTINGS;
//use crate::scd::algorithm5::messages::{Message, WRITE, SYNC};
//
//pub struct Algorithm5 {
////    reg[1..m]:
////    tsa[1..m]:
//    own_id: NodeId,
//    send_end: Mutex<Sender<String>>,
//    recv_end: Mutex<Receiver<HashSet<String>>>,
//    broadcaster: Arc<SCD>,
//    reg: Mutex<Vec<Int>>,
//    tsa: Mutex<Vec<(Int, Option<Int>)>>,
//    condAllHaveTerminated: Condvar,
//}
//
//impl Algorithm5 {
//
//    pub fn new(send_end: Sender<String>, recv_end: Receiver<HashSet<String>>, broadcaster: Arc<SCD>) -> Arc<Self> {
//        let number_of_nodes = SETTINGS.node_ids().len();
//        let algorithm5 = Algorithm5 {
//            own_id: SETTINGS.node_id(),
//            send_end: Mutex::new(send_end),
//            recv_end: Mutex::new(recv_end),
//            broadcaster,
//            reg: Mutex::new(vec![0; number_of_nodes]),
//            tsa: Mutex::new(vec![(0, None); number_of_nodes]),
//            condAllHaveTerminated: Condvar::new(),
//        };
//        Arc::new(algorithm5)
//    }
//
//    pub fn snapshot(&self) -> Vec<Int> {
//        let sync_message = SYNC { sender_identifier: self.own_id };
//        let json_message = self.jsonify_message(&sync_message);
//        let txDes = self.broadcaster.scdBroadcast(json_message);
//        //wait(txDes = ⊥ ∨ hasTerminated(txDes));
//        self.wait_until_operation_has_terminated();
//        //return reg
//        let reg = self.reg.lock().unwrap();
//        *reg
//    }
//
//    pub fn write(&self, r: Int, v: Int) {
//        let sync_message = SYNC { sender_identifier: self.own_id };
//        let json_message = self.jsonify_message(&sync_message);
//        let txDes = self.broadcaster.scdBroadcast(json_message);
//        //wait(txDes = ⊥ ∨ hasTerminated(txDes));
//        self.wait_until_operation_has_terminated(txDes);
//
//        let tsa = self.tsa.lock().unwrap();
//        let write_message = WRITE {
//            r,
//            v,
//            date_tuple: (tsa[r].date, self.own_id),
//        };
//        let json_message = self.jsonify_message(&write_message);
//        let txDes = self.broadcaster.scdBroadcast(json_message);
//        //wait(txDes = ⊥ ∨ hasTerminated(txDes));
//        self.wait_until_operation_has_terminated(txDes)
//
//    }
//
//    pub fn recv_loop() {
////        foreach r ∈ {1, . . . , k} : tsa[r] <ts timestamp where timestamp := max<ts {WRITE[x].timestamp}x∈{1,...,k}
////        do (reg[r], tsa[r]) ← (WRITE[r].υ, timestamp);
//    }
//
//    fn jsonify_message<Me: Message>(&self, message: &Me) -> String {
//        serde_json::to_string(message).expect("Could not serialize a message")
//    }
//
//    fn wait_until_operation_has_terminated(&self, tag: Option<Tag>) {
//        //wait(txDes = ⊥ ∨ hasTerminated(txDes));
//    }
//}