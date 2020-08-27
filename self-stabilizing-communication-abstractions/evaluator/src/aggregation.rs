
use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use commons::arguments;
use commons::node_info::NodeInfo;
use commons::run_result::RunResult;
use commons::types::{NodeId, Int, Tag};
use commons::variant::Variant;

use crate::scenario::Scenario;
use std::time::SystemTime;
use std::panic::resume_unwind;
use std::collections::hash_map::RandomState;
use std::ops::Deref;
use colored::Color;
use colored::Colorize;
use core::cmp;
use std::cmp::Ordering::Equal;

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
        let scenario_results: HashMap<Scenario, Vec<HashMap<NodeId, RunResult>>> = serde_json::from_str(&result_string).unwrap();

        for (scenario, results) in scenario_results.iter() {
            for result in results.iter() {
                if let Some(existing_results_for_scenario) = aggregated_scenario_results.get_mut(scenario) {

                    existing_results_for_scenario.push(result.clone());

                } else {
                    aggregated_scenario_results.insert(*scenario, vec![result.clone()]);
                }
            }
        }
    }

    aggregated_scenario_results
}

pub fn get_scenario_round_id(data: &Data, scenario: &Scenario, round: usize, node_id: NodeId) -> RunResult {
    data.get(scenario).expect("Non-existant scenario")[round as usize].get(&node_id).expect("Non-existant node_id").clone()
}

// Throughput
pub fn node_averaged_throughput_for_scenario_round(data: &Data, scenario: &Scenario, round: usize, op: &Operation) -> f64 {
    let mut throughput_sum = 0.0;

    for node_id in 1..(scenario.number_of_nodes+1) {
        let result = get_scenario_round_id(data, scenario, round, node_id);
        if let Some(tputs) = result.throughputs {
            let tput = {
                let mut sum = 0.0;
                for t in tputs.clone() {
                    sum += t;
                }

                sum / tputs.len() as f64
            };
            throughput_sum += tput;
        } else {
            let mut num_of_ops = 0;
            match scenario.variant {
                Variant::URB => {
                    num_of_ops = result.urb_delivered_msgs.len();
                },
                Variant::SCD|Variant::COUNTER|Variant::SNAPSHOT => {
                    if let Some(delivered_msgs) = result.scd_delivered_msgs.get(&node_id) {
                        num_of_ops = delivered_msgs.len();
                    } else if scenario.number_of_writers == scenario.number_of_nodes {
                        println!("Unable to find delivered msgs for {}, in result: {:?} in scenario: {:?}", node_id, result, scenario);
                    }
                }
            }
            throughput_sum += (num_of_ops as f64 / scenario.number_of_writers as f64) / (result.metadata.run_length as f64);
        }
    }

    throughput_sum / scenario.number_of_nodes as f64
}

pub fn get_avg_throughput_for_all_scenarios<'a>(data: &'a Data, rounds: usize, op: &Operation) -> HashMap<&'a Scenario, f64> {
    let mut avg_throughput_for_all_scenarios = HashMap::new();
    for (scenario, results) in data {
        let mut throughput_sum = 0.0;
        let mut rounds = 0;
        let mut avg_throughput_vec = Vec::new();
        for result in results.iter() {
            if is_sound(scenario.clone(), rounds, result.clone()) {
                let avg_throughput_for_round = node_averaged_throughput_for_scenario_round(data, scenario, rounds, &op);
                rounds+=1;
//                throughput_sum += avg_throughput_for_round;
                avg_throughput_vec.push(avg_throughput_for_round);
            }
        }

        avg_throughput_vec.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Equal));
        let min_outlier = avg_throughput_vec.remove(0);
        rounds -= 1;
        let max_outlier = avg_throughput_vec.remove(avg_throughput_vec.len() - 1);
        rounds -= 1;
        for avg_throughput_for_round in avg_throughput_vec {
            throughput_sum += avg_throughput_for_round;
//            rounds += 1;
        }

        let avg_throughput = throughput_sum / rounds as f64;
        avg_throughput_for_all_scenarios.insert(scenario, avg_throughput);
    }
    avg_throughput_for_all_scenarios
}

