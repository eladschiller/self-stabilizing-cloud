use serde::{Deserialize, Serialize};
use commons::types::Int;
use crate::terminal_output::printlnu;


pub trait Message: Serialize {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SYNC {
    #[serde(rename = "SYNC")]
    pub sender_identifier: Int,
}

impl Message for SYNC {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MSG {
    #[serde(rename = "MSG")]
    pub msg: String,
    pub sender_identifier: Int,
}

impl Message for MSG {}


pub fn json_is_SYNC_Message(json: &str) -> bool {
    json.starts_with("{\"SYNC\":")
}

pub fn json_is_MSG_Message(json: &str) -> bool {
    json.starts_with("{\"MSG\":")
}