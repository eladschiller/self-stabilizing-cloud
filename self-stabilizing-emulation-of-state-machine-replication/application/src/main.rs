#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
#![allow(non_snake_case)]

mod communicator;
mod configuration_manager;
mod mediator;
mod responsible_cell;
mod settings;
mod terminal_output;
mod urb;

use std::fs;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::thread;
use std::time::{Duration,Instant};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use commons::arguments;
use commons::types::Int;

use crate::mediator::Mediator;
use crate::settings::SETTINGS;
use crate::terminal_output::printlnu;
use urb::NodeDelegate;
use crate::urb::types::Tag;

use std::cmp;

use std::collections::VecDeque;


fn main() {
    SETTINGS.node_id();

    let mediator = Mediator::new();

    // This is important when running locally. If some application
    // processes start before all have been built, they will
    // consume so much CPU time that the build processes
    // are very slow, and hence some nodes will be run for
    // a longer time than others.
    // thread::sleep(Duration::from_millis(
    //     1000 * SETTINGS.number_of_nodes() as u64,
    // ));

    // Wait for all nodes to start.
    thread::sleep(Duration::from_millis(1000 * SETTINGS.number_of_nodes() as u64));

    let tx = start_client_threads_and_get_channel_send_ends(&mediator);

    sleep_time_specified_by_arguments();

    let _ = tx.send(());

    // Wait for URB node to handle records in the buffer
    // This is why congestion control is needed.
    //thread::sleep(Duration::from_secs(60));
    thread::sleep(Duration::from_millis(1000 * SETTINGS.number_of_nodes() as u64));

    mediator.stop_all_threads();

    // Wait for all threads to terminate
    thread::sleep(Duration::from_millis(1000 * SETTINGS.number_of_nodes() as u64));

    if SETTINGS.record_evaluation_info() {
        let mut run_result = mediator.run_result();

        run_result.metadata.node_id = SETTINGS.node_id();
        run_result.metadata.run_length = SETTINGS.run_length().as_secs() as Int;

        let json = serde_json::to_string(&*run_result).unwrap();
        // printlnu(format!("{}", &json));
        let mut hasher = DefaultHasher::new();
        run_result.delivered_msgs.hash(&mut hasher);
        printlnu(format!("Total delivered msgs: {}.Hash of delivered msgs: {}", run_result.delivered_msgs.len(),  hasher.finish()));
        fs::write(
            arguments::run_result_file_name_from_node_id(SETTINGS.node_id()),
            json,
        )
        .expect("Could not write the json result file");
    }


    thread::sleep(Duration::from_millis(1000 * SETTINGS.number_of_nodes() as u64));
}

