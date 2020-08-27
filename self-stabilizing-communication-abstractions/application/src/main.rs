#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
#![allow(non_snake_case)]

mod communicator;
mod configuration_manager;
mod mediator;
mod responsible_cell;
mod settings;
mod terminal_output;
mod urb;
mod scd;
mod merge;
mod ping_check;

use std::fs;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, UNIX_EPOCH, SystemTime};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use commons::arguments;
use commons::types::{Int, NodeId};

use crate::mediator::Mediator;
use crate::settings::SETTINGS;
use crate::terminal_output::printlnu;
use urb::NodeDelegate;
use scd::scd::SCD;
use crate::scd::pattern3::Pattern3;
use crate::scd::pattern4::Pattern4;
use crate::scd::algorithm6::Algorithm6;
use crate::merge::mergednode::{MergedNode, StatusCode};
use commons::variant::Variant;
use ring_channel::RecvError;
use crate::urb::thetafd::json_is_ThetafdMessage;
use fastping_rs::Pinger;
use fastping_rs::PingResult::{Idle, Receive};
use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use nix::errno::EWOULDBLOCK;

fn main() {
    SETTINGS.node_id();
    //let (mediator_tx, scd_rx) = mpsc::channel();
    //let (scd_tx, mediator_rx) = mpsc::channel();
    //let (scd_pattern_tx, pattern_rx) = mpsc::channel();
    //let (pattern_tx, scd_pattern_rx) = mpsc::channel();

    let mut link_latencies = HashMap::new();
    if SETTINGS.socket_addrs().get(&SETTINGS.node_id()).unwrap().ip() != IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)) {
        //link_latencies = ping_all_nodes();
    }
    let mut node = MergedNode::new(link_latencies);

    //let mediator = Mediator::new(mediator_tx, mediator_rx);
    //let scd = SCD::new(scd_tx, scd_rx, mediator.node_id(), mediator.node_ids().clone(), scd_pattern_tx, scd_pattern_rx);
