use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use commons::arguments;
use commons::node_info::NodeInfo;
use commons::run_result::RunResult;
use commons::types::NodeId;

use crate::scenario::Scenario;
use crate::aggregation;
use std::net::ToSocketAddrs;

lazy_static! {
    pub static ref ARGUMENTS: Arguments = Arguments::new();
}

pub enum Arguments {
    Install(InstallArguments),
    Gather(GatherArguments),
    Aggregate(AggregateArguments),
}

impl Arguments {
    fn new() -> Arguments {
        let matches = get_matches();

        if let Some(install_matches) = matches.subcommand_matches("install") {
            Arguments::Install(InstallArguments::from_matches(&install_matches))
        } else if let Some(gather_matches) = matches.subcommand_matches("gather") {
            Arguments::Gather(GatherArguments::from_matches(&gather_matches))
        } else if let Some(aggregate_matches) = matches.subcommand_matches("aggregate") {
            Arguments::Aggregate(AggregateArguments::from_matches(&aggregate_matches))
        } else {
            panic!("No correct subcommand was provided.")
        }
    }
}

pub struct InstallArguments {
    pub hosts_file: String,
    pub optimize_string: String,
    pub randomize: bool,
    pub is_local_run: bool,
}

impl InstallArguments {
    fn from_matches(matches: &ArgMatches<'static>) -> InstallArguments {
        InstallArguments {
            hosts_file: arguments::hosts_file_from_matches(matches),
            optimize_string: optimize_string_from_matches(matches),
            randomize: randomize_from_matches(matches),
            is_local_run: arguments::is_local_run_from_string(&matches),
        }
    }
}

pub struct GatherArguments {
    pub hosts_file: String,
    pub node_infos: HashSet<NodeInfo>,
    pub scenarios: HashSet<Scenario>,
    pub rounds: i32,
    pub result_file_path: PathBuf,
    pub optimize_string: String,
    pub print_client_operations_string: String,
    pub run_length_string: String,
    pub is_local_run: bool,
}

impl GatherArguments {
    fn from_matches(matches: &ArgMatches<'static>) -> GatherArguments {
        GatherArguments {
            hosts_file: arguments::hosts_file_from_matches(matches),
            node_infos: arguments::node_infos_from_matches(matches),
            scenarios: scenarios_from_matches(matches),
            rounds: number_of_rounds_from_matches(matches),
            result_file_path: result_file_path_from_matches(matches),
            optimize_string: optimize_string_from_matches(matches),
            print_client_operations_string: arguments::print_client_operations_string_from_matches(
                matches,
            ),
            run_length_string: arguments::run_length_string_from_matches(&matches),
            is_local_run: arguments::is_local_run_from_string(&matches),
        }
    }
}

pub enum Experiment {
    Experiment1,
    Experiment2,
    Experiment3,
    Experiment4,
    Experiment5,
    Experiment6,
    Experiment7,
}

pub struct AggregateArguments {
    pub run_results: HashMap<Scenario, Vec<HashMap<NodeId, RunResult>>>,
    pub rounds: usize,
    pub experiment: Experiment,
}

impl AggregateArguments {
    fn from_matches(matches: &ArgMatches<'static>) -> AggregateArguments {
        let (run_results, rounds) = run_results_from_matches(matches);
        AggregateArguments {
            run_results: run_results,
            rounds: rounds,
            experiment: experiment_from_matches(matches),
        }
    }
}

fn get_matches() -> ArgMatches<'static> {
    App::new("Rusty Self-Stabilizing Abstractions: Evaluator")
        .about("A helper utilty that gathers evaluation results and aggregates them")
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(SubCommand::with_name("install")
            .about("Will install Rust and the source code on the (remote) hosts.")

            .arg(arguments::is_local_run())
            .arg(arguments::hosts_file("The file with node ids, addresses, ports, ssh key paths and usernames."))
            .arg(randomize_argument())
            .arg(arguments::optimize()))

        .subcommand(SubCommand::with_name("gather")
            .about("Will run each scenario once and gather the results in a file. The results-file will be built upon, and if a scenario already exists there, it will not be run again.")

            .arg(arguments::is_local_run())
            .arg(arguments::hosts_file("The file with node ids, addresses, ports, ssh key paths and usernames."))
            .arg(scenario_file_argument())
            .arg(result_file_argument())
            .arg(arguments::optimize())
            .arg(run_length_argument())
            .arg(arguments::print_client_operations())
            .arg(rounds_argment()))

        .subcommand(SubCommand::with_name("aggregate")
            .about("Will aggregate multiple result-files to generate aggregated results, according to what you have programatically defined.")
            .arg(experiment_argument())
            .arg(result_files_argument()))

        .get_matches()
}

fn randomize_argument() -> Arg<'static, 'static> {
    Arg::with_name("randomize")
        .short("r")
        .long("randomize")
        .takes_value(false)
        .help("Randomize the hosts file.")
}

fn scenario_file_argument() -> Arg<'static, 'static> {
    Arg::with_name("scenario-file")
        .required(true)
        .takes_value(true)
        .help("The file with scenarios to run.")
}

