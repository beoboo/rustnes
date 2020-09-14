use crate::types::Word;
use crate::addressing_mode::AddressingMode;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    Address(AddressingMode, Word),
    Keyword(String),
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
