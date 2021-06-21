use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use commons::types::{Int, NodeId};
use super::types::Tag;


pub trait Message: Serialize {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MSG<'a, V: Clone> {
    #[serde(rename = "MSG")]
    pub sender: NodeId,
    pub msg: Cow<'a, Option<V>>,
    pub tag: Tag,
}

impl<'a, V: Serialize + Clone> Message for MSG<'a, V> {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MSGAck {
    #[serde(rename = "MSGAck")]
    pub sender: NodeId,
    pub tag: Tag,
}

impl Message for MSGAck {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct GOSSIP {
    #[serde(rename = "GOSSIP")]
    pub sender: NodeId,
    pub maxSeq: Int,
    pub rxObsS: Int,
    pub txObsS: Int,
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
