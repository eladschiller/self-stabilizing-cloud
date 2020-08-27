use serde::{Deserialize, Serialize};

pub type Int = i32;
pub type NodeId = Int;

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Tag {
    pub id: NodeId,
    pub seq: Int,
}
