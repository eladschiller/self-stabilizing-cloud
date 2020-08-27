//#![allow(dead_code, unused_variables, unused_imports, unused_mut)]

#[macro_use]
extern crate lazy_static;

mod arguments;

use std::collections::HashSet;
use std::process::Child;
use std::{thread, process, fs};
use std::vec::Vec;
use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};

use ctrlc;

use commons::execution;
use commons::node_info::NodeInfo;
use commons::remote_machine::*;

use crate::arguments::ARGUMENTS;
use std::string::ToString;

fn main() {
    stop_all_remote_processes();
    set_ctrl_c_handler();

    if ARGUMENTS.install {
        run_install_script_on_remote_computers();
        upload_source_code_and_hosts_file();
        build_source_code();
    } else if ARGUMENTS.clean {
        clean_remote_directory();
    } else {
        upload_hosts_file_to_all();
        run_application_on_remote_computers();
    }
}

fn upload_hosts_file_to_all() {
    let mut join_handles = Vec::new();

    for node_info in ARGUMENTS.node_infos_for_unique_hosts().iter() {
        let node_info_thread = node_info.clone();
        let join_handle = thread::spawn(move || {
            create_and_upload_hosts_file(&node_info_thread);
        });

        join_handles.push(join_handle);
    }

    for join_handle in join_handles.into_iter() {
        join_handle
            .join()
            .expect("Could not join a thread for source code upload.");
    }
}

fn stop_all_remote_processes() {
    run_function_on_each_unique_host_in_parallell(&stop_remote_processes);
}

fn run_function_on_each_unique_host_in_parallell(function_to_run: &dyn Fn(&NodeInfo) -> Child) {
    run_function_on_hosts_in_parallell(function_to_run, &ARGUMENTS.node_infos_for_unique_hosts());
}

fn run_function_on_hosts_in_parallell(
    function_to_run: &dyn Fn(&NodeInfo) -> Child,
    hosts: &HashSet<NodeInfo>,
) {
    let mut finished_ok = false;
    while !finished_ok {
        finished_ok = true;
        let mut processes = Vec::new();
        let start_instant = Instant::now();

        for node_info in hosts.iter() {
            let process = function_to_run(&node_info);
            processes.push(process);
        }
        let mut finished = false;
        'ploop: while !finished {
            for process in processes.iter_mut() {
                finished = true;
                match process.try_wait() {
                    Ok(Some(_)) => {
                    },
                    Ok(None) => {
                        finished = false;
                        if Duration::from_secs(1200) < start_instant.elapsed() {
                            println!("More than 20 minutes have elapsed since start of parallell function run, restarting.");
                            finished_ok = false;
                            break 'ploop;
                        }
                    },
                    Err(e) => {
                        println!("Error while waiting on process in remote starter: {:?}", e);
                    }
                }
            }
            thread::sleep(Duration::from_secs(5));
        }
        if !finished_ok {
            for process in processes.iter_mut() {
                let _ = process.kill();
            }
        }
    }
}

fn stop_remote_processes(node_info: &NodeInfo) -> Child {
    let stop_command = format!("pkill -u {}", node_info.username);
    execution::execute_remote_command(&stop_command, node_info)
}

fn set_ctrl_c_handler() {
    ctrlc::set_handler(move || {
        println!("I will now exit. But first I will stop all processes I have started on the remote computers.");
        stop_all_remote_processes();
        process::exit(0);

    }).expect("Could not set the CTRL+C handler.");
}

fn run_install_script_on_remote_computers() {
    run_function_on_each_unique_host_in_parallell(&run_install_script_on_remote_computer);
}

fn run_install_script_on_remote_computer(node_info: &NodeInfo) -> Child {
    create_remote_directory(node_info);
    copy_install_script(node_info);
    run_install_script(node_info)
}

fn create_remote_directory(node_info: &NodeInfo) {
    execution::execute_remote_command(&format!("mkdir {}/", REMOTE_DIRECTORY_NAME), node_info)
        .wait()
        .expect("mkdir failed.");
}

fn copy_install_script(node_info: &NodeInfo) {
    execution::scp_copy_of_local_source_path_to_remote_destination_path(
        &node_info.script_path,
        REMOTE_INSTALL_SCRIPT_NAME,
        node_info,
    )
    .wait()
    .expect("script upload failed.");
}

fn run_install_script(node_info: &NodeInfo) -> Child {
    execution::execute_remote_command(
        &format!("{}/{}", REMOTE_DIRECTORY_NAME, REMOTE_INSTALL_SCRIPT_NAME),
        node_info,
    )
}

fn upload_source_code_and_hosts_file() {
    let mut join_handles = Vec::new();

    for node_info in ARGUMENTS.node_infos_for_unique_hosts().iter() {
        let node_info_thread = node_info.clone();
        let join_handle = thread::spawn(move || {
            upload_source_code_and_hosts_file_to_single_remote_computer(&node_info_thread);
        });

        join_handles.push(join_handle);
    }

    for join_handle in join_handles.into_iter() {
        join_handle
            .join()
            .expect("Could not join a thread for source code upload.");
    }
}

fn upload_source_code_and_hosts_file_to_single_remote_computer(node_info: &NodeInfo) {
    update_crate_source_on_remote("application", node_info);
    update_crate_source_on_remote("commons", node_info);
    upload_hosts_file(node_info);
}

