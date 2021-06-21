/*
use std::io::prelude::*;
use std::net::ToSocketAddrs;
use std::os::unix::io::{AsRawFd, RawFd};
use std::{io, mem};

extern crate libc;
extern crate nix;
use log::warn;

use nix::sys::socket::*;
use nix::unistd::close;
use nix::fcntl::*;

#[allow(deprecated)]
extern "C" {
    fn htonl(hostlong: libc::uint32_t) -> libc::uint32_t;
}

fn new_raw_socket() -> io::Result<libc::c_int> {
    let socket = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DCCP, libc::IPPROTO_DCCP) };
    if socket < 0 {
        return Err(io::Error::last_os_error());
    }

    match setsockopt(socket, sockopt::ReuseAddr, &true) {
        Ok(_) => (),
        Err(nix::Error::Sys(_)) => return Err(io::Error::last_os_error()),
        Err(e) => panic!("{}", e),
    };

    match setsockopt(socket, sockopt::ReusePort, &true) {
        Ok(_) => Ok(socket),
        Err(nix::Error::Sys(_)) => Err(io::Error::last_os_error()),
        Err(e) => panic!("{}", e),
    }
}

#[allow(deprecated)]
fn setservicecode(socket: libc::c_int, code: i32) -> io::Result<()> {
    let mut opt = code as libc::c_int;
    opt = unsafe { htonl(opt as libc::uint32_t) as libc::c_int };

    let ret = unsafe {
        libc::setsockopt(
            socket,
            libc::SOL_DCCP,
            libc::DCCP_SOCKOPT_SERVICE,
            &opt as *const _ as *const libc::c_void,
            mem::size_of_val(&opt) as libc::socklen_t,
        )
    };

    if ret < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

#[derive(Debug)]
pub struct DCCPSocket {
    inner: libc::c_int,
}

impl DCCPSocket {
    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        match send(self.as_raw_fd(), buf, MsgFlags::empty()) {
            Ok(n) => Ok(n),
            Err(nix::Error::Sys(_)) => return Err(io::Error::last_os_error()),
            Err(e) => panic!("{}", e),
        }
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        match recv(self.as_raw_fd(), buf, MsgFlags::empty()) {
            Ok(n) => Ok(n),
            Err(nix::Error::Sys(_)) => return Err(io::Error::last_os_error()),
            Err(e) => panic!("{}", e),
        }
    }

    pub fn set_nonblocking(&self) {
        if self.as_raw_fd() < 0 {
            return;
        }
        fcntl(self.as_raw_fd(), FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).expect("Could not set socket to non blocking mode.");
    }

    pub fn connect<A: ToSocketAddrs>(addr: A, servicecode: i32) -> io::Result<Self> {
        let socket = new_raw_socket()?;
        let mut connected = false;

        setservicecode(socket, servicecode)?;

        for addr in addr.to_socket_addrs()? {
            let nix_addr = SockAddr::new_inet(InetAddr::from_std(&addr));
            match connect(socket, &nix_addr) {
                Ok(_) => { connected = true; break; },
                Err(nix::Error::Sys(_)) => continue,
                Err(e) => panic!("{}", e),
            };
        }

        if connected {
            Ok(Self { inner: socket })
        } else {
            Err(io::Error::last_os_error())
        }
    }

}

impl Drop for DCCPSocket {
    fn drop(&mut self) {
        if let Err(e) = close(self.as_raw_fd()) {
            warn!("{}", e);
        }
    }
}

impl Read for DCCPSocket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.recv(buf)
    }
}

impl Write for DCCPSocket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.send(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl AsRawFd for DCCPSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.inner
    }
}

pub struct DCCPListener {
    inner: libc::c_int,
}

impl DCCPListener {
    pub fn bind<A: ToSocketAddrs>(addr: A, servicecode: i32) -> io::Result<Self> {
        let socket = new_raw_socket()?;
        let mut bound = false;

        for addr in addr.to_socket_addrs()? {
            let nix_addr = SockAddr::new_inet(InetAddr::from_std(&addr));
            match bind(socket, &nix_addr) {
                Ok(_) => bound = true,
                Err(nix::Error::Sys(e)) => { warn!("{}", e); continue },
                Err(e) => panic!("{}", e),
            };
        }

        if !bound {
           return  Err(io::Error::last_os_error());
        }

        setservicecode(socket, servicecode)?;

        match listen(socket, 128) {
            Ok(_) => Ok(Self { inner: socket }),
            Err(nix::Error::Sys(_)) => return Err(io::Error::last_os_error()),
            Err(e) => panic!("{}", e),
        }
    }

    pub fn accept(&self) -> io::Result<(DCCPSocket, SockAddr)> {
        match accept(self.as_raw_fd()) {
            Ok(c) => {
                match getpeername(c) {
                    Ok(peer) => Ok((DCCPSocket{inner:c}, peer)),
                    Err(nix::Error::Sys(_)) => {
                        return Err(io::Error::last_os_error());
                    },
                    Err(e) => panic!("{}", e),
                }
            },
            Err(nix::Error::Sys(_)) => {
                return Err(io::Error::last_os_error());
            },
            Err(e) => panic!("{}", e),
        }
    }
}

impl AsRawFd for DCCPListener {
    fn as_raw_fd(&self) -> RawFd {
        self.inner
    }
}

impl Drop for DCCPListener {
    fn drop(&mut self) {
        if let Err(e) = close(self.as_raw_fd()) {
            warn!("{}", e);
        }
    }
}
*/
