use serde::{Deserialize, Serialize};
use commons::types::Int;
use crate::terminal_output::printlnu;
use crate::urb::messages::Message;


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SYNC {
    #[serde(rename = "SYNC")]
    pub sender_id: Int,
}

impl Message for SYNC {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PLUS {
    #[serde(rename = "PLUS")]
    pub i: Int,
}

impl Message for PLUS {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MINUS {
    #[serde(rename = "MINUS")]
    pub i: Int,
}

impl Message for MINUS {}


pub fn json_is_SYNC_Message(json: &str) -> bool {
    json.starts_with("{\"SYNC\":")
}

pub fn json_is_PLUS_Message(json: &str) -> bool {
    json.starts_with("{\"PLUS\":")
}

pub fn json_is_MINUS_Message(json: &str) -> bool {
    json.starts_with("{\"MINUS\":")
}