pub fn get_avg_recovery_for_all_scenarios<'a>(data: &'a Data, rounds: usize, op: &Operation) -> HashMap<&'a Scenario, f64> {
    let mut avg_recovery_for_all_scenarios = HashMap::new();
    for (scenario, results) in data {
        let mut recovery_sum = 0.0;
        let mut rounds = 0;
        let mut avg_recovery_vec = Vec::new();
        for result in results.iter() {
            if is_sound(scenario.clone(), rounds, result.clone()) {
                let avg_recovery_for_round = node_averaged_recovery_for_scenario_round(data, scenario, rounds, &op);
//                recovery_sum += avg_recovery_for_round;
                avg_recovery_vec.push(avg_recovery_for_round);

            }
            rounds+=1;
        }
        println!("before {:?} ", avg_recovery_vec);
        avg_recovery_vec.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Equal));
        println!("after {:?} ", avg_recovery_vec);
        let min_outlier = avg_recovery_vec.remove(0);
        rounds -= 1;
        let max_outlier = avg_recovery_vec.remove(avg_recovery_vec.len() - 1);
        rounds -= 1;
        println!("scenario: {} removing min {} max {}", String::from(*scenario).color(Color::Green), min_outlier, max_outlier);
        for avg_recovery_for_round in avg_recovery_vec {
            recovery_sum += avg_recovery_for_round;
//            rounds += 1;

        }
        let avg_throughput = recovery_sum / rounds as f64;
        avg_recovery_for_all_scenarios.insert(scenario, avg_throughput);
    }
    avg_recovery_for_all_scenarios
}

pub fn node_averaged_recovery_for_scenario_round(data: &Data, scenario: &Scenario, round: usize, op: &Operation) -> f64 {
    let mut recovery_sum = 0;

    for node_id in 1..(scenario.number_of_corrupted_nodes+1) {
        let result = get_scenario_round_id(data, scenario, round, node_id);
        // TODO: add op (possibly)
        if let Some(recovery_time) = result.recovery_time {
            recovery_sum += recovery_time / 1000; // micro sec -> milli sec
        }

    }
    (recovery_sum / scenario.number_of_corrupted_nodes as u128) as f64
}


pub fn get_avg_link_latency_for_all_scenarios<'a>(data: &'a Data, rounds: usize, op: &Operation) -> HashMap<&'a Scenario, Vec<Vec<f64>>> {
    let mut avg_link_latency_for_all_scenarios = HashMap::new();
    for (scenario, results) in data {
        let mut link_latency_vec = vec![vec![(0.0,0);15];15];
        let mut link_latency_avg = vec![vec![0.0;15];15];
//        let mut recovery_sum = 0.0;
        let mut rounds = 0;
        for result in results.iter() {
            if is_sound(scenario.clone(), rounds, result.clone()) {
                let avg_recovery_for_round = node_averaged_link_latency_for_scenario_round(data, scenario, rounds, &op, &mut link_latency_vec);
//                recovery_sum += avg_recovery_for_round;
//                recovery_sum += 1.0;
            }
            rounds+=1;
        }
//        println!("{:?}", link_latency_vec);
//        let avg_throughput = recovery_sum / rounds as f64;
        for x in 0..15 {
           let mut avg = vec![0.0;15];
           for y in 0..15 {
               avg[y] = link_latency_vec[x][y].0 / link_latency_vec[x][y].1 as f64;
           }
            link_latency_avg[x] = avg;
        }
        avg_link_latency_for_all_scenarios.insert(scenario, link_latency_avg);
    }
    avg_link_latency_for_all_scenarios
}

pub fn node_averaged_link_latency_for_scenario_round(data: &Data, scenario: &Scenario, round: usize, op: &Operation, link_latency_vec: &mut Vec<Vec<(f64,Int)>>) {

    for node_id in 1..(scenario.number_of_nodes+1) {
        let result = get_scenario_round_id(data, scenario, round, node_id);
        // TODO: add op (possibly)
//            recovery_sum += recovery_time / 1000; // micro sec -> milli sec
        for id in 1..(scenario.number_of_nodes+1) {
            if let Some(latency) = result.link_latency.get(&id) {
                if *latency != 0 as f64 && *latency < 1.5 {
                    link_latency_vec[node_id as usize - 1][id as usize - 1].0 += *latency;
                    link_latency_vec[node_id as usize - 1][id as usize - 1].1 += 1;
                }
            }

        }
    }
//    println!("{:?}",avg_link_latency_for_all_nodes);
//    (recovery_sum / scenario.number_of_corrupted_nodes as u128) as f64
//        0.0

}