//    let pattern = Pattern3::new(pattern_tx, pattern_rx, Arc::clone(&scd));
//    let pattern = Pattern4::new(pattern_tx, pattern_rx, Arc::clone(&scd));
    //let algorithm6 = Algorithm6::new(pattern_tx, pattern_rx, Arc::clone(&scd));

    // This is important when running locally. If some application
    // processes start before all have been built, they will
    // consume so much CPU time that the build processes
    // are very slow, and hence some nodes will be run for
    // a longer time than others.
    // thread::sleep(Duration::from_millis(
    //     1000 * SETTINGS.number_of_nodes() as u64,
    // ));

    // Wait for all nodes to start.
    thread::sleep(Duration::from_secs(4));

    // let tx = start_client_threads_and_get_channel_send_ends(&scd, &algorithm6);
    let (stop_tx, msg_send, status_recv ) = MergedNode::start_the_do_forever_loop(node);

    printlnu(format!("Writer={},Reader={}", SETTINGS.is_writer(), SETTINGS.is_crashing_node()));

    let run_time = Instant::now();
    if SETTINGS.is_writer() || (SETTINGS.variant() == Variant::SNAPSHOT || SETTINGS.variant() == Variant::COUNTER) {
        let mut iter = 1;
        let mut is_reading = false;
        let max_num_msgs = Int::max_value();
        loop {
            let mut non_blocking_op= true;
            if run_time.elapsed().as_secs() >= SETTINGS.run_length().as_secs() {
                break;
            }
            if iter <= max_num_msgs {
                match SETTINGS.variant() {
                    Variant::URB => {
                        non_blocking_op = true;
                        let _ = msg_send.send(format!("{}", "URB_BROADCAST"));
                    },
                    Variant::SCD => {
                        non_blocking_op = true;
                        let _ = msg_send.send(format!("{}", "SCD_BROADCAST"));
                    },
                    Variant::COUNTER => {
                        if iter % (5 + SETTINGS.node_id()) == 0 && SETTINGS.is_writer()  {
                            non_blocking_op = true;
                            let _ = msg_send.send(format!("{}", "COUNTER_INCREASE"));
                        } else if SETTINGS.is_crashing_node() && !is_reading {
                            non_blocking_op = false;
                            let _ = msg_send.send(format!("{}", "COUNTER_READ"));
                            is_reading = true;
                        }
                    },
                    Variant::SNAPSHOT => {
                        if iter % (5 + SETTINGS.node_id()) == 0 && SETTINGS.is_writer()  {
                            non_blocking_op = false;
                            let _ = msg_send.send(format!("{}", "SNAPSHOT_WRITE"));
                            is_reading = true;
                        } else if SETTINGS.is_crashing_node() && !is_reading {
                            non_blocking_op = false;
                            let _ = msg_send.send(format!("{}", "SNAPSHOT"));
                        }
                    }
                }
//                let _ = msg_send.send(format!("{}", iter));
                let recv = if non_blocking_op {
                    let r = status_recv.recv_timeout(Duration::from_micros(100));
                    if r.is_ok() {
                        Ok(r.unwrap())
                    } else {
                        Err("error")
                    }
                } else {
                    let r = status_recv.recv_timeout(Duration::from_secs(20));
                    if r.is_ok() {
                        Ok(r.unwrap())
                    } else {
                        Err("error")
                    }
                };
                if let Ok(code) = recv {
                    match code {
                        StatusCode::Ok => {
                            if !is_reading {
                                if SETTINGS.print_client_operations() {
                                    printlnu(format!("Sent: '{}'", iter));
                                }
                            }
                        },
                        StatusCode::ResultReady => {
                            is_reading = false;
                        },
                        StatusCode::ResultNotReady => {
                            printlnu(format!("RESULT NOT REaDY"));
                        },
                        StatusCode::ErrNoSpace => {}
                        _ => {}
                    }
                }
                iter +=1;
            }
        }
    } else {
        thread::sleep(SETTINGS.run_length());
    }
    println!("Stopping node");
    let _ = stop_tx.send(());

    //sleep_time_specified_by_arguments();
    //sleep_time_specified_by_arguments();
    loop{
        match status_recv.try_recv() {
            Err(TryRecvError::Empty) => {
                thread::sleep(Duration::from_secs(2));
            }
            Ok(StatusCode::Finished) => {break;}
            _=>{}
        }
    }
    let _ = stop_tx.send(());

    // Wait for URB node to handle records in the buffer
    // This is why congestion control is needed.


    // Wait for all threads to terminate
    thread::sleep(Duration::from_secs(2));

    if SETTINGS.record_evaluation_info() {
        /* let mut run_result = mediator.run_result();

         run_result.metadata.node_id = SETTINGS.node_id();
         run_result.metadata.run_length = SETTINGS.run_length().as_secs() as Int;

         let json = serde_json::to_string(&*run_result).unwrap();
         // printlnu(format!("{}", &json));
         // let mut hasher = DefaultHasher::new();
         // run_result.urb_delivered_msgs.hash(&mut hasher);
         // printlnu(format!("Total delivered msgs: {}.Hash of delivered msgs: {}", run_result.urb_delivered_msgs.len(),  hasher.finish()));

         let mut hasher = DefaultHasher::new();
         let delivered_msgs = scd.delivered_msgs.lock().unwrap();
         delivered_msgs.hash(&mut hasher);
         let broadcasted_msg_number = scd.broadcasted_msg_number.lock().unwrap();
         printlnu(format!("Total broadcasted number: {}. Total delivered msgs: {}.Hash of delivered msgs: {}", *broadcasted_msg_number, delivered_msgs.len(),  hasher.finish()));
         fs::write(
             arguments::run_result_file_name_from_node_id(SETTINGS.node_id()),
             json,
         )
             .expect("Could not write the json result file");*/
    }


    thread::sleep(Duration::from_secs(2));
}

