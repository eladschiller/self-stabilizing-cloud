#![allow(dead_code, unused_variables, unused_imports, unused_mut)]

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process;

use rand::thread_rng;
use rand::seq::SliceRandom;
use ctrlc;
use serde_json;

use commons::execution;
use commons::node_info::NodeInfo;
use commons::run_result::RunResult;
use commons::types::NodeId;
use commons::variant::Variant;

mod arguments;
mod scenario;
mod aggregation;

use arguments::*;
use scenario::*;

fn main() {
    let arguments: &Arguments = &ARGUMENTS;

    ctrlc::set_handler(move || {
        // It seems that when ctrl-c is pressed in the evaluator,
        // somehow, the ctrl-c code of remote_starter is run.
        // This is wanted behavoir, so that the remote_starter
        // can exit the processes running on the remote machines.
        process::exit(0);
    })
    .expect("Could not set the CTRL+C handler.");

    match arguments {
        Arguments::Install(arguments) => run_install_subcommand(arguments),
        Arguments::Gather(arguments) => run_gather_subcommand(arguments),
        Arguments::Aggregate(arguments) => run_aggregate_subcommand(arguments),
    };
}

fn run_install_subcommand(arguments: &InstallArguments) {
    let hosts_file = &arguments.hosts_file;
    if arguments.randomize {
        randomize_hosts_file(hosts_file);
    }
    let optimize_string = &arguments.optimize_string;
    let command = format!(
        "cargo run --manifest-path ../remote_starter/Cargo.toml -- {} -v Algorithm1 -i {}",
        hosts_file, optimize_string
    );

    execution::execute_local_command(&command)
        .wait()
        .expect("Error waiting for the execution of the install command.");
}

fn randomize_hosts_file(file_path: &str) {
    let string = fs::read_to_string(file_path).expect("Unable to read the hosts file.");
    let mut hosts_vec: Vec<String> = string.lines().map(|s| s.to_string()).collect();
    hosts_vec.shuffle(&mut thread_rng());
    for (i, node_string) in hosts_vec.iter_mut().enumerate() {
        let comma_offset = node_string.find(',').unwrap();
        node_string.replace_range(..comma_offset, &format!("{}", i + 1));
    }
    let mut shuffled_string = String::new();
    for host in hosts_vec {
        shuffled_string.push_str(&format!("{}\n", host));
    }
    fs::write(file_path, shuffled_string).unwrap();
}

fn run_gather_subcommand(arguments: &GatherArguments) {
    let result_file_path = &arguments.result_file_path;
    create_result_file_if_not_existing(result_file_path);
    let mut results = read_result_file(result_file_path);

    for scenario in &arguments.scenarios {
        run_scenario_if_not_already_run_and_insert_result(&scenario, arguments, &mut results);
        save_results_to_file(&results, result_file_path);
    }

    let mut trimmed_results = HashMap::new();
    for (scenario, result) in results.iter() {
        if arguments.scenarios.contains(scenario) {
            trimmed_results.insert(*scenario, result.clone());
        }
    }

    save_results_to_file(&trimmed_results, result_file_path);
}

fn create_result_file_if_not_existing(result_file_path: &Path) {
    if result_file_path.is_dir() {
        fs::remove_dir_all(result_file_path).expect("Could not remove a result file directory.");
    }

    if !result_file_path.is_file() {
        let empty_result: HashMap<Scenario, HashMap<NodeId, RunResult>> = HashMap::new();
        let json = serde_json::to_string(&empty_result)
            .expect("Could not serialize the empty result set.");
        fs::write(result_file_path, json).expect("Could not write the empty result file.");
    }
}

fn read_result_file(result_file_path: &Path) -> HashMap<Scenario, HashMap<NodeId, RunResult>> {
    let json = fs::read_to_string(result_file_path).expect("Could not read the result file.");
    serde_json::from_str(&json).expect("Could not parse the result file.")
}