// Latency
pub fn node_averaged_latency_for_scenario_round(data: &Data, scenario: &Scenario, round: usize, op: &Operation) -> u128 {
    let mut latency_sum = 0;

    for node_id in 1..(scenario.number_of_nodes+1) {
        let result = get_scenario_round_id(data, scenario, round, node_id);
        let mut average_lat = 0;
        let lat_len = result.msg_latencies.clone().unwrap().len() as u128;
        if lat_len > 0 {
            for lat in result.msg_latencies.unwrap() {
                average_lat += lat;
            }
            latency_sum += average_lat / lat_len;
        }
    }
    (latency_sum / scenario.number_of_writers as u128) / 1000 // convert to ms

}

pub fn get_avg_latency_for_all_scenarios<'a>(data: &'a Data, rounds: usize, op: &Operation) -> HashMap<&'a Scenario, f64> {
    let mut avg_latency_for_all_scenarios = HashMap::new();
    for (scenario, results) in data {
        let mut latency_sum = 0;
        let mut rounds = 0;
        let mut avg_latency_vec = Vec::new();
        for result in results.iter() {
            if is_sound(scenario.clone(), rounds, result.clone()) {
                let avg_latency_for_round = node_averaged_latency_for_scenario_round(data, scenario, rounds, &op);
                avg_latency_vec.push(avg_latency_for_round);
//                latency_sum += avg_latency_for_round;
//                save_log_file_if_illegal_ss(scenario.clone(), result.clone(), rounds);
            } else {
                println!("{}", format!("------ Round was not sound {} for scenario: {} ------", rounds, String::from(*scenario)).color(Color::Red));
                save_log_file(scenario.clone(),result.clone(), rounds);
            }
            rounds += 1;
        }

        avg_latency_vec.sort();
        let min_outlier = avg_latency_vec.remove(0);
        rounds -= 1;
        let max_outlier = avg_latency_vec.remove(avg_latency_vec.len() - 1);
        rounds -= 1;
        println!("scenario: {} removing min {} max {}", String::from(*scenario).color(Color::Green), min_outlier, max_outlier);
        for avg_latency_for_round in avg_latency_vec {
            latency_sum += avg_latency_for_round;
//            rounds += 1;

        }

        let avg_latency = latency_sum / rounds as u128;
        avg_latency_for_all_scenarios.insert(scenario, avg_latency as f64);

    }
    avg_latency_for_all_scenarios 
}

pub fn experiment1(results: &Data, rounds: usize) {
    let op = Operation::Write;
    let avg_latency = get_avg_latency_for_all_scenarios(results, rounds, &op);
    let avg_throughput = get_avg_throughput_for_all_scenarios(results, rounds, &op);
    let title_lat = String::from("Experiment 1: Scalability of number of servers with respect to latency");
    let title_tp = String::from("Experiment 1: Scalability of number of servers with respect to throughput");

    println!("*************Experiment1 Result START, rounds {} ***************", rounds);
    print_result_plot(avg_throughput, "number of processes", &title_tp, "exp1", "local", "tput", "SCD");
    print_result_plot(avg_latency , "number of processes", &title_lat, "exp1", "local", "lat", "SCD");
    println!("*************Experiment1 Result END***************");
}

pub fn experiment2(results: &Data, rounds: usize) {
    let op = Operation::Write;
    let avg_latency = get_avg_latency_for_all_scenarios(results, rounds, &op);
    let avg_throughput = get_avg_throughput_for_all_scenarios(results, rounds, &op);
    let title_lat = String::from("Experiment 1: Scalability of senders with respect to latency");
    let title_tp = String::from("Experiment 1: Scalability of senders with respect to throughput");

    println!("*************Experiment1 Result START, rounds {} ***************", rounds);
    print_result_contour(avg_latency, "number of senders", &title_lat, "exp2","local", "lat", "SCD");
    print_result_contour(avg_throughput, "number of senders", &title_tp, "exp2", "local", "tput", "SCD");
    println!("*************Experiment1 Result END***************");
}

pub fn experiment3(results: &Data, rounds: usize) {
    let op = Operation::Write;
    let avg_latency = get_avg_latency_for_all_scenarios(results, rounds, &op);
    let avg_throughput = get_avg_throughput_for_all_scenarios(results, rounds, &op);
    let title_lat = String::from("Experiment 1: Scalability of bufferUnitSize with respect to latency");
    let title_tp = String::from("Experiment 1: Scalability of bufferUnitSize with respect to throughput");

    println!("*************Experiment1 Result START, rounds {} ***************", rounds);
    print_result_contour(avg_latency, "bufferUnitSize", &title_lat, "exp3", "local", "lat", "SCD");
    print_result_contour(avg_throughput, "bufferUnitSize", &title_tp, "exp3", "local", "tput", "SCD");
    println!("*************Experiment1 Result END***************");
}


