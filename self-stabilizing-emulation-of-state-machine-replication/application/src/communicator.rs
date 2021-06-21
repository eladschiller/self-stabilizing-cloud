use std::str;

pub mod udp_communicator;
pub mod dccp_communicator;
pub mod dccp;

pub trait CommunicatorDelegate {
    fn json_received(&self, json: &str);
}