fn update_crate_source_on_remote(crate_name: &str, node_info: &NodeInfo) {
    delete_src_folder_of_crate(crate_name, node_info);
    create_crate_folder(crate_name, node_info);

    copy_path_from_local_crate_to_remote_crate("src/", crate_name, node_info);
    copy_path_from_local_crate_to_remote_crate("Cargo.toml", crate_name, node_info);
    copy_path_from_local_crate_to_remote_crate("Cargo.lock", crate_name, node_info);
}

fn delete_src_folder_of_crate(crate_name: &str, node_info: &NodeInfo) {
    execution::execute_remote_command(
        &format!("rm -r {}/{}/src/", REMOTE_DIRECTORY_NAME, crate_name),
        &node_info,
    )
    .wait()
    .expect("Delete src folder process failed.");
}

fn create_crate_folder(crate_name: &str, node_info: &NodeInfo) {
    execution::execute_remote_command(
        &format!("mkdir {}/{}", REMOTE_DIRECTORY_NAME, crate_name),
        &node_info,
    )
    .wait()
    .expect("Could not wait for a remote command.");
}

fn copy_path_from_local_crate_to_remote_crate(path: &str, crate_name: &str, node_info: &NodeInfo) {
    execution::scp_copy_of_local_source_path_to_remote_destination_path(
        &format!("../{}/{}", crate_name, path),
        &format!("{}/{}", crate_name, path),
        &node_info,
    )
    .wait()
    .expect("Could not wait for a remote copy.");
}

fn create_and_upload_hosts_file(node_info: &NodeInfo) {
    let tmp_host = "hosts_tmp.txt";
    let new_hosts = &ARGUMENTS.node_infos;
    let mut lines = Vec::new();
    for node in new_hosts {
        lines.push(format!("{},{},{},{},{}", node.node_id, node.socket_addr.to_string(), node.key_path, node.username, node.script_path));
    }
    fs::write(
        tmp_host,
        lines.join("\n"),
    ).expect("unable to write to hosts tmp file");
    execution::scp_copy_of_local_source_path_to_remote_destination_path(
        tmp_host,
        &format!("application/{}", REMOTE_HOSTS_FILE_NAME),
        node_info,
    )
        .wait()
        .expect("Could not wait for the hosts file copy command.");
}


fn upload_hosts_file(node_info: &NodeInfo) {
    execution::scp_copy_of_local_source_path_to_remote_destination_path(
        &ARGUMENTS.hosts_file,
        &format!("application/{}", REMOTE_HOSTS_FILE_NAME),
        node_info,
    )
    .wait()
    .expect("Could not wait for the hosts file copy command.");
}

fn build_source_code() {
    run_function_on_each_unique_host_in_parallell(&build_source_code_on_remote_computer);
}


fn build_source_code_on_remote_computer(node_info: &NodeInfo) -> Child {
    let pi_path_fix =  if ARGUMENTS.is_local_run {
        "PATH=$PATH:/home/pi/.cargo/bin"
    } else {
        ""
    };
    let command = format!(
        "\"cd {}/application/; {} cargo build {}; \"",
        REMOTE_DIRECTORY_NAME, pi_path_fix, ARGUMENTS.release_mode_string
    );
    println!("{}", command);
    execution::execute_remote_command(&command, &node_info)
}

fn clean_remote_directory() {
    run_function_on_each_unique_host_in_parallell(&rm_remote_directory_on_remote_computer);
}

fn rm_remote_directory_on_remote_computer(node_info: &NodeInfo) -> Child {
    let command = format!(
        "\"rm -rf {}\"",
        REMOTE_DIRECTORY_NAME
    );
    execution::execute_remote_command(&command, &node_info)
}

fn run_application_on_remote_computers() {
    run_function_on_all_hosts_in_parallell(&run_application_on_remote_computer);
}

fn run_function_on_all_hosts_in_parallell(function_to_run: &dyn Fn(&NodeInfo) -> Child) {
    run_function_on_hosts_in_parallell(function_to_run, &ARGUMENTS.node_infos);
}

fn run_application_on_remote_computer(node_info: &NodeInfo) -> Child {
    let mut writer_s = "";
    let mut failing_s = "";
    let mut window_s = "".to_string();
    let mut crashing_s = "";
    if node_info.is_failing {
        failing_s = "-f";
    }
    if node_info.is_writer {
        writer_s = "-w";
    }
    if ARGUMENTS.window_size.is_some() {
        window_s = format!("-s {}", ARGUMENTS.window_size.unwrap());
    }

    if node_info.is_crashing {
        crashing_s = "-b";
    }
    let pi_path_fix = if ARGUMENTS.is_local_run {
        "PATH=$PATH:/home/pi/.cargo/bin"
    } else {
        ""
    };

    let command_string = format!(
        "\"cd {}/application/; {} RUST_BACKTRACE=1 cargo run {} -- {} {} -l {} -c {:?} {} {} {} {} {} -v {:?} {} -d {}\"",
        REMOTE_DIRECTORY_NAME,
        pi_path_fix,
        ARGUMENTS.release_mode_string,
        node_info.node_id,
        REMOTE_HOSTS_FILE_NAME,
        ARGUMENTS.run_length_string,
        commons::arguments::color_from_node_id(node_info.node_id),
        ARGUMENTS.record_evaluation_info_string,
        ARGUMENTS.print_client_operations_string,
        writer_s,
        failing_s,
        window_s,
        ARGUMENTS.variant,
        crashing_s,
        ARGUMENTS.delta,
    );
    println!("{}",command_string);

    execution::execute_remote_command(&command_string, &node_info)
}