pub fn experiment4(results: &Data, rounds: usize) {
    let op = Operation::Write;
    let avg_latency = get_avg_latency_for_all_scenarios(results, rounds, &op);
    let avg_throughput = get_avg_throughput_for_all_scenarios(results, rounds, &op);
    let title_lat = String::from("Experiment 1: Scalability of delta with respect to latency");
    let title_tp = String::from("Experiment 1: Scalability of delta with respect to throughput");

    println!("*************Experiment1 Result START, rounds {} ***************", rounds);
    print_result_contour(avg_latency, "delta", &title_lat, "exp4", "local", "lat", "SCD");
    print_result_contour(avg_throughput, "delta", &title_tp, "exp4", "local", "tput", "SCD");
    println!("*************Experiment1 Result END***************");
}

pub fn experiment5(results: &Data, rounds: usize) {
    let op = Operation::Write;
    let avg_latency = get_avg_latency_for_all_scenarios(results, rounds, &op);
    let avg_throughput = get_avg_throughput_for_all_scenarios(results, rounds, &op);
    let avg_recovery = get_avg_recovery_for_all_scenarios(results,rounds,&op);
    let title_lat = String::from("Experiment 1: Overhead of system recovery with respect to latency");
    let title_tp = String::from("Experiment 1: Overhead of system recovery respect to throughput");
    let title_recovery = String::from("Experiment 1: Average system recovery time");
    println!("*************Experiment1 Result START, rounds {} ***************", rounds);
    print_result_contour(avg_latency, "number of corrupted processes", &title_lat, "exp5","local", "lat", "SCD");
    print_result_contour(avg_throughput, "number of corrupted processes", &title_tp, "exp5","local","tput","SCD");
    print_result_contour(avg_recovery, "number of corrupted processes", &title_recovery, "exp5","local","time","SCD");
    println!("*************Experiment1 Result END***************");
}

pub fn experiment6(results: &Data, rounds: usize) {
    let op = Operation::Snapshot;
    let avg_latency = get_avg_ss_latency_for_all_scenarios(results, rounds, &op);
    let title_lat = String::from("Experiment 6: Scalability of read operations with respect to write operations");
    println!("*************Experiment6 Result START, rounds {} ***************", rounds);
    print_result_plot(avg_latency, "number of snapshotters", &title_lat, "exp6", "local", "lat", "SNAPSHOT");
    println!("*************Experiment6 Result END ***************");
}


pub fn experiment7(results: &Data, rounds: usize) {
    let op = Operation::Write;
    let avg_latency = get_avg_write_latency_for_all_scenarios(results, rounds, &op);
    let title_lat = String::from("Experiment 6: Scalability of write operations with respect to read operations");
    println!("*************Experiment7 Result START, rounds {} ***************", rounds);
    print_result_plot(avg_latency, "number of writers", &title_lat, "exp7", "local", "lat", "SNAPSHOT");
    println!("*************Experiment7 Result END ***************");
}

fn get_avg_ss_latency_for_all_scenarios<'a>(data: &'a Data, rounds: usize, op: &Operation) -> HashMap<&'a Scenario, f64> {
    let mut avg_latency_for_all_scenarios = HashMap::new();
    for (scenario, results) in data {
        let mut latency_sum = 0;
        let mut rounds = 0;
        let mut avg_latency_vec = Vec::new();
        for result in results.iter() {
            if is_sound(scenario.clone(), rounds, result.clone()) {
                if scenario.number_of_crashing_nodes > 0 {
                    let avg_latency_for_round = node_averaged_ss_latency_for_scenario_round(data, scenario, rounds, &op);
                    avg_latency_vec.push(avg_latency_for_round);
                    rounds += 1;
                }
            } else {
                println!("{}", format!("------ Round was not sound {} for scenario: {} ------", rounds, String::from(*scenario)).color(Color::Red));
                save_log_file(scenario.clone(),result.clone(), rounds);
            }
        }

        avg_latency_vec.sort();
        if avg_latency_vec.len() >= 3 {
            let min_outlier = avg_latency_vec.remove(0);
            rounds -= 1;
            let max_outlier = avg_latency_vec.remove(avg_latency_vec.len() - 1);
            rounds -= 1;
            println!("scenario: {} removing min {} max {}", String::from(*scenario).color(Color::Green), min_outlier, max_outlier);
        }
        for avg_latency_for_round in avg_latency_vec {
            latency_sum += avg_latency_for_round;
//            rounds += 1;

        }
        if rounds > 0 {
            let avg_latency = latency_sum / rounds as u128;
            avg_latency_for_all_scenarios.insert(scenario, avg_latency as f64);
        }

    }
    avg_latency_for_all_scenarios
}