fn start_client_threads_and_get_channel_send_ends(mediator: &Arc<Mediator>) -> Sender<()> {
    let (tx, rx) = mpsc::channel();
    let arc_mediator = Arc::clone(mediator);
    thread::spawn(move || {
        client_to_urb(rx, arc_mediator);
    });
    tx
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct MultivaluedObject{
    seq:i32,
    v:i32,
    proposals:Vec<i32>,
    BC:Vec<BinaryObject>,
    tx_des:Tag,
    one_term:bool,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct BinaryObject{
    seq:i32,
    k:i32,
    r:i32,
    est:Vec<i32>,
    my_leader:i32,
    new_r:i32,
    tx_des:Tag,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct GlobalResetObject{
    phase: i32,
}

fn client_to_urb(rx: Receiver<()>, mediator: Arc<Mediator>) {
    /* Start, local variables omega_failure_detector */
    let number_of_nodes: i32 = SETTINGS.number_of_nodes(); /* total number of nodes in system */   
    let node_id: i32 = SETTINGS.node_id(); /* my node id */
    let mut r: i32 = 0; /* current round number */
    let max_gap_extrema_count: i32 = 21474836; /* max gap between the extrema of count values */
    let mut rec_from: Vec<i32> = vec![0; number_of_nodes as usize]; /* set of identities of the nodes that replied to the most recent query */
    let mut prev_rec_from : Vec<i32>; // = vec![0; number_of_nodes as usize]; /* Line 14 */
    let mut replied_r: Vec<i32>;
    let mut count: Vec<i32> = vec![0; number_of_nodes as usize]; /* set of nodes with counters for suspected failures */
    let mut first_bool = true;
    let mut count_responses; // = 0;
    let mut message_nr = 0; // USED FOR DEBUGGING ONLY, NOT NEEDED IN ALGORITHM
    let t: i32; // = 1; // t < n/2 !!! CHANGE THIS BASED ON NODES WE ALLOW TO BE SLOW OR FAIL !!!
    if number_of_nodes%2 == 0 {
        t = (number_of_nodes/2)-1;
    }else{
        t = number_of_nodes/2;
    }
    /* End, local variables omega_failure_detector */

    /* Start, local variables for binary consensus*/
    let mut txDes:Tag = Tag{id:-1, seq:-1}; /* URB transmission descriptor for decision sharing*/
    let n_minus_t: i32 = number_of_nodes - t; //any set of nMinust is a mojority set
    let mut rec_phase_0_from: Vec<i32> = vec![0; number_of_nodes as usize];
    let mut num_rec_phase_0: i32 = 0;
    let mut rec_phase_0_from_leader: bool = false;
    let mut rec_phase_0_same_leader: Vec<i32> = vec![0; number_of_nodes as usize];
    let mut rec_same_leader: i32 = 0;
    let mut get_v_from_leader: i32 = -1;
    let mut rec_phase_1_from: Vec<i32> = vec![0; number_of_nodes as usize];
    let mut num_rec_phase_1: i32 = 0;
    let mut rec: Vec<i32> = vec![-2; number_of_nodes as usize];
    /* End, local variables for binary consensus*/

    /* Start, local variables for multivalued consensus*/
    let m = 3; /* numer of multivalued objects */
    let mut CS: Vec<MultivaluedObject> = vec![MultivaluedObject{seq: -1, v: -1, proposals:vec![-1; number_of_nodes as usize], BC:vec![BinaryObject{seq:-1,k:-1,r:-1,est:vec![-1,-1,-1],my_leader:-1,new_r:-1,tx_des:Tag{id:-1,seq:-1}}; number_of_nodes as usize], tx_des:Tag{id:-1, seq:-1}, one_term:false}; m as usize];
    /* End, local variables for multivalued consensus*/

    /* Start, local variables for TO_URB */
    let mut to_broadcast_number:i32 = 0;
    let small_delta:i32 = 0;
    let mut obs_s:i32 = 0; // highest obsolete sequence number
    let mut sn:i32 = 0; // Start at -1 to start using multivalued obj at index 0
    let mut sync_bool:bool = true;
    let mut sync_rate_control:i32 = 0;
    let mut max_seq:i32 = 0;
    let mut rec_syn_ack:Vec<i32> = vec![-1;number_of_nodes as usize]; //List of SynAcks for sn
    let mut all_ready:(i32,i32) = (9999999, 9999999);
    let mut all_ready_converted:i32 = 0;
    let mut min_ready_vec = vec![(-1,-1);number_of_nodes as usize];
    let mut max_ready_vec = vec![(-1,-1);number_of_nodes as usize];
    let mut cond1:bool;
    let mut undelivered_bag:Vec<(i32,i32)> = Vec::new(); //Bag of undelivered TOmsg
    let mut delivered_bag:Vec<(i32,i32)> = Vec::new(); //Bag of delivered TOmsg
    let mut all_seq:VecDeque<(i32,i32)> = VecDeque::new(); // Its length if all execution is correct should be 1
    let mut reset_for_each:VecDeque<i32> = VecDeque::new();
    /* End, local variables for TO_URB */

    /* Start, global reset variables */
    let mut reset:bool = false; // We set this to true when we want to stop invoking new operations such as bin_prop, mul_prop
    let mut prp: Vec<i32> = vec![0;number_of_nodes as usize];
    let mut all: Vec<bool> = vec![false;number_of_nodes as usize];
    let mut all_seen: Vec<bool> = vec![false;number_of_nodes as usize];
    let mut echo_vec: Vec<(i32,bool)> = vec![(0,false);number_of_nodes as usize];
    let mut seq_res = 0;
    let mut seq_res_rec: Vec<i32> = vec![0;number_of_nodes as usize];
    let mut new_max_prp = 0;
    /* end, global reset variables */

    let now = Instant::now();
    let mut printOne = true;
    let mut abs_loop_iter = 0;

    loop {

        /* Start, an omega construction */
        r += 1;
        prev_rec_from = vec![0; number_of_nodes as usize];
        replied_r = vec![0; number_of_nodes as usize];
        replied_r[(node_id-1) as usize] = 1; // We need to assume that we are in the set of nodes that replied to the msot recent query at the start
        while first_bool {
            let json_alive = serde_json::to_string(&format!("ALIVE,{},{},{:?},", node_id, r, &count)).expect("Could not serialize a message");
            for node_x in 1..number_of_nodes+1 {
                if node_x != node_id && replied_r[(node_x-1) as usize] == 0{ // optimization implemented here
                    //printlnu(format!("Sent: {} |||| to node_id = {}", &json_alive, node_x));
                    mediator.send_json_to(&json_alive, node_x);
                }
                for x in 0..number_of_nodes*100{
                    let alive = mediator.get_alive_msg();
                    if alive.contains("ALIVE"){
                        mediator.rm_alive_msg();
                        let tokens: Vec<&str> = alive.split(",").collect();
                        let n_j = tokens[1].parse::<i32>().unwrap();
                        let r_j = tokens[2].parse::<i32>().unwrap();
                        for x in 0..number_of_nodes{
                            count[x as usize] = cmp::max(count[x as usize], tokens[(x as usize + 3)].to_string().replace("[","").replace("]","").replace(" ", "").parse::<i32>().unwrap());
                        }
                        check(&mut count, max_gap_extrema_count);
                        let json_response = serde_json::to_string(&format!("RESPONSE,{},{},{:?},{:?},", node_id, r_j, count, rec_from)).expect("Could not serialize a message");
                        //printlnu(format!("Sent: {} |||| to node_id = {}", &json_response, n_j));
                        mediator.send_json_to(&json_response, n_j);
                    }
                }
                let response = mediator.get_response_msg();
                if response.contains("RESPONSE"){
                    mediator.rm_response_msg();
                    //printlnu(format!("Got response: {}",response));
                    let tokens: Vec<&str> = response.split(",").collect();
                    let n_j = tokens[1].parse::<i32>().unwrap();
                    let r_j = tokens[2].parse::<i32>().unwrap();
                    for x in 0..number_of_nodes{
                        count[x as usize] = cmp::max(count[x as usize], tokens[(x as usize + 3)].to_string().replace("[","").replace("]","").replace(" ", "").parse::<i32>().unwrap());
                    }
                    check(&mut count, max_gap_extrema_count);
                    if r_j == r {
                        replied_r[(n_j as usize - 1)] = 1;
                        for x in 0..number_of_nodes{
                            prev_rec_from[x as usize] = cmp::max(prev_rec_from[x as usize], tokens[((x+ number_of_nodes) as usize + 3 )].to_string().replace("[","").replace("]","").replace(" ", "").parse::<i32>().unwrap());
                        }
                    }
                }
                count_responses = 0;
                for x in 0..number_of_nodes{
                    if replied_r[x as usize] == 1 {
                        count_responses += 1;
                    }
                }
                if count_responses >= number_of_nodes-t{
                    //printlnu(format!("Round {} Finished, time(ms) = {}, count = {:?}, leader = {}", r, now.elapsed().as_millis(), count, leader(&count)));
                    for n in 0..number_of_nodes{
                        let min_counts = count.iter().min();
                        let mut count_min = 0;
                        match min_counts {
                            Some(min) => count_min = *min,
                            None      => println!( "Vector is empty" ),
                        }
                        if (prev_rec_from[n as usize] == 0) && (n+1 != node_id) {
                            if count[n as usize] <  (max_gap_extrema_count + count_min) {
                                count[n as usize] += 1;
                            }
                        }
                    }
                    rec_from = vec![0; number_of_nodes as usize];
                    for n in 0..number_of_nodes {
                        if replied_r[n as usize] == 1 {
                            rec_from[n as usize] = 1;
                        }
                    }
                    first_bool = false;
                    check(&mut count, max_gap_extrema_count);
                    //printlnu(format!("count={:?}", count));
                    break;
                }
            }         
        }
        first_bool = true;
        //printlnu(format!("count={:?}", count));
        /* End, an omega construction */

        /* Start, a self-stabilizing algorithm for indulgent zero-degrading binary consensus */
        for cs_i in 0..m{
            if CS[cs_i as usize].seq != -1 {
                for bc_i in 0..number_of_nodes{
                    if CS[cs_i as usize].BC[bc_i as usize].est[0] != -1 && CS[cs_i as usize].BC[bc_i as usize].k != -1 && CS[cs_i as usize].BC[bc_i as usize].seq != -1 && (CS[cs_i as usize].BC[bc_i as usize].tx_des.seq == -1 || has_terminated(&CS[cs_i as usize].BC[bc_i as usize].tx_des, &mediator)){
                        /*Start work in BC obj*/
                        let mut received_decide = mediator.get_decide_msg();
                        if received_decide.contains("DECIDE"){
                            mediator.rm_decide_msg();
                            decide(received_decide, &mut CS, obs_s, m, number_of_nodes, &count);
                        }
                        rec = vec![-2; number_of_nodes as usize];
                        if CS[cs_i as usize].BC[bc_i as usize].est[2] == -1 && CS[cs_i as usize].BC[bc_i as usize].tx_des.id != -1 && has_terminated(&CS[cs_i as usize].BC[bc_i as usize].tx_des, &mediator){ // If est[2] = ⊥ AND txDes != ⊥ AND hasTerminated(txDes)
                            CS[cs_i as usize].BC[bc_i as usize].tx_des = Tag { id: -1, seq: -1};
                        }
                        if CS[cs_i as usize].BC[bc_i as usize].est[2] != -1 && (CS[cs_i as usize].BC[bc_i as usize].tx_des.id == -1 || has_terminated(&CS[cs_i as usize].BC[bc_i as usize].tx_des, &mediator)) { // If est[2] !=  ⊥ AND (txDes = ⊥ OR hasTerminated(txDes))
                            //printlnu(format!("ME DECIDE,{},{},{},", CS[cs_i as usize] as usize].BC[bc_i as usize].est[2], CS[cs_i as usize] as usize].BC[bc_i as usize].seq, CS[cs_i as usize] as usize].BC[bc_i as usize].k));
                            CS[cs_i as usize].BC[bc_i as usize].tx_des = mediator.urbBroadcast(format!("DECIDE,{},{},{},", CS[cs_i as usize].BC[bc_i as usize].est[2], CS[cs_i as usize].BC[bc_i as usize].seq, CS[cs_i as usize].BC[bc_i as usize].k)); // I have received a URB which set est[2] to a real value, now I URB this aswell                                        
                            continue;
                        }
                        CS[cs_i as usize].BC[bc_i as usize].my_leader = leader(&count);  /* read omega fd*/
                        CS[cs_i as usize].BC[bc_i as usize].r = cmp::max(CS[cs_i as usize].BC[bc_i as usize].r,CS[cs_i as usize].BC[bc_i as usize].new_r)+1;
                        let mut cond_var_1: bool = true;
                        let mut cond_var_2: bool = true;
                        num_rec_phase_0 = 0;
                        //printlnu(format!("Entering phase 0: r={}, cs_i={}, bc_i={}", CS[cs_i as usize].BC[bc_i as usize].r, cs_i, bc_i));
                        while cond_var_1 && cond_var_2{ /* Phase 0 : select a value with the help of omega */
                            /* HANDLE OTHER MESSAGES FROM FD and SYNC TOURB*/
                            for x in 0..number_of_nodes{
                                let mut received_sync = mediator.get_sync_msg();
                                if received_sync.contains("SYNC"){
                                    mediator.rm_sync_msg();
                                    let tokens:Vec<&str> = received_sync.split(",").collect();
                                    let snJ: i32 = tokens[1].parse::<i32>().unwrap(); // sn
                                    let pJ: i32 = tokens[2].parse::<i32>().unwrap(); // nodeid
                                    let mut getSeq:i32 = getSeq(obs_s, &mut CS);
                                    let mut sendTupleMax = ready_max(&undelivered_bag,&delivered_bag);
                                    let mut sendTupleMin = ready_min(&undelivered_bag,&delivered_bag);
                                    let json_syncack = serde_json::to_string(&format!("S_ACK,{},{},{},{},{},{},{},{},", snJ, getSeq, obs_s,sendTupleMax.0, sendTupleMax.1,sendTupleMin.0, sendTupleMin.1, node_id)).expect("Could not serialize a message");
                                    mediator.send_json_to(&json_syncack, pJ);
                                }
                                let alive = mediator.get_alive_msg();
                                if alive.contains("ALIVE"){
                                    mediator.rm_alive_msg();
                                    let tokens: Vec<&str> = alive.split(",").collect();
                                    let n_j = tokens[1].parse::<i32>().unwrap();
                                    let r_j = tokens[2].parse::<i32>().unwrap();
                                    for x in 0..number_of_nodes{
                                        count[x as usize] = cmp::max(count[x as usize], tokens[(x as usize + 3)].to_string().replace("[","").replace("]","").replace(" ", "").parse::<i32>().unwrap());
                                    }
                                    check(&mut count, max_gap_extrema_count);
                                    let json_response = serde_json::to_string(&format!("RESPONSE,{},{},{:?},{:?},", node_id, r_j, count, rec_from)).expect("Could not serialize a message");
                                    //printlnu(format!("Sent: {} |||| to node_id = {}", &json_response, n_j));
                                    mediator.send_json_to(&json_response, n_j);
                                }
                            }
                            /* HANDLE OTHER MESSAGES FROM FD and SYNC TOURB*/
                            let mut received_decide = mediator.get_decide_msg();
                            if received_decide.contains("DECIDE"){
                                mediator.rm_decide_msg();
                                decide(received_decide, &mut CS, obs_s, m, number_of_nodes, &count);
                            }
                            if CS[cs_i as usize].BC[bc_i as usize].est[2] != -1 || CS[cs_i as usize].BC[bc_i as usize].tx_des.seq != -1 {
                                cond_var_1 = false;
                            }
                            if num_rec_phase_0 >= n_minus_t || (rec_phase_0_from_leader || CS[cs_i as usize].BC[bc_i as usize].my_leader != leader(&count)){
                                cond_var_2 = false;
                            }
                            let json_broadcast_phase_0 = serde_json::to_string(&format!("PHASE,0,true,{},{},{},{},{},{},{},", CS[cs_i as usize].BC[bc_i as usize].r, CS[cs_i as usize].BC[bc_i as usize].est[0], CS[cs_i as usize].BC[bc_i as usize].my_leader, CS[cs_i as usize].BC[bc_i as usize].new_r, node_id, CS[cs_i as usize].BC[bc_i as usize].seq, CS[cs_i as usize].BC[bc_i as usize].k)).expect("Could not serialize a message");
                            for node_x in 1..number_of_nodes+1{
                                mediator.send_json_to(&json_broadcast_phase_0, node_x);
                            }
                            for x in 0..number_of_nodes*100{
                                let mut received_phase = mediator.get_phase_msg();
                                if received_phase.contains("PHASE"){
                                    mediator.rm_phase_msg();
                                    phase(received_phase, &mut CS, obs_s, m, &mediator, &count, node_id, number_of_nodes, cs_i, bc_i, &mut rec_phase_0_from, &mut rec_phase_1_from, &mut rec_phase_0_from_leader, &mut get_v_from_leader, &mut rec_phase_0_same_leader, &mut rec);
                                    num_rec_phase_0 = 0;
                                    rec_same_leader = 0;
                                    for x in 0..number_of_nodes{
                                        if rec_phase_0_from[x as usize] == 1 {
                                            num_rec_phase_0 += 1;
                                        }
                                        if rec_phase_0_same_leader[x as usize] == 1{
                                            rec_same_leader += 1;
                                        }
                                    }
                                }
                            }  
                        }
                        rec_phase_0_from = vec![0; number_of_nodes as usize];
                        rec_phase_0_from_leader = false;
                        rec_phase_0_same_leader = vec![0; number_of_nodes as usize];
                        if (rec_same_leader > number_of_nodes/2) && (get_v_from_leader != -1) {
                            CS[cs_i as usize].BC[bc_i as usize].est[1] = get_v_from_leader;
                            //printlnu(format!("r={}, cs_i={}, bc_i={}, received from leader={}, value={}", CS[cs_i as usize].BC[bc_i as usize].r, cs_i, bc_i, CS[cs_i as usize].BC[bc_i as usize].my_leader, get_v_from_leader));
                        }else{
                            CS[cs_i as usize].BC[bc_i as usize].est[1] = -1;
                        }
                        rec_same_leader = 0;
                        get_v_from_leader = -1;
                        let mut cond_var_3: bool = true;
                        let mut cond_var_4: bool = true;
                        num_rec_phase_1 = 0;
                        rec_phase_1_from = vec![0; number_of_nodes as usize];
                        //printlnu(format!("Entering phase 1. Does quasi-agreement proprty hold? : r={}, cs_i={}, bc_i={}, est[1]={}, my_leader={}", CS[cs_i as usize].BC[bc_i as usize].r, cs_i, bc_i, CS[cs_i as usize].BC[bc_i as usize].est[1], CS[cs_i as usize].BC[bc_i as usize].my_leader));
                        while cond_var_3 && cond_var_4 { /* Phase 1 : try to decide on an est[1] value */
                            /* HANDLE OTHER MESSAGES FROM FD and SYNC TOURB*/
                            for x in 0..number_of_nodes{
                                let mut received_sync = mediator.get_sync_msg();
                                if received_sync.contains("SYNC"){
                                    mediator.rm_sync_msg();
                                    let tokens:Vec<&str> = received_sync.split(",").collect();
                                    let snJ: i32 = tokens[1].parse::<i32>().unwrap(); // sn
                                    let pJ: i32 = tokens[2].parse::<i32>().unwrap(); // nodeid
                                    let mut getSeq:i32 = getSeq(obs_s, &mut CS);
                                    let mut sendTupleMax = ready_max(&undelivered_bag,&delivered_bag);
                                    let mut sendTupleMin = ready_min(&undelivered_bag,&delivered_bag);
                                    let json_syncack = serde_json::to_string(&format!("S_ACK,{},{},{},{},{},{},{},{},", snJ, getSeq, obs_s,sendTupleMax.0, sendTupleMax.1,sendTupleMin.0, sendTupleMin.1, node_id)).expect("Could not serialize a message");
                                    mediator.send_json_to(&json_syncack, pJ);
                                }
                                let alive = mediator.get_alive_msg();
                                if alive.contains("ALIVE"){
                                    mediator.rm_alive_msg();
                                    let tokens: Vec<&str> = alive.split(",").collect();
                                    let n_j = tokens[1].parse::<i32>().unwrap();
                                    let r_j = tokens[2].parse::<i32>().unwrap();
                                    for x in 0..number_of_nodes{
                                        count[x as usize] = cmp::max(count[x as usize], tokens[(x as usize + 3)].to_string().replace("[","").replace("]","").replace(" ", "").parse::<i32>().unwrap());
                                    }
                                    check(&mut count, max_gap_extrema_count);
                                    let json_response = serde_json::to_string(&format!("RESPONSE,{},{},{:?},{:?},", node_id, r_j, count, rec_from)).expect("Could not serialize a message");
                                    //printlnu(format!("Sent: {} |||| to node_id = {}", &json_response, n_j));
                                    mediator.send_json_to(&json_response, n_j);
                                }
                            }
                            /* HANDLE OTHER MESSAGES FROM FD and SYNC TOURB*/
                            let mut received_decide = mediator.get_decide_msg();
                            if received_decide.contains("DECIDE"){
                                mediator.rm_decide_msg();
                                decide(received_decide, &mut CS, obs_s, m, number_of_nodes, &count);
                            }
                            if CS[cs_i as usize].BC[bc_i as usize].est[2] != -1 || CS[cs_i as usize].BC[bc_i as usize].tx_des.seq != -1 {
                                cond_var_3 = false;
                            }
                            if num_rec_phase_1 >= n_minus_t {
                                cond_var_4 = false;
                            }
                            let json_broadcast_phase_1 = serde_json::to_string(&format!("PHASE,1,true,{},{},{},{},{},{},{},", CS[cs_i as usize].BC[bc_i as usize].r, CS[cs_i as usize].BC[bc_i as usize].est[1], CS[cs_i as usize].BC[bc_i as usize].my_leader, CS[cs_i as usize].BC[bc_i as usize].new_r, node_id, CS[cs_i as usize].BC[bc_i as usize].seq, CS[cs_i as usize].BC[bc_i as usize].k)).expect("Could not serialize a message");
                            for node_x in 1..number_of_nodes+1{
                                mediator.send_json_to(&json_broadcast_phase_1, node_x);
                            }
                            for x in 0..number_of_nodes*100{
                                let mut received_phase = mediator.get_phase_msg();
                                if received_phase.contains("PHASE"){
                                    mediator.rm_phase_msg();
                                    phase(received_phase, &mut CS, obs_s, m, &mediator, &count, node_id, number_of_nodes, cs_i, bc_i, &mut rec_phase_0_from, &mut rec_phase_1_from, &mut rec_phase_0_from_leader, &mut get_v_from_leader, &mut rec_phase_0_same_leader, &mut rec);
                                    num_rec_phase_1 = 0;
                                    for x in 0..number_of_nodes{
                                        if rec_phase_1_from[x as usize] == 1 {
                                            num_rec_phase_1 += 1;
                                        }
                                    }
                                }
                            }
                        }
                        rec_phase_1_from = vec![0; number_of_nodes as usize];
                        let mut n_minus_t_0 = rec.iter().filter(|&n| *n == 0).count();
                        let mut n_minus_t_1 = rec.iter().filter(|&n| *n == 1).count();
                        if rec.contains(&0) && rec.contains(&1) {
                            //panic!("pls dont rec={:?}, cs_i={}, bc_i={}", rec, cs_i, bc_i);
                            //printlnu(format!("Phase 1 done, rec has different values so i reset est 1"));
                            CS[cs_i as usize].BC[bc_i as usize].est[1] = -1;//vec![-1; number_of_nodes as usize];
                            continue;
                        }
                        else if (rec.contains(&0) || rec.contains(&1)) && rec.contains(&-1) {
                            if rec.contains(&0){
                                CS[cs_i as usize].BC[bc_i as usize].est[0] = 0;
                            }else{
                                CS[cs_i as usize].BC[bc_i as usize].est[0] = 1;
                            }
                        }        
                        else if (n_minus_t_0 >= n_minus_t as usize || n_minus_t_1 >= n_minus_t as usize) && CS[cs_i as usize].BC[bc_i as usize].tx_des.seq == -1 {
                            if rec.contains(&0){
                                //printlnu(format!("I decided 0 r={}, cs_i={}, bc_i={}, my_leader={}, rec={:?}", CS[cs_i as usize].BC[bc_i as usize].r, cs_i, bc_i, CS[cs_i as usize].BC[bc_i as usize].my_leader, rec));
                                CS[cs_i as usize].BC[bc_i as usize].tx_des = mediator.urbBroadcast(format!("DECIDE,{},{},{},",0, CS[cs_i as usize].BC[bc_i as usize].seq, CS[cs_i as usize].BC[bc_i as usize].k));
                            }else{
                                //printlnu(format!("I decided 1 r={}, cs_i={}, bc_i={}, my_leader={}, rec={:?}", CS[cs_i as usize].BC[bc_i as usize].r, cs_i, bc_i, CS[cs_i as usize].BC[bc_i as usize].my_leader, rec));
                                CS[cs_i as usize].BC[bc_i as usize].tx_des = mediator.urbBroadcast(format!("DECIDE,{},{},{},",1, CS[cs_i as usize].BC[bc_i as usize].seq, CS[cs_i as usize].BC[bc_i as usize].k));
                            }
                        }
                        else{
                            //printlnu(format!("Could not decide r={}, starting new round later! rec={:?}, cond_var_3={}, cond_var_4={}", CS[cs_i as usize].BC[bc_i as usize].r, rec, cond_var_3, cond_var_4));
                            continue;
                        }
                        /*End work in BC obj*/
                    }
                    //If BC obj was not activated we may need to read phase messages here in order to activate it
                }
            }
        }
        /* Check if there are any decide msgs*/
        let mut received_decide = mediator.get_decide_msg();
        if received_decide.contains("DECIDE"){
            mediator.rm_decide_msg();
            decide(received_decide, &mut CS, obs_s, m, number_of_nodes, &count);
        }
        /* Check if there are any decide msgs*/
        /* End, a self-stabilizing algorithm for indulgent zero-degrading binary consensus */

        /* Start, Self-stabilizing multivalued consensus */
        for cs_i in 0..m{
            if CS[cs_i as usize].v != -1 && (CS[cs_i as usize].tx_des.seq == -1 || has_terminated(&CS[cs_i as usize].tx_des, &mediator)) {
                CS[cs_i as usize].one_term = CS[cs_i as usize].one_term || (has_terminated(&CS[cs_i as usize].tx_des, &mediator) && CS[cs_i as usize].tx_des.seq != -1);
                CS[cs_i as usize].tx_des = mediator.urbBroadcast(format!("PROPOSAL,{},{},{},",CS[cs_i as usize].seq, CS[cs_i as usize].v, node_id));
            }
            /* invoke BC objects seperatly */
            if CS[cs_i as usize].one_term && k(CS[cs_i as usize].seq, number_of_nodes, &CS) < (number_of_nodes-1) && CS[cs_i as usize].BC[(k(CS[cs_i as usize].seq, number_of_nodes, &CS)+1) as usize].seq == -1 && (k(CS[cs_i as usize].seq, number_of_nodes, &CS)  == -1 || (CS[cs_i as usize].BC[k(CS[cs_i as usize].seq, number_of_nodes, &CS) as usize].est[2] != -1) && test(CS[cs_i as usize].seq, &mut CS, obs_s)) {
                let v:i32;
                if CS[cs_i as usize].proposals[(k(CS[cs_i as usize].seq, number_of_nodes, &CS)+1) as usize] != -1{
                    v = 1;
                }else{
                    v = 0;
                }
                bin_propose(CS[cs_i as usize].seq, k(CS[cs_i as usize].seq, number_of_nodes, &CS)+1, v, &mut CS, m, obs_s, &count);
            }
            /* invoke BC objects seperatly */
        }

        for x in 0..number_of_nodes*100{
            let mut received_proposal = mediator.get_proposal_msg();
            if received_proposal.contains("PROPOSAL"){
                mediator.rm_proposal_msg();
                proposal(received_proposal, &mut CS, obs_s, m, number_of_nodes);
            }
        }
        /* End, Self-stabilizing multivalued consensus */

        /* Start, TO-URB*/
        if abs_loop_iter%50 == 0 {
            if reset == false {
                to_broadcast_number += 1;
                mediator.urbBroadcast(format!("TOURB,{},{},",to_broadcast_number,node_id));
            }
        }

        for x in 0..number_of_nodes*100{
        let mut received_to_urb = mediator.get_to_urb_msg();
        if received_to_urb.contains("TOURB") {
            mediator.rm_to_urb_msg();
            let tokens:Vec<&str> = received_to_urb.split(",").collect();
            let toseqJ: i32 = tokens[1].parse::<i32>().unwrap(); // total order seq
            let senderJ: i32 = tokens[2].parse::<i32>().unwrap(); // sender id
            //Sort and insert
            let mut insertQuestion = 0;
            let mut rec_tuple = (senderJ, toseqJ);
            //Check if tuple already has arrived
            for x in 0..(undelivered_bag.len()) {
                if undelivered_bag[x] == rec_tuple {
                    insertQuestion = 1;
                }
            }
            //Check if tuple already has been delivered
            for x in 0..(delivered_bag.len()) {
                if delivered_bag[x] == rec_tuple {
                    insertQuestion = 1;
                }
            }
            //If tuple not already in undelivered_bag, add it to the bag and sort it
            //Sort Order = (1,1) (2,1) (3,1) (1,2) (2,2) (3,2) ... 
            if insertQuestion == 0 {
                //printlnu(format!("Inserting tuple = {:?}",rec_tuple));
                undelivered_bag.push(rec_tuple);
                undelivered_bag.sort();
                undelivered_bag.sort_by(|a, b| a.1.cmp(&b.1));
                //printlnu(format!("Current undelivered_bag = {:?}, ready_max={:?}",undelivered_bag,ready_max(&undelivered_bag,&delivered_bag)));
            }
        }
        }
        if reset == false {
            let json_sync = serde_json::to_string(&format!("SYNC,{},{},", sn, node_id)).expect("Could not serialize a message");
            for node_x in 1..number_of_nodes+1{
                mediator.send_json_to(&json_sync, node_x);
                let mut received_sync = mediator.get_sync_msg();
                if received_sync.contains("SYNC"){
                    mediator.rm_sync_msg();
                    let tokens:Vec<&str> = received_sync.split(",").collect();
                    let snJ: i32 = tokens[1].parse::<i32>().unwrap(); // sn
                    let pJ: i32 = tokens[2].parse::<i32>().unwrap(); // node_id
                    let mut getSeq:i32 = getSeq(obs_s, &mut CS);
                    let mut sendTupleMax = ready_max(&undelivered_bag,&delivered_bag);
                    let mut sendTupleMin = ready_min(&undelivered_bag,&delivered_bag);
                    let json_syncack = serde_json::to_string(&format!("S_ACK,{},{},{},{},{},{},{},{},", snJ, getSeq, obs_s,sendTupleMax.0, sendTupleMax.1,sendTupleMin.0, sendTupleMin.1, node_id)).expect("Could not serialize a message");
                    mediator.send_json_to(&json_syncack, pJ);
                }
                let mut received_syncack = mediator.get_syncack_msg();
                if received_syncack.contains("S_ACK"){
                    mediator.rm_syncack_msg();
                    let tokens:Vec<&str> = received_syncack.split(",").collect();
                    let snJ: i32 = tokens[1].parse::<i32>().unwrap(); // sn
                    let getSeqJ: i32 = tokens[2].parse::<i32>().unwrap();
                    let obs_sJ: i32 = tokens[3].parse::<i32>().unwrap();
                    let readyMaxAJ: i32 = tokens[4].parse::<i32>().unwrap();
                    let readyMaxBJ: i32 = tokens[5].parse::<i32>().unwrap();
                    let readyMinAJ: i32 = tokens[6].parse::<i32>().unwrap();
                    let readyMinBJ: i32 = tokens[7].parse::<i32>().unwrap();
                    let nJ: i32 = tokens[8].parse::<i32>().unwrap(); //Sender ID
                    let recTupleMax = (readyMaxAJ, readyMaxBJ);
                    max_ready_vec[(nJ-1) as usize] = recTupleMax;
                    let recTupleMin = (readyMinAJ, readyMinBJ);
                    min_ready_vec[(nJ-1) as usize] = recTupleMin;
                    if snJ == sn {
                        let mut cmpMin = 9999999;  // = tupleToInt(minReadyVec[0]);
                        for i in 0..max_ready_vec.len() {
                            if !(max_ready_vec[i] == (-1,-1)) {
                                cmpMin = cmp::min(cmpMin, tuple_to_int(max_ready_vec[i]));
                            }
                        } 
                        all_ready_converted = cmpMin;
                        max_seq = cmp::max(max_seq, getSeqJ);
                        if !all_seq.contains(&(getSeqJ,obs_sJ)){ // Union, only push to set if not already there
                            all_seq.push_front((getSeqJ,obs_sJ)); 
                        }   
                        rec_syn_ack[(nJ-1) as usize] = 1;
                        let mut allRec = 0;
                        for i in 0..number_of_nodes {
                            if rec_syn_ack[i as usize] == 1 {
                                allRec += 1;
                                //printlnu(format!("recSynAck = {:?} + sn = {}", recSynAck, sn));
                            }
                        }
                        //printlnu(format!("recSynAck = {:?} + sn = {}", recSynAck, sn));
                        //printlnu(format!("allRec={}, Trusted:{:?}, Trusted_length={}",allRec, mediator.trusted(), mediator.trusted().len()));
                        if allRec == mediator.trusted().len() {
                            sync_bool = false;
                            sn += 1;
                            //printlnu(format!("sn={}",sn));
                            //printlnu(format!("Done with SYNC for seq = {}, minReady={:?}, maxSeq={}, allSeq={:?}", sn, minReadyVec, maxSeq, allSeq));
                        }
                    }
        
                }
            }
        }
        
        if !sync_bool{
            let mut x = obs_s;
            let mut y = getSeq(obs_s, &mut CS);
            let mut z = max_seq;
            if !((x+1 == y && y == z) || (x == y && y == z) || (x == y && y == z-1)) {
                let mut max = cmp::max(x,y);
                obs_s = cmp::max(max,z);
            }
            reset_for_each.clear();
            if x < y {
                reset_for_each.push_front(x % 3);
            }
            reset_for_each.push_front(y % 3);
            if all_seq.len() == 1 {
                reset_for_each.push_front((z+1) % 3);
            }
            for i in 0..3 {
                if !(reset_for_each.contains(&i)) {
                    CS[i as usize].seq = -1;
                    CS[i as usize].v = -1;
                    CS[i as usize].proposals = vec![-1;SETTINGS.number_of_nodes() as usize];
                    CS[i as usize].BC = vec![BinaryObject{seq:-1,k:-1,r:-1,est:vec![-1,-1,-1],my_leader:-1,new_r:-1,tx_des:Tag{id:-1,seq:-1}}; number_of_nodes as usize];
                    CS[i as usize].tx_des = Tag{seq: -1, id: -1};
                    CS[i as usize].one_term = false;
                }
            }

            if all_seq.len() == 1 && obs_s == getSeq(obs_s, &mut CS) && big_delta(&small_delta, &mut min_ready_vec,&mut max_ready_vec,&mut CS, &mediator) && reset == false {
                //printlnu(format!("I propose, alreadyconverted = {} | obs_s = {} | getSeq = {} | maxSeq+1 = {}", all_ready_converted ,obs_s, getSeq(obs_s, &mut CS), max_seq+1));
                multi_propose(max_seq+1, all_ready_converted, m, &mut CS, obs_s);
            }

            if obs_s+1 == getSeq(obs_s, &mut CS) && CS[((obs_s+1)%3) as usize].seq != -1 && multi_result(obs_s+1, m, &mut CS, obs_s) != -1{
                if multi_result(obs_s+1, m, &mut CS, obs_s) != -2 {
                    bulk_read_and_deliver(multi_result(obs_s+1, m, &mut CS, obs_s), &mut undelivered_bag, &mut delivered_bag);
                    printlnu(format!("Time since we started = {} ms |||| number of messages we have delivered = {} |||| number of times we have total order delivered = {}", now.elapsed().as_millis(), delivered_bag[delivered_bag.len()-1].1*number_of_nodes ,obs_s+1));
                }else {
                    printlnu(format!("TRANSIENT FAULT"));
                } // Debugging
                obs_s += 1;
            }
            for i in 0..number_of_nodes {
                rec_syn_ack[i as usize] = 0; //Reset
            }
            sync_bool = true;
            let mut maxSeq:i32 = 0;
            all_seq.clear();
        }


        /* End, TO-URB*/

        /* Start, global reset */

        //Check if global reset is needed
        if max_int(r, sn) && reset == false {
            printlnu(format!("Global Reset"));
            reset = true;
            let mut echo_enable_count = 0;
            let mut zero_true_count = 0;
            if all_seen_fn(&mut all_seen, &mut all) {
                for i in 0..number_of_nodes {
                    if ((prp[(node_id-1) as usize], my_all(node_id-1, &mut prp, &mut all, &mut all_seen)) == echo_vec[i as usize]) && greater_equal(i, node_id-1, &mut prp, &mut all, &mut all_seen) {
                        echo_enable_count += 1;
                    }

                    if (prp[i as usize], all[i as usize]) == (0, true) {
                        zero_true_count += 1;
                    }
                }

                if (echo_enable_count == number_of_nodes) && (zero_true_count == number_of_nodes){
                    all_seen = vec![false;number_of_nodes as usize];
                    prp[(node_id-1) as usize] = 1;
                    all[(node_id-1) as usize] = false;
                }
            }
        }

        if prp.contains(&1) || prp.contains(&2) {
            reset = true;
        } else {
            reset = false;
        }

        let mut new_prp = prp[(node_id-1) as usize];
        prp[(node_id-1) as usize] = max_prp(&mut prp, &mut all, &mut all_seen);
        if new_prp != prp[(node_id-1) as usize] {
            new_max_prp = 1;
        }

        let mut echo_no_all_count = 0;
        for i in 0..number_of_nodes {
            if (prp[(node_id-1) as usize] == echo_vec[i as usize].0) && greater_equal(i, node_id-1, &mut prp, &mut all, &mut all_seen) {
                echo_no_all_count += 1;
            }
        }
        if echo_no_all_count == number_of_nodes {
            all[(node_id-1) as usize] = true;
        } else {
            if !(all[(node_id-1) as usize] == true && prp[(node_id-1) as usize] == 1) && !(all[(node_id-1) as usize] == true && prp[(node_id-1) as usize] == 2) || new_max_prp == 1 {
                all[(node_id-1) as usize] = false;
                new_max_prp = 0;
            }
        }

        for i in 0..number_of_nodes {
            if all[i as usize] == true {
                all_seen[i as usize] = true;
            }
        }

        let mut corr_deg_cond = false;
        let mut phase_cond = false;
        for k in 0..number_of_nodes {
            if !(corr_deg(k, node_id-1, &mut prp, &mut all, &mut all_seen)) {
                corr_deg_cond = true;
            }
        }
        for k in 0..number_of_nodes {
            if (((prp[(node_id-1) as usize] + 1) % 3) == prp[k as usize]) && all_seen[k as usize] != true {
                phase_cond = true;
            }
        }

        if corr_deg_cond || phase_cond {
            for k in 0..number_of_nodes {
                //printlnu(format!("Something went wrong, resetting global reset phase"));
                prp[k as usize] = 0;
                all[k as usize] = false;
            }
        }

        let mut echo_cond_var = false;
        let mut echo_cond_count = 0;
        for i in 0..number_of_nodes {
            if ((prp[(node_id-1) as usize], my_all(node_id-1, &mut prp, &mut all, &mut all_seen)) == echo_vec[i as usize]) && greater_equal(i, node_id-1, &mut prp, &mut all, &mut all_seen) {
                echo_cond_count += 1;
            }
        }
        
        if all_seen_fn(&mut all_seen, &mut all) && (echo_cond_count == number_of_nodes) {
            if prp[(node_id-1) as usize] == 1 {
                all_seen = vec![false;number_of_nodes as usize];
                prp[(node_id-1) as usize] = 2;
                all[(node_id-1) as usize] = false;
            } else if prp[(node_id-1) as usize] == 2 {
                //printlnu(format!("Resetting - Leaving final state"));
                all_seen = vec![false;number_of_nodes as usize];
                prp[(node_id-1) as usize] = 0;
                all[(node_id-1) as usize] = false;
                reset = false;
                sn = 0;
            }   
        }

        let mut send_my_all = my_all(node_id-1, &mut prp, &mut all, &mut all_seen);
        seq_res += 1;
        for node_x in 1..number_of_nodes+1 {
            //if node_x != node_id { //Dont send to own node?
                let json_echo = serde_json::to_string(&format!("ECHO,{},{},{:?},{},{:?},{},", node_id, prp[(node_id-1) as usize], send_my_all, prp[(node_x-1) as usize], all[(node_x-1) as usize], seq_res)).expect("Could not serialize a message");
                mediator.send_json_to(&json_echo, node_x);
            //}
        }

        for x in 0..number_of_nodes*100 {
            let echo = mediator.get_echo_msg();
            if echo.contains("ECHO"){
                mediator.rm_echo_msg();
                let tokens: Vec<&str> = echo.split(",").collect();
                let n_j = tokens[1].parse::<i32>().unwrap();
                let prp_j = tokens[2].parse::<i32>().unwrap();
                let all_j = tokens[3].parse::<bool>().unwrap();
                let prp_i = tokens[4].parse::<i32>().unwrap();
                let all_i = tokens[5].parse::<bool>().unwrap();
                let seq_res_j = tokens[6].parse::<i32>().unwrap();

                if seq_res_rec[(n_j-1) as usize] < seq_res_j {
                    if !(prp_j == 1 && all_j == false && prp[(n_j-1) as usize] == 1 && all[(n_j-1) as usize] == true) 
                    && !(prp_j == 2 && all_j == false && prp[(n_j-1) as usize] == 2 && all[(n_j-1) as usize] == true) {
                        if n_j != node_id {
                            prp[(n_j-1) as usize] = prp_j;
                            all[(n_j-1) as usize] = all_j;
                        }
                    }

                    echo_vec[(n_j-1) as usize] = (prp_i,all_i);
                    seq_res_rec[(n_j-1) as usize] = seq_res_j;
                }
            }
        }

        /* End, global reset */

        abs_loop_iter += 1;
        match rx.try_recv() {
            Err(TryRecvError::Empty) => {}
            _ => break
        }    
    }
    printlnu(format!("FINISHED"));
}

/* Start, macros and operations */

fn max_int(r:i32, sn:i32) -> bool{ //maxPrp()
    // Variables which we will check: r (FD), to_boradcast_number (TO-URB), obs_s (TO-URB), sn (TO-URB)
    let max_int = i32::MAX;
    if r == max_int || sn == max_int {
        return true;
    }
    return false;
}

fn phase_to_int(prp:i32, all:bool) -> i32{
    match (prp,all) {
        (0,false) => return 0,
        (0,true) => return 1,
        (1,false) => return 2,
        (1,true) => return 3,
        (2,false) => return 4,
        (2,true) => return 5,
        _ => panic!("INVALID INPUT TO phase_to_int"),
    }
}

fn max_prp(prp: &mut Vec<i32>, all: &mut Vec<bool>, all_seen: &mut Vec<bool>) -> i32 {
    let mut cond_var = 0;
    for i in 0..SETTINGS.number_of_nodes() {
        if (greater_equal(i, SETTINGS.node_id()-1, prp, all, all_seen)) {
            return mod_max(all_seen, prp);
        }
    }
    return prp[(SETTINGS.node_id()-1) as usize];
}

fn mod_max(all_seen: &mut Vec<bool>, prp: &mut Vec<i32>) -> i32{
    let max_prp = prp.iter().max();
    let mut prp_max = 0;
    match max_prp {
        Some(max) => prp_max = *max,
        None      => println!( "Vector is empty" ),
    }

    if prp.contains(&1) && !(prp.contains(&2)) && (prp[(SETTINGS.node_id()-1) as usize] != prp_max) {
        for i in 0..SETTINGS.number_of_nodes() {
            all_seen[i as usize] = false;
        }
        return prp_max;
    }
    return prp[(SETTINGS.node_id()-1) as usize];
}

fn all_seen_fn(all_seen: &mut Vec<bool>, all: &mut Vec<bool>) -> bool{ 
    let mut all_seen_count = 0;
    for i in 0..SETTINGS.number_of_nodes() {
        if all_seen[i as usize] == true {
            all_seen_count += 1;
        }
    }
    if all_seen_count == SETTINGS.number_of_nodes() && all[(SETTINGS.node_id()-1) as usize] {
        return true;
    }
    return false;
}

fn my_all(k:i32, prp: &mut Vec<i32>, all: &mut Vec<bool>, all_seen: &mut Vec<bool>) -> bool{
    if all[k as usize] == true /*&& k != SETTINGS.node_id()*/ {
        return true;
    }
    if k == (SETTINGS.node_id()-1) {
        for i in 0..SETTINGS.number_of_nodes() {
            if (all_seen[i as usize] == true) && (prp[i as usize] == ((prp[(SETTINGS.node_id()-1) as usize]+1)%3)) {
                return true;
            }
        }
    }
    return false;
}

fn greater_equal(k:i32, i:i32, prp: &mut Vec<i32>, all: &mut Vec<bool>, all_seen: &mut Vec<bool>) -> bool{ //GEQ()
    /*if (degree(k,prp,all,all_seen)-degree(i,prp,all,all_seen)) < 0 || (degree(k,prp,all,all_seen)-degree(i,prp,all,all_seen)) < 0{
        panic!("neg");
    }*/
    let mut deg = degree(k,prp,all,all_seen)-degree(i,prp,all,all_seen);
    if (deg.wrapping_rem_euclid(6)) == 1 || (deg.wrapping_rem_euclid(6)) == 0 {
        return true;
    }
    return false;
}

fn corr_deg(k:i32, i:i32, prp: &mut Vec<i32>, all: &mut Vec<bool>, all_seen: &mut Vec<bool>) -> bool{
   let mut xi = degree(i,prp,all,all_seen);
   let mut xk = degree(k,prp,all,all_seen);
   if (xi == xk) || (xi == ((xk+1)%6)) || (xk == ((xi+1)%6)) {
       return true;
   }
   return false;
}

fn degree(k:i32, prp: &mut Vec<i32>, all: &mut Vec<bool>, all_seen: &mut Vec<bool>) -> i32 {
    if my_all(k, prp, all, all_seen) {
        let mut return_variable = 2*prp[k as usize] + 1;
        return return_variable;
    } else {
        return 2*prp[k as usize];
    }
}

fn bulk_read_and_deliver(result:i32, undelivered_bag:&mut Vec<(i32,i32)> , delivered_bag:&mut Vec<(i32,i32)>) {
    let mut resultTuple = int_to_tuple(result);
    if delivered_bag.contains(&resultTuple) || (undelivered_bag.contains(&resultTuple) == false) {
        return
    }
    loop {
        if undelivered_bag[0] != resultTuple {
            delivered_bag.push(undelivered_bag[0]);
            undelivered_bag.remove(0);
        } else {
            delivered_bag.push(undelivered_bag[0]);
            undelivered_bag.remove(0);
            break;
        }
    }
}

fn big_delta(small_delta:&i32, minReady:&mut Vec<(i32,i32)>, maxReady:&mut Vec<(i32,i32)>, CS:&mut Vec<MultivaluedObject>, mediator: &Arc<Mediator>) -> bool{
    let mut l:i32 = 0; // May not be possible to have it init as 0
    let mut txDes0Term:bool = has_terminated(&CS[0].tx_des, mediator); // These return false as -1 has not terminated
    let mut txDes1Term:bool = has_terminated(&CS[1].tx_des, mediator);
    let mut txDes2Term:bool = has_terminated(&CS[2].tx_des, mediator);
    let mut allHaveTerminated:bool = false;
    if txDes0Term && txDes1Term && txDes2Term{
        allHaveTerminated = true;
    }
    //printlnu(format!("txDes0Term={}, txDes1Term={}, txDes2Term={}",txDes0Term,txDes1Term,txDes2Term));
    // big_delta NOT FINISHED IMPLEMENTATION
    for i in 0..(SETTINGS.number_of_nodes()-1) {
        l += tuple_to_int(maxReady[i as usize]) - tuple_to_int(minReady[i as usize]);
    }

    let mut ret:bool;
    if (allHaveTerminated && 0 < l) ||  *small_delta <= l{
        ret = true;
    }else{
        ret = false;
    }
    ret
}

fn tuple_to_int(tuple_to_int:(i32,i32)) -> i32{
    let mut n = SETTINGS.number_of_nodes();
    let mut tuple_cmp = (1,1);
    let mut index = 1;
    loop {
        if tuple_to_int == tuple_cmp {
            return index;
        } else {
            index += 1;
            if tuple_cmp.0 == n {
                tuple_cmp.0 = 1;
                tuple_cmp.1 = tuple_cmp.1 + 1;
            } else {
                tuple_cmp.0 = tuple_cmp.0 + 1;
            }
        }
    }
}

fn int_to_tuple(intToTuple:i32) -> (i32,i32){
    let mut n = SETTINGS.number_of_nodes();
    let mut indexCalc = 1;
    let mut returnTuple = (1,1);
    
    loop {
        if indexCalc == intToTuple {
            return returnTuple;
        } else {
            if returnTuple.0 == n {
                returnTuple.0 = 1;
                returnTuple.1 = returnTuple.1 + 1;
            } else {
                returnTuple.0 = returnTuple.0 + 1;
            }
        }
        indexCalc += 1;
    }
}

fn ready_min(undelivered_bag:&Vec<(i32,i32)> , delivered_bag:&Vec<(i32,i32)>) -> (i32,i32){
    let mut ready_min:(i32,i32) = (0,0);
    let mut next_to_deliver:(i32,i32);
    let mut n = SETTINGS.number_of_nodes();

    if delivered_bag.len() == 0 {
        next_to_deliver = (1,1);
        return next_to_deliver
    } else {
        next_to_deliver = delivered_bag[delivered_bag.len()-1];
        if next_to_deliver.0 == n {
            next_to_deliver.0 = 1;
            next_to_deliver.1 = next_to_deliver.1 + 1;
        } else {
            next_to_deliver.0 = next_to_deliver.0 + 1;
        }
        if undelivered_bag.contains(&next_to_deliver) {
            return next_to_deliver
        }
    }
    return next_to_deliver
}

fn ready_max(undelivered_bag:&Vec<(i32,i32)> , delivered_bag:&Vec<(i32,i32)>) -> (i32,i32){
    let mut ready_max:(i32,i32) = (1,1);
    let mut max_bag_length = cmp::max(undelivered_bag.len(), delivered_bag.len());
    let mut num_nodes: i32 = SETTINGS.number_of_nodes();
    let mut all_has:i32;
    for seq in 1..max_bag_length{ 
        all_has = 0;
        for node_itr in 1..num_nodes+1{
            if undelivered_bag.contains(&(node_itr,seq as i32)) || delivered_bag.contains(&(node_itr,seq as i32)){
                ready_max = (node_itr,seq as i32);
                all_has += 1;
            }else{
                all_has = 0;
                break;
            }
        }   
        if all_has != num_nodes{
            break;
        }
    }
    ready_max
}

fn multi_result(s:i32, m:i32, CS:&mut Vec<MultivaluedObject>, obs_s:i32) -> i32{
    let n = SETTINGS.number_of_nodes();
    if CS[(s%m) as usize].seq == -1 || !test(s, CS, obs_s){
        return -1 //line 41 no result yet
    }
    else if k(s,n,&CS) >= (n-1) || !test(CS[(s%m) as usize].seq, CS, obs_s) || CS[(s%m) as usize].seq != s || CS[(s%m) as usize].v == -1 {
        //printlnu(format!("Transient 1 fault caused by: {:?}", CS[(s%m) as usize]));
        //panic!();
        return -2 //line 42
    }
    else if CS[(s%m) as usize].BC[(k(s,n,&CS)+1) as usize].seq == -1 || (CS[(s%m) as usize].BC[(k(s,n,&CS)+1) as usize].est[2] == -1 || !test(s, CS, obs_s)) { //Careful here k(s) can return n and n+1 will be out of bounds for BC[]
        return -1 //line 43 no result yet
    }
    else if CS[(s%m) as usize].proposals[(k(s,n,&CS)+1) as usize] == -1 {
        //printlnu(format!("Transient 2 fault caused by: {:?}", CS[(s%m) as usize]));
        //panic!();
        return -2 //line 44
    }
    else{
        return CS[(s%m) as usize].proposals[(k(s,n,&CS)+1) as usize]; //line 44
    }
}

fn multi_propose(s:i32, v:i32, m:i32, CS:&mut Vec<MultivaluedObject>, obs_s:i32){ 
    if test(s, CS, obs_s) && v != -1 && CS[(s%m) as usize].seq == -1{
        CS[(s%m) as usize].seq = s;
        CS[(s%m) as usize].v = v;
        CS[(s%m) as usize].proposals = vec![-1;SETTINGS.number_of_nodes() as usize];
        CS[(s%m) as usize].BC = vec![BinaryObject{seq: -1, k:-1, r:-1, est:vec![-1,-1,-1], my_leader:-1, new_r:-1,tx_des:Tag{id:-1,seq:-1}};SETTINGS.number_of_nodes() as usize];
        CS[(s%m) as usize].tx_des = Tag{id:-1, seq:-1};
    }
}

fn k(s:i32,n:i32,CS:&Vec<MultivaluedObject>) -> i32{ /* k(s) is the highest (consecutive) BC entry index with the decision False, cf. k (line 4) */
    let mut get_i:i32 = -1;
    for i in 0..n{
        if CS[(s%3) as usize].BC[i as usize].est[2] == 0 { // Check that the result is false == 0
            get_i = i;
        }else {
            return get_i;
        }
    }
    get_i
}

fn proposal(received_proposal: String, CS: &mut Vec<MultivaluedObject>, obs_s: i32, m: i32, number_of_nodes: i32) {
    let tokens:Vec<&str> = received_proposal.split(",").collect();
    let vJ: i32 = tokens[2].parse::<i32>().unwrap();
    let nodeJ: i32 = tokens[3].parse::<i32>().unwrap();
    let sJ: i32 = tokens[1].parse::<i32>().unwrap();
    if test(sJ, CS, obs_s) && vJ != -1 {
        if CS[(sJ%m) as usize].seq != -1 && CS[(sJ%m) as usize].proposals[(nodeJ-1) as usize] == -1 {
            CS[(sJ%m) as usize].proposals[(nodeJ-1) as usize] = vJ;
        }
        else if CS[(sJ%m) as usize].seq == -1{
            CS[(sJ%m) as usize].seq = sJ;
            CS[(sJ%m) as usize].v = vJ;
            CS[(sJ%m) as usize].proposals = vec![-1; number_of_nodes as usize];
            CS[(sJ%m) as usize].BC = vec![BinaryObject{seq:-1,k:-1,r:-1,est:vec![-1,-1,-1],my_leader:-1,new_r:-1,tx_des:Tag{id:-1,seq:-1}}; number_of_nodes as usize];
            CS[(sJ%m) as usize].tx_des = Tag{id:-1, seq:-1};
            CS[(sJ%m) as usize].one_term = false;
            CS[(sJ%m) as usize].proposals[(nodeJ-1) as usize] = vJ;
        }
    }

}

fn decide(received_decide: String, CS: &mut Vec<MultivaluedObject>, obs_s: i32, m: i32, number_of_nodes: i32, count: &Vec<i32>){
    let tokens:Vec<&str> = received_decide.split(",").collect();
    let seqJ: i32 = tokens[2].parse::<i32>().unwrap();
    let kJ: i32 = tokens[3].parse::<i32>().unwrap();
    let vJ: i32 = tokens[1].parse::<i32>().unwrap();
    if test(seqJ, CS, obs_s) && CS[(seqJ%m) as usize].seq != -1 {
        if CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].seq == -1 {
            CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].seq = seqJ;
            CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].r = 0;
            CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].est = vec![vJ,vJ,-1];
            CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].my_leader = leader(count);
            CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].new_r = 0;
        }
        if CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].est[2] == -1 {
            CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].est[2] = vJ;
        }
    }
}

