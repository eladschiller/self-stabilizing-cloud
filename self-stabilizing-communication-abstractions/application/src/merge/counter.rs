use super::mergednode::MergedNode;
use crate::urb::messages::Message;
use crate::scd::algorithm6::messages::{PLUS, MINUS, json_is_MINUS_Message, json_is_PLUS_Message};
use crate::terminal_output::printlnu;
use commons::types::Int;
use crate::settings::SETTINGS;
use std::time::Instant;

impl MergedNode {
    pub fn increase(&mut self) {
//        scdBroadcast PLUS(i)
        let message = PLUS { i: self.node_id };

        let json_message = self.jsonify_message(&message);

        self.scd_broadcast(json_message);
    }

    pub fn decrease(&mut self) {
//        scdBroadcast MINUS(i)
        let message = MINUS { i: self.node_id };

        let json_message = self.jsonify_message(&message);

        self.scd_broadcast(json_message);
    }

    pub fn read(&mut self) -> Option<Int> {
//        allHaveTerminated();
        //TODO change to condvar
//        while !self.all_have_terminated(){
//
//        }
//
        let now = Instant::now();
        printlnu(format!("-------------    Initiating counter read"));
////        return (counter);
//        if self.scd_all_have_terminated() {
//            Some(self.counter)
//        } else {
//            self.is_reading = true;
//            None
//        }
        self.wait_until_all_terminate();
        self.run_result.read_latencies.as_mut().unwrap().push(now.elapsed().as_micros());
        printlnu(format!("-------------    counter: {}", self.counter));
        Some(self.counter)
    }

    pub fn counter_received(&mut self, message: Vec<String>) {
        let mut k = 0;
        let mut l = 0;
        for msg in message {
            if json_is_PLUS_Message(&msg) {
                if let Ok(plus_message) = serde_json::from_str::<PLUS>(&msg) {
                    k += 1;
                }
            }

            if json_is_MINUS_Message(&msg) {
                if let Ok(minus_message) = serde_json::from_str::<MINUS>(&msg) {
                    l += 1;
                }
            }

        }
        self.counter = self.counter + k - l;
    }

    pub fn wait_until_all_terminate(&mut self) {
        let mut has_terminated = false;
        let mut iter = 0;
        while !has_terminated {
            self.bare_bone_loop_iter(iter % SETTINGS.delta() == 0);
            has_terminated = self.scd_all_have_terminated();
            iter += 1;
        }
    }
}