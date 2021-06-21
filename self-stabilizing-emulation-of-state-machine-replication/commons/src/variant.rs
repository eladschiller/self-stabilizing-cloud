use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Copy, Hash)]
pub enum Variant {
    URB,
    Algorithm2,
    Algorithm3,
    Algorithm4,
}

impl FromStr for Variant {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "TOURB" {
            Ok(Variant::URB)
        } else if s == "Algorithm2" {
            Ok(Variant::Algorithm2)
        } else if s == "Algorithm3" {
            Ok(Variant::Algorithm3)
        } else if s == "Algorithm4" {
            Ok(Variant::Algorithm4)
        } else {
            panic!("Unknown variant.");
        }
    }
}
