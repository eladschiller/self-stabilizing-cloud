pub mod messages;

use super::scd::SCD;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::mpsc::{Sender, Receiver};
use messages::*;
use std::borrow::Cow;
use std::collections::HashSet;
use commons::types::{Int, NodeId};
use std::thread;
use std::time::Duration;
use crate::settings::SETTINGS;
use crate::terminal_output::printlnu;

pub struct Pattern3 {
    own_id: NodeId,
    send_end: Mutex<Sender<String>>,
    recv_end: Mutex<Receiver<HashSet<String>>>,
    broadcaster: Arc<SCD>,
    done: Mutex<bool>,
}

impl Pattern3 {
    pub fn new(send_end: Sender<String>, recv_end: Receiver<HashSet<String>>, broadcaster: Arc<SCD>) -> Arc<Self> {
        let pattern3 = Pattern3 {
            own_id: SETTINGS.node_id(),
            send_end: Mutex::new(send_end),
            recv_end: Mutex::new(recv_end),
            broadcaster: broadcaster,
            done: Mutex::new(false),
        };
        Arc::new(pattern3)
    }

    pub fn op(&self) {

        let mut done = self.done.lock().unwrap();
        *done = false;
        drop(done);

        //let tx = scdBroadcast TYPE(a, b, . . . , i)
        let message = SYNC{ sender_identifier: self.own_id };

        let json_message = self.jsonify_message(&message);

        let tx = self.broadcaster.scdBroadcast(json_message);

        //wait (tx = None or hasTerminated(txDes)) or allHaveTerminated()
        self.wait_until_operations_are_done();

        //Compute the return value r before calling return (r);
    }

    fn wait_until_operations_are_done(&self) {
//        upon scdDelivered({MSG[1..k], SYNC[1..l]})
        let recv_end = self.recv_end.lock().unwrap();
        let mut done = self.done.lock().unwrap();

        while !*done {
            match recv_end.recv() {
                Ok(msg) => {
                    //change done variable here
                    for message in msg.iter() {
//                        let json_message = self.jsonify_message(message);
                        printlnu(format!("recv end {:?}", message));

                        if json_is_SYNC_Message(&message) {
                            if let Ok(sync_message) = serde_json::from_str::<SYNC>(&message) {
                                printlnu(format!("sync {:?}", sync_message));
                                if sync_message.sender_identifier == self.own_id {
                                    *done = true;
                                    break;
                                }
                            }
                        }

                        if json_is_MSG_Message(&message) {
                            //do algorithm step on MSG[x] specific to the implemented task
                            printlnu(format!("msg"));
                            thread::sleep(Duration::from_millis(10));
                            *done = true;
                            break;
                        }
                    }
                }
                Err(e) => {
                    printlnu(format!("Error when trying to receive msg from SCD: {:?}", e));
                }
            }
        }
    }

    fn jsonify_message<Me: Message>(&self, message: &Me) -> String {
        serde_json::to_string(message).expect("Could not serialize a message")
    }

}