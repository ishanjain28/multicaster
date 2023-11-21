mod a;
mod aaaa;
mod cname;
mod https;
mod nsec;
mod ptr;
mod srv;
mod txt;
use crate::{ParserError, Type};

#[derive(Debug)]
pub enum RData {
    A(a::Record),
    Aaaa(aaaa::Record),
    Cname(cname::Record),
    Ptr(ptr::Record),
    Txt(txt::Record),
    Srv(srv::Record),
    Https(https::Record),

    Unknown(Type, Vec<u8>),
}

impl RData {
    pub fn parse(rtype: Type, data: &[u8], original: &[u8]) -> Result<Self, ParserError> {
        use RData::*;
        match rtype {
            Type::A => Ok(A(a::Record::parse(data, original)?)),
            Type::Cname => Ok(Cname(cname::Record::parse(data, original)?)),
            Type::Ptr => Ok(Ptr(ptr::Record::parse(data, original)?)),
            Type::Txt => Ok(Txt(txt::Record::parse(data, original)?)),
            Type::Aaaa => Ok(Aaaa(aaaa::Record::parse(data, original)?)),
            Type::Srv => Ok(Srv(srv::Record::parse(data, original)?)),
            Type::Https => Ok(Https(https::Record::parse(data, original)?)),

            _ => Ok(Unknown(rtype, data.to_vec())),
        }
    }
}
