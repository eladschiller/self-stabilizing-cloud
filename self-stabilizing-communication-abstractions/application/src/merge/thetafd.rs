use super::mergednode::MergedNode;
use std::collections::HashSet;
use commons::types::{NodeId, Int};
use commons::constants;
use crate::urb::thetafd::ThetafdMessage;
use crate::terminal_output::printlnu;
use bit_vec::BitVec;

impl MergedNode {
    //Thetafd
    pub(crate) fn theta_iter(&self) {
        for id in self.node_ids.clone() {
            self.send_theta_hb(id);
        }
    }
    pub fn trusted(&mut self) -> BitVec {
        let vector = &self.theta_vector;
        for (idx, x) in vector.iter() {
            if *x >= constants::THETAFD_W  {
                if self.current_trusted.get(*idx as usize - 1).unwrap() {
                    self.current_trusted.set(*idx as usize - 1, false);
                    printlnu(format!("Node {} is not trusted {:?}", idx, vector));
                }
            }
        }
        self.current_trusted.clone()
    }

    pub fn on_theta_hb(&mut self, msg: ThetafdMessage){
        let sender_id = msg.sender;
        for(idx, val) in self.theta_vector.iter_mut() {
            *val = if *idx == sender_id || *idx == self.node_id {
                0
            } else {
//                printlnu(format!("idx {} sender_id {} *val = {} *val + 1 = {} msg {:?}",idx, sender_id, *val, *val+1, msg));
                *val + 1
            }
        }
    }

    fn send_theta_hb(&self, reciever_id: Int) {
        let message = ThetafdMessage {sender:self.node_id};
        let json_msg = serde_json::to_string(&message).expect("Could not serialize theta msg");
        self.send_json_to(&json_msg, reciever_id);
    }


}