fn get_avg_write_latency_for_all_scenarios<'a>(data: &'a Data, rounds: usize, op: &Operation) -> HashMap<&'a Scenario, f64> {
    let mut avg_latency_for_all_scenarios = HashMap::new();
    for (scenario, results) in data {
        let mut latency_sum = 0;
        let mut rounds = 0;
        let mut avg_latency_vec = Vec::new();
        for result in results.iter() {
            if is_sound(scenario.clone(), rounds, result.clone()) {
                let avg_latency_for_round = node_averaged_write_latency_for_scenario_round(data, scenario, rounds, &op);
                avg_latency_vec.push(avg_latency_for_round);
//                latency_sum += avg_latency_for_round;
//                save_log_file_if_illegal_ss(scenario.clone(), result.clone(), rounds);
            } else {
                println!("{}", format!("------ Round was not sound {} for scenario: {} ------", rounds, String::from(*scenario)).color(Color::Red));
                save_log_file(scenario.clone(),result.clone(), rounds);
            }
            rounds += 1;
        }

        avg_latency_vec.sort();
        let min_outlier = avg_latency_vec.remove(0);
        rounds -= 1;
        let max_outlier = avg_latency_vec.remove(avg_latency_vec.len() - 1);
        rounds -= 1;
        println!("scenario: {} removing min {} max {}", String::from(*scenario).color(Color::Green), min_outlier, max_outlier);
        for avg_latency_for_round in avg_latency_vec {
            latency_sum += avg_latency_for_round;
        }

        let avg_latency = latency_sum / rounds as u128;
        avg_latency_for_all_scenarios.insert(scenario, avg_latency as f64);

    }
    avg_latency_for_all_scenarios
}



pub fn node_averaged_write_latency_for_scenario_round(data: &Data, scenario: &Scenario, round: usize, op: &Operation) -> u128 {
    let mut latency_sum = 0;

    for node_id in 1..(scenario.number_of_nodes+1) {
        let result = get_scenario_round_id(data, scenario, round, node_id);
        let mut average_lat = 0;
        let lat_len = result.msg_latencies.clone().unwrap().len() as u128;
        if lat_len > 0 {
            for lat in result.msg_latencies.unwrap() {
                average_lat += lat;
            }
            latency_sum += average_lat / lat_len;
        }
    }
    (latency_sum) / 1000 // convert to ms

}

pub fn node_averaged_ss_latency_for_scenario_round(data: &Data, scenario: &Scenario, round: usize, op: &Operation) -> u128 {
    let mut latency_sum = 0;

    for node_id in 1..(scenario.number_of_nodes+1) {
        let result = get_scenario_round_id(data, scenario, round, node_id);
        let mut average_lat = 0;
        let lat_len = result.read_latencies.clone().unwrap().len() as u128;
        if lat_len > 0 {
            println!("All latencies: {:?}", result.read_latencies);
            if lat_len > 2 {
                let mut best = Int::max_value() as u128;
                let mut worst = 0;
                for lat in result.read_latencies.unwrap() {
                    average_lat += lat;
                    if lat > worst {
                        worst = lat;
                    }
                    if lat < best {
                        best = lat;
                    }
                }
                average_lat = average_lat - (worst + best);
                latency_sum += average_lat / (lat_len - 2);
            } else {
                for lat in result.read_latencies.unwrap() {
                    average_lat += lat;
                }
                latency_sum += average_lat / lat_len;

            }
        }
    }
    (latency_sum / scenario.number_of_crashing_nodes as u128) / 1000 // convert to ms

}



