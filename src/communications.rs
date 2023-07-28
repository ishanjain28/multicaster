use log::{info, trace, warn};
use multicast_socket::{Message, MulticastOptions, MulticastSocket};
use std::{net::SocketAddrV4, sync::mpsc::Sender};

pub struct Communications {
    tx_chan: Sender<Message>,
}

impl Communications {
    pub fn new(tx_chan: Sender<Message>) -> Result<Self, String> {
        Ok(Communications { tx_chan })
    }

    pub async fn start_listeners(&mut self) -> Result<(), String> {
        info!("listener started");

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
                    self.tx_chan
                        .send(msg)
                        .expect("error in sending to mpsc channel");
                }
                Err(e) if e.to_string().contains("EAGAIN") => continue,
                Err(e) => {
                    warn!("error in reading from socket {:?} ", e);
                }
            };
        }
    }
}
