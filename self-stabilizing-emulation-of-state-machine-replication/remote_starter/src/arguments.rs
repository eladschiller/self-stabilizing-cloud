use std::collections::HashSet;

use clap::{App, AppSettings, Arg, ArgMatches};

use commons::arguments;
use commons::node_info::NodeInfo;
use commons::types::Int;
//use commons::variant::Variant;

lazy_static! {
    pub static ref ARGUMENTS: Arguments = Arguments::new();
}

#[derive(Debug)]
pub struct Arguments {
    pub hosts_file: String,
    pub node_infos: HashSet<NodeInfo>,
    pub number_of_writers: Int,
    pub number_of_snapshotters: Int,
    //pub variant: Variant,
    pub delta: Int,
    pub release_mode_string: String,
    pub print_client_operations_string: String,
    pub run_length_string: String,
    pub record_evaluation_info_string: String,
    pub install: bool,
    pub clean: bool,
}

impl Arguments {
    fn new() -> Arguments {
        let matches = get_matches();

        Arguments {
            hosts_file: arguments::hosts_file_from_matches(&matches),
            node_infos: arguments::node_infos_from_matches(&matches),
            number_of_writers: arguments::number_of_writers_from_matches(&matches),
            number_of_snapshotters: arguments::number_of_snapshotters_from_matches(&matches),
            //variant: arguments::variant_from_matches(&matches),
            delta: arguments::delta_from_matches(&matches),
            release_mode_string: arguments::release_mode_string_from_matches(&matches),
            print_client_operations_string: arguments::print_client_operations_string_from_matches(
                &matches,
            ),
            run_length_string: arguments::run_length_string_from_matches(&matches),
            record_evaluation_info_string: arguments::record_evaluation_info_string_from_matches(
                &matches,
            ),
            install: install_from_matches(&matches),
            clean: clean_from_matches(&matches),
        }
    }

    pub fn node_infos_for_unique_hosts(&self) -> HashSet<NodeInfo> {
        let mut node_ids_for_unique_hosts = HashSet::new();
        let mut handled_hosts = HashSet::new();

        for node_info in self.node_infos.iter() {
            if handled_hosts.insert(node_info.ip_addr_string()) {
                node_ids_for_unique_hosts.insert(node_info.clone());
            }
        }

        node_ids_for_unique_hosts
    }
}

fn get_matches() -> ArgMatches<'static> {
    App::new("Rusty Self-Stabilizing Snapshots: Remote starter")
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::VersionlessSubcommands)
        .about("A helper utility that starts multiple nodes on remote machines via SSH.")
        .arg(arguments::hosts_file(
            "The file with node ids, addresses, ports, ssh key paths and usernames.",
        ))
        .arg(arguments::number_of_writers())
        .arg(arguments::number_of_snapshotters())
        //.arg(arguments::variant())
        .arg(arguments::delta())
        .arg(arguments::run_length())
        .arg(arguments::record_evaluation_info())
        .arg(arguments::optimize())
        .arg(install_argument())
        .arg(clean_argument())
        .arg(arguments::print_client_operations())
        .get_matches()
}

fn install_argument() -> Arg<'static, 'static> {
    Arg::with_name("install")
        .takes_value(false)
        .short("i")
        .long("install")
        .help("With this option, Rust will be installed, the source code and configuration files will be uploaded and the application will be built. Without this option, the application will be launched.")
}

fn clean_argument() -> Arg<'static, 'static> {
    Arg::with_name("clean")
        .takes_value(false)
        .short("c")
        .long("clean")
        .help("This option will remove the remote directory.")
}
fn install_from_matches(matches: &ArgMatches<'static>) -> bool {
    matches.is_present("install")
}

fn clean_from_matches(matches: &ArgMatches<'static>) -> bool {
    matches.is_present("clean")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_that_node_infos_for_unique_hosts_includes_correct_node_infos() {
        let socket_addrs: Vec<SocketAddr> = vec![
            "1.2.3.4:1236".parse().unwrap(),
            "1.2.3.4:1235".parse().unwrap(),
            "1.2.3.4:1234".parse().unwrap(),
            "4.4.4.4:1236".parse().unwrap(),
            "1.10.1.1:123".parse().unwrap(),
        ];

        let mut node_infos = HashSet::new();
        for socket_addr in socket_addrs.iter() {
            let node_info = NodeInfo {
                node_id: 1,
                socket_addr: socket_addr.clone(),
                key_path: "".to_string(),
                username: "".to_string(),
                script_path: "".to_string(),
            };

            node_infos.insert(node_info);
        }

        let arguments = Arguments {
            hosts_file: "".to_string(),
            node_infos: node_infos,
            number_of_writers: 0,
            number_of_snapshotters: 0,
            //variant: Variant::Algorithm1,
            release_mode_string: "".to_string(),
            print_client_operations_string: "".to_string(),
            run_length_string: "".to_string(),
            record_evaluation_info_string: "".to_string(),
            install: false,
        };

        let node_infos_for_unique_hosts = arguments.node_infos_for_unique_hosts();

        assert_eq!(node_infos_for_unique_hosts.len(), 3);

        let mut found_ip_addrs = HashSet::new();
        for node_info in node_infos_for_unique_hosts.iter() {
            found_ip_addrs.insert(node_info.ip_addr_string());
        }

        assert!(found_ip_addrs.contains("1.2.3.4"));
        assert!(found_ip_addrs.contains("4.4.4.4"));
        assert!(found_ip_addrs.contains("1.10.1.1"));
    }
}
