use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, ToSocketAddrs};
use std::fs;
use fastping_rs::Pinger;
use fastping_rs::PingResult::{Idle, Receive};

fn main() {
    let socket_addrs = socket_addrs_from_matches();
    let link_latencies = ping_all_nodes(socket_addrs.clone());
    let mut s = String::new();
    for (node_id, addr) in link_latencies {
        s.push_str(format!("{}:{},", node_id, addr).as_str());
    }
    let _ = fs::write(
        "latency.txt",
        format!("{:?}",s)
    );
}

fn ping_all_nodes(s_addrs: HashMap<i32, SocketAddr>) -> HashMap<i32, f64> {
    let mut ping_recv_set = HashSet::new();
    let mut ping_attempt = 0;
    let mut latency_vector = HashMap::new();
    let (pinger, results) = match Pinger::new(None, None) {
        Ok((pinger, results)) => (pinger, results),
        Err(e) => panic!("Error creating pinger: {}", e)
    };
    for (_, addrs) in s_addrs.clone() {
        pinger.add_ipaddr(format!("{}", addrs.ip()).as_ref());
    }
    pinger.run_pinger();
    'outer: loop {
        match results.recv() {
            Ok(result) => {
                match result {
                    Idle { addr: _ } => {
//                        println!("Idle Address {}.", addr);
                        ping_attempt += 1;
                        if ping_attempt == 3 {
                            break 'outer;
                        }
                    },
                    Receive { addr, rtt } => {
//                        println!("Node {} Receive from Address {} in {:?}.", SETTINGS.node_id(), addr, rtt);
                        for (node, addrs) in &s_addrs {
                            if addrs.ip() == addr {
                                ping_recv_set.insert(node);
//                                println!("Node {} counter {} nodes {} addr.ip {:?} addr {:?}", SETTINGS.node_id(), ping_recv_counter, SETTINGS.socket_addrs().keys().len(),addrs.ip(), addr);
                                latency_vector.insert(*node, rtt.as_secs_f64());
                                if ping_recv_set.len() == s_addrs.keys().len() {
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
            },
            Err(_) => panic!("Worker threads disconnected before the solution was found!"),
        }
    }
    latency_vector
}

fn socket_addrs_from_matches() -> HashMap<i32, SocketAddr> {
    let hosts_file_path = "../application/hosts.txt";
    let string = fs::read_to_string(hosts_file_path).expect("Unable to read file");
    socket_addrs_from_string(string)
}

fn socket_addrs_from_string(string: String) -> HashMap<i32, SocketAddr> {
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

