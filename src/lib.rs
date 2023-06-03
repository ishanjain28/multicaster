use once_cell::sync::Lazy;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub static IPv4: Lazy<IpAddr> = Lazy::new(|| Ipv4Addr::new(224, 0, 0, 123).into());
pub static IPv6: Lazy<IpAddr> =
    Lazy::new(|| Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0x0123).into());
