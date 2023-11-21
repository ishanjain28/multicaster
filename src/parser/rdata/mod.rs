mod a;
mod aaaa;
mod cname;
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
    Nsec(nsec::Record),

    Unknown(Type, Vec<u8>),
}

impl RData {
    pub fn parse(rtype: Type, data: &[u8], original: &[u8]) -> Result<Self, ParserError> {
        match rtype {
            Type::A => Ok(RData::A(a::Record::parse(data, original)?)),
            Type::Cname => Ok(RData::Cname(cname::Record::parse(data, original)?)),
            Type::Ptr => Ok(RData::Ptr(ptr::Record::parse(data, original)?)),
            Type::Txt => Ok(RData::Txt(txt::Record::parse(data, original)?)),
            Type::Aaaa => Ok(RData::Aaaa(aaaa::Record::parse(data, original)?)),
            Type::Srv => Ok(RData::Srv(srv::Record::parse(data, original)?)),
            Type::Nsec => Ok(RData::Nsec(nsec::Record::parse(data, original)?)),
            Type::Https => todo!(),
        }
    }
}
