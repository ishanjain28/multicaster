use crate::ParserError;

#[derive(Debug, Default)]
pub struct Header {
    pub id: u16,
    pub fields: u16,
    pub qd_count: u16,
    pub an_count: u16,
    pub ns_count: u16,
    pub ar_count: u16,
}

impl Header {
    pub fn parse(data: &[u8]) -> Result<Self, ParserError> {
        if data.len() < 12 {
            return Err(ParserError::HeaderError(
                "input length is less than 20 bytes",
            ));
        }

        Ok(Header {
            id: u16::from_be_bytes([data[0], data[1]]),
            fields: u16::from_be_bytes([data[2], data[3]]),
            qd_count: u16::from_be_bytes([data[4], data[5]]),
            an_count: u16::from_be_bytes([data[6], data[7]]),
            ns_count: u16::from_be_bytes([data[8], data[9]]),
            ar_count: u16::from_be_bytes([data[10], data[11]]),
        })
    }

    pub fn qr(&self) -> bool {
        ((self.fields >> 15) & 1) == 1
    }

    pub fn opcode(&self) -> u8 {
        ((self.fields >> 11) & 0b01111) as u8
    }

    pub fn aa(&self) -> bool {
        ((self.fields >> 10) & 1) == 1
    }

    pub fn tc(&self) -> bool {
        ((self.fields >> 9) & 1) == 1
    }

    pub fn rd(&self) -> bool {
        ((self.fields >> 8) & 1) == 1
    }

    pub fn ra(&self) -> bool {
        ((self.fields >> 7) & 1) == 1
    }

    pub fn z(&self) -> u8 {
        ((self.fields >> 4) & 0b111) as u8
    }

    pub fn rcode(&self) -> u8 {
        (self.fields & 0b111) as u8
    }

    pub fn size() -> usize {
        12
    }
}
