use crate::ParserError;

#[derive(Debug, Default)]
pub struct Question {
    pub qname: String,
    pub qtype: u16,
    pub unicast_preferred: bool,
    pub qclass: u16,
}

impl Question {
    pub fn parse(data: &[u8], original: &[u8]) -> Result<(Self, usize), ParserError> {
        let (qname, mut read) = Self::read_label(data, original)?;

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

    fn read_label(data: &[u8], original: &[u8]) -> Result<(String, usize), ParserError> {
        let mut parse_data = data;
        let mut out = Vec::new();
        let mut offset = 0;
        let mut byte = parse_data[offset];
        let mut bytes_read = 1;

        let mut return_pos: Option<usize> = None;

        loop {
            match byte {
                0 => {
                    out.pop();

                    return Ok((
                        String::from_utf8(out).map_err(|_| ParserError::LabelIsNotUTF8)?,
                        if let Some(p) = return_pos {
                            p + 2
                        } else {
                            bytes_read
                        },
                    ));
                }
                0xc0 => {
                    let nof = parse_data[offset + 1] as usize;
                    if nof >= original.len() {
                        return Err(ParserError::UnexpectedEOP);
                    }

                    return_pos = match return_pos {
                        Some(x) => Some(std::cmp::max(x, offset)),
                        None => Some(offset),
                    };

                    offset = 0;
                    parse_data = &original[nof..];
                    byte = parse_data[0];
                }

                _ => {
                    out.extend(&parse_data[offset + 1..offset + 1 + byte as usize]);

                    out.push(b'.');

                    offset += byte as usize + 1;
                    bytes_read += byte as usize + 1;

                    byte = parse_data[offset];
                }
            }
        }
    }
}