fn start_client_threads_and_get_channel_send_ends(
    broadcaster: &Arc<SCD>,
//    pattern: &Arc<Pattern3>,
//    pattern: &Arc<Pattern4>,
    algorithm6: &Arc<Algorithm6>,
) -> Sender<()> {
    let (tx, rx) = mpsc::channel();

    let broadcaster = Arc::clone(broadcaster);
//    let pattern = Arc::clone(pattern);
    let algorithm_node = Arc::clone(algorithm6);

    thread::spawn(move || {
        client_application_sequentially_consistent_counter(rx, broadcaster, algorithm_node);
    });

//    thread::spawn(move || {
//        client_application_broadcast(rx, broadcaster, pattern);
//    });

//    thread::spawn(move || {
//        client_urb_broadcast(rx, broadcaster);
//    });

    tx
}

fn client_application_sequentially_consistent_counter(rx: Receiver<()>, broadcaster: Arc<SCD>, algorithm: Arc<Algorithm6>) {
    let mut iteration = 0;
    let alg_recv_node = Arc::clone(&algorithm);
    thread::spawn(move || {
        alg_recv_node.recv_loop();
    });
    loop {
        iteration += 1;
        printlnu(format!("Start broadcast {}", iteration));
        algorithm.increase();
        algorithm.read();
        printlnu(format!("End broadcast {}", iteration));
        thread::sleep(Duration::from_millis(100));
    }

}

//fn client_application_broadcast(rx: Receiver<()>, broadcaster: Arc<SCD>, pattern: Arc<Pattern3>) {
fn client_application_broadcast(rx: Receiver<()>, broadcaster: Arc<SCD>, pattern: Arc<Pattern4>) {
    let mut application_iteration = 0;
    loop {
        application_iteration += 1;
        printlnu(format!("Start broadcast {}", application_iteration));
        pattern.op();
        printlnu(format!("End broadcast {}", application_iteration));
        thread::sleep(Duration::from_millis(100));
    }

}

fn client_urb_broadcast(rx: Receiver<()>, broadcaster: Arc<SCD>) {
    let mut broadcast_number = 0;
    loop {
        broadcast_number += 1;

        if SETTINGS.print_client_operations() {
            printlnu(format!("Start broadcast {}", broadcast_number));
        }

        // broadcaster.scdBroadcast(format!("URB payload from {}: {}", SETTINGS.node_id(), broadcast_number));
        broadcaster.scdBroadcast(broadcast_number.to_string());

        if SETTINGS.print_client_operations() {
            printlnu(format!("End broadcast {}", broadcast_number));
        }

        thread::sleep(Duration::from_millis(100));
        match rx.try_recv() {
            Err(TryRecvError::Empty) => {}
            _ => break
        }
    }
}

fn sleep_time_specified_by_arguments() {
    if SETTINGS.run_length() == Duration::from_secs(0) {
        loop {
            thread::sleep(Duration::from_secs(60));
        }
    } else {
        thread::sleep(SETTINGS.run_length());
    }
}

fn ping_all_nodes() -> HashMap<NodeId, f64> {
    let mut latency_vector = HashMap::new();
    let string = fs::read_to_string("latency.txt").expect("Unable to read the hosts file.");
    let (_, sstr1) = string.split_at(1);
    let (sstr2, _) = sstr1.split_at(sstr1.len()-2);
    let split: Vec<&str> = sstr2.split(",").collect();
    for s in split.clone() {
        let s1: Vec<&str> = s.split(":").collect();
        let mut node_id = -1;
        let mut latency= 0.0;
        if s1.len() >= 2 {
            if let Ok(id) = s1[0].parse::<i32>() {
                node_id = id;
            } else {
                node_id = -1;
            }
            if let Ok(lat) = s1[1].parse::<f64>() {
                latency = lat
            } else {
                latency = 0 as f64;
            }
        }
        latency_vector.insert(node_id, latency);
    }
    latency_vector
}

