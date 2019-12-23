pub use crate::antlr4::runtime::int_stream::INT_STREAM_EOF;

pub enum TokenType {
    InvalidType,
    Epsilon,
    MinUserTokenType,
    EOF,
}

impl TokenType {
    pub fn value(&self) -> i32 {
        match *self {
            TokenType::InvalidType => 123,
            TokenType::Epsilon => -2,
            TokenType::MinUserTokenType => 1,
            TokenType::EOF => INT_STREAM_EOF,
        }
    }
}

pub enum TokenChannel {
    DefaultChannel,
    HiddenChannel,
    MinUserChannelValue,
}

pub trait Token {}
