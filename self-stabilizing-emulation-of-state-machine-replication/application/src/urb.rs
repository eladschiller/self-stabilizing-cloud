pub mod hbfd;
pub mod thetafd;
pub mod urb_node;
pub mod types;
mod messages;

use std::collections::HashSet;
use std::sync::MutexGuard;
use std::time::Duration;

use commons::types::{Int, NodeId};
use commons::run_result::RunResult;

pub const THETA_HB_TIMEOUT: Duration = Duration::from_millis(1000);
pub const HBFD_HB_TIMEOUT: Duration = Duration::from_millis(1000);

pub trait NodeDelegate {
    fn send_json_to(&self, json: &str, receiver: NodeId);

    fn node_id(&self) -> NodeId;
    fn node_ids(&self) -> &HashSet<NodeId>;
    fn number_of_nodes(&self) -> Int;

    fn record_evaluation_info(&self) -> bool;
    fn run_result(&self) -> MutexGuard<RunResult>;
}
