use std::sync::{mpsc::{self, Sender, Receiver, TryRecvError}, Arc, Weak, Mutex, MutexGuard};
use std::collections::HashMap;
use std::thread;

use serde::{Deserialize, Serialize};

use commons::types::{Int, NodeId};
use crate::communicator::CommunicatorDelegate;
use crate::urb::NodeDelegate;
// use crate::terminal_output::printlnu;
use super::HBFD_HB_TIMEOUT;

pub struct Hbfd<D>{
    delegate: Weak<D>,
    hb: Mutex<HashMap<NodeId, Int>>,
    stop_thread_handler: Mutex<Sender<()>>,
}

impl <D> Hbfd<D>
    where D : CommunicatorDelegate + NodeDelegate + Send + Sync + 'static {

    pub fn new(delegate: Weak<D>) -> Arc<Hbfd<D>> {
        let mediator = delegate.upgrade().unwrap();
        let node_ids = mediator.node_ids();

        let mut hashmap = HashMap::new();

        for node_id in node_ids {
            hashmap.insert(*node_id, 0);
        }

        let hb = Mutex::new(hashmap);
        let (tx, rx) = mpsc::channel();

        let hbfd = Hbfd{
            delegate,
            hb,
            stop_thread_handler: Mutex::new(tx), 
        };

        let hbfd= Arc::new(hbfd);

        Hbfd::start_hbfd_thread(&hbfd, rx);
        hbfd
    }

    pub fn get_hb(&self) -> Vec<Int> {
        let hb = self.hb.lock().unwrap();
        let mut hb_array = Vec::new();
        for node_id in 1..self.delegate().node_ids().len() + 1 {
            hb_array.push(*hb.get(&(node_id as Int)).unwrap());
        }
        hb_array
    }

    fn id(&self) -> NodeId {
        self.delegate().node_id()
    }

    fn start_hbfd_thread(hbfd: &Arc<Self>, rx: Receiver<()>) {
        
        let hbfd = Arc::clone(&hbfd);
        thread::spawn(move || {
            hbfd.do_forever_loop(rx);
        });
    }

    pub fn stop_thread(&self) {
        let tx = self.stop_thread_handler.lock().unwrap();
        tx.send(()).unwrap();
    }

    fn do_forever_loop(&self, rx: Receiver<()>) {
        loop {
            for id in self.delegate().node_ids() {
                self.send_heartbeat(*id);
            }
            thread::sleep(HBFD_HB_TIMEOUT);

            match rx.try_recv() {
                Err(TryRecvError::Empty) => {}
                _ => {
                    break;
                }
            }
        }
    }

    pub fn on_heartbeat(&self, msg: HbfdMessage) {
        // printlnu(format!("Received hbfd heartbeat from {}.", msg.sender));
        let sender_id = msg.sender;
        let mut hb = self.hb.lock().unwrap();

        *hb.get_mut(&sender_id).unwrap() += 1; 
    }

    fn send_heartbeat(&self, receiver_id: Int) {
        if receiver_id == self.id() {
            let mut hb = self.hb.lock().unwrap();
            *hb.get_mut(&receiver_id).unwrap() += 1;
        } else {
            let msg = HbfdMessage { sender: self.id() };
            let json_msg = serde_json::to_string(&msg).expect("Could not serialize a message");
            self.delegate().send_json_to(&json_msg, receiver_id);
        }
    }

    fn delegate(&self) -> Arc<D> {
        self.delegate.upgrade().expect("Cannot upgrade delegate in Hbfd.")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HbfdMessage {
    #[serde(rename = "HbfdMessage")]
    pub sender: NodeId,
}

pub fn json_is_HbfdMessage(json: &str) -> bool {
    json.starts_with("{\"HbfdMessage\":")
}