fn phase(received_phase: String, CS: &mut Vec<MultivaluedObject>, obs_s: i32, m: i32, mediator: &Arc<Mediator>, count: &Vec<i32>, node_id: i32, number_of_nodes: i32, cs_i: i32, bc_i: i32, rec_phase_0_from: &mut Vec<i32>, rec_phase_1_from: &mut Vec<i32>, rec_phase_0_from_leader: &mut bool, get_v_from_leader: &mut i32, rec_phase_0_same_leader: &mut Vec<i32>, rec: &mut Vec<i32>){
    let tokens:Vec<&str> = received_phase.split(",").collect();
    let nJ: i32 = tokens[1].parse::<i32>().unwrap(); // The 0 from PHASE,0
    let aJ: String = tokens[2].to_string(); // true or false (Ack needed)
    let rJ: i32 = tokens[3].parse::<i32>().unwrap(); // the round
    let vJ: i32 = tokens[4].parse::<i32>().unwrap(); // the value
    let myLeaderJ = tokens[5].parse::<i32>().unwrap(); // the leader of the node who send the broadcast
    let newRJ = tokens[6].parse::<i32>().unwrap(); // the new round number
    let nodeJ = tokens[7].parse::<i32>().unwrap(); // The node who sent the broadcast
    let seqJ = tokens[8].parse::<i32>().unwrap(); // the sequence number
    let kJ = tokens[9].parse::<i32>().unwrap(); // The k value
    if (!test(seqJ, CS, obs_s) || CS[(seqJ%m) as usize].seq == -1) && aJ == "true".to_string(){
        let json_ack_phase = serde_json::to_string(&format!("PHASE,{},false,{},{},{},{},{},{},{},", nJ, rJ, vJ, leader(count), CS[cs_i as usize].BC[bc_i as usize].new_r, node_id, seqJ, kJ)).expect("Could not serialize a message");
        mediator.send_json_to(&json_ack_phase, nodeJ);
        return;
    }
    if CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].seq == -1 {
        CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].seq = seqJ;
        CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].k = kJ;
        CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].r = rJ;
        CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].est = vec![vJ,-1,-1];
        CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].my_leader = leader(count);
        CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].new_r = cmp::max(rJ, newRJ);
        CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].tx_des = Tag{id: -1, seq: -1};
    }else {
        let mut max_0 = cmp::max(rJ, CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].new_r);
        CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].new_r = cmp::max(max_0, newRJ);
    }
    if /*test(seqJ, CS, obs_s) &&*/ nJ == 1 && CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].est[1] == -1 {
        //CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].est[1] = vJ;
        //printlnu(format!("IS THIS OK? from:{} got vJ={}, cs_i={}, bc_i={}, seqJ={}, kJ={}", nodeJ, vJ, cs_i, bc_i, seqJ%m, kJ%number_of_nodes));
    }
    if aJ == "true".to_string() {
        let json_ack_phase = serde_json::to_string(&format!("PHASE,{},false,{},{},{},{},{},{},{},", nJ, rJ, vJ, CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].my_leader, cmp::max(CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].r, CS[(seqJ%m) as usize].BC[(kJ%number_of_nodes) as usize].new_r), node_id, seqJ, kJ)).expect("Could not serialize a message");
        mediator.send_json_to(&json_ack_phase, nodeJ);
    }
    if aJ == "true".to_string() && seqJ == CS[cs_i as usize].BC[bc_i as usize].seq && kJ == CS[cs_i as usize].BC[bc_i as usize].k && rJ == CS[cs_i as usize].BC[bc_i as usize].r{
        if nJ == 0 {
            rec_phase_0_from[(nodeJ-1) as usize] = 1;
            if myLeaderJ == CS[cs_i as usize].BC[bc_i as usize].my_leader{
                rec_phase_0_same_leader[(nodeJ-1) as usize] = 1;
                //printlnu(format!("Set rec_phase_0_same_leader[{}]=1, cs_i={}, bc_i={}", nodeJ-1, cs_i, bc_i));
            }
            if nodeJ == CS[cs_i as usize].BC[bc_i as usize].my_leader {
                //printlnu(format!("p0, est[0]={}, leader={}, value={}, cs_i={}, bc_i={}, seqJ={}, kJ={}", CS[cs_i as usize].BC[bc_i as usize].est[0], nodeJ, vJ, cs_i, bc_i, seqJ, kJ));
                *get_v_from_leader = vJ;
                *rec_phase_0_from_leader = true;
                //printlnu(format!("From leader Node_id:{} got vJ={}, cs_i={}, seqJ={}, bc_i={}, kJ={}, r={}", nodeJ, vJ, cs_i, seqJ, bc_i, kJ, rJ));
            }
        }
        if nJ == 1 {
            //printlnu(format!("From node={}, received vJ={}, at cs_i={}, bc_i={}", nodeJ, vJ, cs_i, bc_i));
            rec_phase_1_from[(nodeJ-1) as usize] = 1;
            rec[(nodeJ-1) as usize] = vJ;
            //printlnu(format!("p1, est[1]={}, leader={}, value={}, cs_i={}, bc_i={}, seqJ={}, kJ={}", CS[cs_i as usize].BC[bc_i as usize].est[1], leader(count), vJ, cs_i, bc_i, seqJ, kJ));
        }
    }else{
        //printlnu(format!("it did not match, aJ={}, rJ={}, r={}, seqJ={}, seq={}, kJ={}, k={}", aJ, rJ, CS[cs_i as usize].BC[bc_i as usize].r, seqJ, CS[cs_i as usize].BC[bc_i as usize].seq, kJ, CS[cs_i as usize].BC[bc_i as usize].k));
    }
    if aJ == "false".to_string() && seqJ == CS[cs_i as usize].BC[bc_i as usize].seq && kJ == CS[cs_i as usize].BC[bc_i as usize].k && rJ == CS[cs_i as usize].BC[bc_i as usize].r{ 
        if nJ == 0 {
            rec_phase_0_from[(nodeJ-1) as usize] = 1;
        }
        if nJ == 1 {
            rec_phase_1_from[(nodeJ-1) as usize] = 1;
        }
    }
}