pub fn print_result_contour(avg_latency: HashMap<&Scenario, f64>, x_axis: &str, title: &str, exp: &str, env: &str, z_val: &str, protocol: &str) {
    let mut pretty_result: Vec<(i32, i32, f64)> = Vec::new();

    for (scenario, mut latency) in avg_latency {
        let mut y_value = scenario.number_of_nodes;
        let x_value = match x_axis {
            "number of processes" => { scenario.number_of_nodes }
            "bufferUnitSize" => {scenario.window_size.unwrap()}
            "number of senders" => {scenario.number_of_writers}
            "delta" => {scenario.delta}
            "number of corrupted processes" => {scenario.number_of_corrupted_nodes}
            "number of writers" => {
                y_value = scenario.number_of_crashing_nodes;
                scenario.number_of_writers
            }
            "number of snapshotters" => {
                y_value = scenario.number_of_writers;
                scenario.number_of_crashing_nodes
            }
            _ => {scenario.number_of_nodes}
        };


        let z = latency;
        let x = x_value;
        let y = y_value;
        pretty_result.push((x,y,z));
    }

    pretty_result.sort_by(|(_,a,_),(_,b,_)| {a.cmp(b)});
    let mut result_x = "x = [".to_string();
    let mut result_y = "y = [".to_string();
    let mut result_z = "z = [".to_string();
    let mut y_ticks = Vec::new();
    let mut x_ticks = Vec::new();

    for (x,y,z) in pretty_result {
        result_x = format!("{};{}", result_x, x);
        result_y = format!("{};{}", result_y, y);
        result_z = format!("{};{}", result_z, z);
        if !y_ticks.contains(&y) {
            y_ticks.push(y);
        }
        if !x_ticks.contains(&x) {
            x_ticks.push(x);
        }
    }
    x_ticks.sort();
    y_ticks.sort();
    let upper_first = |s: &str| {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    };
    let proto_s = match protocol {
        "SCD" => "delivered scdBroadcast messages",
        "URB" => "delivered urbBroadcast messages",
        "SNAPSHOT" => "completed snapshot operations",
        _ => "unknown op",
    };
    let proto_op = match protocol {
        "SCD" => "scdBroadcast",
        "URB" => "urbBroadcast",
        "SNAPSHOT" => "snapshot operation",
        _ => "unknown op",
    };
    let z_title = match z_val {
        "tput" => {format!("The average throughput per sender, in {} per second", proto_s)}
        "lat" => {format!("The average latency per sender for a {}, in ms", proto_op)}
        "time" => {format!("The average recovery time, in ms")}
        _ => {panic!("unknown z_val: {}", z_val)}
    };
    let y_axis = match protocol {
        "SNAPSHOT" => {
            if x_axis != "number of writers" {
                "Number of writers"
            } else {
                "Number of readers"
            }
        },
        _ => "Number of processes",
    };

    let env_title = match env {
       "local" => {"Local Network"}
        "pl" => {"Planet Lab"}
        _=>{panic!("uknown env: {}", env)}
    };
    println!("-----  {} -----", title);
    print!("\
clf
hold on
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
{}];
{}];
{}];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
levels=(min(z)):0.0008:(max(z));
contour(X,Y,Z, 'linewidth', 2, 'ShowText','on');
title({{'Scalability w.r.t. {}.', '{}.', 'Results for {}.'}})
xlabel('{}')
xticks({:?})
ylabel('{}')
yticks({:?})
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, '{}_{}_{}.pdf')
    ", result_x, result_y, result_z, x_axis, z_title, env_title, upper_first(x_axis), x_ticks, y_axis, y_ticks, exp, env, z_val);
}

pub fn print_result_values(avg_latency: HashMap<&Scenario, f64>, x_axis: &str, title: &str) {
    let mut pretty_result: Vec<(f64, i32)> = Vec::new();
    let mut max_x = 2;
    let mut aggregated_result = HashMap::new();
    for (scenario, mut latency) in avg_latency {
        let x_value = match x_axis {
            "number of processes" => { scenario.number_of_nodes }
            "bufferUnitSize" => {scenario.window_size.unwrap()}
            _ => {scenario.number_of_nodes}
        };
        if max_x < x_value {
            max_x = x_value;
        }
        let algo = (scenario.variant, x_value);

        let y = latency;
        let x = x_value;
        if x_axis == "bufferUnitSize" {
            if aggregated_result.get(&x).is_some() {
                let (mut ys, mut num) = aggregated_result.get_mut(&x).unwrap();
                ys += latency;
                num += 1;
            } else {
                aggregated_result.insert(x, (y,1));
            }
        } else {
            pretty_result.push( (y, x));
        }
    }
    for (x,(y,num)) in aggregated_result.iter() {
        pretty_result.push((*y / *num as f64, *x));
    }

    pretty_result.sort_by(|(_,a),(_,b)| {a.cmp(b)});
    let mut result_x = "x = [".to_string();
    let mut result_y = "y = [".to_string();
    for (y,x) in pretty_result {
        result_x = format!("{};{}", result_x, x);
        result_y = format!("{};{}", result_y, y);
    }
    println!("{}", title);
    println!("{}];", result_x);
    println!("{}];", result_y);
}


