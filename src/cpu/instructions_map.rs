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
            0x00 => Instruction::new(OpCode::BRK, AddressingMode::Implied, 7),
            0x01 => Instruction::new(OpCode::ORA, AddressingMode::IndirectIndexedX, 6),
            0x05 => Instruction::new(OpCode::ORA, AddressingMode::ZeroPage, 3),
            0x06 => Instruction::new(OpCode::ASL, AddressingMode::ZeroPage, 5),
            0x08 => Instruction::new(OpCode::PHP, AddressingMode::Implied, 3),
            0x09 => Instruction::new(OpCode::ORA, AddressingMode::Immediate, 2),
            0x0A => Instruction::new(OpCode::ASL, AddressingMode::Accumulator, 2),
            0x0D => Instruction::new(OpCode::ORA, AddressingMode::Absolute, 4),
            0x0E => Instruction::new(OpCode::ASL, AddressingMode::Absolute, 6),
            0x10 => Instruction::new(OpCode::BPL, AddressingMode::Relative, 2),
            0x11 => Instruction::new(OpCode::ORA, AddressingMode::YIndexedIndirect, 5),
            0x15 => Instruction::new(OpCode::ORA, AddressingMode::ZeroPageX, 4),
            0x16 => Instruction::new(OpCode::ASL, AddressingMode::ZeroPageX, 6),
            0x18 => Instruction::new(OpCode::CLC, AddressingMode::Implied, 2),
            0x19 => Instruction::new(OpCode::ORA, AddressingMode::AbsoluteY, 4),
            0x1D => Instruction::new(OpCode::ORA, AddressingMode::AbsoluteX, 4),
            0x1E => Instruction::new(OpCode::ASL, AddressingMode::AbsoluteX, 7),
            0x20 => Instruction::new(OpCode::JSR, AddressingMode::Absolute, 6),
            0x21 => Instruction::new(OpCode::AND, AddressingMode::IndirectIndexedX, 6),
            0x24 => Instruction::new(OpCode::BIT, AddressingMode::ZeroPage, 3),
            0x25 => Instruction::new(OpCode::AND, AddressingMode::ZeroPage, 3),
            0x26 => Instruction::new(OpCode::ROL, AddressingMode::ZeroPage, 5),
            0x28 => Instruction::new(OpCode::PLP, AddressingMode::Implied, 4),
            0x29 => Instruction::new(OpCode::AND, AddressingMode::Immediate, 2),
            0x2A => Instruction::new(OpCode::ROL, AddressingMode::Accumulator, 2),
            0x2C => Instruction::new(OpCode::BIT, AddressingMode::Absolute, 4),
            0x2D => Instruction::new(OpCode::AND, AddressingMode::Absolute, 4),
            0x2E => Instruction::new(OpCode::ROL, AddressingMode::Absolute, 6),
            0x30 => Instruction::new(OpCode::BMI, AddressingMode::Relative, 2),
            0x31 => Instruction::new(OpCode::AND, AddressingMode::YIndexedIndirect, 5),
            0x35 => Instruction::new(OpCode::AND, AddressingMode::ZeroPageX, 4),
            0x36 => Instruction::new(OpCode::ROL, AddressingMode::ZeroPageX, 6),
            0x38 => Instruction::new(OpCode::SEC, AddressingMode::Implied, 2),
            0x39 => Instruction::new(OpCode::AND, AddressingMode::AbsoluteY, 4),
            0x3D => Instruction::new(OpCode::AND, AddressingMode::AbsoluteX, 4),
            0x3E => Instruction::new(OpCode::ROL, AddressingMode::AbsoluteX, 7),
            0x40 => Instruction::new(OpCode::RTI, AddressingMode::Implied, 6),
            0x41 => Instruction::new(OpCode::EOR, AddressingMode::IndirectIndexedX, 6),
            0x45 => Instruction::new(OpCode::EOR, AddressingMode::ZeroPage, 3),
            0x46 => Instruction::new(OpCode::LSR, AddressingMode::ZeroPage, 5),
            0x48 => Instruction::new(OpCode::PHA, AddressingMode::Implied, 3),
            0x49 => Instruction::new(OpCode::EOR, AddressingMode::Immediate, 2),
            0x4A => Instruction::new(OpCode::LSR, AddressingMode::Accumulator, 2),
            0x4C => Instruction::new(OpCode::JMP, AddressingMode::Absolute, 3),
            0x4D => Instruction::new(OpCode::EOR, AddressingMode::Absolute, 4),
            0x4E => Instruction::new(OpCode::LSR, AddressingMode::Absolute, 6),
            0x50 => Instruction::new(OpCode::BVC, AddressingMode::Relative, 2),
            0x51 => Instruction::new(OpCode::EOR, AddressingMode::YIndexedIndirect, 5),
            0x55 => Instruction::new(OpCode::EOR, AddressingMode::ZeroPageX, 4),
            0x56 => Instruction::new(OpCode::LSR, AddressingMode::ZeroPageX, 6),
            0x58 => Instruction::new(OpCode::CLI, AddressingMode::Implied, 2),
            0x59 => Instruction::new(OpCode::EOR, AddressingMode::AbsoluteY, 4),
            0x5D => Instruction::new(OpCode::EOR, AddressingMode::AbsoluteX, 4),
            0x5E => Instruction::new(OpCode::LSR, AddressingMode::AbsoluteX, 7),
            0x60 => Instruction::new(OpCode::RTS, AddressingMode::Implied, 6),
            0x61 => Instruction::new(OpCode::ADC, AddressingMode::IndirectIndexedX, 6),
            0x65 => Instruction::new(OpCode::ADC, AddressingMode::ZeroPage, 3),
            0x66 => Instruction::new(OpCode::ROR, AddressingMode::ZeroPage, 5),
            0x68 => Instruction::new(OpCode::PLA, AddressingMode::Implied, 4),
            0x69 => Instruction::new(OpCode::ADC, AddressingMode::Immediate, 2),
            0x6A => Instruction::new(OpCode::ROR, AddressingMode::Accumulator, 2),
            0x6C => Instruction::new(OpCode::JMP, AddressingMode::Indirect, 5),
            0x6D => Instruction::new(OpCode::ADC, AddressingMode::Absolute, 4),
            0x6E => Instruction::new(OpCode::ROR, AddressingMode::Absolute, 6),
            0x70 => Instruction::new(OpCode::BVS, AddressingMode::Relative, 2),
            0x71 => Instruction::new(OpCode::ADC, AddressingMode::YIndexedIndirect, 5),
            0x75 => Instruction::new(OpCode::ADC, AddressingMode::ZeroPageX, 4),
            0x76 => Instruction::new(OpCode::ROR, AddressingMode::ZeroPageX, 6),
            0x78 => Instruction::new(OpCode::SEI, AddressingMode::Implied, 2),
            0x79 => Instruction::new(OpCode::ADC, AddressingMode::AbsoluteY, 4),
            0x7D => Instruction::new(OpCode::ADC, AddressingMode::AbsoluteX, 4),
            0x7E => Instruction::new(OpCode::ROR, AddressingMode::AbsoluteX, 7),
            0x81 => Instruction::new(OpCode::STA, AddressingMode::IndirectIndexedX, 6),
            0x84 => Instruction::new(OpCode::STY, AddressingMode::ZeroPage, 3),
            0x85 => Instruction::new(OpCode::STA, AddressingMode::ZeroPage, 3),
            0x86 => Instruction::new(OpCode::STX, AddressingMode::ZeroPage, 3),
            0x88 => Instruction::new(OpCode::DEY, AddressingMode::Implied, 2),
            0x8A => Instruction::new(OpCode::TXA, AddressingMode::Implied, 2),
            0x8C => Instruction::new(OpCode::STY, AddressingMode::Absolute, 4),
            0x8D => Instruction::new(OpCode::STA, AddressingMode::Absolute, 4),
            0x8E => Instruction::new(OpCode::STX, AddressingMode::Absolute, 4),
            0x90 => Instruction::new(OpCode::BCC, AddressingMode::Relative, 2),
            0x91 => Instruction::new(OpCode::STA, AddressingMode::YIndexedIndirect, 6),
            0x94 => Instruction::new(OpCode::STY, AddressingMode::ZeroPageX, 4),
            0x95 => Instruction::new(OpCode::STA, AddressingMode::ZeroPageX, 4),
            0x96 => Instruction::new(OpCode::STX, AddressingMode::ZeroPageY, 4),
            0x98 => Instruction::new(OpCode::TYA, AddressingMode::Implied, 2),
            0x99 => Instruction::new(OpCode::STA, AddressingMode::AbsoluteY, 5),
            0x9A => Instruction::new(OpCode::TXS, AddressingMode::Implied, 2),
            0x9D => Instruction::new(OpCode::STA, AddressingMode::AbsoluteX, 5),
            0xA0 => Instruction::new(OpCode::LDY, AddressingMode::Immediate, 2),
            0xA1 => Instruction::new(OpCode::LDA, AddressingMode::IndirectIndexedX, 6),
            0xA2 => Instruction::new(OpCode::LDX, AddressingMode::Immediate, 2),
            0xA4 => Instruction::new(OpCode::LDY, AddressingMode::ZeroPage, 3),
            0xA5 => Instruction::new(OpCode::LDA, AddressingMode::ZeroPage, 3),
            0xA6 => Instruction::new(OpCode::LDX, AddressingMode::ZeroPage, 3),
            0xA8 => Instruction::new(OpCode::TAY, AddressingMode::Implied, 2),
            0xA9 => Instruction::new(OpCode::LDA, AddressingMode::Immediate, 2),
            0xAA => Instruction::new(OpCode::TAX, AddressingMode::Implied, 2),
            0xAC => Instruction::new(OpCode::LDY, AddressingMode::Absolute, 4),
            0xAD => Instruction::new(OpCode::LDA, AddressingMode::Absolute, 4),
            0xAE => Instruction::new(OpCode::LDX, AddressingMode::Absolute, 4),
            0xB0 => Instruction::new(OpCode::BCS, AddressingMode::Relative, 2),
            0xB1 => Instruction::new(OpCode::LDA, AddressingMode::YIndexedIndirect, 5),
            0xB4 => Instruction::new(OpCode::LDY, AddressingMode::ZeroPageX, 4),
            0xB5 => Instruction::new(OpCode::LDA, AddressingMode::ZeroPageX, 4),
            0xB6 => Instruction::new(OpCode::LDX, AddressingMode::ZeroPageY, 4),
            0xB8 => Instruction::new(OpCode::CLV, AddressingMode::Implied, 2),
            0xB9 => Instruction::new(OpCode::LDA, AddressingMode::AbsoluteY, 4),
            0xBA => Instruction::new(OpCode::TSX, AddressingMode::Implied, 2),
            0xBC => Instruction::new(OpCode::LDY, AddressingMode::AbsoluteX, 4),
            0xBD => Instruction::new(OpCode::LDA, AddressingMode::AbsoluteX, 4),
            0xBE => Instruction::new(OpCode::LDX, AddressingMode::AbsoluteY, 4),
            0xC0 => Instruction::new(OpCode::CPY, AddressingMode::Immediate, 2),
            0xC1 => Instruction::new(OpCode::CMP, AddressingMode::IndirectIndexedX, 6),
            0xC4 => Instruction::new(OpCode::CPY, AddressingMode::ZeroPage, 3),
            0xC5 => Instruction::new(OpCode::CMP, AddressingMode::ZeroPage, 3),
            0xC6 => Instruction::new(OpCode::DEC, AddressingMode::ZeroPage, 5),
            0xC8 => Instruction::new(OpCode::INY, AddressingMode::Implied, 2),
            0xC9 => Instruction::new(OpCode::CMP, AddressingMode::Immediate, 2),
            0xCA => Instruction::new(OpCode::DEX, AddressingMode::Implied, 2),
            0xCC => Instruction::new(OpCode::CPY, AddressingMode::Absolute, 4),
            0xCD => Instruction::new(OpCode::CMP, AddressingMode::Absolute, 4),
            0xCE => Instruction::new(OpCode::DEC, AddressingMode::Absolute, 6),
            0xD0 => Instruction::new(OpCode::BNE, AddressingMode::Relative, 2),
            0xD1 => Instruction::new(OpCode::CMP, AddressingMode::YIndexedIndirect, 5),
            0xD5 => Instruction::new(OpCode::CMP, AddressingMode::ZeroPageX, 4),
            0xD6 => Instruction::new(OpCode::DEC, AddressingMode::ZeroPageX, 6),
            0xD8 => Instruction::new(OpCode::CLD, AddressingMode::Implied, 2),
            0xD9 => Instruction::new(OpCode::CMP, AddressingMode::AbsoluteY, 4),
            0xDD => Instruction::new(OpCode::CMP, AddressingMode::AbsoluteX, 4),
            0xDE => Instruction::new(OpCode::DEC, AddressingMode::AbsoluteX, 7),
            0xE0 => Instruction::new(OpCode::CPX, AddressingMode::Immediate, 2),
            0xE1 => Instruction::new(OpCode::SBC, AddressingMode::IndirectIndexedX, 6),
            0xE4 => Instruction::new(OpCode::CPX, AddressingMode::ZeroPage, 3),
            0xE5 => Instruction::new(OpCode::SBC, AddressingMode::ZeroPage, 3),
            0xE6 => Instruction::new(OpCode::INC, AddressingMode::ZeroPage, 5),
            0xE8 => Instruction::new(OpCode::INX, AddressingMode::Implied, 2),
            0xE9 => Instruction::new(OpCode::SBC, AddressingMode::Immediate, 2),
            0xEA => Instruction::new(OpCode::NOP, AddressingMode::Implied, 2),
            0xEC => Instruction::new(OpCode::CPX, AddressingMode::Absolute, 4),
            0xED => Instruction::new(OpCode::SBC, AddressingMode::Absolute, 4),
            0xEE => Instruction::new(OpCode::INC, AddressingMode::Absolute, 6),
            0xF0 => Instruction::new(OpCode::BEQ, AddressingMode::Relative, 2),
            0xF1 => Instruction::new(OpCode::SBC, AddressingMode::YIndexedIndirect, 5),
            0xF5 => Instruction::new(OpCode::SBC, AddressingMode::ZeroPageX, 4),
            0xF6 => Instruction::new(OpCode::INC, AddressingMode::ZeroPageX, 6),
            0xF8 => Instruction::new(OpCode::SED, AddressingMode::Implied, 2),
            0xF9 => Instruction::new(OpCode::SBC, AddressingMode::AbsoluteY, 4),
            0xFD => Instruction::new(OpCode::SBC, AddressingMode::AbsoluteX, 4),
            0xFE => Instruction::new(OpCode::INC, AddressingMode::AbsoluteX, 7),
            _ => panic!(format!("[InstructionsMap::find] Unexpected op code: {:#04X}", op_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn find_instruction() {
        let map = InstructionsMap::new();
        let instruction = map.find(0x00);

        assert_that!(instruction, type_of::<Instruction>());
    }
}