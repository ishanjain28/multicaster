use crate::{ParserError, Qname};

#[derive(Debug)]
pub struct Record {}

impl Record {
    pub fn parse(data: &[u8], original: &[u8]) -> Result<Self, ParserError> {
        println!("{:02x?}", &data[..]);
        let (domain_name, mut read) = Qname::read(data, original)?;
        println!("{:02x?}", &data[read..]);

        read += 1;
        let rr_bitmap_len = data[read];

        let rr_bitmap = u16::from_be_bytes([data[read], data[read + 1]]);

        println!("{:0x}", rr_bitmap_len);

        Ok(Self {})
    }
}
