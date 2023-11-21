use crate::{ParserError, Qname};

#[derive(Debug)]
pub struct Record {
    priority: u16,
    weight: u16,
    port: u16,
    target: String,
}

impl Record {
    pub fn parse(mut data: &[u8], original: &[u8]) -> Result<Self, ParserError> {
        let priority = u16::from_be_bytes([data[0], data[1]]);
        data = &data[2..];
        let weight = u16::from_be_bytes([data[0], data[1]]);
        data = &data[2..];
        let port = u16::from_be_bytes([data[0], data[1]]);
        data = &data[2..];

        let (target, _) = Qname::read(data, original)?;

        Ok(Self {
            priority,
            weight,
            port,
            target,
        })
    }
}
