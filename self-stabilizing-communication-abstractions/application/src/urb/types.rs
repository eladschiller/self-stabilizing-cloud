use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use commons::types::{Int, NodeId, Tag};
use bit_vec::BitVec;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct BufferRecord<V> {
    pub urb_tag: Option<Tag>,
    pub msg: Option<V>,
    pub delivered: bool,
    pub recBy: BitVec,
    pub recBy_trusted: BitVec,
    pub prevHB: Vec<Int>,
    pub urb_needed: bool,
    pub scd_needed: bool,
    pub meta: Option<String>,
    pub creation_instant: Option<Instant>,
}

