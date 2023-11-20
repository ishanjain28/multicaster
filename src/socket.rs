#![allow(unused)]

use log::trace;
// This code has been adapted from multicast_socket crate
use nix::sys::socket::{self as sock, AddressFamily, SockaddrIn, SockaddrLike, SockaddrStorage};
use serde::de::value;
use socket2::{Domain, Protocol, Socket, Type};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    io::{self, IoSlice, IoSliceMut, Result as IoResult},
    mem,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    os::unix::io::AsRawFd,
    time::Duration,
};

pub struct MulticastOptions {
    pub read_timeout: Duration,
    pub buffer_size: usize,
}

impl Default for MulticastOptions {
    fn default() -> Self {
        MulticastOptions {
            read_timeout: Duration::from_secs(1),
            buffer_size: 512,
        }
    }
}

#[derive(Debug)]
pub struct MulticastSocket {
    socket: socket2::Socket,
    interfaces: HashMap<String, Vec<IpAddr>>,
    multicast_group: MulticastGroup,
    buffer_size: usize,
}

#[derive(Debug, Clone)]
pub struct MulticastGroup {
    pub ipv4: SocketAddrV4,
    pub port: u16,
}

impl MulticastSocket {
    pub fn new(
        options: MulticastOptions,
        interfaces: HashMap<String, Vec<IpAddr>>,
        multicast_group: MulticastGroup,
    ) -> Result<Self, std::io::Error> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_read_timeout(Some(options.read_timeout))?;
        socket.set_multicast_loop_v4(false)?;
        socket.set_reuse_address(true)?;
        socket.set_reuse_port(true)?;

        // Ipv4PacketInfo translates to `IP_PKTINFO`. Checkout the [ip
        // manpage](https://man7.org/linux/man-pages/man7/ip.7.html) for more details. In summary
        // setting this option allows for determining on which interface a packet was received.
        sock::setsockopt(socket.as_raw_fd(), sock::sockopt::Ipv4PacketInfo, &true)
            .map_err(nix_to_io_error)?;

        for (if_name, addresses) in interfaces.iter() {
            trace!(
                "joining groups = {:?} interface = {:?}",
                multicast_group.ipv4.ip(),
                Ipv4Addr::UNSPECIFIED
            );

            socket.join_multicast_v4(multicast_group.ipv4.ip(), &Ipv4Addr::UNSPECIFIED);

            for address in addresses {
                if let IpAddr::V4(v4_addr) = address {
                    if v4_addr.is_loopback() {
                        continue;
                    }

                    socket.join_multicast_v4(multicast_group.ipv4.ip(), v4_addr)?;

                    trace!(
                        "joined ipv4 multicast group {} {}",
                        multicast_group.ipv4.ip(),
                        v4_addr
                    );
                }
            }
        }

        socket.bind(
            &SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), multicast_group.port).into(),
        )?;

        Ok(MulticastSocket {
            socket,
            interfaces,
            buffer_size: options.buffer_size,
            multicast_group,
        })
    }

    pub fn all_interfaces() -> IoResult<HashMap<String, Vec<IpAddr>>> {
        let interfaces = get_if_addrs::get_if_addrs()?.into_iter();
        // We have to filter the same interface if it has multiple ips
        // https://stackoverflow.com/questions/49819010/ip-add-membership-fails-when-set-both-on-interface-and-its-subinterface-is-that
        let mut map = HashMap::new();

        for interface in interfaces {
            map.entry(interface.name.clone())
                .or_insert_with(Vec::new)
                .push(interface.ip());
        }

        // TODO: remove loopback?
        map.remove("lo");

        Ok(map)
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub data: Vec<u8>,
    pub origin_address: Option<SocketAddr>,
    pub interface: Interface,
}

#[derive(Debug, Clone)]
pub enum Interface {
    Default,
    Index(i32),
    IpAddr(Ipv6Addr),
}

#[inline]
fn ifname_to_ifidx(name: String) -> u32 {
    let out = name.as_ptr() as *const _;
    unsafe { libc::if_nametoindex(out) }
}

fn nix_to_io_error(e: nix::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}

impl MulticastSocket {
    pub fn receive(&self) -> IoResult<Message> {
        let mut data_buffer = vec![0; self.buffer_size];
        let mut control_buffer = nix::cmsg_space!(libc::in6_pktinfo, libc::in_pktinfo);

        let (origin_address, interface, bytes_read) = {
            let message = sock::recvmsg(
                self.socket.as_raw_fd(),
                &mut [IoSliceMut::new(&mut data_buffer)],
                Some(&mut control_buffer),
                sock::MsgFlags::empty(),
            )
            .map_err(nix_to_io_error)?;

            let origin_address = match message.address {
                //v4 @ Some(SockaddrIn) => v4,
                Some(sock::SockAddr::Inet(inet)) => Some(inet.to_std()),
                _ => None,
            };
            //            let origin_address = match origin_address {
            //                Some(SocketAddr::V6(v6)) => v6,
            //                _ => SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0),
            //            };

            println!("{:?}", origin_address);
            let mut interface = Interface::Default;

            for cmsg in message.cmsgs() {
                if let sock::ControlMessageOwned::Ipv6PacketInfo(pktinfo) = cmsg {
                    interface = Interface::Index(pktinfo.ipi6_ifindex as _);
                    trace!("control packet ipv6: {:?}", pktinfo);
                }
                if let sock::ControlMessageOwned::Ipv4PacketInfo(pktinfo) = cmsg {
                    interface = Interface::Index(pktinfo.ipi_ifindex as _);

                    trace!("control packet ipv4: {:?}", pktinfo);
                }
            }

            (origin_address, interface, message.bytes)
        };

        Ok(Message {
            data: data_buffer[0..bytes_read].to_vec(),
            origin_address,
            interface,
        })
    }

    pub fn send(&self, buf: &[u8], interface: &Interface) -> io::Result<usize> {
        Ok(0)
        //    match interface {
        //        Interface::Default => todo!(),
        //        Interface::Index(index) => {
        //            // TODO: Send over ipv4 and ipv6
        //            pkt_info.ipi_ifindex = *index as _;
        //        }
        //        Interface::Ip(IpAddr::V4(v4)) => {
        //            let mut pkt_info: libc::in_pktinfo = unsafe { mem::zeroed() };
        //            pkt_info.ipi_spec_dst = libc::in_addr {
        //                s_addr: (*v4).into(),
        //            };
        //        }
        //        Interface::Ip(IpAddr::V6(v6)) => {
        //            let mut pkt_info: libc::in6_pktinfo = unsafe { mem::zeroed() };
        //        }
        //    }

        //    match interface {
        //        Interface::Default => {}
        //        Interface::Ipv6(address) => {}
        //        Interface::Ipv4(address) => {
        //            pkt_info.ipi_spec_dst = libc::in_addr {
        //                s_addr: (*address).into(),
        //            }
        //        }

        //        Interface::Index(index) => pkt_info.ipi_ifindex = *index as _,
        //    };

        //    let destination = SockaddrIn::from(self.multicast_group);

        //    sock::sendmsg(
        //        self.socket.as_raw_fd(),
        //        &[IoSlice::new(buf)],
        //        &[sock::ControlMessage::Ipv4PacketInfo(&pkt_info)],
        //        sock::MsgFlags::empty(),
        //        Some(&destination),
        //    )
        //    .map_err(nix_to_io_error)
    }
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
