use serde::{Deserialize, Serialize};
use commons::types::{Int, NodeId};
use crate::terminal_output::printlnu;
use crate::urb::messages::Message;
use crate::merge::snapshot::Timestamp;


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SYNC {
    #[serde(rename = "SYNC")]
    pub sender_id: Int,
}

impl Message for SYNC {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct WRITE {
    #[serde(rename = "WRITE")]
    pub r: Int,
    pub v: Int,
    pub timestamp: Timestamp,
}

impl Message for WRITE {}


pub fn json_is_SYNC_Message(json: &str) -> bool {
    json.starts_with("{\"SYNC\":")
}

pub fn json_is_MSG_Message(json: &str) -> bool {
    json.starts_with("{\"WRITE\":")
}