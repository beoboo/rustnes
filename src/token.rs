use crate::types::Byte;
use crate::addressing::Addressing;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    Keyword(String),
    ImmediateAddress(Byte),
    EOF,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: u32,
}

impl Token {
    pub fn new(token_type: TokenType, line: u32) -> Token {
        Token {
            token_type,
            line,
        }
    }

    pub fn is_eof(&self) -> bool {
        self.token_type == TokenType::EOF
    }
}
