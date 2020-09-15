use crate::op_code::OpCode;
use crate::types::Byte;
use crate::addressing_mode::AddressingMode;
use crate::cpu::instruction::Instruction;

#[derive(Clone, Debug)]
pub struct InstructionsMap {}

impl InstructionsMap {
    pub fn new() -> InstructionsMap {
        InstructionsMap {}
    }

    pub fn find(&self, op_id: Byte) -> Instruction {
        match &op_id {
            0x00 => Instruction::new(OpCode::BRK, AddressingMode::Implied, 1),
            0x10 => Instruction::new(OpCode::BPL, AddressingMode::Relative, 2),
            0x18 => Instruction::new(OpCode::CLC, AddressingMode::Implied, 1),
            0x38 => Instruction::new(OpCode::SEC, AddressingMode::Implied, 1),
            0x4C => Instruction::new(OpCode::JMP, AddressingMode::Absolute, 3),
            0x58 => Instruction::new(OpCode::CLI, AddressingMode::Implied, 2),
            0x69 => Instruction::new(OpCode::ADC, AddressingMode::Immediate, 2),
            0x78 => Instruction::new(OpCode::SEI, AddressingMode::Implied, 2),
            0x8D => Instruction::new(OpCode::STA, AddressingMode::Absolute, 4),
            0x9A => Instruction::new(OpCode::TXS, AddressingMode::Implied, 2),
            0xA2 => Instruction::new(OpCode::LDX, AddressingMode::Immediate, 2),
            0xA9 => Instruction::new(OpCode::LDA, AddressingMode::Immediate, 2),
            0xAD => Instruction::new(OpCode::LDA, AddressingMode::Absolute, 4),
            0xEA => Instruction::new(OpCode::NOP, AddressingMode::Implied, 0),
            0xD8 => Instruction::new(OpCode::CLD, AddressingMode::Implied, 1),
            0xF8 => Instruction::new(OpCode::SED, AddressingMode::Implied, 1),
            0xE9 => Instruction::new(OpCode::SBC, AddressingMode::Immediate, 2),
            _ => panic!(format!("Unexpected op code: {:#04X}", op_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn find_instruction() {
        let map = InstructionsMap::new();
        let instruction = map.find(0x00);

        assert_that!(instruction, type_of::<Instruction>());
    }
}