use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Failed to parse DNS Header. Error: {0}")]
    HeaderError(&'static str),

    #[error("Unexpected end of packet")]
    UnexpectedEOP,

    #[error("Label is not UTF-8")]
    LabelIsNotUTF8,

    #[error("Unknown rtype: {0}")]
    UnknownRType(u16),
}
