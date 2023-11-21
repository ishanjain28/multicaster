use crate::{parser::qname, ParserError, Qname};

#[derive(Debug)]
pub struct Record {
    domain_name: String,
}

impl Record {
    pub fn parse(data: &[u8], original: &[u8]) -> Result<Self, ParserError> {
        let (domain_name, _) = Qname::read(data, original)?;

        Ok(Self { domain_name })
    }
}
