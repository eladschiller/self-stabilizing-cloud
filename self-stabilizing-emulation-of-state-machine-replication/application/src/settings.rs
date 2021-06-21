use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::time::Duration;

use clap::{App, AppSettings, Arg, ArgMatches};
use colored::*;
use lazy_static::lazy_static;

use commons::arguments;
use commons::types::{Int, NodeId};

lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new();
}

#[derive(Debug)]
pub struct Settings {
    node_id: NodeId,
    socket_addrs: HashMap<NodeId, SocketAddr>,
    terminal_color: Color,
    print_client_operations: bool,
    run_length: Duration,
    record_evaluation_info: bool,
}

impl Settings {
    fn new() -> Settings {
        let matches = get_matches();

        Settings {
            node_id: node_id_from_matches(&matches),
            socket_addrs: socket_addrs_from_matches(&matches),
            terminal_color: color_from_matches(&matches),
            print_client_operations: print_client_operations_from_matches(&matches),
            run_length: run_length_from_matches(&matches),
            record_evaluation_info: record_evaluation_info_from_matches(&matches),
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn socket_addrs(&self) -> &HashMap<NodeId, SocketAddr> {
        &self.socket_addrs
    }

    pub fn terminal_color(&self) -> Color {
        self.terminal_color
    }

    #[allow(dead_code)]
    pub fn record_evaluation_info(&self) -> bool {
        self.record_evaluation_info
    }

    pub fn print_client_operations(&self) -> bool {
        self.print_client_operations
    }

    pub fn number_of_nodes(&self) -> Int {
        self.socket_addrs.len() as Int
    }

    pub fn run_length(&self) -> Duration {
        self.run_length
    }

}

fn get_matches() -> ArgMatches<'static> {

    App::new("Rusty Self-Stabilizing URB: Application")
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::VersionlessSubcommands)
        .about("The application code, that is an instance of a urb node.")
        .arg(node_id_argument())
        .arg(arguments::hosts_file(
            "The file with host ids, addresses and ports.",
        ))
        .arg(color_argument())
        .arg(arguments::print_client_operations())
        .arg(arguments::run_length())
        .arg(arguments::record_evaluation_info())
        .get_matches()
}

fn node_id_argument() -> Arg<'static, 'static> {
    Arg::with_name("node-id")
        .required(true)
        .takes_value(true)
        .help("The integer id of this node instance.")
}

fn node_id_from_matches(matches: &ArgMatches<'static>) -> NodeId {
    matches.value_of("node-id").unwrap().parse().unwrap()
}

fn socket_addrs_from_matches(matches: &ArgMatches<'static>) -> HashMap<NodeId, SocketAddr> {
    let hosts_file_path = matches.value_of("hosts-file").unwrap();
    let string = fs::read_to_string(hosts_file_path).expect("Unable to read file");
    socket_addrs_from_string(string)
}

fn socket_addrs_from_string(string: String) -> HashMap<NodeId, SocketAddr> {
    let mut socket_addrs = HashMap::new();

    for line in string.lines() {
        let components: Vec<&str> = line.split(",").collect();
        let id = components[0].parse().unwrap();
        let socket_addr = components[1]
            .to_socket_addrs()
            .expect("Could not transform to socket addrs.")
            .next()
            .expect("No socket addrs provided.");

        socket_addrs.insert(id, socket_addr);
    }
    socket_addrs
}

fn color_argument() -> Arg<'static, 'static> {
    let colors = &["Black", "Red", "Green", "Yellow", "Blue", "Magenta", "Cyan"];
    Arg::with_name("color")
        .short("c")
        .long("color")
        .takes_value(true)
        .possible_values(colors)
        .default_value("Black")
        .help("The color of the terminal output")
}

fn color_from_matches(matches: &ArgMatches<'static>) -> Color {
    matches.value_of("color").unwrap().parse().unwrap()
}

fn print_client_operations_from_matches(matches: &ArgMatches<'static>) -> bool {
    matches.is_present("print-client-operations")
}

fn run_length_from_matches(matches: &ArgMatches<'static>) -> Duration {
    let seconds = arguments::run_length_string_from_matches(matches)
        .parse()
        .unwrap();
    Duration::from_secs(seconds)
}

fn record_evaluation_info_from_matches(matches: &ArgMatches<'static>) -> bool {
    matches.is_present("record-evaluation-info")
}
