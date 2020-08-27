use super::mergednode::MergedNode;
use crate::terminal_output::printlnu;
use commons::types::Int;
use crate::urb::hbfd::HbfdMessage;

impl MergedNode {
    //Hbfd
    pub(crate) fn hbfd_iter(&mut self) {
        for id in self.node_ids.clone() {
            self.send_hb_hb(id);
        }
    }

    pub fn get_hb(&self) -> Vec<Int> {
        let hb = &self.hb;
        let mut hb_vec = Vec::new();
        for node_id in 1..(self.node_ids.len() + 1) {
            hb_vec.push(*hb.get(&(node_id as Int)).unwrap())
        }
        hb_vec
    }

    pub fn on_hb_hb(&mut self, msg: HbfdMessage) {
        let sender_id = msg.sender;
        *self.hb.get_mut(&sender_id).unwrap() += 1;
    }

    fn send_hb_hb(&mut self, receiver_id: Int) {
        if receiver_id == self.node_id {
            *self.hb.get_mut(&receiver_id).unwrap() += 1;
        } else {
            let msg = HbfdMessage {sender:self.node_id};
            let json_msg = serde_json::to_string(&msg).expect("Could not serialize a hb message");
            self.send_json_to(&json_msg, receiver_id);
        }
    }
}