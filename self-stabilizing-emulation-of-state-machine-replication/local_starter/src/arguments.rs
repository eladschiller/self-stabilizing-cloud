use clap::{App, AppSettings, Arg, ArgMatches};

use commons::arguments;
use commons::types::Int;
use commons::variant::Variant;

lazy_static! {
    pub static ref ARGUMENTS: Arguments = Arguments::new();
}

pub struct Arguments {
    pub number_of_nodes: Int,
    pub release_mode_string: String,
    pub print_client_operations_string: String,
    pub run_length_string: String,
    pub record_evaluation_info_string: String,
    pub variant: Variant,
}

impl Arguments {
    fn new() -> Arguments {
        let matches = get_matches();

        Arguments {
            number_of_nodes: number_of_nodes_from_matches(&matches),
            release_mode_string: arguments::release_mode_string_from_matches(&matches),
            print_client_operations_string: arguments::print_client_operations_string_from_matches(
                &matches,
            ),
            run_length_string: arguments::run_length_string_from_matches(&matches),
            record_evaluation_info_string: arguments::record_evaluation_info_string_from_matches(
                &matches,
            ),
            variant: arguments::variant_from_matches(&matches),
        }
    }
}

fn get_matches() -> ArgMatches<'static> {
    App::new("Rusty Self-Stabilizing TO_URB: Local starter")
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::VersionlessSubcommands)
        .about("A helper utility that starts multiple nodes on your local computer.")
        .arg(number_of_nodes_argument())
        .arg(arguments::optimize())
        .arg(arguments::print_client_operations())
        .arg(arguments::run_length())
        .arg(arguments::record_evaluation_info())
        .arg(arguments::variant())
        .get_matches()
}

pub fn number_of_nodes_argument() -> Arg<'static, 'static> {
    Arg::with_name("number-of-nodes")
        .required(true)
        .takes_value(true)
        .help("The number of local nodes to run.")
}

fn number_of_nodes_from_matches(matches: &ArgMatches<'static>) -> Int {
    matches
        .value_of("number-of-nodes")
        .expect("Number of nodes arg not existing.")
        .parse()
        .expect("Could not parse number of nodes.")
}
