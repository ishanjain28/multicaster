use crate::Config;
use dns_parser::Packet;
use log::{info, trace, warn};
use multicast_socket::{Interface as MulticastInterface, MulticastOptions, MulticastSocket};
use std::{ffi::CString, net::SocketAddrV4};

pub struct Mdns {
    socket: MulticastSocket,
    config: Config,
}

impl Mdns {
    pub fn new(config: Config) -> Self {
        // mdns
        let mdns_address = SocketAddrV4::new([224, 0, 0, 251].into(), 5353);
        let multicast_socket = MulticastSocket::with_options(
            mdns_address,
            // TODO(ishan): Listen on ALL Interfaces, including ipv6
            multicast_socket::all_ipv4_interfaces().expect("could not fetch all interfaces"),
            MulticastOptions {
                loopback: false,
                buffer_size: 4096,
                ..Default::default()
            },
        )
        .expect("error in creating multicast socket");

        Self {
            socket: multicast_socket,
            config,
        }
    }

    pub fn listener_loop(&self) {
        info!("listener started");

        loop {
            match self.socket.receive() {
                Ok(msg) => self.process_packet(msg),
                Err(e) if e.to_string().contains("EAGAIN") => continue,
                Err(e) => {
                    warn!("error in reading from socket {:?} ", e);
                }
            };
        }
    }

    pub fn process_packet(&self, msg: multicast_socket::Message) {
        // TODO: Generalize this to parse any type of supported packet
        let packet = Packet::parse(&msg.data).expect("failed to parse packet as a dns packet");

        let src_ifname = if let MulticastInterface::Index(idx) = msg.interface {
            ifidx_to_ifname(idx as u32)
        } else {
            "lo".to_string()
        };

        trace!(
            "EVENT src-if = {} if-index {:?} address = {}, packet: {:?} answers = {:?}",
            src_ifname,
            msg.interface,
            msg.origin_address,
            packet.questions.iter().map(|q| q.qname).collect::<Vec<_>>(),
            packet.answers.iter().map(|q| q.name).collect::<Vec<_>>()
        );

        let interfaces = get_if_addrs::get_if_addrs().unwrap();
        trace!("Interfaces: {:?}", interfaces);
        for conf in &self.config.mdns {
            let mut dst_ifs = vec![];

            for query in &packet.questions {
                if conf.destinations.contains(&src_ifname)
                    && (conf.filters.is_empty() || conf.filters.contains(&query.qname.to_string()))
                {
                    dst_ifs.extend(
                        conf.sources
                            .iter()
                            .filter_map(|dst_if| interfaces.iter().find(|x| &x.name == dst_if)),
                    );
                }
            }

            for answer in &packet.answers {
                if conf.sources.contains(&src_ifname)
                    && (conf.filters.is_empty() || conf.filters.contains(&answer.name.to_string()))
                {
                    dst_ifs.extend(
                        conf.destinations
                            .iter()
                            .filter_map(|dst_if| interfaces.iter().find(|x| &x.name == dst_if)),
                    );
                }
            }

            for dst_if in dst_ifs {
                let dst_ifid = ifname_to_ifidx(dst_if.name.to_string());

                info!(
                    "forwarding packet questions {:?} answers = {:?} from {} to {}",
                    packet.questions, packet.answers, src_ifname, dst_if.name
                );
                // TODO(ishan): Take a note of transaction id
                // and avoid feedback loops

                self.socket
                    .send(&msg.data, &MulticastInterface::Index(dst_ifid as i32))
                    .expect("error in sending mdns packet");
            }
        }
    }
}

fn ifidx_to_ifname(idx: u32) -> String {
    let out = CString::new("askdjhaskdjakdjadksa").unwrap();

    unsafe {
        let ptr = out.into_raw();
        let response = libc::if_indextoname(idx, ptr);

        CString::from_raw(response).into_string().unwrap()
    }
}

fn ifname_to_ifidx(name: String) -> u32 {
    let out = name.as_ptr() as *const _;
    unsafe { libc::if_nametoindex(out) }
}
