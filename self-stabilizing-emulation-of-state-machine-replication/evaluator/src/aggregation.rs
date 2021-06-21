
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use commons::arguments;
use commons::node_info::NodeInfo;
use commons::run_result::RunResult;
use commons::types::{NodeId, Int};
use commons::variant::Variant;

use crate::scenario::Scenario;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod example_data;

pub type Data = HashMap<Scenario, Vec<HashMap<NodeId, RunResult>>>;
const LINE_SPECS: [&'static str; 7] = ["-+r", "-og", "-*b", "-^c", "-xm", "-sr", "-dk"];

#[derive(PartialEq, Debug)]
pub enum Operation{
    Snapshot,
    Write,
}

pub fn result_map_from_result_strings(result_strings: HashSet<String>) -> Data {
    let mut aggregated_scenario_results: HashMap<Scenario, Vec<HashMap<NodeId, RunResult>>> = HashMap::new();
    for result_string in result_strings {
        let scenario_results: HashMap<Scenario, HashMap<NodeId, RunResult>> = serde_json::from_str(&result_string).unwrap();

        for (scenario, result) in scenario_results.iter() {
            if let Some(existing_results_for_scenario) = aggregated_scenario_results.get_mut(scenario) {

                existing_results_for_scenario.push(result.clone());

            } else {
                aggregated_scenario_results.insert(*scenario, vec![result.clone()]);
            }
        }
    }

    aggregated_scenario_results
}

pub fn get_scenario_round_id(data: &Data, scenario: &Scenario, round: usize, node_id: NodeId) -> RunResult {
    data.get(scenario).expect("Non-existant scenario")[round as usize].get(&node_id).expect("Non-existant node_id").clone()
}

pub fn node_averaged_latency_for_scenario_round(data: &Data, scenario: &Scenario, round: usize, op: &Operation) -> f64 {
    let mut latency_sum = 0.0;

    for node_id in 1..(scenario.number_of_nodes+1) {
        let result = get_scenario_round_id(data, scenario, round, node_id);

        let number_of_ops;
        if *op == Operation::Write && result.metadata.is_writer {
            number_of_ops = result.write_ops;
        } else if *op == Operation::Snapshot && result.metadata.is_snapshotter {
            number_of_ops = result.snapshot_ops;
        } else {
            continue;
        }
        // Converting to ms.
        let latency = (result.metadata.run_length * 1000) as f64 / (number_of_ops as f64);
        latency_sum += latency;
    }
    match op {
        Operation::Write => latency_sum as f64 / (scenario.number_of_writers as f64),
        Operation::Snapshot => latency_sum as f64 / (scenario.number_of_snapshotters as f64),
    }
}

pub fn get_avg_latency_for_all_scenarios<'a>(data: &'a Data, rounds: usize, op: &Operation) -> HashMap<&'a Scenario, f64> {
    let mut avg_latency_for_all_scenarios = HashMap::new();
    for (scenario, result) in data {
        let mut latency_sum = 0.0;
        for round in 0..rounds {
            let avg_latency_for_round = node_averaged_latency_for_scenario_round(data, scenario, round, &op);
            latency_sum += avg_latency_for_round;
        }
        let avg_latency = latency_sum / rounds as f64;
        avg_latency_for_all_scenarios.insert(scenario, avg_latency);
    }
    avg_latency_for_all_scenarios 
}

pub fn experiment1(results: &Data, rounds: usize) {
    let op = Operation::Write;
    let avg_latency = get_avg_latency_for_all_scenarios(results, rounds, &op);
    let title = String::from("Experiment 1: Scalability of write operations with respect to write operations");
    println!("*************Experiment1 Result START***************");
    print_latency_matlab_code(avg_latency,"writers", &title, &op);
    println!("*************Experiment1 Result END***************");
}

