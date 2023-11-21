use crate::socket::{Interface as MulticastInterface, MulticastOptions, MulticastSocket};
use crate::{Config, DnsPacket};
use dns_parser::Packet;
use log::{info, trace, warn};
use nix::errno::Errno;
use std::net::Ipv4Addr;
use std::{ffi::CString, net::SocketAddrV4};

pub struct Mdns {
    socket: MulticastSocket,
    config: Config,
}

impl Mdns {
    pub fn new(config: Config) -> Self {
        // mdns
        let multicast_socket = MulticastSocket::new(
            MulticastOptions::default(),
            MulticastSocket::all_interfaces().unwrap(),
            SocketAddrV4::new(Ipv4Addr::new(224, 0, 0, 251), 5353),
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
                Err(e)
                    if e.get_ref().map_or(false, |e| {
                        e.downcast_ref::<nix::Error>()
                            .is_some_and(|c| *c == Errno::EAGAIN)
                    }) =>
                {
                    continue;
                }
                Err(e) => {
                    warn!("error in reading from socket {:?} ", e);
                }
            };
        }
    }

    pub fn process_packet(&self, msg: crate::socket::Message) {
        // TODO: Generalize this to parse any type of supported packet
        let packet = DnsPacket::parse(&msg.data).unwrap_or_else(|e| {
            trace!("{:0x?}", msg.data);

            panic!(
                "failed to parse packet as a dns packet. origin = {:?} interface = {:?} error = {:?}, loose_string = {:02x?}",
                msg.origin_address,msg.interface, e, msg.data,
            )
        });

        let src_ifname = if let MulticastInterface::Index(idx) = msg.interface {
            ifidx_to_ifname(idx as u32)
        } else {
            "lo".to_string()
        };

        trace!(
            "EVENT src-if = {} if-index {:?} address = {:?}, packet: {:?} answers = {:?}",
            src_ifname,
            msg.interface,
            msg.origin_address,
            packet.questions.iter().collect::<Vec<_>>(),
            packet.answers.iter().collect::<Vec<_>>()
        );

        let interfaces = get_if_addrs::get_if_addrs().unwrap();
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
                    "forwarding packet packet {:?} from {} to {}",
                    packet, src_ifname, dst_if.name
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
