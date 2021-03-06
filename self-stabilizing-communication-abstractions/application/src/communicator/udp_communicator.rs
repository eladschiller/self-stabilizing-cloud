use std::sync::{Arc, Weak, Mutex};
use std::collections::HashMap;
use std::net::UdpSocket;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str;
use std::thread;

use commons::types::NodeId;
use super::CommunicatorDelegate;

pub struct UDPCommunicator<D> {
    socket: UdpSocket,
    socket_addrs: HashMap<NodeId, SocketAddr>,
    delegate: Weak<D>,
}

impl<D: CommunicatorDelegate + Send + Sync + 'static> UDPCommunicator<D> {
    pub fn new(
        own_socket_addr: SocketAddr,
        socket_addrs: HashMap<NodeId, SocketAddr>,
        delegate: Weak<D>,
    ) -> Arc<UDPCommunicator<D>> {
        let own_socket_addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            own_socket_addr.port(),
        );
        let socket = UdpSocket::bind(own_socket_addr).expect("Could not create socket.");

        let communicator = UDPCommunicator {
            socket: socket,
            socket_addrs: socket_addrs,
            delegate: delegate,
        };
        let communicator = Arc::new(communicator);
        // let recv_thread_communicator = Arc::clone(&communicator);
        // thread::spawn(move || {
        //     recv_thread_communicator.recv_loop();
        // });
        communicator
    }

    pub fn start_recv_thread(communicator: &Arc<Self>) {
        let recv_thread_communicator = Arc::clone(&communicator);
        thread::spawn(move || {
            recv_thread_communicator.recv_loop();
        });
    }

    pub fn stop_thread(&self) {
        
    }

    pub fn recv_loop(&self) {
        loop {
            // let mut buf = [0; 4096];
            let mut buf = [0; 100000];

            let amt = self
                .socket
                .recv(&mut buf)
                .expect("Error receiving from socket");
            let json_string = str::from_utf8(&buf[0..amt]).expect("Error converting bytes to utf8");

            if let Some(delegate) = self.delegate() {
                delegate.json_received(json_string);
            }
        }
    }

    fn delegate(&self) -> Option<Arc<D>> {
        self.delegate.upgrade()
    }

    pub fn send_json_to(&self, json: &str, receiver_id: NodeId) {
        let bytes = json.as_bytes();
        let dst_socket_addr = self
            .socket_addrs
            .get(&receiver_id)
            .expect("Could not find receiver among the socket addresses");
        self.socket
            .send_to(bytes, dst_socket_addr)
            .expect("Could not send on the socket");
    }
}