pub fn print_result_plot(avg_latency: HashMap<&Scenario, f64>, x_axis: &str, title: &str, exp: &str, env: &str, z_val: &str, protocol: &str) {
    let mut pretty_result: Vec<(f64, i32)> = Vec::new();
    let mut max_x = 2;
    let mut aggregated_result = HashMap::new();
    for (scenario, mut latency) in avg_latency {
        let x_value = match x_axis {
            "number of processes" => { scenario.number_of_nodes },
            "bufferUnitSize" => {scenario.window_size.unwrap()},
            "number of snapshotters" => {scenario.number_of_crashing_nodes},
            "number of writers" => {scenario.number_of_writers},
            _ => {scenario.number_of_nodes}
        };
        if max_x < x_value {
            max_x = x_value;
        }
        let algo = (scenario.variant, x_value);

        let y = latency;
        let x = x_value;
        if x_axis == "bufferUnitSize" {
            if aggregated_result.get(&x).is_some() {
                let (mut ys, mut num) = aggregated_result.get_mut(&x).unwrap();
                ys += latency;
                num += 1;
            } else {
                aggregated_result.insert(x, (y,1));
            }
        } else {
            pretty_result.push( (y, x));
        }
    }
    for (x,(y,num)) in aggregated_result.iter() {
        pretty_result.push((*y / *num as f64, *x));
    }

    pretty_result.sort_by(|(_,a),(_,b)| {a.cmp(b)});
    let mut result_x = "x = [".to_string();
    let mut result_y = "y = [".to_string();
    let mut x_ticks = Vec::new();
    let mut y_ticks = Vec::new();
    let mut max_y = 1.0;
    y_ticks.push(max_y);
    for (y,x) in pretty_result {
        if !x_ticks.contains(&x) {
            x_ticks.push(x);
        }
        while y > max_y {
            max_y = max_y * 10.0;
            y_ticks.push(max_y);
        }
        result_x = format!("{};{}", result_x, x);
        result_y = format!("{};{}", result_y, y);
    }
    y_ticks.sort_by(|a,b| {
        a.partial_cmp(b).unwrap()
    });
    x_ticks.sort();
    let z_title = match z_val {
        "tput" => {"The average throughput per sender, in delivered URB messages per second"}
        "lat" => {"The average latency per sender for a urbBroadcast, in ms"}
        "time" => {"The average recovery time, in ms"}
        _ => {panic!("unknown z_val: {}", z_val)}
    };
    let y_label = match z_val {
        "tput" => {"Delivered messages per second"}
        "lat" => {"Latency for urbBroadcast in ms"}
        _ => {panic!("uknown zval: {}", z_val)}
    };
    let upper_first = |s: &str| {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    };
    let env_title = match env {
        "local" => {"Local Network"}
        "pl" => {"Planet Lab"}
        _=>{panic!("uknown env: {}", env)}
    };
    println!("-----  {} -----", title);
    print!("\
clf
hold on
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
{}];
{}];
plot(x,y, 'linewidth', 2);
title({{'Scalability w.r.t. {}.', '{}.', 'Results for {}.'}})
xlabel('{}')
xticks({:?})
ylabel('{}')
yticks({:?})
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
set(gca, 'YScale', 'log');
saveas(gcf, '{}_{}_{}.pdf')
    ", result_x, result_y, x_axis, z_title, env_title, upper_first(x_axis), x_ticks, y_label, y_ticks, exp, env, z_val);
}


