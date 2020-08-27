use serde::{Deserialize, Serialize};
use commons::types::{Tag, Int, NodeId};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct Entry {
    pub msg: String,
    pub tag: Tag,
    pub cl: VectorClock,
    pub delivered: bool,
    pub urb_tag: Tag,
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Debug)]
pub struct VectorClock {
    pub vc: Vec<Int>,
}

impl VectorClock {
    pub fn new(size: usize, val: Int) -> Self {
        VectorClock {
            vc: vec![val; size],
        }
    }

    pub fn get(&self, node_id: NodeId) -> Int {
        self.vc[node_id as usize - 1]
    }

    pub fn set(&mut self, node_id: NodeId, val: Int) {
        self.vc[node_id as usize - 1] = val
    }

    pub fn inner(&self) -> &Vec<Int> {
        &self.vc
    }
}