pub fn print_latency_matlab_code(avg_latency: HashMap<&Scenario, f64>, x_axis:&str, title: &str, op: &Operation) {
    let mut pretty_result: HashMap<(Variant, i32), Vec<f64>> = HashMap::new();
    for (scenario, latency) in avg_latency {
        let algo = (scenario.variant, scenario.delta);
        let latency_index = match x_axis {
            "snapshotters" => scenario.number_of_snapshotters as usize / 5 as usize,
            "writers" => scenario.number_of_writers as usize / 5 as usize,
            _ => panic!("Unrecognized x_axis."),
        }; 
        match pretty_result.get_mut(&algo) {
            Some(array) => {
                array[latency_index] = latency;
            }
            None => {
                let mut array = Vec::new();
                for i in 0..4 {
                    array.push(-1.0);
                }
                array[latency_index] = latency;
                pretty_result.insert(algo, array);   
            }
        }
    }
    println!("x = [1, 5, 10, 15];");
    println!("figure\nhold on\n");
    println!("title('{}', 'fontsize', 18)", title);
    println!("xlabel('Number of {}', 'fontsize', 18)", x_axis);
    println!("ylabel('{:?} latency [ms]', 'fontsize', 18)", op);
    println!("ylim([0 inf])");
    println!("set(gca,'FontSize', 15)");

    let mut legend = String::new();
    let mut lines = Vec::new();
    for (algo, latency_array) in pretty_result {
        let line_name = match algo.0 {
            Variant::Algorithm4 => format!("{:?}_{}", algo.0, algo.1),
            _ => format!("{:?}", algo.0),
        };
        lines.push((line_name, latency_array));
    }

    lines.sort_by(|a, b| a.0.cmp(&b.0));

    // Ugly fix for printing in a nice order.
    let length = lines.len();
    lines.swap(length - 1, length - 2);

    for ((line_name, latency_array), line_spec) in lines.iter().zip(&LINE_SPECS) {
        println!("{} = {:?};", line_name, latency_array);
        println!("plot(x, {}, '{}')", line_name, line_spec);
        legend.push_str(&format!("'{}', ", line_name.replace("_","-")));
    }
    println!("legend({{ {} }}, 'Location', 'northeast')\n", legend);
}

pub fn experiment2(results: &Data, rounds: usize) {
    let op = Operation::Write;
    let avg_latency = get_avg_latency_for_all_scenarios(results, rounds, &op);

    println!("*************Experiment2 Result START***************");
    let title = String::from("Experiment 2: Scalability of write operations with respect to snapshot operations");
    print_latency_matlab_code(avg_latency, "snapshotters", &title, &op);
    println!("*************Experiment2 Result END***************");
}

pub fn experiment3(results: &Data, rounds: usize) {
    let op = Operation::Snapshot;
    let avg_latency = get_avg_latency_for_all_scenarios(results, rounds, &op);

    println!("*************Experiment3 Result START***************");
    let title = String::from("Experiment 3: Scalability of snapshot operations with respect to snapshot operations");
    print_latency_matlab_code(avg_latency, "snapshotters", &title, &op);
    println!("*************Experiment3 Result END***************");

}

pub fn experiment4(results: &Data, rounds: usize) {
    let op = Operation::Snapshot;
    let avg_latency = get_avg_latency_for_all_scenarios(results, rounds, &op);

    println!("*************Experiment4 Result START***************");
    let title = String::from("Experiment 4: Scalability of snapshot operations with respect to write operations");
    print_latency_matlab_code(avg_latency, "writers", &title, &op);
    println!("*************Experiment4 Result END***************");
}

pub fn experiment5(results: &Data, rounds: usize) {
    let snapshot_op = Operation::Snapshot;
    let write_op = Operation::Write;
    let avg_snapshot_latency = get_avg_latency_for_all_scenarios(results, rounds, &snapshot_op); 
    let avg_write_latency = get_avg_latency_for_all_scenarios(results, rounds, &write_op); 

    println!("*************Experiment5 Result START***************");
    print_experiment5_matlab_code(&avg_snapshot_latency, &avg_write_latency);
    println!("*************Experiment5 Result END***************");
}

pub fn print_experiment5_matlab_code(avg_snapshot_latency: &HashMap<&Scenario, f64>, avg_write_latency: &HashMap<&Scenario, f64>) {
    let mut x_axis: Vec<Int> = (0..1100).step_by(100).collect();
    let mut snapshot_latency_array: Vec<f64> = vec![0.0; 11];
    let mut write_latency_array: Vec<f64> = vec![0.0; 11];
    for (scenario, snapshot_latency) in avg_snapshot_latency {
        let index = scenario.delta / 100;    
        snapshot_latency_array[index as usize] = *snapshot_latency;
    }

    for (scenario, write_latency) in avg_write_latency {
        let index = scenario.delta / 100;    
        write_latency_array[index as usize] = *write_latency;
    }

    println!("x = {:?};", x_axis);
    println!("figure\nhold on\n");
    println!("title('{}', 'fontsize', 18)", "Experiment 5: The trade-off parameter \\delta''s effect on operation latencies");
    println!("xlabel('\\delta', 'fontsize', 18 )");
    println!("ylabel('Operation latency [ms]', 'fontsize', 18)");

    println!("ylim([0 inf])");
    println!("set(gca,'FontSize', 15)");

    println!("write_latency = {:?};", write_latency_array);
    println!("snapshot_latency = {:?};", snapshot_latency_array);
    
    println!("plot(x, write_latency, '-*r');");
    println!("plot(x, snapshot_latency, '-ob');");

    println!("legend({{ 'Write latency', 'Snapshot latency' }}, 'Location', 'northwest')\n");
}
/*
TODOs:
- Function to get average of values, except highest and lowest.
- Function to write a matlab array string.
- Function to write an entire matlab file for all experiments.
- Function that calculates node_average_write_latency for specified number of writers, multiple values from an array.
- Functions for doing calculations on messages sent/received.
*/
