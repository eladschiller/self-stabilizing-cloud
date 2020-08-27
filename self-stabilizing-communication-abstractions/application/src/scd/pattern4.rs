pub mod messages;

use std::sync::{Mutex, Arc, Condvar};
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashSet;
use commons::types::{NodeId, Tag};

use crate::scd::scd::SCD;
use crate::scd::pattern4::messages::{SYNC, MSG, json_is_SYNC_Message, json_is_MSG_Message};
use crate::settings::SETTINGS;
use crate::scd::pattern4::messages::Message;
use std::thread;
use crate::terminal_output::printlnu;

pub struct Pattern4 {
    own_id: NodeId,
    send_end: Mutex<Sender<String>>,
    recv_end: Mutex<Receiver<HashSet<String>>>,
    broadcaster: Arc<SCD>,
    wait_cond: Condvar,
}


impl Pattern4 {
    pub fn new(send_end: Sender<String>, recv_end: Receiver<HashSet<String>>, broadcaster: Arc<SCD>) -> Arc<Self>{
        let pattern4 = Pattern4 {
            own_id: SETTINGS.node_id(),
            send_end: Mutex::new(send_end),
            recv_end: Mutex::new(recv_end),
            broadcaster,
            wait_cond: Condvar::new(),
        };
        Arc::new(pattern4)
    }

    pub fn op(&self) {
//        let txDes := scdBroadcast TYPE(a, b, . . . , i)
        let message = SYNC{ sender_identifier: self.own_id };

        let json_message = self.jsonify_message(&message);

        let txDes = self.broadcaster.scdBroadcast(json_message);

//        wait(txDes = ⊥ ∨ hasTerminated(txDes) or allHaveTerminated());
        self.wait(&txDes);

//        Compute the return value r before calling return (r);
//        return;
    }

    fn wait(&self, txDes: &Option<Tag>) {
//        wait(txDes = ⊥ ∨ hasTerminated(txDes) or allHaveTerminated());
        //TODO probably need a thread to listen for incoming msgs
        loop {
            self.listen_to_incoming_msg();
            if txDes.is_none() {
                return;
            } else {
                //TODO use condvar
                if self.broadcaster.hasTerminated(txDes.as_ref().unwrap()) || self.broadcaster.allHaveTerminated() {
                    break;
                }
            }
        }
    }

    fn jsonify_message<Me: Message>(&self, message: &Me) -> String {
        serde_json::to_string(message).expect("Could not serialize a message")
    }

    fn listen_to_incoming_msg(&self) {
        let recv_end = self.recv_end.lock().unwrap();
        match recv_end.recv() {
            Ok(msg) => {
                //change done variable here
                for message in msg.iter() {

                    if json_is_SYNC_Message(&message) {
                        if let Ok(sync_message) = serde_json::from_str::<SYNC>(&message) {
                            printlnu(format!("sync {:?}", sync_message));
                        }
                    }

                    if json_is_MSG_Message(&message) {
                        //do algorithm step on MSG[x] specific to the implemented task
                        if let Ok(msg) = serde_json::from_str::<MSG>(&message) {
                            printlnu(format!("msg {:?}", msg));
                        }
                    }
                }
            }
            Err(e) => {
                printlnu(format!("Error when trying to receive msg from SCD: {:?}", e));
            }
        }

    }

}