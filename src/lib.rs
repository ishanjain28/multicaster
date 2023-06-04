// TODO(ishan): Eventually we'll have a listener and transmitter module for every thing we want to
// support. So, 1 for MDNS, another for WSDD?
// Or a common listener/transmitter and then different modules to parse and transmit each type of
// traffic
pub mod config;
pub use config::*;
pub mod listener;

use crate::listener::{join_multicast, new_socket};
use once_cell::sync::Lazy;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    thread,
};

pub static IPV4: Lazy<IpAddr> = Lazy::new(|| Ipv4Addr::new(0, 0, 0, 0).into());
pub static IPV6: Lazy<IpAddr> = Lazy::new(|| {
    Ipv6Addr::new(0x2a0a, 0x6040, 0x4004, 0x10, 0xf1c6, 0xf2b0, 0x9f45, 0xb425).into()
});

pub fn start(config: crate::Config) {
    // TODO(ishan): Start listeners and transmitters on v4 and v6 here

    let mut handles = vec![];

    for (_, v) in config.config {
        let handle = thread::spawn(move || {
            let mut ipv4_socket = new_socket(&SocketAddr::new(*IPV4, v.port))
                .expect("error in bind op on ipv4 address");

            let mut ipv6_socket = new_socket(&SocketAddr::new(*IPV6, v.port))
                .expect("error in bind op on ipv6 address");

            for group in v.multicast_groups {
                join_multicast(&mut ipv4_socket, &group)
                    .expect("error in joining ipv4 multicast group");
                join_multicast(&mut ipv6_socket, &group)
                    .expect("error in joining ipv6 multicast group");
                println!(
                    "Listening for multicast packets on ipv4/ipv6 in group {}:{}",
                    group, v.port
                );
            }
            let mut buf = [0u8; 64];
            match ipv4_socket.recv_from(&mut buf) {
                Ok((len, remote_addr)) => {
                    let buf = &buf[..len];
                    println!("received {}bytes response from {}", len, remote_addr);

                    let v = String::from_utf8_lossy(buf);

                    println!("output = {:?}", v);
                }
                Err(e) => {
                    println!("got an error: {}", e);
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap()
    }
}
