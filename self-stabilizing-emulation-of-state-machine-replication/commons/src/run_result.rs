#![allow(non_snake_case)]

use std::collections::HashSet;
use std::collections::BTreeSet;
//use std::iter::FromIterator;

use serde::{Deserialize, Serialize};

use crate::types::{Int, NodeId};
//use crate::variant::Variant;

type Tag = (NodeId, Int);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RunResult {

    pub MSG_message: MessageTypeResult,
    pub MSGAckck_message: MessageTypeResult,
    pub GOSSIP_message: MessageTypeResult,
    pub delivered_msgs: BTreeSet<Tag>,

    pub metadata: Metadata,
}

impl RunResult {
    pub fn new() -> RunResult {
        RunResult {

            MSG_message: MessageTypeResult::new(),
            MSGAckck_message: MessageTypeResult::new(),
            GOSSIP_message: MessageTypeResult::new(),
            delivered_msgs: BTreeSet::new(),

            metadata: Metadata::new(),
        }
    }
    /*
    fn all_nodes_set(number_of_nodes: Int) -> HashSet<NodeId> {
        HashSet::from_iter(1..(number_of_nodes + 1) as NodeId)
    }
    */
    /*
    fn all_other_nodes_set(number_of_nodes: Int, node_id: NodeId) -> HashSet<NodeId> {
        let mut all_nodes_set = Self::all_nodes_set(number_of_nodes);
        all_nodes_set.remove(&node_id);
        all_nodes_set
    }
    */
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageTypeResult {
    pub sent: Int,
    pub received: Int,
    pub nodes_received_from: HashSet<NodeId>,
}

impl MessageTypeResult {
    pub fn new() -> MessageTypeResult {
        MessageTypeResult {
            sent: 0,
            received: 0,
            nodes_received_from: HashSet::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    pub node_id: NodeId,
    //pub is_snapshotter: bool,
    //pub is_writer: bool,
    pub run_length: Int,
    //pub variant: Variant,
}

impl Metadata {
    pub fn new() -> Metadata {
        Metadata {
            node_id: 0,
            //is_snapshotter: false,
            //is_writer: false,
            run_length: 0,
            //variant: Variant::Algorithm1,
        }
    }
}
