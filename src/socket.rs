#![allow(unused)]

use log::trace;
// This code has been adapted from multicast_socket crate
use nix::sys::{
    self,
    socket::{self as sock, AddressFamily, SockaddrIn, SockaddrLike, SockaddrStorage},
};
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
            buffer_size: 4096,
        }
    }
}

#[derive(Debug)]
pub struct MulticastSocket {
    socket: socket2::Socket,
    interfaces: HashMap<String, Vec<IpAddr>>,
    multicast_group: SocketAddrV4,
    buffer_size: usize,
}

impl MulticastSocket {
    pub fn new(
        options: MulticastOptions,
        interfaces: HashMap<String, Vec<IpAddr>>,
        multicast_group: SocketAddrV4,
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
            trace!("joining groups = {:?}", multicast_group);

            for address in addresses {
                if let IpAddr::V4(v4_addr) = address {
                    if v4_addr.is_loopback() {
                        continue;
                    }

                    socket.join_multicast_v4(multicast_group.ip(), v4_addr)?;

                    trace!(
                        "joined ipv4 multicast group {} {}",
                        multicast_group.ip(),
                        v4_addr
                    );
                }
            }
        }

        socket.bind(
            &SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), multicast_group.port()).into(),
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
    pub origin_address: Option<SocketAddrV4>,
    pub interface: Interface,
}

#[derive(Debug, Clone)]
pub enum Interface {
    Default,
    Index(i32),
    IpAddr(IpAddr),
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
        let mut control_buffer = nix::cmsg_space!(libc::in_pktinfo);

        let (origin_address, interface, bytes_read) = {
            let message = sock::recvmsg(
                self.socket.as_raw_fd(),
                &mut [IoSliceMut::new(&mut data_buffer)],
                Some(&mut control_buffer),
                sock::MsgFlags::empty(),
            )
            .map_err(nix_to_io_error)?;

            let origin_address = message.address.map(|x: sock::SockaddrIn| {
                SocketAddrV4::new(Ipv4Addr::from_bits(x.ip()), x.port())
            });

            let mut interface = Interface::Default;

            for cmsg in message.cmsgs() {
                if let sock::ControlMessageOwned::Ipv4PacketInfo(pktinfo) = cmsg {
                    interface = Interface::Index(pktinfo.ipi_ifindex as _);
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
        let mut pkt_info: libc::in_pktinfo = unsafe { mem::zeroed() };

        match interface {
            Interface::Default => todo!(),
            Interface::Index(i) => {
                pkt_info.ipi_ifindex = *i as _;
            }
            Interface::IpAddr(IpAddr::V4(addr)) => {
                pkt_info.ipi_spec_dst = libc::in_addr {
                    s_addr: (*addr).into(),
                };
            }

            _ => unreachable!(),
        }

        sock::sendmsg(
            self.socket.as_raw_fd(),
            &[IoSlice::new(buf)],
            &[sock::ControlMessage::Ipv4PacketInfo(&pkt_info)],
            sock::MsgFlags::empty(),
            Some(&SockaddrIn::from(self.multicast_group)),
        )
        .map_err(nix_to_io_error)
    }
}
