use crate::types::Byte;
use crate::addressing_mode::AddressingMode;
use crate::cpu::instruction::Instruction;
use crate::cpu::op_code::OpCode;

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
            0x20 => Instruction::new(OpCode::JSR, AddressingMode::Absolute, 6),
            0x2A => Instruction::new(OpCode::ROL, AddressingMode::Accumulator, 2),
            0x2C => Instruction::new(OpCode::BIT, AddressingMode::Absolute, 4),
            0x18 => Instruction::new(OpCode::CLC, AddressingMode::Implied, 1),
            0x38 => Instruction::new(OpCode::SEC, AddressingMode::Implied, 1),
            0x4C => Instruction::new(OpCode::JMP, AddressingMode::Absolute, 3),
            0x48 => Instruction::new(OpCode::PHA, AddressingMode::Implied, 3),
            0x58 => Instruction::new(OpCode::CLI, AddressingMode::Implied, 2),
            0x60 => Instruction::new(OpCode::RTS, AddressingMode::Implied, 6),
            0x68 => Instruction::new(OpCode::PLA, AddressingMode::Implied, 4),
            0x69 => Instruction::new(OpCode::ADC, AddressingMode::Immediate, 2),
            0x78 => Instruction::new(OpCode::SEI, AddressingMode::Implied, 2),
            0x85 => Instruction::new(OpCode::STA, AddressingMode::ZeroPage, 3),
            0x86 => Instruction::new(OpCode::STX, AddressingMode::ZeroPage, 3),
            0x88 => Instruction::new(OpCode::DEY, AddressingMode::Implied, 2),
            0x8C => Instruction::new(OpCode::STY, AddressingMode::Absolute, 4),
            0x8D => Instruction::new(OpCode::STA, AddressingMode::Absolute, 4),
            0x8E => Instruction::new(OpCode::STX, AddressingMode::Absolute, 4),
            0x90 => Instruction::new(OpCode::BCC, AddressingMode::Relative, 2),
            0x91 => Instruction::new(OpCode::STA, AddressingMode::YIndexedIndirect, 6),
            0x9A => Instruction::new(OpCode::TXS, AddressingMode::Implied, 2),
            0x9D => Instruction::new(OpCode::STA, AddressingMode::AbsoluteX, 5),
            0xA0 => Instruction::new(OpCode::LDY, AddressingMode::Immediate, 2),
            0xA2 => Instruction::new(OpCode::LDX, AddressingMode::Immediate, 2),
            0xA5 => Instruction::new(OpCode::LDA, AddressingMode::ZeroPage, 3),
            0xA6 => Instruction::new(OpCode::LDX, AddressingMode::ZeroPage, 3),
            0xA9 => Instruction::new(OpCode::LDA, AddressingMode::Immediate, 2),
            0xAD => Instruction::new(OpCode::LDA, AddressingMode::Absolute, 4),
            0xB1 => Instruction::new(OpCode::LDA, AddressingMode::YIndexedIndirect, 5),
            0xBD => Instruction::new(OpCode::LDA, AddressingMode::AbsoluteX, 4),
            0xCA => Instruction::new(OpCode::DEX, AddressingMode::Implied, 2),
            0xC9 => Instruction::new(OpCode::CMP, AddressingMode::Immediate, 2),
            0xE0 => Instruction::new(OpCode::CPX, AddressingMode::Immediate, 2),
            0xEA => Instruction::new(OpCode::NOP, AddressingMode::Implied, 0),
            0xD0 => Instruction::new(OpCode::BNE, AddressingMode::Relative, 1),
            0xD8 => Instruction::new(OpCode::CLD, AddressingMode::Implied, 1),
            0xE8 => Instruction::new(OpCode::INX, AddressingMode::Implied, 2),
            0xF8 => Instruction::new(OpCode::SED, AddressingMode::Implied, 1),
            0xE9 => Instruction::new(OpCode::SBC, AddressingMode::Immediate, 2),
            _ => panic!(format!("[InstructionsMap::find] Unexpected op code: {:#04X}", op_id))
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