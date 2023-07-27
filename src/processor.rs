use crate::{Config, Event};
use dns_parser::Packet;
use log::{info, warn};
use multicast_socket::{Interface, MulticastOptions, MulticastSocket};
use std::{ffi::CString, net::SocketAddrV4, sync::mpsc::Receiver};

pub struct Processor<'a> {
    reader: &'a Receiver<Event>,
    config: Config,
}

impl<'a> Processor<'a> {
    pub fn new(reader: &'a Receiver<Event>, config: Config) -> Result<Self, String> {
        Ok(Self { reader, config })
    }

    pub fn start_read_loop(&self) {
        // mdns
        let mdns_address = SocketAddrV4::new([224, 0, 0, 251].into(), 5353);
        let socket = MulticastSocket::with_options(
            mdns_address,
            multicast_socket::all_ipv4_interfaces().expect("could not fetch all interfaces"),
            MulticastOptions {
                loopback: false,
                buffer_size: 4096,
                ..Default::default()
            },
        )
        .expect("error in creating multicast socket");

        for evt in self.reader {
            // TODO: Generalize this to parse any type of supported packet
            let packet =
                Packet::parse(&evt.msg.data).expect("failed to parse packet as a dns packet");

            let interfaces = get_if_addrs::get_if_addrs().unwrap();

            let src_ifname = if let Interface::Index(idx) = evt.msg.interface {
                ifidx_to_ifname(idx as u32)
            } else {
                "lo".to_string()
            };

            info!(
                "EVENT src-if = {} if-index {:?} address = {}, packet: {:?} answers = {:?}",
                src_ifname,
                evt.msg.interface,
                evt.msg.origin_address,
                packet.questions.iter().map(|q| q.qname).collect::<Vec<_>>(),
                packet.answers.iter().map(|q| q.name).collect::<Vec<_>>()
            );
            for conf in &self.config.mdns {
                for query in &packet.questions {
                    let forward = conf.destinations.contains(&src_ifname)
                        && (conf.filters.is_empty()
                            || conf.filters.contains(&query.qname.to_string()));

                    if forward {
                        let dst_ifs = conf
                            .sources
                            .clone()
                            .into_iter()
                            .filter_map(|dst_if| interfaces.iter().find(|x| x.name == dst_if));

                        for dst_if in dst_ifs {
                            let dst_ifid = ifname_to_ifidx(dst_if.name.to_string());

                            info!(
                                "forwarding {:?} from {:?} to {:?}({})",
                                packet, src_ifname, dst_if, dst_ifid
                            );
                            // TODO(ishan): Take a note of transaction id
                            // and avoid feedback loops
                            socket
                                .send(&evt.msg.data, &Interface::Index(dst_ifid as i32))
                                .expect("error in sending mdns packet");
                        }
                    }
                }

                for answer in &packet.answers {
                    let forward = conf.sources.contains(&src_ifname)
                        && (conf.filters.is_empty()
                            || conf.filters.contains(&answer.name.to_string()));

                    if forward {
                        let dst_ifs = conf
                            .destinations
                            .clone()
                            .into_iter()
                            .filter_map(|dst_if| interfaces.iter().find(|x| x.name == dst_if));

                        // TODO(ishan): Stop blasting this every where.
                        // Try to limit answers traffic between destinations
                        // it should not be broadcasting an answer to all networks just because 1
                        // asked for it
                        for dst_if in dst_ifs {
                            let dst_ifid = ifname_to_ifidx(dst_if.name.to_string());

                            info!(
                                "forwarding {:?} from {:?} to {:?}({})",
                                packet, src_ifname, dst_if, dst_ifid
                            );

                            socket
                                .send(&evt.msg.data, &Interface::Index(dst_ifid as i32))
                                .expect("error in sending mdns packet");
                        }
                    }
                }
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
