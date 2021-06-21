use serde::{Deserialize, Serialize};

use commons::types::Int;
use commons::variant::Variant;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Copy, Clone)]
#[serde(into = "String")]
#[serde(from = "String")]
pub struct Scenario {
    pub number_of_nodes: Int,
    pub number_of_snapshotters: Int,
    pub number_of_writers: Int,
    pub variant: Variant,
    pub delta: Int,
}
// This struct is serialized in String because it's used as a key. And json only allows string keys.

impl Scenario {
    pub fn new(
        number_of_nodes: Int,
        number_of_snapshotters: Int,
        number_of_writers: Int,
        variant: Variant,
        delta: Int
    ) -> Scenario {
        Scenario {
            number_of_nodes: number_of_nodes,
            number_of_snapshotters: number_of_snapshotters,
            number_of_writers: number_of_writers,
            variant: variant,
            delta: delta,
        }
    }
}

impl From<Scenario> for String {
    fn from(scenario: Scenario) -> String {
        format!(
            "Scenario,{},{},{},{:?},{}",
            scenario.number_of_nodes,
            scenario.number_of_snapshotters,
            scenario.number_of_writers,
            scenario.variant,
            scenario.delta
        )
    }
}

impl From<String> for Scenario {
    fn from(string: String) -> Scenario {
        let components: Vec<&str> = string.split(",").collect();
        let scenario_name = components[0];
        let number_of_nodes = components[1]
            .parse()
            .expect("Could not parse number_of_nodes");
        let number_of_snapshotters = components[2]
            .parse()
            .expect("Could not parse number_of_snapshotters");
        let number_of_writers = components[3]
            .parse()
            .expect("Could not parse number_of_writers");
        let variant = components[4].parse().expect("Could not parse variant");

        let delta = components[5].parse().expect("Could not parse delta");

        if scenario_name != "Scenario" {
            panic!("Scenario name doesn't match.");
        }

        Scenario {
            number_of_nodes: number_of_nodes,
            number_of_snapshotters: number_of_snapshotters,
            number_of_writers: number_of_writers,
            variant: variant,
            delta: delta,
        }
    }
}
