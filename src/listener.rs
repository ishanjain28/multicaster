use std::{
    io::{Error, Result as IoResult},
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    os::fd::AsRawFd,
};

use nix::sys::socket::sockopt::ReuseAddr;

pub fn new_socket(addr: &SocketAddr) -> Result<UdpSocket, Error> {
    let socket = UdpSocket::bind(addr)?;

    #[cfg(unix)]
    nix::sys::socket::setsockopt(socket.as_raw_fd(), ReuseAddr, &true)?;

    //    socket.set_read_timeout(Some(Duration::from_millis(100)))?;

    Ok(socket)
}

pub fn join_multicast(socket: &mut UdpSocket, group: &IpAddr) -> IoResult<()> {
    // TODO(ishan): Eventually, this should only listen on the
    // interfaces specified in config.toml
    match group {
        IpAddr::V4(ref mdns_v4) => {
            socket.multicast_loop_v4()?;
            socket.join_multicast_v4(mdns_v4, &Ipv4Addr::new(0, 0, 0, 0))
        }
        IpAddr::V6(ref mdns_v6) => {
            socket.multicast_loop_v6()?;

            socket.join_multicast_v6(mdns_v6, 0)
        }
    }
}
