use std::collections::HashMap;

use crate::instructions::addressing_mode::AddressingMode;
use crate::instructions::instruction::Instruction;
use crate::instructions::op_code::OpCode;
use crate::types::Byte;
use std::borrow::Borrow;

pub mod op_code;
mod instruction;
pub mod addressing_mode;

// #[derive(Debug)]
pub struct Instructions {
    map: HashMap<Byte, Instruction>
}

impl Instructions {
    pub fn new() -> Instructions {
        let mut map = HashMap::new();

        map.insert(0x00, Instruction::new(OpCode::BRK, AddressingMode::Implied, 7));
        map.insert(0x01, Instruction::new(OpCode::ORA, AddressingMode::IndirectIndexedX, 6));
        map.insert(0x05, Instruction::new(OpCode::ORA, AddressingMode::ZeroPage, 3));
        map.insert(0x06, Instruction::new(OpCode::ASL, AddressingMode::ZeroPage, 5));
        map.insert(0x08, Instruction::new(OpCode::PHP, AddressingMode::Implied, 3));
        map.insert(0x09, Instruction::new(OpCode::ORA, AddressingMode::Immediate, 2));
        map.insert(0x0A, Instruction::new(OpCode::ASL, AddressingMode::Accumulator, 2));
        map.insert(0x0D, Instruction::new(OpCode::ORA, AddressingMode::Absolute, 4));
        map.insert(0x0E, Instruction::new(OpCode::ASL, AddressingMode::Absolute, 6));
        map.insert(0x10, Instruction::new(OpCode::BPL, AddressingMode::Relative, 2));
        map.insert(0x11, Instruction::new(OpCode::ORA, AddressingMode::YIndexedIndirect, 5));
        map.insert(0x15, Instruction::new(OpCode::ORA, AddressingMode::ZeroPageX, 4));
        map.insert(0x16, Instruction::new(OpCode::ASL, AddressingMode::ZeroPageX, 6));
        map.insert(0x18, Instruction::new(OpCode::CLC, AddressingMode::Implied, 2));
        map.insert(0x19, Instruction::new(OpCode::ORA, AddressingMode::AbsoluteY, 4));
        map.insert(0x1D, Instruction::new(OpCode::ORA, AddressingMode::AbsoluteX, 4));
        map.insert(0x1E, Instruction::new(OpCode::ASL, AddressingMode::AbsoluteX, 7));
        map.insert(0x20, Instruction::new(OpCode::JSR, AddressingMode::Absolute, 6));
        map.insert(0x21, Instruction::new(OpCode::AND, AddressingMode::IndirectIndexedX, 6));
        map.insert(0x24, Instruction::new(OpCode::BIT, AddressingMode::ZeroPage, 3));
        map.insert(0x25, Instruction::new(OpCode::AND, AddressingMode::ZeroPage, 3));
        map.insert(0x26, Instruction::new(OpCode::ROL, AddressingMode::ZeroPage, 5));
        map.insert(0x28, Instruction::new(OpCode::PLP, AddressingMode::Implied, 4));
        map.insert(0x29, Instruction::new(OpCode::AND, AddressingMode::Immediate, 2));
        map.insert(0x2A, Instruction::new(OpCode::ROL, AddressingMode::Accumulator, 2));
        map.insert(0x2C, Instruction::new(OpCode::BIT, AddressingMode::Absolute, 4));
        map.insert(0x2D, Instruction::new(OpCode::AND, AddressingMode::Absolute, 4));
        map.insert(0x2E, Instruction::new(OpCode::ROL, AddressingMode::Absolute, 6));
        map.insert(0x30, Instruction::new(OpCode::BMI, AddressingMode::Relative, 2));
        map.insert(0x31, Instruction::new(OpCode::AND, AddressingMode::YIndexedIndirect, 5));
        map.insert(0x35, Instruction::new(OpCode::AND, AddressingMode::ZeroPageX, 4));
        map.insert(0x36, Instruction::new(OpCode::ROL, AddressingMode::ZeroPageX, 6));
        map.insert(0x38, Instruction::new(OpCode::SEC, AddressingMode::Implied, 2));
        map.insert(0x39, Instruction::new(OpCode::AND, AddressingMode::AbsoluteY, 4));
        map.insert(0x3D, Instruction::new(OpCode::AND, AddressingMode::AbsoluteX, 4));
        map.insert(0x3E, Instruction::new(OpCode::ROL, AddressingMode::AbsoluteX, 7));
        map.insert(0x40, Instruction::new(OpCode::RTI, AddressingMode::Implied, 6));
        map.insert(0x41, Instruction::new(OpCode::EOR, AddressingMode::IndirectIndexedX, 6));
        map.insert(0x45, Instruction::new(OpCode::EOR, AddressingMode::ZeroPage, 3));
        map.insert(0x46, Instruction::new(OpCode::LSR, AddressingMode::ZeroPage, 5));
        map.insert(0x48, Instruction::new(OpCode::PHA, AddressingMode::Implied, 3));
        map.insert(0x49, Instruction::new(OpCode::EOR, AddressingMode::Immediate, 2));
        map.insert(0x4A, Instruction::new(OpCode::LSR, AddressingMode::Accumulator, 2));
        map.insert(0x4C, Instruction::new(OpCode::JMP, AddressingMode::Absolute, 3));
        map.insert(0x4D, Instruction::new(OpCode::EOR, AddressingMode::Absolute, 4));
        map.insert(0x4E, Instruction::new(OpCode::LSR, AddressingMode::Absolute, 6));
        map.insert(0x50, Instruction::new(OpCode::BVC, AddressingMode::Relative, 2));
        map.insert(0x51, Instruction::new(OpCode::EOR, AddressingMode::YIndexedIndirect, 5));
        map.insert(0x55, Instruction::new(OpCode::EOR, AddressingMode::ZeroPageX, 4));
        map.insert(0x56, Instruction::new(OpCode::LSR, AddressingMode::ZeroPageX, 6));
        map.insert(0x58, Instruction::new(OpCode::CLI, AddressingMode::Implied, 2));
        map.insert(0x59, Instruction::new(OpCode::EOR, AddressingMode::AbsoluteY, 4));
        map.insert(0x5D, Instruction::new(OpCode::EOR, AddressingMode::AbsoluteX, 4));
        map.insert(0x5E, Instruction::new(OpCode::LSR, AddressingMode::AbsoluteX, 7));
        map.insert(0x60, Instruction::new(OpCode::RTS, AddressingMode::Implied, 6));
        map.insert(0x61, Instruction::new(OpCode::ADC, AddressingMode::IndirectIndexedX, 6));
        map.insert(0x65, Instruction::new(OpCode::ADC, AddressingMode::ZeroPage, 3));
        map.insert(0x66, Instruction::new(OpCode::ROR, AddressingMode::ZeroPage, 5));
        map.insert(0x68, Instruction::new(OpCode::PLA, AddressingMode::Implied, 4));
        map.insert(0x69, Instruction::new(OpCode::ADC, AddressingMode::Immediate, 2));
        map.insert(0x6A, Instruction::new(OpCode::ROR, AddressingMode::Accumulator, 2));
        map.insert(0x6C, Instruction::new(OpCode::JMP, AddressingMode::Indirect, 5));
        map.insert(0x6D, Instruction::new(OpCode::ADC, AddressingMode::Absolute, 4));
        map.insert(0x6E, Instruction::new(OpCode::ROR, AddressingMode::Absolute, 6));
        map.insert(0x70, Instruction::new(OpCode::BVS, AddressingMode::Relative, 2));
        map.insert(0x71, Instruction::new(OpCode::ADC, AddressingMode::YIndexedIndirect, 5));
        map.insert(0x75, Instruction::new(OpCode::ADC, AddressingMode::ZeroPageX, 4));
        map.insert(0x76, Instruction::new(OpCode::ROR, AddressingMode::ZeroPageX, 6));
        map.insert(0x78, Instruction::new(OpCode::SEI, AddressingMode::Implied, 2));
        map.insert(0x79, Instruction::new(OpCode::ADC, AddressingMode::AbsoluteY, 4));
        map.insert(0x7D, Instruction::new(OpCode::ADC, AddressingMode::AbsoluteX, 4));
        map.insert(0x7E, Instruction::new(OpCode::ROR, AddressingMode::AbsoluteX, 7));
        map.insert(0x81, Instruction::new(OpCode::STA, AddressingMode::IndirectIndexedX, 6));
        map.insert(0x84, Instruction::new(OpCode::STY, AddressingMode::ZeroPage, 3));
        map.insert(0x85, Instruction::new(OpCode::STA, AddressingMode::ZeroPage, 3));
        map.insert(0x86, Instruction::new(OpCode::STX, AddressingMode::ZeroPage, 3));
        map.insert(0x88, Instruction::new(OpCode::DEY, AddressingMode::Implied, 2));
        map.insert(0x8A, Instruction::new(OpCode::TXA, AddressingMode::Implied, 2));
        map.insert(0x8C, Instruction::new(OpCode::STY, AddressingMode::Absolute, 4));
        map.insert(0x8D, Instruction::new(OpCode::STA, AddressingMode::Absolute, 4));
        map.insert(0x8E, Instruction::new(OpCode::STX, AddressingMode::Absolute, 4));
        map.insert(0x90, Instruction::new(OpCode::BCC, AddressingMode::Relative, 2));
        map.insert(0x91, Instruction::new(OpCode::STA, AddressingMode::YIndexedIndirect, 6));
        map.insert(0x94, Instruction::new(OpCode::STY, AddressingMode::ZeroPageX, 4));
        map.insert(0x95, Instruction::new(OpCode::STA, AddressingMode::ZeroPageX, 4));
        map.insert(0x96, Instruction::new(OpCode::STX, AddressingMode::ZeroPageY, 4));
        map.insert(0x98, Instruction::new(OpCode::TYA, AddressingMode::Implied, 2));
        map.insert(0x99, Instruction::new(OpCode::STA, AddressingMode::AbsoluteY, 5));
        map.insert(0x9A, Instruction::new(OpCode::TXS, AddressingMode::Implied, 2));
        map.insert(0x9D, Instruction::new(OpCode::STA, AddressingMode::AbsoluteX, 5));
        map.insert(0xA0, Instruction::new(OpCode::LDY, AddressingMode::Immediate, 2));
        map.insert(0xA1, Instruction::new(OpCode::LDA, AddressingMode::IndirectIndexedX, 6));
        map.insert(0xA2, Instruction::new(OpCode::LDX, AddressingMode::Immediate, 2));
        map.insert(0xA4, Instruction::new(OpCode::LDY, AddressingMode::ZeroPage, 3));
        map.insert(0xA5, Instruction::new(OpCode::LDA, AddressingMode::ZeroPage, 3));
        map.insert(0xA6, Instruction::new(OpCode::LDX, AddressingMode::ZeroPage, 3));
        map.insert(0xA8, Instruction::new(OpCode::TAY, AddressingMode::Implied, 2));
        map.insert(0xA9, Instruction::new(OpCode::LDA, AddressingMode::Immediate, 2));
        map.insert(0xAA, Instruction::new(OpCode::TAX, AddressingMode::Implied, 2));
        map.insert(0xAC, Instruction::new(OpCode::LDY, AddressingMode::Absolute, 4));
        map.insert(0xAD, Instruction::new(OpCode::LDA, AddressingMode::Absolute, 4));
        map.insert(0xAE, Instruction::new(OpCode::LDX, AddressingMode::Absolute, 4));
        map.insert(0xB0, Instruction::new(OpCode::BCS, AddressingMode::Relative, 2));
        map.insert(0xB1, Instruction::new(OpCode::LDA, AddressingMode::YIndexedIndirect, 5));
        map.insert(0xB4, Instruction::new(OpCode::LDY, AddressingMode::ZeroPageX, 4));
        map.insert(0xB5, Instruction::new(OpCode::LDA, AddressingMode::ZeroPageX, 4));
        map.insert(0xB6, Instruction::new(OpCode::LDX, AddressingMode::ZeroPageY, 4));
        map.insert(0xB8, Instruction::new(OpCode::CLV, AddressingMode::Implied, 2));
        map.insert(0xB9, Instruction::new(OpCode::LDA, AddressingMode::AbsoluteY, 4));
        map.insert(0xBA, Instruction::new(OpCode::TSX, AddressingMode::Implied, 2));
        map.insert(0xBC, Instruction::new(OpCode::LDY, AddressingMode::AbsoluteX, 4));
        map.insert(0xBD, Instruction::new(OpCode::LDA, AddressingMode::AbsoluteX, 4));
        map.insert(0xBE, Instruction::new(OpCode::LDX, AddressingMode::AbsoluteY, 4));
        map.insert(0xC0, Instruction::new(OpCode::CPY, AddressingMode::Immediate, 2));
        map.insert(0xC1, Instruction::new(OpCode::CMP, AddressingMode::IndirectIndexedX, 6));
        map.insert(0xC4, Instruction::new(OpCode::CPY, AddressingMode::ZeroPage, 3));
        map.insert(0xC5, Instruction::new(OpCode::CMP, AddressingMode::ZeroPage, 3));
        map.insert(0xC6, Instruction::new(OpCode::DEC, AddressingMode::ZeroPage, 5));
        map.insert(0xC8, Instruction::new(OpCode::INY, AddressingMode::Implied, 2));
        map.insert(0xC9, Instruction::new(OpCode::CMP, AddressingMode::Immediate, 2));
        map.insert(0xCA, Instruction::new(OpCode::DEX, AddressingMode::Implied, 2));
        map.insert(0xCC, Instruction::new(OpCode::CPY, AddressingMode::Absolute, 4));
        map.insert(0xCD, Instruction::new(OpCode::CMP, AddressingMode::Absolute, 4));
        map.insert(0xCE, Instruction::new(OpCode::DEC, AddressingMode::Absolute, 6));
        map.insert(0xD0, Instruction::new(OpCode::BNE, AddressingMode::Relative, 2));
        map.insert(0xD1, Instruction::new(OpCode::CMP, AddressingMode::YIndexedIndirect, 5));
        map.insert(0xD5, Instruction::new(OpCode::CMP, AddressingMode::ZeroPageX, 4));
        map.insert(0xD6, Instruction::new(OpCode::DEC, AddressingMode::ZeroPageX, 6));
        map.insert(0xD8, Instruction::new(OpCode::CLD, AddressingMode::Implied, 2));
        map.insert(0xD9, Instruction::new(OpCode::CMP, AddressingMode::AbsoluteY, 4));
        map.insert(0xDD, Instruction::new(OpCode::CMP, AddressingMode::AbsoluteX, 4));
        map.insert(0xDE, Instruction::new(OpCode::DEC, AddressingMode::AbsoluteX, 7));
        map.insert(0xE0, Instruction::new(OpCode::CPX, AddressingMode::Immediate, 2));
        map.insert(0xE1, Instruction::new(OpCode::SBC, AddressingMode::IndirectIndexedX, 6));
        map.insert(0xE4, Instruction::new(OpCode::CPX, AddressingMode::ZeroPage, 3));
        map.insert(0xE5, Instruction::new(OpCode::SBC, AddressingMode::ZeroPage, 3));
        map.insert(0xE6, Instruction::new(OpCode::INC, AddressingMode::ZeroPage, 5));
        map.insert(0xE8, Instruction::new(OpCode::INX, AddressingMode::Implied, 2));
        map.insert(0xE9, Instruction::new(OpCode::SBC, AddressingMode::Immediate, 2));
        map.insert(0xEA, Instruction::new(OpCode::NOP, AddressingMode::Implied, 2));
        map.insert(0xEC, Instruction::new(OpCode::CPX, AddressingMode::Absolute, 4));
        map.insert(0xED, Instruction::new(OpCode::SBC, AddressingMode::Absolute, 4));
        map.insert(0xEE, Instruction::new(OpCode::INC, AddressingMode::Absolute, 6));
        map.insert(0xF0, Instruction::new(OpCode::BEQ, AddressingMode::Relative, 2));
        map.insert(0xF1, Instruction::new(OpCode::SBC, AddressingMode::YIndexedIndirect, 5));
        map.insert(0xF5, Instruction::new(OpCode::SBC, AddressingMode::ZeroPageX, 4));
        map.insert(0xF6, Instruction::new(OpCode::INC, AddressingMode::ZeroPageX, 6));
        map.insert(0xF8, Instruction::new(OpCode::SED, AddressingMode::Implied, 2));
        map.insert(0xF9, Instruction::new(OpCode::SBC, AddressingMode::AbsoluteY, 4));
        map.insert(0xFD, Instruction::new(OpCode::SBC, AddressingMode::AbsoluteX, 4));
        map.insert(0xFE, Instruction::new(OpCode::INC, AddressingMode::AbsoluteX, 7));

        Instructions {
            map
        }
    }

    pub fn find(&self, op_id: Byte) -> Instruction {
        *self.map.get(&op_id).unwrap_or(Instruction::new(OpCode::NOP, AddressingMode::Implied, 2).borrow())
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn find_instruction() {
        let map = Instructions::new();
        let instruction = map.find(0x00);

        assert_that!(instruction, type_of::<Instruction>());
    }
}