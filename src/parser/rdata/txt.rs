use crate::{ParserError, Qname};

#[derive(Debug)]
pub struct Record {
    sets: Vec<u8>,
}

impl Record {
    pub fn parse(data: &[u8], original: &[u8]) -> Result<Self, ParserError> {
        Ok(Self {
            sets: data.to_vec(),
        })
    }
}
