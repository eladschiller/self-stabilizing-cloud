use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Copy, Hash)]
pub enum Variant {
    URB,
    SCD,
    COUNTER,
    SNAPSHOT,
}

impl FromStr for Variant {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "URB" {
            Ok(Variant::URB)
        } else if s == "SCD" {
            Ok(Variant::SCD)
        } else if s == "COUNTER" {
            Ok(Variant::COUNTER)
        } else if s == "SNAPSHOT" {
            Ok(Variant::SNAPSHOT)
        } else {
            panic!("Unknown variant.");
        }
    }
}
