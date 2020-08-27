use std::net::SocketAddr;

use crate::types::NodeId;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct NodeInfo {
    pub node_id: NodeId,
    pub socket_addr: SocketAddr,
    pub key_path: String,
    pub username: String,
    pub script_path: String,
    pub is_writer: bool,
    pub is_failing: bool,
    pub is_crashing: bool,
}

impl NodeInfo {
    pub fn ip_addr_string(&self) -> String {
        format!("{}", self.socket_addr.ip())
    }
}
