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
}

impl InstallArguments {
    fn from_matches(matches: &ArgMatches<'static>) -> InstallArguments {
        InstallArguments {
            hosts_file: arguments::hosts_file_from_matches(matches),
            optimize_string: optimize_string_from_matches(matches),
            randomize: randomize_from_matches(matches),
        }
    }
}

pub struct GatherArguments {
    pub hosts_file: String,
    pub node_infos: HashSet<NodeInfo>,
    pub scenarios: HashSet<Scenario>,
    pub result_file_path: PathBuf,
    pub optimize_string: String,
    pub print_client_operations_string: String,
    pub run_length_string: String,
}

impl GatherArguments {
    fn from_matches(matches: &ArgMatches<'static>) -> GatherArguments {
        GatherArguments {
            hosts_file: arguments::hosts_file_from_matches(matches),
            node_infos: arguments::node_infos_from_matches(matches),
            scenarios: scenarios_from_matches(matches),
            result_file_path: result_file_path_from_matches(matches),
            optimize_string: optimize_string_from_matches(matches),
            print_client_operations_string: arguments::print_client_operations_string_from_matches(
                matches,
            ),
            run_length_string: arguments::run_length_string_from_matches(&matches),
        }
    }
}

pub enum Experiment {
    Experiment1,
    Experiment2,
    Experiment3,
    Experiment4,
    Experiment5,
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
    App::new("Rusty Self-Stabilizing Snapshots: Evaluator")
        .about("A helper utilty that gathers evaluation results and aggregates them")
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::VersionlessSubcommands)

        .subcommand(SubCommand::with_name("install")
            .about("Will install Rust and the source code on the (remote) hosts.")

            .arg(arguments::hosts_file("The file with node ids, addresses, ports, ssh key paths and usernames."))
            .arg(randomize_argument())
            .arg(arguments::optimize()))

        .subcommand(SubCommand::with_name("gather")
            .about("Will run each scenario once and gather the results in a file. The results-file will be built upon, and if a scenario already exists there, it will not be run again.")

            .arg(arguments::hosts_file("The file with node ids, addresses, ports, ssh key paths and usernames."))
            .arg(scenario_file_argument())
            .arg(result_file_argument())
            .arg(arguments::optimize())
            .arg(run_length_argument())
            .arg(arguments::print_client_operations()))

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
    scenarios_from_string(string)
}

fn scenarios_from_string(string: String) -> HashSet<Scenario> {
    let mut scenarios = HashSet::new();

    for line in string.lines() {
        if line.starts_with("//") {
            continue;
        }
        let components: Vec<&str> = line.split(",").collect();
        let number_of_nodes = components[0]
            .parse()
            .expect("Could not parse the number of nodes.");
        let number_of_snapshotters = components[1]
            .parse()
            .expect("Could not parse the number of snapshotters.");
        let number_of_writers = components[2]
            .parse()
            .expect("Could not parse the number of writers.");
        let variant = components[3].parse().expect("Could not parse variant");
        let delta = components[4].parse().expect("Could not parse delta");

        let scenario = Scenario::new(
            number_of_nodes,
            number_of_snapshotters,
            number_of_writers,
            variant,
            delta
        );

        scenarios.insert(scenario);
    }

    scenarios
}

fn result_file_path_from_matches(matches: &ArgMatches<'static>) -> PathBuf {
    let as_str = matches
        .value_of("result-file")
        .expect("result file not provided.");
    PathBuf::from(as_str)
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
        _ => panic!("Unknown experiment!"),
    }
}
fn randomize_from_matches(
    matches: &ArgMatches<'static>,
    ) -> bool { 
    matches.is_present("randomize")
}
 
