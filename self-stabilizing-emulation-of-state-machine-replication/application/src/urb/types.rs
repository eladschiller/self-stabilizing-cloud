use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use commons::types::{Int, NodeId};

// #[derive(Debug)]
// pub struct UrbBuffer<V> {
//     pub records: Vec<BufferRecord<V>>,
// }

// impl<V> UrbBuffer<V> {
//     pub fn new() -> Self {
//         UrbBuffer {
//             records: Vec::new(),
//         }
//     }

//     pub fn contains_tag(&self, tag: &Tag) -> bool {
//         for record in &self.records {
//             if record.tag == *tag {
//                 return true;
//             }
//         }
//         false
//     }
// }

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
pub struct Tag {
    pub id: NodeId,
    pub seq: Int,
}

#[derive(Debug)]
pub struct BufferRecord<V> {
    pub tag: Tag,
    pub msg: Option<V>,
    pub delivered: bool,
    pub recBy: HashSet<NodeId>,
    pub prevHB: Vec<Int>,
}

