#![allow(non_snake_case)]
use std::collections::HashSet;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard};

use commons::run_result::RunResult;
use commons::types::{Int, NodeId};

//use crate::terminal_output::printlnu;
use crate::communicator::{udp_communicator::UDPCommunicator, CommunicatorDelegate};
//use crate::communicator::dccp_communicator::DCCPCommunicator;
use crate::configuration_manager::ConfigurationManager;
use crate::responsible_cell::ResponsibleCell;
use crate::settings::SETTINGS;
use crate::urb::{NodeDelegate, urb_node::UrbNode};
use crate::urb::types::Tag;

pub struct Mediator {
    communicator: ResponsibleCell<Option<Arc<UDPCommunicator<Mediator>>>>,
    // communicator: ResponsibleCell<Option<Arc<DCCPCommunicator<Mediator>>>>,
    configuration_manager: ConfigurationManager,
    run_result: Mutex<RunResult>,

    node: ResponsibleCell<Option<Arc<UrbNode<Mediator, String>>>>,
    node_do_forever_loop_handle_tx: Mutex<Option<Sender<()>>>,
}

impl Mediator {
    pub fn new() -> Arc<Mediator> {
        let node_id = SETTINGS.node_id();
        let socket_addrs = SETTINGS.socket_addrs().clone();
        let node_ids = socket_addrs.keys().map(|node_id| *node_id).collect();
        let mediator = Mediator {
            communicator: ResponsibleCell::new(None),
            configuration_manager: ConfigurationManager::new(node_id, node_ids),
            run_result: Mutex::new(RunResult::new()),
            node: ResponsibleCell::new(None),
            node_do_forever_loop_handle_tx: Mutex::new(None),
        };
        let mediator: Arc<Mediator> = Arc::new(mediator);

        let own_socket_addr = socket_addrs
            .get(&node_id)
            .expect("Could not find own socket address.");

        let communicator =
            UDPCommunicator::new(*own_socket_addr, socket_addrs, Arc::downgrade(&mediator));

        // let communicator =
        //     DCCPCommunicator::new(*own_socket_addr, socket_addrs.clone(), Arc::downgrade(&mediator));
        *mediator.communicator.get_mut() = Some(communicator);

        let node = UrbNode::new(Arc::downgrade(&mediator)); 

        *mediator.node.get_mut() = Some(node);
        UDPCommunicator::start_recv_thread(&*mediator.communicator.get().as_ref().unwrap());
        // DCCPCommunicator::start_recv_thread(&*mediator.communicator.get().as_ref().unwrap());

        let mut node_do_forever_loop_handle_tx =
            mediator.node_do_forever_loop_handle_tx.lock().unwrap();
        *node_do_forever_loop_handle_tx = Some(UrbNode::start_the_do_forever_loop(mediator.node()));
        drop(node_do_forever_loop_handle_tx);

        mediator
    }

    pub fn stop_all_threads(&self) {
        let node_do_forever_loop_handle_tx = self.node_do_forever_loop_handle_tx.lock().unwrap();
        if let Some(handle) = &*node_do_forever_loop_handle_tx {
            let _ = handle.send(());
        }
    }

    fn communicator(&self) -> &UDPCommunicator<Mediator> {
        self.communicator
            .get()
            .as_ref()
            .expect("Communicator not set on Mediator.")
    }

    fn node(&self) -> &Arc<UrbNode<Mediator, String>> {
        self.node.get().as_ref().expect("Node not set on Mediator.")
    }
    pub fn trusted(&self) -> HashSet<NodeId>{
        self.node().trusted()
    }

    pub fn urbBroadcast(&self, msg: String) -> Tag{
        let tx_des = self.node().urbBroadcast(msg);
        tx_des
    }

    pub fn get_echo_msg(&self) -> String{
        return self.node().get_echo_msg();
    }
    pub fn rm_echo_msg(&self){
        self.node().rm_echo_msg();
    }
    pub fn get_alive_msg(&self) -> String{
        return self.node().get_alive_msg();
    }
    pub fn rm_alive_msg(&self){
        self.node().rm_alive_msg();
    }
    pub fn get_response_msg(&self) -> String{
        return self.node().get_response_msg();
    }
    pub fn rm_response_msg(&self){
        self.node().rm_response_msg();
    }
    pub fn get_phase_msg(&self) -> String{
        return self.node().get_phase_msg();
    }
    pub fn rm_phase_msg(&self){
        self.node().rm_phase_msg();
    }
    pub fn get_decide_msg(&self) -> String{
        return self.node().get_decide_msg();
    }
    pub fn rm_decide_msg(&self){
        self.node().rm_decide_msg();
    }
    pub fn get_proposal_msg(&self) -> String{
        return self.node().get_proposal_msg();
    }
    pub fn rm_proposal_msg(&self){
        self.node().rm_proposal_msg();
    }
    pub fn get_to_urb_msg(&self) -> String{
        return self.node().get_to_urb_msg();
    }
    pub fn rm_to_urb_msg(&self){
        self.node().rm_to_urb_msg();
    }
    pub fn get_sync_msg(&self) -> String{
        return self.node().get_sync_msg();
    }
    pub fn rm_sync_msg(&self){
        self.node().rm_sync_msg();
    }
    pub fn get_syncack_msg(&self) -> String{
        return self.node().get_syncack_msg();
    }
    pub fn rm_syncack_msg(&self){
        self.node().rm_syncack_msg();
    }

    pub fn has_terminated(&self, tx_des: &Tag) -> bool {
        let terminated = self.node().has_terminated(tx_des);
        terminated
    }

    fn configuration_manager(&self) -> &ConfigurationManager {
        &self.configuration_manager
    }


    #[allow(dead_code)]
    pub fn transition_to_arbitrary_state(&self) {
        self.node().transition_to_arbitrary_state();
    }
}

impl CommunicatorDelegate for Mediator {
    fn json_received(&self, json: &str) {
        self.node().json_received(json);
    }
}

impl NodeDelegate for Mediator {
    fn send_json_to(&self, json: &str, receiver: NodeId) {
        self.communicator().send_json_to(json, receiver);
    }

    fn node_id(&self) -> NodeId {
        self.configuration_manager().node_id()
    }

    fn node_ids(&self) -> &HashSet<NodeId> {
        self.configuration_manager().node_ids()
    }

    fn number_of_nodes(&self) -> Int {
        self.configuration_manager().number_of_nodes()
    }

    fn record_evaluation_info(&self) -> bool {
        SETTINGS.record_evaluation_info()
    }

    fn run_result(&self) -> MutexGuard<RunResult> {
        self.run_result.lock().unwrap()
    }
}