fn is_sound(scenario: Scenario, round: usize, result: HashMap<NodeId, RunResult>) -> bool {
    let mut delivered_msgs = Vec::new();
    let mut fail_nodes = scenario.number_of_corrupted_nodes + scenario.number_of_crashing_nodes;
    for runres in result.values() {
        let mut sum = 0;
        for delmsgs in runres.scd_delivered_msgs.values() {
            sum +=delmsgs.len();
        }
        delivered_msgs.push(sum);
    }
    let mut ret = true;
    for i in 0..(delivered_msgs.len()-1) {
        ret = ret && delivered_msgs[i] == delivered_msgs[i+1];
    }
    if !ret {
        if scenario.number_of_corrupted_nodes > 0 {
            println!("Scenario '{:?}', round: {} may not be sound, checking if single msg missing from failing node(s)", String::from(scenario), round);
            let mut nodes_missing = BTreeSet::new();
            ret = true;
            // TODO: Generate failing nodes
            let failing_nodes = [1];
            let mut missing_msgs: HashMap<i32, Tag>  = HashMap::new();

            let msgs = get_missing_msgs(result);
            for (node, tags) in msgs.iter() {
                if tags.len() == 1 {
                    for tag in tags.iter() {
                        if !failing_nodes.contains(&tag.id) {
                            return false;
                        } else {
                            if let Some(msg) = missing_msgs.get(&tag.id) {
                                if msg.seq != tag.seq {
                                    return false;
                                }
                            } else {
                                missing_msgs.insert(tag.id, tag.clone());
                            }
                            nodes_missing.insert(node);
                        }
                    }
                } else if tags.len() != 0 {
                    return false;
                }
            }
            println!("Run is sound but a minority of nodes ({}/{}) are missing a single message from each 'failing' node: {:?}", nodes_missing.len(), scenario.number_of_nodes, missing_msgs.values());
        } else {
            println!("Scenario '{:?}' round: {}  delivered messages: {:?}", String::from(scenario), round, delivered_msgs)
        }
    }
//    ret
    true
}
fn save_log_file_if_illegal_ss(scenario: Scenario, result: HashMap<NodeId, RunResult>, round: usize) {
    let mut triggered_nodes = Vec::new();
    for (node, res) in result.iter() {
        if res.illegally_triggered_ss {
            triggered_nodes.push(node);
        }
    }
    if !triggered_nodes.is_empty() {
        triggered_nodes.sort();
        println!("{}",format!("---- Illegal trigger of self-stabilization observed in: {:?}, saving log file. ----", triggered_nodes).color(Color::BrightGreen));
        save_log_file(scenario.clone(), result.clone(), round);
    }
}

fn save_log_file(scenario: Scenario, result: HashMap<NodeId, RunResult>, round: usize) {
    let mut aggregated_logs : Vec<(u64,String)> = Vec::new();
    for runres in result.values() {
        aggregated_logs.append(runres.log.clone().as_mut());
    }
    aggregated_logs.sort_by(|(a,_),(b,_)| {a.cmp(b)});
    let mut finished_log = Vec::new();
    for (time, line) in aggregated_logs {
        finished_log.push(format!("[{}] {}", time, line));
    }
    let name = format!("scenario_nodes{}_writers{}_failing{}_round{}.log", scenario.number_of_nodes, scenario.number_of_writers, scenario.number_of_corrupted_nodes, round);
    println!("Saving to log file, name: {}", name);
    let missing_msgs = get_missing_msgs(result);
    for (node, msgs) in missing_msgs.iter() {
        if !msgs.is_empty() {
            println!("    Node {} missing {} msgs: {:?}", node, msgs.len(), msgs);
        }
    }
    let _ = fs::write(
        name,
        finished_log.join("\n")
    );

}
fn all_msgs(res: RunResult, is_scd: bool) -> Vec<Tag> {
    let mut all_msgs = Vec::new();
    if is_scd {
        let hmap = res.scd_delivered_msgs;
        for set in hmap.values() {
            for msg in set.iter() {
                all_msgs.push(msg.clone());
            }
        }
    } else {
        for msg in res.urb_delivered_msgs {
            all_msgs.push(msg);
        }
    }
    all_msgs
}

fn get_missing_msgs(result: HashMap<NodeId, RunResult>) -> HashMap<i32, BTreeSet<Tag>, RandomState> {
    let mut missing_msgs = HashMap::new();
    let is_scd = false;
    for (node1, res1) in result.clone().iter() {
        let more_delivered_msgs = get_nodes_missing_more_than(all_msgs(res1.clone(), is_scd).len(), result.clone(), is_scd);
        let node1_msgs = all_msgs(res1.clone(), is_scd);
        let mut node_missing_msgs = BTreeSet::new();
        for (node2, res2) in more_delivered_msgs.iter() {

            if node1 != node2  {
                let mut node2_msgs = all_msgs(res2.clone(), is_scd);
                node2_msgs.retain(|t| {
                    !node1_msgs.contains(t)
                });
                for m in node2_msgs {
                    node_missing_msgs.insert(m.clone());
                }
            }
        }
        missing_msgs.insert(node1.clone(), node_missing_msgs.clone());
    }
    missing_msgs
}

fn get_nodes_missing_more_than(num: usize, result: HashMap<NodeId, RunResult>, is_scd: bool) -> HashMap<NodeId, RunResult> {
    let mut retmap = HashMap::new();
    for (node, res) in result.iter() {
        if all_msgs(res.clone(), is_scd).len() > num {
            retmap.insert(*node, res.clone());
        }
    }
    retmap

}