fn test(s: i32, CS: &mut Vec<MultivaluedObject>, obs_s: i32) -> bool {
    if s == CS[0].seq || s == CS[1].seq || s == CS[2].seq || s == getSeq(obs_s, CS)+1 {
        return true;
    }
    return false;
}

fn getSeq(obs_s: i32, CS: &mut Vec<MultivaluedObject>) -> i32{
    let mut max0:i32 = cmp::max(CS[0].seq,CS[1].seq);
    let mut max1:i32 = cmp::max(max0,CS[2].seq);
    let mut max2:i32 = cmp::max(max1,obs_s);
    max2
}

fn leader(count: &Vec<i32>) -> i32 { // operation leader
    let min_c = count.iter().min();
    let mut c_min = 0;
    match min_c{
        Some(min) => c_min = *min,
        None      => println!( "Vector is empty" ),
    }
    let index = count.iter().position(|&r|r == c_min).unwrap();
    index as i32 +1// This is the index of the lowest value in count aka the leader, +1 cuz index starts at 0 which is node 1
}

fn check(count: &mut Vec<i32>, max_gap_extrema_count: i32){ // macro check
    let max_c = count.iter().max();
    let mut c_max = 0;
    match max_c {
        Some(max) => c_max = *max,
        None      => println!( "Vector is empty" ),
    }
    let minC = count.iter().min();
    let mut c_min = 0;
    match minC {
        Some(min) => c_min = *min,
        None      => println!( "Vector is empty" ),
    }
    if c_max - c_min > max_gap_extrema_count{
        for x in 0..SETTINGS.number_of_nodes(){
            let newMax = cmp::max(count[x as usize],c_max-max_gap_extrema_count);
            count[x as usize] = newMax;
        }
    }
}

