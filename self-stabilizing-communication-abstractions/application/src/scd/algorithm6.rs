pub mod messages;

use std::sync::{Mutex, Arc, Condvar};
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashSet;
use commons::types::{Int, NodeId};

use crate::scd::scd::SCD;
use crate::scd::algorithm6::messages::{PLUS, MINUS, json_is_PLUS_Message, json_is_MINUS_Message};
use crate::settings::SETTINGS;
use crate::terminal_output::printlnu;
use std::thread;
use crate::urb::messages::Message;

pub struct Algorithm6 {
    own_id: NodeId,
    send_end: Mutex<Sender<String>>,
    recv_end: Mutex<Receiver<HashSet<String>>>,
    broadcaster: Arc<SCD>,
    counter: Mutex<Int>,
    condAllHaveTerminated: Condvar,
}

impl Algorithm6 {
    pub fn new(send_end: Sender<String>, recv_end: Receiver<HashSet<String>>, broadcaster: Arc<SCD>) -> Arc<Self> {
        let algorithm6 = Algorithm6 {
            own_id: SETTINGS.node_id(),
            send_end: Mutex::new(send_end),
            recv_end: Mutex::new(recv_end),
            broadcaster,
            counter: Mutex::new(0),
            condAllHaveTerminated: Condvar::new(),
        };
        Arc::new(algorithm6)
    }

    pub fn increase(&self) {
//        scdBroadcast PLUS(i)
        let message = PLUS { i: self.own_id };

        let json_message = self.jsonify_message(&message);

        self.broadcaster.scdBroadcast(json_message);
    }

    pub fn decrease(&self) {
//        scdBroadcast MINUS(i)
        let message = MINUS { i: self.own_id };

        let json_message = self.jsonify_message(&message);

        self.broadcaster.scdBroadcast(json_message);
    }

    pub fn read(&self) -> Int {
//        allHaveTerminated();
        //TODO change to condvar
        while !self.broadcaster.allHaveTerminated(){

        }

        let counter = self.counter.lock().unwrap();
        printlnu(format!("counter: {}", counter));
//        return (counter);
        *counter
    }


    fn jsonify_message<Me: Message>(&self, message: &Me) -> String {
        serde_json::to_string(message).expect("Could not serialize a message")
    }

    pub fn recv_loop(&self) {
        let recv_end = self.recv_end.lock().unwrap();

        loop {
            match recv_end.recv() {
                Ok(message) => {
                    let mut k = 0;
                    let mut l = 0;
//                    printlnu(format!("recv end {:?}", message));
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
                    let mut counter = self.counter.lock().unwrap();
                    *counter = *counter + k - l;
                    drop(counter);
                }

                Err(e) => {
                    printlnu(format!("Error when trying to receive msg from SCD: {:?}", e));
                }
            }
        }
    }

}