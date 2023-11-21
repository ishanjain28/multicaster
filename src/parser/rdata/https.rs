use crate::{ParserError, Qname};
use std::net::Ipv6Addr;

#[derive(Debug)]
pub struct Record {
    pub svc_priority: u16,
    pub target_name: String,
    pub svc_params: Vec<SvcParam>,
}

#[derive(Debug)]
pub struct SvcParam {
    key: u16,
    value: Vec<u8>,
}

impl Record {
    pub fn parse(mut data: &[u8], original: &[u8]) -> Result<Self, ParserError> {
        let svc_priority = u16::from_be_bytes([data[0], data[1]]);
        data = &data[2..];

        let (target_name, read) = Qname::read(data, original)?;
        data = &data[read..];

        let mut svc_params = vec![];

        while !data.is_empty() {
            let key = u16::from_be_bytes([data[0], data[1]]);
            let value_length = u16::from_be_bytes([data[2], data[3]]) as usize;

            let value = data[4..4 + value_length].to_vec();

            svc_params.push(SvcParam { key, value });

            data = &data[4 + value_length..];
        }

        if !data.is_empty() {
            return Err(ParserError::UnexpectedEOP);
        }

        Ok(Self {
            svc_priority,
            target_name,
            svc_params,
        })
    }
}