fn has_terminated(tx_des: &Tag, mediator: &Arc<Mediator>) -> bool { // The predicate is true when sender knows that all non-failing nodes has delivered m
    let has_terminated = mediator.has_terminated(tx_des);
    has_terminated
}

fn bin_propose(s: i32, k: i32, v: i32, CS: &mut Vec<MultivaluedObject>, m: i32, obs_s: i32, count: &Vec<i32>) {
    if test(s, CS, obs_s) && CS[(s%m) as usize].seq != -1 && CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].seq == -1{
        CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].seq = s;
        CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].k = k;
        CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].r = 0;
        CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].est = vec![v,-1,-1];
        CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].my_leader = leader(count);
        CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].new_r = 0;
        CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].tx_des = Tag{id: -1, seq: -1};
    }else{
        printlnu(format!("I did not start BC obj"));
    }
}

fn bin_result(s: i32, k: i32, CS:&mut Vec<MultivaluedObject>, m:i32, obs_s: i32) -> i32 {
    if !test(s, CS, obs_s) || CS[(s%m) as usize].seq == -1 || CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].seq == -1{
        return -1;
    }
    return CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].est[2];
}

fn deactivate(s: i32, k: i32, CS: &mut Vec<MultivaluedObject>, m: i32, obs_s: i32) {
    if !test(s, CS, obs_s) || CS[(s%m) as usize].seq != -1 {
        CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].seq = -1;
        CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].k = -1;
        CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].r = -1;
        CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].est = vec![-1,-1,-1];
        CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].my_leader = -1;
        CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].new_r = -1;
        CS[(s%m) as usize].BC[(k%SETTINGS.number_of_nodes()) as usize].tx_des = Tag{id: -1, seq: -1};
    }
}
/* End, macros and operations */

fn sleep_time_specified_by_arguments() {
    if SETTINGS.run_length() == Duration::from_secs(0) {
        loop {
            thread::sleep(Duration::from_secs(60));
        }
    } else {
        thread::sleep(SETTINGS.run_length());
    }
}
