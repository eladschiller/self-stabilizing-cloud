#[macro_use]
extern crate lazy_static;

mod arguments;

use std::fs;
use std::path::Path;
use std::process::Child;
use std::vec::Vec;

use commons::execution;
use commons::types::NodeId;

use crate::arguments::ARGUMENTS;
use std::string::ToString;
use commons::arguments::variant;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

fn main() {
    create_hosts_file();
    build_application();
    run_application();
}

fn create_hosts_file() {
    let hosts_file_string = hosts_file_string();
    let file_path = Path::new("hosts.txt");
    if file_path.exists() {
        if let Ok(existing_string) = fs::read_to_string(file_path) {
            if existing_string == hosts_file_string {
                return;
            }
        }

        fs::remove_file(file_path).expect("Could not remove existing hosts.txt file");
    }

    fs::write(file_path, hosts_file_string).expect("Could not write the new hosts.txt file.");
}

fn hosts_file_string() -> String {
    let mut string = String::new();
    let port_offset = 62000;

    for node_id in 1..ARGUMENTS.number_of_nodes + 1 {
        string.push_str(&format!(
            "{},127.0.0.1:{}\n",
            node_id,
            node_id + port_offset
        ));
    }

    string
}

fn build_application() {
    let build_command = format!(
        "cargo build {} --manifest-path ../application/Cargo.toml",
        ARGUMENTS.release_mode_string
    );
    execution::execute_local_command(&build_command)
        .wait()
        .expect("Build failed");
}

fn run_application() {
    let mut run_processes = Vec::new();
    let mut nr_of_writers = ARGUMENTS.number_of_writers;
    let mut nr_of_failing = ARGUMENTS.number_of_failing;
    let mut nr_of_crashing = ARGUMENTS.number_of_crashing;
    for node_id in 1..ARGUMENTS.number_of_nodes + 1 {
        let is_writer = nr_of_writers > 0;
        let is_failing = nr_of_failing > 0;
        let is_crashing = nr_of_crashing > 0;
        let run_process = run_single_application_instance(node_id, is_writer, is_failing, is_crashing);
        nr_of_writers += -1;
        nr_of_failing += -1;
        nr_of_crashing += -1;
        run_processes.push(run_process);
    }

    for run_process in run_processes.iter_mut() {
        run_process
            .wait()
            .expect("Could not wait for the run process.");
    }
}

fn run_single_application_instance(node_id: NodeId, is_writer: bool, is_failing: bool, is_crashing: bool) -> Child {
    let mut writer_s = "";
    let mut failing_s = "";
    let mut window_s = "".to_string();
    let mut crashing_s = "";
    if is_writer {
        writer_s = "-w";
    }
    if is_failing {
        failing_s = "-f";
    }
    if is_crashing {
        crashing_s = "-b"
    }
    if ARGUMENTS.window_size.is_some() {
        window_s = format!("-s {}", ARGUMENTS.window_size.unwrap());
    }

    let color = commons::arguments::color_from_node_id(node_id);
    let command = format!("cargo run {} --manifest-path ../application/Cargo.toml -- {} hosts.txt -c {:?} -l {} {} {} {} {} {} -v {:?} {} -d {}",
        ARGUMENTS.release_mode_string,
        node_id,
        color,
        ARGUMENTS.run_length_string,
        ARGUMENTS.print_client_operations_string,
        ARGUMENTS.record_evaluation_info_string,
        writer_s,
        failing_s,
        window_s,
        ARGUMENTS.variant,
        crashing_s,
        ARGUMENTS.delta,
    );

    execution::execute_local_command(&command)
}
