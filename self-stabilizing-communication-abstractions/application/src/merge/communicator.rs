use super::mergednode::MergedNode;
use commons::types::NodeId;
use crate::urb::thetafd::json_is_ThetafdMessage;
use crate::urb::messages::{Message, json_is_MSG_message, MSG, json_is_MSGAck_message, json_is_GOSSIP_message, GOSSIP, MSGAck};
use crate::urb::hbfd::json_is_HbfdMessage;
use crate::merge::mergednode::MessageType;
use crate::scd::messages::{json_is_SCDGOSSIP_message, SCDGOSSIP, json_is_FORWARD_message, FORWARD};
use std::str;
use crate::terminal_output::printlnu;
use crate::settings::SETTINGS;

impl MergedNode {
    //Communicator
    pub fn send_json_to(&self, json: &str, receiver_id: NodeId) {
        let bytes = json.as_bytes();
        let dst_socket_addr = self
            .socket_addrs
            .get(&receiver_id)
            .expect("Could not find receiver among the socket addresses");
        while let Err(e) = self.socket.send_to(bytes, dst_socket_addr) {
            if SETTINGS.print_client_operations() {
                printlnu(format!("Unable to send on socket, trying again. Err: {:?}",e));
            }
        }
    }

    pub fn recv_messages(&mut self) {
        let mut buf = [0;100000];
        loop {
            match self.socket.recv(&mut buf) {
                Ok(amt) => {
                    let json_string = str::from_utf8(&buf[0..amt]).expect("Error converting bytes to utf8");
                    self.json_received(json_string);
                }
                Err(e)=> {
                    break;
                }
            }
        }
    }

    //
    // Message sending, reception and serialization
    //

    pub fn jsonify_message<Me: Message>(&self, message: &Me) -> String {
        serde_json::to_string(message).expect("Could not serialize a message")
    }

    pub fn send_json_message_to(&self, json: &str, receiver_id: NodeId) {
        self.send_json_to(json, receiver_id);
    }

    pub fn json_received(&mut self, json: &str) {
        if json_is_HbfdMessage(&json) {
            if let Ok(hbfd_message) = serde_json::from_str(&json) {
                return self.on_hb_hb(hbfd_message);
            }
        }

        if json_is_ThetafdMessage(&json) {
            if let Ok(thetafd_message) = serde_json::from_str(&json) {
                return self.on_theta_hb(thetafd_message);
            }
        }

        let mut msgs_buffer_txs = self.msgs_buffer_txs.as_mut();
        match &mut msgs_buffer_txs {
            Some(buffer_txs) => {
                if json_is_MSG_message(&json) {
                    if let Ok(MSG_message) = serde_json::from_str::<MSG<String>>(&json) {
                        let _ = buffer_txs.get_mut(&MessageType::MSG).unwrap().get_mut(&MSG_message.sender).unwrap().send(json.to_owned());
                        return;
                    }
                } else if json_is_MSGAck_message(&json) {
                    if let Ok(MSGAck_message) = serde_json::from_str::<MSGAck>(&json) {
                        let _ = buffer_txs.get_mut(&MessageType::MSGAck).unwrap().get_mut(&MSGAck_message.sender).unwrap().send(json.to_owned());
                        return;
                    }
                } else if json_is_GOSSIP_message(&json) {
                    if let Ok(GOSSIP_message) = serde_json::from_str::<GOSSIP>(&json) {
                        let _ = buffer_txs.get_mut(&MessageType::GOSSIP).unwrap().get_mut(&GOSSIP_message.sender).unwrap().send(json.to_owned());
                        return;
                    }
                } else if json_is_FORWARD_message(&json) {
                    if let Ok(Forward_message) = serde_json::from_str::<FORWARD>(&json) {
                        let _ = buffer_txs.get_mut(&MessageType::FORWARD).unwrap().get_mut(&Forward_message.msg_tag.id).unwrap().send(json.to_owned());
                    }
                } else if json_is_SCDGOSSIP_message(&json) {
                    if let Ok(SCDGOSSIP_message) = serde_json::from_str::<SCDGOSSIP>(&json) {
                        let _ = buffer_txs.get_mut(&MessageType::SCDGOSSIP).unwrap().get_mut(&SCDGOSSIP_message.sender).unwrap().send(json.to_owned());
                        return;
                    }
                }
            },
            None => return,
        }
    }
}