use std::collections::HashSet;
use std::fs;
use std::net::ToSocketAddrs;

use clap::{Arg, ArgMatches};
use colored::Color;
use colored::Color::*;

use crate::node_info::NodeInfo;
use crate::types::{Int, NodeId};
use crate::variant::Variant;

pub fn hosts_file(help_text: &'static str) -> Arg<'static, 'static> {
    Arg::with_name("hosts-file")
        .required(true)
        .takes_value(true)
        .help(help_text)
}

pub fn hosts_file_from_matches(matches: &ArgMatches<'static>) -> String {
    matches
        .value_of("hosts-file")
        .expect("hosts-file not found in matches.")
        .to_string()
}

pub fn node_infos_from_matches(matches: &ArgMatches<'static>) -> HashSet<NodeInfo> {
    let hosts_file_path = hosts_file_from_matches(matches);
    let string = fs::read_to_string(hosts_file_path).expect("Unable to read the hosts file.");
    node_infos_from_string(string)
}

pub fn node_infos_from_string(string: String) -> HashSet<NodeInfo> {
    let mut node_infos = HashSet::new();

    for line in string.lines() {
        let components: Vec<&str> = line.split(",").collect();
        let node_id = components[0].parse().expect("Could not parse node id.");
        let socket_addr = components[1]
            .to_socket_addrs()
            .expect("Could not transform to socket addrs.")
            .next()
            .expect("No socket addrs provided.");
        let key_path = components[2].to_string();
        let username = components[3].to_string();
        let script_path = components[4].to_string();

        let node_info = NodeInfo {
            node_id: node_id,
            socket_addr: socket_addr,
            key_path: key_path,
            username: username,
            script_path: script_path,
        };

        node_infos.insert(node_info);
    }

    node_infos
}

pub fn number_of_writers() -> Arg<'static, 'static> {
    Arg::with_name("number-of-writers")
        .required(false)
        .takes_value(true)
        .default_value("0")
        .short("w")
        .long("number-of-writers")
        .help("The number of nodes that should write.")
}

pub fn number_of_writers_from_matches(matches: &ArgMatches<'static>) -> Int {
    matches
        .value_of("number-of-writers")
        .expect("Number of writers arg not existing.")
        .parse()
        .expect("Could not parse number of writers.")
}

pub fn number_of_snapshotters() -> Arg<'static, 'static> {
    Arg::with_name("number-of-snapshotters")
        .required(false)
        .takes_value(true)
        .default_value("0")
        .short("s")
        .long("number-of-snapshotters")
        .help("The number of nodes that should snapshot.")
}

pub fn number_of_snapshotters_from_matches(matches: &ArgMatches<'static>) -> Int {
    matches
        .value_of("number-of-snapshotters")
        .expect("Number of snapshotters arg not existing.")
        .parse()
        .expect("Could not parse number of snapshotters.")
}

pub fn variant() -> Arg<'static, 'static> {
    Arg::with_name("variant")
        .required(false)
        .default_value("TOURB")
        .takes_value(true)
        .short("v")
        .help("Which snapshot algorithm to run.")
}

pub fn variant_from_matches(matches: &ArgMatches<'static>) -> Variant {
    matches
        .value_of("variant")
        .expect("Variant arg not existing.")
        .parse()
        .expect("Could not parse variant arg.")
}

pub fn delta() -> Arg<'static, 'static> {
    Arg::with_name("delta")
        .required(false)
        .takes_value(true)
        .default_value("0")
        .short("d")
        .long("delta")
        .help("The delta parameter for Algorithm4.")
}

pub fn delta_from_matches(matches: &ArgMatches<'static>) -> Int {
    matches
        .value_of("delta")
        .expect("delta arg not existing.")
        .parse()
        .expect("Could not parse delta arg.")
}

pub fn run_length() -> Arg<'static, 'static> {
    Arg::with_name("run-length")
        .required(false)
        .takes_value(true)
        .default_value("0")
        .short("l")
        .long("run-length")
        .help("The number of seconds the program should run for. If 0 is given, the program will run until aborted with Ctrl-C.")
}

pub fn run_length_string_from_matches(matches: &ArgMatches<'static>) -> String {
    matches
        .value_of("run-length")
        .expect("run length arg not existing.")
        .to_string()
}

pub fn record_evaluation_info() -> Arg<'static, 'static> {
    Arg::with_name("record-evaluation-info")
        .short("e")
        .long("record-evaluation-info")
        .takes_value(false)
        .help("Record information used for the evaluation, such as latency and number of messages sent. If not done, the performance might be slightly higher.")
}

pub fn record_evaluation_info_string_from_matches(matches: &ArgMatches<'static>) -> String {
    match matches.is_present("record-evaluation-info") {
        true => "--record-evaluation-info".to_string(),
        false => "".to_string(),
    }
}

pub fn optimize() -> Arg<'static, 'static> {
    Arg::with_name("optimize")
        .takes_value(false)
        .short("o")
        .long("optimize")
        .help("With this option, cargo will build/run in release mode. This uses optimizations and yields higher performance.")
}

pub fn release_mode_string_from_matches(matches: &ArgMatches<'static>) -> String {
    match matches.is_present("optimize") {
        true => "--release".to_string(),
        false => "".to_string(),
    }
}

pub fn print_client_operations() -> Arg<'static, 'static> {
    Arg::with_name("print-client-operations")
        .takes_value(false)
        .short("p")
        .long("print-client-operations")
        .help("Print when a snapshot/write operation starts/ends. If not included, the performance might be slightly higher.")
}

pub fn print_client_operations_string_from_matches(matches: &ArgMatches<'static>) -> String {
    match matches.is_present("print-client-operations") {
        true => "--print-client-operations".to_string(),
        false => "".to_string(),
    }
}

pub fn color_from_node_id(node_id: NodeId) -> Color {
    let colors = vec![Black, Red, Green, Yellow, Blue, Magenta, Cyan];
    colors[(node_id as usize) % colors.len()]
}

pub fn run_result_file_name_from_node_id(node_id: NodeId) -> String {
    format!("node{:0>6}.eval", node_id)
}

pub fn write_string_from_node_id_and_number_of_writers(
    number_of_nodes: Int,
    node_id: NodeId,
    number_of_writers: Int,
) -> &'static str {
    // match node_id <= number_of_writers {
    //     true => "--write",
    //     false => "",
    // }
    // Allocate from both ends so that snapshoters and writers don't overlap by default
    let writer_threshold = number_of_nodes - number_of_writers;
    match node_id > writer_threshold {
        true => "--write",
        false => "",
    }
}

pub fn snapshot_string_from_node_id_and_number_of_snapshotters(
    node_id: NodeId,
    number_of_snapshotters: Int,
) -> &'static str {
    match node_id <= number_of_snapshotters {
        true => "--snapshot",
        false => "",
    }
}
