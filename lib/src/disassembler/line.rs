use crate::instructions::addressing_mode::AddressingMode;

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub instruction: String,
    pub addressing_mode: AddressingMode,
}

impl<'a> Line {
    pub fn new(instruction: &'a str, addressing_mode: AddressingMode) -> Self {
        Line {
            instruction: instruction.to_string(),
            addressing_mode,
        }
    }
}