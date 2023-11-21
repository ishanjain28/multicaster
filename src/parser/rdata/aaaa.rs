use crate::ParserError;
use std::net::Ipv6Addr;

#[derive(Debug)]
pub struct Record {
    pub address: Ipv6Addr,
}

impl Record {
    pub fn parse(mut data: &[u8], original: &[u8]) -> Result<Self, ParserError> {
        let address = Ipv6Addr::from([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            data[9], data[10], data[11], data[12], data[13], data[14], data[15],
        ]);

        Ok(Self { address })
    }
}
