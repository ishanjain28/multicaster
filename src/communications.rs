use crate::Config;
use get_if_addrs::IfAddr;
use log::{debug, info, trace, warn};
use net2::UdpBuilder;
use std::{
    collections::HashMap,
    io::{Error as IoError, Result as IoResult},
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

pub struct Communications {
    // TODO(ishan): Accommodate IPv6 sockets as well here
    sockets: HashMap<(String, IpAddr, u16), CommSocket>,
    handles: Vec<JoinHandle<()>>,

    tx_chan: Sender<Event>,
    rx_chan: Receiver<Event>,
}

#[derive(Debug)]
pub struct Event {
    pub src_if: String,
    pub local_addr: IpAddr,
    pub remote_addr: SocketAddr,
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub struct CommSocket(UdpSocket);

impl CommSocket {
    pub fn new(addr: IpAddr, port: u16) -> Result<CommSocket, IoError> {
        // TODO(ishan): Remove expect
        let addr = SocketAddr::new(addr, port);

        let socket = match addr {
            SocketAddr::V4(_) => UdpBuilder::new_v4()?.reuse_address(true)?.bind(addr)?,
            SocketAddr::V6(_) => UdpBuilder::new_v6()?.reuse_address(true)?.bind(addr)?,
        };

        socket.set_read_timeout(Some(Duration::from_millis(100)))?;
        socket.set_nonblocking(false)?;

        Ok(CommSocket(socket))
    }

    pub fn join_multicast_group(&self, ip_addr: &IpAddr, group: &IpAddr) -> IoResult<()> {
        match (group, ip_addr) {
            (IpAddr::V4(ref group), IpAddr::V4(ref v4)) => {
                self.0.set_multicast_loop_v4(false)?;
                // TODO(ishan): This should be an input from config.toml
                self.0.join_multicast_v4(group, v4)
            }
            (IpAddr::V6(ref group), IpAddr::V6(_)) => {
                self.0.set_multicast_loop_v6(false)?;
                self.0.join_multicast_v6(group, 0)
            }

            (_, _) => unreachable!(),
        }
    }

    // TODO(ishan): Add a method to send messages
    pub fn send(&self, buf: &[u8], addr: SocketAddr) -> Result<usize, IoError> {
        self.0.set_ttl(1)?;
        self.0.send_to(buf, addr)
    }
}

impl Communications {
    pub fn new(config: Config) -> Result<Self, String> {
        let mut sockets = HashMap::new();

        // This channel is used to send packets from this module to the processor
        // TODO(ishan): Eventually, swap out std::mpsc for some thing faster
        // or switch to a different model completely that doesn't use channels like this
        let (rx, tx) = mpsc::channel();

        let interfaces = get_if_addrs::get_if_addrs().unwrap();
        trace!("Interfaces list: {:?}", interfaces);

        for v in &config.mdns {
            // Find addresses on the specified sources interfaces
            // Listen on all addresses with SO_REUSE_ADDR and use interface's IP address
            // when joining multicast group
            // Only working with IPv4 addresses for now

            for src_if_name in &v.sources {
                let src_if = interfaces.iter().find(|x| &x.name == src_if_name).unwrap();

                let listen_addr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

                let ip_addr = match src_if.addr {
                    IfAddr::V4(ref v4) => IpAddr::V4(v4.ip),
                    IfAddr::V6(ref v6) => IpAddr::V6(v6.ip),
                };

                let socket =
                    CommSocket::new(listen_addr, v.port).expect("error in bind on address");

                info!("started listening on {}:{}", listen_addr, v.port);

                for group in v.multicast_groups.iter() {
                    socket.join_multicast_group(&ip_addr, group).map_err(|e| {
                        format!("error in joining multicast group {}: {}", group, e)
                    })?;

                    info!(
                        "joined multicast group {} with address = {}",
                        group, ip_addr
                    );
                }
                sockets.insert((src_if_name.clone(), ip_addr, v.port), socket);
            }
        }

        info!("{:?}", sockets);
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
        info!("listener started");

        for ((src_if, local_addr, _), v) in sockets {
            let socket = v
                .try_clone()
                .map_err(|e| format!("error in cloning socket for read ops: {}", e))?;

            let tx_chan = self.tx_chan.clone();

            let handle = thread::spawn(move || {
                // TODO(ishan): What should be the size here ?

                loop {
                    let mut buf = [0u8; 9000];

                    match socket.recv_from(&mut buf) {
                        Ok((len, remote_addr)) => {
                            let buf = &buf[..len];

                            tx_chan
                                .send(Event {
                                    src_if: src_if.clone(),
                                    remote_addr,
                                    local_addr,
                                    payload: buf.to_vec(),
                                })
                                .expect("error in sending to mpsc channel");

                            debug!("read {}bytes from {}", len, remote_addr);
                        }
                        Err(_) => {
                            // TODO: Log all buy EAGAIN
                            //     warn!("error in reading from socket {}: {}", port, e);
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

    fn clone_sockets(&self) -> Result<HashMap<(String, IpAddr, u16), UdpSocket>, std::io::Error> {
        let mut sockets = HashMap::new();

        for (k, v) in self.sockets.iter() {
            // TODO(ishan): find a better solution here
            sockets.insert(k.clone(), v.0.try_clone()?);
        }

        Ok(sockets)
    }

    pub fn get_reader(&self) -> &Receiver<Event> {
        &self.rx_chan
    }

    // TODO(ishan): Add a function to send messages from a port
}
