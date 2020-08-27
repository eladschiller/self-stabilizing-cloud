use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use commons::types::{Tag, Int, NodeId};
use crate::scd::types::VectorClock;
use crate::urb::messages::Message;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct FORWARD<'a> {
    #[serde(rename = "FORWARD")]
    pub msg: Cow<'a, String>,
    pub msg_tag: Tag,
    pub forward_tag: Tag,
    pub cl: VectorClock,
}

impl<'a> Message for FORWARD<'a> {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct GOSSIP {
    #[serde(rename = "GOSSIP")]
    pub clock: Int,
}

impl Message for GOSSIP {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SCDGOSSIP {
    #[serde(rename = "SCDGOSSIP")]
    pub sender: NodeId,
    pub scd_maxSeq: Int,
    pub scd_rxObsS: Int,
    pub scd_txObsS: Int,
    pub scd_rxSpace: Option<Int>,
    pub scd_txSpace: Option<Int>,
}

impl Message for SCDGOSSIP {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SCDMETA {
    #[serde(rename = "SCDMETA")]
    pub tag: Tag,
    pub cl: VectorClock,
    pub delivered: bool,
    pub txDes: Option<Tag>,
    pub transmission_counter: Int,
}

impl Message for SCDMETA {}


pub fn json_is_FORWARD_message(json: &str) -> bool {
    json.starts_with("{\"FORWARD\":")
}

pub fn json_is_GOSSIP_message(json: &str) -> bool {
    json.starts_with("{\"GOSSIP\":")
}

pub fn json_is_SCDGOSSIP_message(json: &str) -> bool {
    json.starts_with("{\"SCDGOSSIP\":")
}