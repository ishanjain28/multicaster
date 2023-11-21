use crate::{ParserError, Qname, RData};

#[derive(Debug)]
pub struct ResourceRecord {
    pub name: String,
    pub rtype: Type,
    pub class: u16,
    pub cache_flush: bool,
    pub ttl: u32,
    pub rdlength: u16,
    pub rdata: RData,
}

impl ResourceRecord {
    pub fn parse(data: &[u8], original: &[u8]) -> Result<(Self, usize), ParserError> {
        let (name, mut read) = Qname::read(data, original)?;
        if read + 10 > data.len() {
            return Err(ParserError::UnexpectedEOP);
        }

        let rtype = Type::parse(u16::from_be_bytes([data[read], data[read + 1]]))?;
        read += 2;

        let (cache_flush, class) =
            Self::parse_class(u16::from_be_bytes([data[read], data[read + 1]]));
        read += 2;

        let ttl = u32::from_be_bytes([data[read], data[read + 1], data[read + 2], data[read + 3]]);
        read += 4;

        let rdlength = u16::from_be_bytes([data[read], data[read + 1]]);
        read += 2;
        if read + rdlength as usize > data.len() {
            return Err(ParserError::UnexpectedEOP);
        }

        let rdata = RData::parse(rtype, &data[read..read + rdlength as usize], original)?;
        read += rdlength as usize;

        Ok((
            Self {
                name,
                rtype,
                class,
                cache_flush,
                ttl,
                rdlength,
                rdata,
            },
            read,
        ))
    }

    fn parse_class(v: u16) -> (bool, u16) {
        (((v >> 15) & 1) == 1, v & !(1 << 15))
    }
}

#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub enum Type {
    A = 1,
    Cname = 5,
    Ptr = 12,
    Txt = 16,
    Aaaa = 28,
    Srv = 33,
    Nsec = 47,
    Https = 65,
    Opt = 41,
}

impl Type {
    fn parse(v: u16) -> Result<Self, ParserError> {
        use Type::*;
        match v {
            1 => Ok(A),
            5 => Ok(Cname),
            12 => Ok(Ptr),
            16 => Ok(Txt),
            28 => Ok(Aaaa),
            33 => Ok(Srv),
            47 => Ok(Nsec),
            65 => Ok(Https),
            41 => Ok(Opt),
            v => Err(ParserError::UnknownRType(v)),
        }
    }
}
