#![allow(non_snake_case)]

use std::collections::{HashSet, BTreeSet, HashMap};
use std::iter::FromIterator;

use serde::{Deserialize, Serialize};

use crate::types::{Int, NodeId, Tag};
use std::time::Instant;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RunResult {

    pub broadcasted_msgs: BTreeSet<Tag>,
    pub urb_delivered_msgs: BTreeSet<Tag>,
    pub scd_delivered_msgs: HashMap<NodeId, BTreeSet<Tag>>,
    pub msg_latencies: Option<Vec<u128>>,
    pub read_latencies: Option<Vec<u128>>,
    pub log: Vec<(u64, String)>,
    pub illegally_triggered_ss: bool,
    pub metadata: Metadata,
    pub link_latency: HashMap<Int, f64>,
    pub recovery_time: Option<u128>,
    pub throughputs: Option<Vec<f64>>,
}

impl RunResult {
    pub fn new() -> RunResult {
        RunResult {

            broadcasted_msgs: BTreeSet::new(),
            urb_delivered_msgs: BTreeSet::new(),
            scd_delivered_msgs: HashMap::new(),
            msg_latencies: Some(Vec::new()),
            read_latencies: Some(Vec::new()),
            log: Vec::new(),

            illegally_triggered_ss: false,
            metadata: Metadata::new(),
            link_latency: HashMap::new(),
            recovery_time: None,
            throughputs: None
        }
    }
    #[allow(dead_code)]
    fn all_nodes_set(number_of_nodes: Int) -> HashSet<NodeId> {
        HashSet::from_iter(1..(number_of_nodes + 1) as NodeId)
    }

    #[allow(dead_code)]
    fn all_other_nodes_set(number_of_nodes: Int, node_id: NodeId) -> HashSet<NodeId> {
        let mut all_nodes_set = Self::all_nodes_set(number_of_nodes);
        all_nodes_set.remove(&node_id);
        all_nodes_set
    }
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
    pub run_length: Int,
}

impl Metadata {
    pub fn new() -> Metadata {
        Metadata {
            node_id: 0,
            run_length: 0,
        }
    }
}
