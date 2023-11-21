use crate::{ParserError, Qname};

#[derive(Debug, Default)]
pub struct Question {
    pub qname: String,
    pub qtype: u16,
    pub unicast_preferred: bool,
    pub qclass: u16,
}

impl Question {
    pub fn parse(data: &[u8], original: &[u8]) -> Result<(Self, usize), ParserError> {
        let (qname, mut read) = Qname::read(data, original)?;

        let qtype = u16::from_be_bytes([data[read], data[read + 1]]);
        read += 2;
        let mut qclass = u16::from_be_bytes([data[read], data[read + 1]]);
        read += 2;

        let unicast_preferred = (qclass & (1 << 15)) == 1 << 15;
        qclass &= !(1 << 15);

        Ok((
            Self {
                qname,
                qtype,
                unicast_preferred,
                qclass,
            },
            read,
        ))
    }
}