fn run_scenario_if_not_already_run_and_insert_result(
    scenario: &Scenario,
    arguments: &GatherArguments,
    results: &mut HashMap<Scenario, HashMap<NodeId, RunResult>>,
) {
    if !results.contains_key(&scenario) {
        let result = run_scenario(scenario, arguments);
        results.insert(*scenario, result);
    }
}

fn run_scenario(scenario: &Scenario, arguments: &GatherArguments) -> HashMap<NodeId, RunResult> {
    loop {
        match run_scenario_once(scenario, arguments) {
            Some(result) => return result,
            None => {}
        }
    }
}

fn run_scenario_once(
    scenario: &Scenario,
    arguments: &GatherArguments,
) -> Option<HashMap<NodeId, RunResult>> {
    execute_command_for_scenario_and_arguments(scenario, arguments);

    let results_for_this_scenario =
        collect_results_from_scenario_and_arguments(scenario, arguments);

    match results_for_this_scenario {
        CollectResult::Success(results_for_this_scenario) => {
            return Some(results_for_this_scenario)
        }
        CollectResult::Failure(soundness_violator) => {
            println!(
                "The result for {:?} is not sound, violated by {}.",
                scenario, soundness_violator
            );
            return None;
        }
    }
}

fn execute_command_for_scenario_and_arguments(scenario: &Scenario, arguments: &GatherArguments) {
    let mut command = format!("cargo run --manifest-path ../remote_starter/Cargo.toml -- {} -v {:?} -s {} -w {} -e {} -l {} {}",
            arguments.hosts_file,
            scenario.variant,
            scenario.number_of_snapshotters,
            scenario.number_of_writers,
            arguments.optimize_string,
            arguments.run_length_string,
            arguments.print_client_operations_string);

    if scenario.delta != 0 {
        command.push_str(&format!(" -d {}", scenario.delta));
    }
    execution::execute_local_command(&command)
        .wait()
        .expect("Could not wait for the gather command for remote_starter.");
}

fn collect_results_from_scenario_and_arguments(
    scenario: &Scenario,
    arguments: &GatherArguments,
) -> CollectResult {
    let mut results_for_this_scenario = HashMap::new();

    for node_info in &arguments.node_infos {
        let run_result = collect_result_for_node_info(&node_info);

        if run_result.is_sound(scenario.number_of_nodes, node_info.node_id) {
            results_for_this_scenario.insert(node_info.node_id, run_result);
        } else {
            return CollectResult::Failure(node_info.node_id);
        }
    }

    return CollectResult::Success(results_for_this_scenario);
}

enum CollectResult {
    Success(HashMap<NodeId, RunResult>),
    Failure(NodeId),
}

fn collect_result_for_node_info(node_info: &NodeInfo) -> RunResult {
    let file_name = commons::arguments::run_result_file_name_from_node_id(node_info.node_id);
    execution::scp_copy_of_remote_source_path_to_local_destination_path(
        &format!("application/{}", file_name),
        &file_name,
        &node_info,
    )
    .wait()
    .expect("Could not wait for the scp download of a result file.");

    let json = fs::read_to_string(&file_name).expect("Could not read a run result.");
    serde_json::from_str(&json).expect("Could not parse a run result.")
}

fn save_results_to_file(
    results: &HashMap<Scenario, HashMap<NodeId, RunResult>>,
    result_file_path: &Path,
) {
    let json = serde_json::to_string(&results).expect("Could not serialize the result.");
    fs::write(result_file_path, &json).expect("Could not write the result file.");
}

fn run_aggregate_subcommand(arguments: &AggregateArguments) {
    let results = &arguments.run_results;
    match &arguments.experiment {
        Experiment::Experiment1 => aggregation::experiment1(results, arguments.rounds),
        Experiment::Experiment2 => aggregation::experiment2(results, arguments.rounds),
        Experiment::Experiment3 => aggregation::experiment3(results, arguments.rounds),
        Experiment::Experiment4 => aggregation::experiment4(results, arguments.rounds),
        Experiment::Experiment5 => aggregation::experiment5(results, arguments.rounds),
        
    };
    // let s = serde_json::to_string(&results).unwrap();
    // println!("{}", s);
}
