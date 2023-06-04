use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

use crate::{
    listener::{join_multicast, new_socket},
    Config,
};
use log::{debug, info, warn};
use once_cell::sync::Lazy;
pub static IPV4: Lazy<IpAddr> = Lazy::new(|| Ipv4Addr::new(0, 0, 0, 0).into());
pub static IPV6: Lazy<IpAddr> = Lazy::new(|| {
    Ipv6Addr::new(0x2a0a, 0x6040, 0x4004, 0x10, 0xf1c6, 0xf2b0, 0x9f45, 0xb425).into()
});

pub struct Communications {
    // TODO(ishan): Accommodate IPv6 sockets as well here
    sockets: HashMap<u16, UdpSocket>,
    handles: Vec<JoinHandle<()>>,

    tx_chan: Sender<Event>,
    rx_chan: Receiver<Event>,
}

pub struct Event {
    pub remote_addr: SocketAddr,
    pub payload: Vec<u8>,
}

impl Communications {
    pub fn new(input: Config) -> Result<Self, String> {
        let mut sockets = HashMap::new();

        // This channel is used to send packets from this module to the processor
        // TODO(ishan): Eventually, swap out std::mpsc for some thing faster
        // or switch to a different model completely that doesn't use channels like this
        let (rx, tx) = mpsc::channel();

        for (_, v) in input.config.iter() {
            let socket =
                new_socket(&SocketAddr::new(*IPV4, v.port)).expect("error in bind on ipv4 address");

            info!("started listening on {}:{}", *IPV4, v.port);

            for group in v.multicast_groups.iter() {
                join_multicast(&socket, group)
                    .map_err(|e| format!("error in joining multicast group {}: {}", group, e))?;

                info!("joined multicast group {}", group);
            }

            sockets.insert(v.port, socket);
        }

        Ok(Communications {
            sockets,
            handles: vec![],
            rx_chan: tx,
            tx_chan: rx,
        })
    }

    pub fn start_listeners(&mut self) -> Result<(), String> {
        let sockets = self
            .clone_sockets()
            .map_err(|e| format!("error in cloning sockets: {}", e))?;

        for (port, v) in sockets {
            let socket = v
                .try_clone()
                .map_err(|e| format!("error in cloning socket for read ops: {}", e))?;
            let tx_chan = self.tx_chan.clone();

            let handle = thread::spawn(move || {
                // TODO(ishan): What should be the size here ?

                loop {
                    let mut buf = [0u8; 2000];

                    match socket.recv_from(&mut buf) {
                        Ok((len, remote_addr)) => {
                            let buf = &buf[..len];

                            tx_chan
                                .send(Event {
                                    remote_addr,
                                    payload: buf.to_vec(),
                                })
                                .expect("error in sending to mpsc channel");

                            debug!("read {}bytes from {}", len, remote_addr);
                        }
                        Err(e) => {
                            warn!("error in reading from socket {}: {}", port, e);
                            continue;
                        }
                    };
                }
            });

            self.handles.push(handle);
        }

        Ok(())
    }

    pub fn wait(&mut self) {
        while let Some(handle) = self.handles.pop() {
            handle.join().unwrap();
        }
    }

    fn clone_sockets(&self) -> Result<HashMap<u16, UdpSocket>, std::io::Error> {
        let mut sockets = HashMap::new();

        for (k, v) in self.sockets.iter() {
            sockets.insert(*k, v.try_clone()?);
        }

        Ok(sockets)
    }

    pub fn get_reader(&self) -> &Receiver<Event> {
        &self.rx_chan
    }

    // TODO(ishan): Add a function to send messages from a port
}
