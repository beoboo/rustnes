use crate::instructions::addressing_mode::AddressingMode;
use crate::types::Word;

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub instruction: String,
    pub addressing_mode: AddressingMode,
    pub address: Word,
}

impl<'a> Line {
    pub fn new(instruction: &'a str, addressing_mode: AddressingMode, address: Word) -> Self {
        Line {
            instruction: instruction.to_string(),
            addressing_mode,
            address,
        }
    }
}