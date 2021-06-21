use std::sync::{mpsc::{self, Sender, Receiver, TryRecvError}, Arc, Weak, Mutex, MutexGuard};
use std::thread;
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use commons::types::{Int, NodeId};
use commons::constants;
use crate::communicator::CommunicatorDelegate;
use crate::urb::NodeDelegate;
// use crate::terminal_output::printlnu;
use super::THETA_HB_TIMEOUT;

pub struct Thetafd<D> {
    delegate: Weak<D>,
    vector: Mutex<HashMap<NodeId, Int>>,
    current_trusted: Mutex<HashSet<NodeId>>,
    stop_thread_handler: Mutex<Sender<()>>,
}

impl<D> Thetafd<D> 
    where D : CommunicatorDelegate + NodeDelegate + Send + Sync + 'static {
    pub fn new(delegate: Weak<D>) -> Arc<Self> {
        let mediator = delegate.upgrade().unwrap();
        let node_ids = mediator.node_ids();
        let mut hashmap = HashMap::new();

        for node_id in node_ids {
            hashmap.insert(*node_id, 0);
        }

        let vector = Mutex::new(hashmap);
        let current_trusted = Mutex::new(node_ids.clone());

        let (tx, rx) = mpsc::channel();
        let thetafd = Thetafd {
            delegate,
            vector,
            current_trusted,
            stop_thread_handler: Mutex::new(tx), 
        };

        let thetafd= Arc::new(thetafd);
        Thetafd::start_thetafd_thread(&thetafd, rx);
        thetafd
    }

    fn id(&self) -> NodeId {
        self.delegate().node_id()
    }

    pub fn trusted(&self) -> HashSet<NodeId>{
        let mut trusted = HashSet::new();
        let vector = self.get_vector();

        for (idx, x) in vector.iter() {
            if *x < constants::THETAFD_W {
                trusted.insert(*idx);
            }
        }
        let mut current_trusted = self.current_trusted.lock().unwrap();
        current_trusted.retain(|id| trusted.contains(id) );
        current_trusted.clone()
    }

    fn get_vector(&self) -> MutexGuard<HashMap<NodeId, Int>> {
        self.vector.lock().unwrap()
    }

    fn start_thetafd_thread(thetafd: &Arc<Self>, rx: Receiver<()>) {
        let thetafd = Arc::clone(&thetafd);
        thread::spawn(move || {
            thetafd.do_forever_loop(rx);
        });
    }

    pub fn stop_thread(&self) {
        let tx = self.stop_thread_handler.lock().unwrap();
        tx.send(()).unwrap();
    }

    fn do_forever_loop(&self, rx:Receiver<()>) {
        loop {
            for id in self.delegate().node_ids() {
                self.send_heartbeat(*id);
            }

            thread::sleep(THETA_HB_TIMEOUT);

            match rx.try_recv() {
                Err(TryRecvError::Empty) => {}
                _ => {
                    break;
                }
            }
        }
    }

    pub fn on_heartbeat(&self, msg: ThetafdMessage) {

        // printlnu(format!("Received theta heartbeat from {}.", msg.sender));
        let sender_id = msg.sender;
        let mut vector = self.get_vector();
        *vector.get_mut(&sender_id).unwrap() = 0;
        for (&idx, val) in vector.iter_mut() {
            *val = if idx == sender_id || idx == self.id() {
                0
            } else {
                *val + 1
            }
        }
    }

    fn send_heartbeat(&self, receiver_id: Int) {
        let message = ThetafdMessage {sender: self.id()};
        let json_msg = serde_json::to_string(&message).expect("Could not serialize a message");
        self.delegate().send_json_to(&json_msg, receiver_id);
    }

    fn delegate(&self) -> Arc<D> {
        self.delegate.upgrade().unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThetafdMessage {
    #[serde(rename = "ThetafdMessage")]
    pub sender: NodeId,
}

pub fn json_is_ThetafdMessage(json: &str) -> bool {
    json.starts_with("{\"ThetafdMessage\":")
}
