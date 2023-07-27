use crate::Config;
use log::{info, trace};
use multicast_socket::{Message, MulticastOptions, MulticastSocket};
use std::{
    net::SocketAddrV4,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

pub struct Communications {
    // TODO(ishan): Accommodate IPv6 sockets as well here
    handles: Vec<JoinHandle<()>>,
    tx_chan: Sender<Event>,
    rx_chan: Receiver<Event>,
}

#[derive(Debug)]
pub struct Event {
    pub msg: Message,
}

impl Communications {
    pub fn new(config: Config) -> Result<Self, String> {
        // This channel is used to send packets from this module to the processor
        // TODO(ishan): Eventually, swap out std::mpsc for some thing faster
        // or switch to a different model completely that doesn't use channels like this
        let (rx, tx) = mpsc::channel();

        Ok(Communications {
            handles: vec![],
            rx_chan: tx,
            tx_chan: rx,
        })
    }

    pub fn start_listeners(&mut self) -> Result<(), String> {
        info!("listener started");

        let tx_chan = self.tx_chan.clone();

        let handle = thread::spawn(move || {
            // TODO(ishan): What should be the size here ?

            let interfaces = get_if_addrs::get_if_addrs().unwrap();
            trace!("Interfaces list: {:?}", interfaces);

            // mdns
            let mdns_address = SocketAddrV4::new([224, 0, 0, 251].into(), 5353);
            let multicast_socket = MulticastSocket::with_options(
                mdns_address,
                multicast_socket::all_ipv4_interfaces().expect("could not fetch all interfaces"),
                MulticastOptions {
                    loopback: false,
                    buffer_size: 4096,
                    ..Default::default()
                },
            )
            .expect("error in creating multicast socket");

            loop {
                match multicast_socket.receive() {
                    Ok(msg) => {
                        tx_chan
                            .send(Event { msg })
                            .expect("error in sending to mpsc channel");
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

        Ok(())
    }

    pub fn wait(&mut self) {
        while let Some(handle) = self.handles.pop() {
            handle.join().unwrap();
        }
    }

    pub fn get_reader(&self) -> &Receiver<Event> {
        &self.rx_chan
    }

    // TODO(ishan): Add a function to send messages from a port
}
