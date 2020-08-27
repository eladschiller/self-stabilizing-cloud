use std::collections::{HashSet, HashMap};
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

pub fn node_infos_from_host_pool(matches: &ArgMatches<'static>) -> HashSet<NodeInfo> {
    let hosts = node_infos_from_matches(matches);
    if number_of_nodes_from_matches(&matches) <= 0{
        return hosts;
    }
    let mut new_hosts = HashSet::new();
    let mut metamap = HashMap::new();
    for node in hosts {
        metamap.insert(node.ip_addr_string(), (node.socket_addr.port(), node.key_path, node.script_path, node.username));
    }
    let mut nr_of_writers = number_of_writers_from_matches(matches);
    let mut nr_of_failing = number_of_failing_from_matches(matches);
    let mut nr_of_crashing = number_of_crashing_from_matches(matches);

    let mut metaiter = metamap.iter().cycle();
    let nr_of_nodes = number_of_nodes_from_matches(&matches);
    // TODO: pick port numbers better
    for i in 1..=nr_of_nodes {
        if let Some((ip, (port, key_path, script_path, username))) = metaiter.next() {
            let saddr = format!("{}:{}", ip, port+ (i - 1) as u16).to_socket_addrs()
                .expect("Unable to transform to socket address")
                .next().expect("No socket addrs provided");
            let is_writer = nr_of_writers > 0;
            let is_failing = nr_of_failing > 0;
            let is_crashing = nr_of_crashing > 0 && nr_of_crashing + i > nr_of_nodes;
            new_hosts.insert(NodeInfo {
                node_id: i,
                socket_addr: saddr,
                key_path: key_path.clone(),
                username: username.clone(),
                script_path: script_path.clone(),
                is_writer: is_writer,
                is_failing: is_failing,
                is_crashing: is_crashing,
            });
            nr_of_writers += -1;
            nr_of_failing += -1;
        }
    }
    new_hosts
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
            is_writer: true,
            is_failing: false,
            is_crashing: false,
        };

        node_infos.insert(node_info);
    }

    node_infos
}

pub fn number_of_writers() -> Arg<'static, 'static> {
    Arg::with_name("number-of-writers")
        .required(false)
        .takes_value(true)
        .default_value("1000")
        .short("w")
        .long("number-of-writers")
        .help("The number of nodes that should write.")
}

pub fn number_of_writers_from_matches(matches: &ArgMatches<'static>) -> Int {
    if let Some(failing) = matches.value_of("number-of-writers"){
        failing.parse().expect("Couldn't parse number of writers")
    } else {
        1000
    }
}

pub fn number_of_failing() -> Arg<'static, 'static> {
    Arg::with_name("number-of-failing")
        .required(false)
        .takes_value(true)
        .default_value("0")
        .short("f")
        .long("number-of-failing")
        .help("Number of nodes that will modify/corrupt their internal variables once")
}

pub fn number_of_failing_from_matches(matches: &ArgMatches<'static>) -> Int {
    if let Some(failing) = matches.value_of("number-of-failing"){
        failing.parse().expect("Couldn't parse number of failing nodes")
    } else {
        0
    }
}

pub fn number_of_crashing() -> Arg<'static, 'static> {
    Arg::with_name("number-of-crashing")
        .required(false)
        .takes_value(true)
        .default_value("0")
        .short("b")
        .long("number-of-crashing")
        .help("Number of nodes that will crash during the execution")
}

pub fn number_of_crashing_from_matches(matches: &ArgMatches<'static>) -> Int {
    if let Some(crashing) = matches.value_of("number-of-crashing"){
        crashing.parse().expect("Couldn't parse number of crashing nodes")
    } else {
        0
    }
}

pub fn number_of_nodes() -> Arg<'static, 'static> {
    Arg::with_name("number-of-nodes")
        .required(false)
        .takes_value(true)
        .default_value("0")
        .short("n")
        .long("number-of-nodes")
        .help("The number of nodes in the system.")
}

pub fn number_of_nodes_from_matches(matches: &ArgMatches<'static>) -> Int {
    matches
        .value_of("number-of-nodes")
        .expect("Number of nodes arg not existing.")
        .parse()
        .expect("Could not parse number of nodes.")
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
        .takes_value(true)
        .default_value("SCD")
        .short("v")
        .help("Which protocol/application algorithm to run.")
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
        .default_value("1")
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

pub fn window_size_argument() -> Arg<'static, 'static> {
    Arg::with_name("window-size")
        .required(false)
        .takes_value(true)
        .short("s")
        .long("window-size")
        .help("Optional window size argument, WINDOW_SIZE constant used by default.")
}

pub fn is_local_run() -> Arg<'static, 'static> {
    Arg::with_name("local-run")
        .required(false)
        .takes_value(false)
        .short("u")
        .help("Optional local network run argument, when present will use pi path fix in remote execution")
}

pub fn is_local_run_from_string(matches: &ArgMatches<'static>) -> bool {
    matches.is_present("local-run")
}

pub fn window_size_from_matches(matches: &ArgMatches<'static>) -> Option<Int> {
    if let Some(window_s) = matches.value_of("window-size") {
        return Some(window_s.parse().expect("Unable to parse window size"));
    }
    None
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

pub fn is_writer() -> Arg<'static, 'static> {
    Arg::with_name("writer")
        .takes_value(false)
        .short("w")
        .long("writer")
        .help("Node is a writer, meaning it will send messages")
}
pub fn is_writer_string_from_matches(matches: &ArgMatches<'static>) -> String {
    match matches.is_present("writer") {
        true => "--writer".to_string(),
        false => "".to_string(),
    }
}

pub fn is_failing_node() -> Arg<'static,'static> {
    Arg::with_name("failing")
        .takes_value(false)
        .short("f")
        .long("failing")
        .help("Node will modify internal variables once during execution")
}

pub fn is_failing_from_matches(matches: &ArgMatches<'static>) -> String {
    match matches.is_present("failing") {
        true => "--failing".to_string(),
        false => "".to_string(),
    }
}

pub fn is_crashing_node() -> Arg<'static,'static> {
    Arg::with_name("crashing")
        .takes_value(false)
        .short("b")
        .long("crashing")
        .help("Node will crash once during execution")
}

pub fn is_crash_from_matches(matches: &ArgMatches<'static>) -> String {
    match matches.is_present("crashing") {
        true => "--crashing".to_string(),
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
