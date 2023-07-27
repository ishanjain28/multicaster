use crate::{communications::CommSocket, Config, Event};
use dns_parser::Packet;
use get_if_addrs::IfAddr;
use log::{info, trace, warn};
use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::mpsc::Receiver,
};

pub struct Processor<'a> {
    reader: &'a Receiver<Event>,
    config: Config,
    sockets: HashMap<String, CommSocket>,
}

impl<'a> Processor<'a> {
    pub fn new(reader: &'a Receiver<Event>, config: Config) -> Result<Self, String> {
        let mut sockets = HashMap::new();

        let interfaces = get_if_addrs::get_if_addrs().unwrap();
        trace!("Interfaces list: {:?}", interfaces);

        for conf in &config.mdns {
            let listen_addr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

            for if_name in conf
                .destinations
                .clone()
                .into_iter()
                .chain(conf.sources.clone())
            {
                if sockets.contains_key(&if_name) {
                    continue;
                }

                let dst_if = match interfaces.iter().find(|x| x.name == if_name) {
                    Some(v) => v,
                    None => {
                        warn!("could not find interface {}", if_name);
                        continue;
                    }
                };

                let dst_ip_addr = match dst_if.addr {
                    IfAddr::V4(ref v4) => IpAddr::V4(v4.ip),
                    IfAddr::V6(ref v6) => IpAddr::V6(v6.ip),
                };

                let socket =
                    CommSocket::new(listen_addr, conf.port).expect("error in bind on address");

                info!("started listening on {}:{}", listen_addr, conf.port);

                for group in &conf.multicast_groups {
                    socket
                        .join_multicast_group(&dst_ip_addr, group)
                        .map_err(|e| {
                            format!("error in joining multicast group {}: {}", group, e)
                        })?;
                }

                sockets.insert(if_name, socket);
            }
        }

        Ok(Self {
            reader,
            config,
            sockets,
        })
    }

    pub fn start_read_loop(&self) {
        for evt in self.reader {
            // TODO: Generalize this to parse any type of supported packet
            let packet =
                Packet::parse(&evt.payload).expect("failed to parse packet as a dns packet");

            //   info!("EVENT {:?}", evt);
            //info!("{:?}", packet);

            for query in packet.questions {
                info!("src_if = {} query = {}", evt.src_if, query.qname);

                // Find a config that has src-if and filter
                let conf = self.config.mdns.iter().filter(|config| {
                    (config.destinations.contains(&evt.src_if)
                        && config.filters.contains(&query.qname.to_string()))
                        || (config.destinations.contains(&evt.src_if) && config.filters.is_empty())
                });

                let mut sources: HashSet<String> = conf
                    .clone()
                    .flat_map(|config| config.sources.clone())
                    .collect();
                sources.remove(&evt.src_if);

                let multicast_groups: HashSet<IpAddr> = conf
                    .flat_map(|config| config.multicast_groups.clone())
                    .collect();

                if !sources.is_empty() {
                    info!("conf = {:?}", sources);
                }
                for src in sources {
                    // Send this MDNS query to all matching sources
                    // When they respond, We'll forward the message to matching destinations

                    if let Some(socket) = self.sockets.get(&src) {
                        for group in multicast_groups.clone() {
                            socket
                                .send(&evt.payload, SocketAddr::new(group, 1900))
                                .expect("error in sending message");
                        }
                    }
                }
            }
        }
    }
}
