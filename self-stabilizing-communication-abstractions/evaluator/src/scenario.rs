use serde::{Deserialize, Serialize};

use commons::types::Int;
use commons::variant::Variant;
use std::collections::HashSet;
use commons::node_info::NodeInfo;
use commons::constants::WINDOW_SIZE;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Copy, Clone)]
#[serde(into = "String")]
#[serde(from = "String")]
pub struct Scenario {
    pub number_of_nodes: Int,
    pub number_of_corrupted_nodes: Int,
    pub number_of_crashing_nodes: Int,
    pub number_of_writers: Int,
    pub delta: Int,
    pub window_size: Option<Int>,
    pub variant: Variant,
}
// This struct is serialized in String because it's used as a key. And json only allows string keys.

impl Scenario {
    pub fn new(
        number_of_nodes: Int,
        number_of_writers: Int,
        number_of_corrupted_nodes: Int,
        variant: Variant,
        number_of_crashing_nodes: Int,
        delta: Int,
    ) -> Scenario {
        Scenario {
            number_of_nodes,
            number_of_corrupted_nodes: number_of_corrupted_nodes,
            number_of_crashing_nodes: number_of_crashing_nodes,
            number_of_writers,
            delta: delta,
            window_size: None,
            variant,
        }
    }
    pub fn window_size(self) -> Int {
        if self.window_size.is_some() {
            return self.window_size.unwrap();
        }
        WINDOW_SIZE
    }
}

impl From<Scenario> for String {
    fn from(scenario: Scenario) -> String {
        let s = format!(
            "Scenario,{},{:?},{},{},{},{}",
            scenario.number_of_nodes,
            scenario.variant,
            scenario.number_of_writers,
            scenario.number_of_corrupted_nodes,
            scenario.number_of_crashing_nodes,
            scenario.delta,
        );
        if scenario.window_size.is_some() {
           return format!("{},{}", s, scenario.window_size.unwrap());
        }
        s
    }
}

impl From<String> for Scenario {
    fn from(string: String) -> Scenario {
        let components: Vec<&str> = string.split(",").collect();
        let scenario_name = components[0];
        let number_of_nodes = components[1]
            .parse()
            .expect("Could not parse number_of_nodes");
        let variant = components[2].parse().expect("Could not parse variant");
        let mut number_of_writers = number_of_nodes;
        if components.len() > 3 {
            if let Ok(nr_writers) = components[3].parse() {
                number_of_writers = nr_writers;
            }
        }
        let mut number_of_failing = 0;
        if components.len() > 4 {
            if let Ok(nr_failing) = components[4].parse() {
                number_of_failing = nr_failing;
            }
        }

        let mut number_of_crashing = 0;
        if components.len() > 5 {
            if let Ok(nr_crashing) = components[5].parse() {
                number_of_crashing = nr_crashing;
            }
        }

        let mut delta = 50;
        if components.len() > 6 {
            if let Ok(delta_p) = components[6].parse() {
                delta = delta_p;
            }
        }

        let mut window_size = None;
        if components.len() > 7 {
            if let Ok(window_s) = components[7].parse() {
                window_size = Some(window_s);
            }
        }

        if scenario_name != "Scenario" {
            panic!("Scenario name doesn't match.");
        }

        Scenario {
            number_of_nodes,
            number_of_corrupted_nodes: number_of_failing,
            number_of_crashing_nodes: number_of_crashing,
            number_of_writers,
            variant,
            window_size,
            delta: delta
        }
    }
}
