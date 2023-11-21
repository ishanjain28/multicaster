use crate::ParserError;
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct Record {
    pub address: Ipv4Addr,
}

impl Record {
    pub fn parse(data: &[u8], original: &[u8]) -> Result<Self, ParserError> {
        let address = Ipv4Addr::from([data[0], data[1], data[2], data[3]]);

        Ok(Self { address })
    }
}
