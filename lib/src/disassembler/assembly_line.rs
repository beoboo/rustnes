use crate::instructions::addressing_mode::AddressingMode;
use crate::types::Word;
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct AssemblyLine {
    pub address: Word,
    pub instruction: String,
    pub addressing_mode: AddressingMode,
    pub operand: Word,
}

impl<'a> AssemblyLine {
    pub fn new(address: Word, instruction: &'a str, addressing_mode: AddressingMode, operand: Word) -> Self {
        Self {
            address,
            instruction: instruction.to_string(),
            addressing_mode,
            operand,
        }
    }
}

fn relative_address(pos: Word, relative: Word) -> String {
    let pos = if relative > 0x80 {
        pos + relative - 0xFF
    } else {
        pos + relative
    };

    format!(" [{:#06X}]", pos + 1)
}

impl Display for AssemblyLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let operand = if self.addressing_mode == AddressingMode::Relative { relative_address(self.address, self.operand) } else { String::from("") };

        write!(f, "{:#06X} {}{}", self.address, self.instruction, operand)
    }
}