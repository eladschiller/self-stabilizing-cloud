use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use commons::types::{Tag, Int, NodeId};
use crate::scd::messages::SCDGOSSIP;


pub trait Message: Serialize {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MSG<'a, V: Clone> {
    #[serde(rename = "MSG")]
    pub sender: NodeId,
    pub msg: Cow<'a, Option<V>>,
    pub tag: Tag,
    pub recv_by: Vec<u8>,
    pub recv_by_trusted: Vec<u8>,
    pub gossip: CombinedGossip,
}

impl<'a, V: Serialize + Clone> Message for MSG<'a, V> {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MSGAck {
    #[serde(rename = "MSGAck")]
    pub sender: NodeId,
    pub tag: Tag,
    pub recv_by: Vec<u8>,
}

impl Message for MSGAck {}


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CombinedGossip {
    #[serde(rename = "COMBINED_GOSSIP")]
    pub urb_gossip: GOSSIP,
    pub scd_gossip: SCDGOSSIP,
}

impl Message for CombinedGossip {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct GOSSIP {
    #[serde(rename = "GOSSIP")]
    pub sender: NodeId,
    pub urb_maxSeq: Int,
    pub urb_rxObsS: Int,
    pub urb_txObsS: Int,
}

impl Message for GOSSIP {}

pub fn json_is_MSG_message(json: &str) -> bool {
    json.starts_with("{\"MSG\":")
}

pub fn json_is_MSGAck_message(json: &str) -> bool {
    json.starts_with("{\"MSGAck\":")
}

pub fn json_is_GOSSIP_message(json: &str) -> bool {
    json.starts_with("{\"GOSSIP\":")
}
