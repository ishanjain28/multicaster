use crate::{ParserError, Qname};

#[derive(Debug)]
pub struct Record {}

impl Record {
    pub fn parse(data: &[u8], original: &[u8]) -> Result<Self, ParserError> {
        unimplemented!()
    }
}
