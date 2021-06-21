/*
use std::collections::HashMap;
use std::time::Duration;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str;
use std::sync::{Arc, Weak};
use std::thread;
use std::os::unix::io::AsRawFd;
use std::io::ErrorKind;

use commons::types::NodeId;
use super::CommunicatorDelegate;
use crate::urb::NodeDelegate;
use super::dccp::*;

use mio::{Ready, Poll, PollOpt, Token, Events};
use mio::unix::EventedFd;
use nix::sys::socket::{SockAddr, InetAddr};

const SERVICE_CODE:i32 = 42;

pub struct DCCPCommunicator<D> {
    sockets: HashMap<NodeId, DCCPSocket>,
    delegate: Weak<D>,
}

impl<D: CommunicatorDelegate + NodeDelegate + Send + Sync + 'static> DCCPCommunicator<D> {
    pub fn new(
        own_socket_addr: SocketAddr,
        socket_addrs: HashMap<NodeId, SocketAddr>,
        delegate: Weak<D>,
    ) -> Arc<DCCPCommunicator<D>> {

        let sockets = establish_full_connection(own_socket_addr, socket_addrs.clone());

        let communicator = DCCPCommunicator {
            sockets: sockets,
            delegate: delegate,
        };
        let communicator = Arc::new(communicator);
        communicator
    }

    pub fn start_recv_thread(communicator: &Arc<Self>) {
        let recv_thread_communicator = Arc::clone(&communicator);
        thread::spawn(move || {
            recv_thread_communicator.recv_loop();
        });
    }

    pub fn recv_loop(&self) {
        let poll = match Poll::new() {
            Ok(poll) => poll,
            Err(e) => panic!("failed to create Poll instance; err={:?}", e),
        };

        // Register all sockets for interest of readiness.
        for (&id, conn) in &self.sockets {
            if id == -1 {
                continue;
            }
            let token_for_id = Token(id as usize);
            conn.set_nonblocking();
            poll.register(&EventedFd(&conn.as_raw_fd()),
                      token_for_id,
                      Ready::readable(),
                      PollOpt::edge()).expect("Error when registering socket.");
        }

        let mut events = Events::with_capacity(1024);

        loop {
            poll.poll(&mut events, None).expect("Error when polling.");

            for event in &events {
                let Token(node_id) = event.token();
                self.handle_socket_event(node_id as NodeId);
            }
        }
    }

    fn handle_socket_event(&self, node_id: NodeId) {
        let mut buf = [0; 4096];
        let socket = self.sockets.get(&(node_id as NodeId)).expect(&format!("No socket for Node {}", node_id));
        loop {
            // Needs to handle events in a loop because of edge triggering. See
            // Rustdoc of Poll for more details.
            match socket.recv(&mut buf) {
                Ok(0) => {
                    // Socket is orderly shutdown.
                    break;
                },
                Ok(amt) => {
                    let json_string = str::from_utf8(&buf[0..amt]).expect("Error converting bytes to utf8");

                    if let Some(delegate) = self.delegate() {
                        delegate.json_received(json_string);
                    }

                },
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No more operation can be performed on this socket.
                    break;
                },
                Err(e) => println!("Error occurs when receiving msg. e: {:?}", e),
            }
        }
    }

    fn delegate(&self) -> Option<Arc<D>> {
        self.delegate.upgrade()
    }

    pub fn send_json_to(&self, json: &str, receiver_id: NodeId) {
        let bytes = json.as_bytes();
        let receiver_id = if receiver_id == self.delegate().unwrap().node_id() {
            -1
        } else {
            receiver_id
        };
        let socket = self.sockets.get(&receiver_id).expect(&format!("No socket for Node {}", receiver_id));
        if let Err(e) = socket.send(bytes) {
            match e.kind() {
                ErrorKind::ConnectionReset | ErrorKind::BrokenPipe => {},
                _ => println!("Sending error: {}", e),
            }
        } 
    }
}

// Establish a fully connected grapgh by connecting actively to higher IDs and listening for
// connections from lower IDs.
// TODO: Here used a lot of cloned socket_addrs because of ownership and lifetime issues. Maybe
// there is a better way to not clone.
pub fn establish_full_connection(own_socket_addr: SocketAddr, socket_addrs: HashMap<NodeId, SocketAddr>) -> HashMap<NodeId, DCCPSocket> {


    let converted_own_addr = SockAddr::Inet(InetAddr::from_std(&own_socket_addr));
    let self_id = get_id_from_addr(converted_own_addr, &socket_addrs);
    let socket_addrs_copy = socket_addrs.clone();

    let connect_thread = thread::spawn(move || {
        // Wait for all node to get ready to accept connections.
        thread::sleep(Duration::from_millis(1000));
        let mut sockets = HashMap::new();
        for (&id, addr) in &socket_addrs_copy {
            if id >= self_id {
                let conn = DCCPSocket::connect(addr, SERVICE_CODE).expect("Fail when establishing connection.");
                conn.send(format!("{}", self_id).as_bytes()).expect("Sending identity failed.");
                sockets.insert(id, conn);
            } 
        }
        sockets
    });

    let own_socket_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        own_socket_addr.port(),
    );

    let listen_socket = DCCPListener::bind(own_socket_addr, SERVICE_CODE).expect("Could not create socket.");
    let mut received_conn_counter = 0;
    let mut sockets_for_lower_ids = HashMap::new();
    let mut buf = [0; 4];
    while received_conn_counter < self_id {
        let (conn, peer_addr) = listen_socket.accept().unwrap();

        let amt = conn.recv(&mut buf).unwrap(); 
        let id_str = str::from_utf8(&buf[0..amt]).expect("Unable to convert to string");
        let id: NodeId = id_str.parse().expect("Unable to parse string as identity.");

        // println!("Connection accepted from {:?}, whose id is {}", peer_addr, id);
        assert_eq!(id <= self_id, true);
        received_conn_counter += 1;
        sockets_for_lower_ids.insert(id, conn);
    }

    let mut sockets_for_higher_ids = connect_thread.join().unwrap();

    // Since we will have 2 socket ends for ourself, change one key to -1 to indicate it will be
    // used as the sending end. 
    let self_socket = sockets_for_higher_ids.remove(&self_id).unwrap();
    sockets_for_higher_ids.insert(-1, self_socket);

    sockets_for_lower_ids.into_iter().chain(sockets_for_higher_ids).collect()
   
}

pub fn get_id_from_addr(addr: SockAddr, socket_addrs: &HashMap<NodeId, SocketAddr>) -> NodeId {
    if let SockAddr::Inet(addr) = addr {
        for (id, socket_addr) in socket_addrs {
            let inet = InetAddr::from_std(&socket_addr);
            if inet == addr {
                return *id;
            } 
        } 
    }
    panic!("Unknown host connected.")
}
*/