fn result_file_argument() -> Arg<'static, 'static> {
    Arg::with_name("result-file")
        .required(true)
        .takes_value(true)
        .help("The file in which the results are stored.")
}

fn run_length_argument() -> Arg<'static, 'static> {
    Arg::with_name("run-length")
        .required(false)
        .takes_value(true)
        .default_value("3")
        .short("l")
        .long("run-length")
        .help("The number of seconds the program should run for. If 0 is given, the program will run forever. Avoid this value.")
}

fn result_files_argument() -> Arg<'static, 'static> {
    Arg::with_name("result-files")
        .required(true)
        .takes_value(true)
        .min_values(1)
        .help(
            "The files with results. Each file should have the same scenarios as the other files.",
        )
}

fn rounds_argment() -> Arg<'static, 'static> {
    Arg::with_name("rounds")
        .required(false)
        .takes_value(true)
        .short("r")
        .long("rounds")
        .help("Number of rounds to run the scenarios.")
}

fn experiment_argument() -> Arg<'static, 'static> {
    Arg::with_name("experiment")
        .required(true)
        .takes_value(true)
        .short("e")
        .long("experiment")
        .help("The experiment that you are aggregating with the data.")
}

fn scenarios_from_matches(matches: &ArgMatches<'static>) -> HashSet<Scenario> {
    let scenarios_file_path = matches
        .value_of("scenario-file")
        .expect("Scenario file argument not found.");
    let string =
        fs::read_to_string(scenarios_file_path).expect("Unable to read the scenarios file.");
    scenarios_from_string(string, matches)
}

fn scenarios_from_string(string: String, matches: &ArgMatches<'static>) -> HashSet<Scenario> {
    let mut scenarios = HashSet::new();

    for line in string.lines() {
        if line.starts_with("//") {
            continue;
        } else {
            let mut scenario = Scenario::from(line.to_string());

            scenarios.insert(scenario);
        }
    }

    scenarios
}

pub(crate) fn node_info_for_scenario(scenario: Scenario, hosts: HashSet<NodeInfo>) -> HashSet<NodeInfo>{
    if scenario.number_of_nodes <= 0{
        return hosts;
    }
    let mut new_hosts = HashSet::new();
    let mut metamap = HashMap::new();
    for node in hosts {
        metamap.insert(node.ip_addr_string(), (node.socket_addr.port(), node.key_path, node.script_path, node.username));
    }
    let mut metaiter = metamap.iter().cycle();
    // TODO: pick port numbers better
    for i in 1..=scenario.number_of_nodes {
        if let Some((ip, (mut port, key_path, script_path, username))) = metaiter.next() {
            let saddr = format!("{}:{}", ip, port+ (i - 1) as u16).to_socket_addrs()
                .expect("Unable to transform to socket address")
                .next().expect("No socket addrs provided");
            new_hosts.insert(NodeInfo {
                node_id: i,
                socket_addr: saddr,
                key_path: key_path.clone(),
                username: username.clone(),
                script_path: script_path.clone(),
                is_writer: true,
                is_failing: false,
                is_crashing: false,
            });
        }
    }
    new_hosts

}

fn result_file_path_from_matches(matches: &ArgMatches<'static>) -> PathBuf {
    let as_str = matches
        .value_of("result-file")
        .expect("result file not provided.");
    PathBuf::from(as_str)
}

fn number_of_rounds_from_matches(matches: &ArgMatches<'static>) -> i32 {
    if matches.value_of("rounds").is_some() {
        matches.value_of("rounds").unwrap().parse().expect("Could not parse number of rounds")
    } else {
        1
    }
}

fn optimize_string_from_matches(matches: &ArgMatches<'static>) -> String {
    match matches.is_present("optimize") {
        true => "--optimize".to_string(),
        false => "".to_string(),
    }
}

fn run_results_from_matches(
    matches: &ArgMatches<'static>,
) -> (HashMap<Scenario, Vec<HashMap<NodeId, RunResult>>>, usize) {
    let result_strings = matches.values_of("result-files").unwrap().map(|result_file| fs::read_to_string(result_file).unwrap()).collect();
    let rounds = matches.values_of("result-files").unwrap().len();
    let run_results = aggregation::result_map_from_result_strings(result_strings);
    (run_results, rounds)
}

fn experiment_from_matches(
    matches: &ArgMatches<'static>,
    ) -> Experiment {
    let experiment: i32 = matches.value_of("experiment").unwrap().parse().expect("Could not parse experiment");
    match experiment {
        1 => Experiment::Experiment1,
        2 => Experiment::Experiment2,
        3 => Experiment::Experiment3,
        4 => Experiment::Experiment4,
        5 => Experiment::Experiment5,
        6 => Experiment::Experiment6,
        7 => Experiment::Experiment7,
        _ => panic!("Unknown experiment!"),
    }
}
fn randomize_from_matches(
    matches: &ArgMatches<'static>,
    ) -> bool { 
    matches.is_present("randomize")
}
 
