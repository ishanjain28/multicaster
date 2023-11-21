use crate::ParserError;

#[derive(Debug, Default)]
pub struct Qname {}

impl Qname {
    pub fn read(data: &[u8], original: &[u8]) -> Result<(String, usize), ParserError> {
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

                v if v & 0b1100_0000 == 0b1100_0000 => {
                    let nof = (((v & 0b0011_1111) as usize) << 8) | parse_data[offset + 1] as usize;

                    if nof >= original.len() {
                        return Err(ParserError::UnexpectedEOP);
                    }

                    if return_pos.is_none() {
                        return_pos = Some(offset);
                    }

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
