use std::collections::HashMap;

use crate::addressing_mode::AddressingMode;
use crate::assembler::instruction::Instruction;
use crate::error::Error;
use crate::types::Byte;

type Map = HashMap<String, Instruction>;


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddressingModeMap {
    map: Map
}

pub fn insert(map: &mut Map, op_code: &str, implied: bool, relative: bool, addressing_mode: AddressingMode, value: Byte) {
    let instruction = map.entry(op_code.to_string()).or_insert(Instruction::new(op_code, implied, relative));

    instruction.modes.insert(addressing_mode, value);
}

impl AddressingModeMap {
    pub fn new() -> AddressingModeMap {
        let mut map = Map::new();

        // ADC
        insert(&mut map, "ADC", false, false, AddressingMode::Immediate, 0x69);
        insert(&mut map, "ADC", false, false, AddressingMode::ZeroPage, 0x65);
        insert(&mut map, "ADC", false, false, AddressingMode::ZeroPageX, 0x75);
        insert(&mut map, "ADC", false, false, AddressingMode::Absolute, 0x6D);
        insert(&mut map, "ADC", false, false, AddressingMode::AbsoluteX, 0x7D);
        insert(&mut map, "ADC", false, false, AddressingMode::AbsoluteY, 0x79);
        insert(&mut map, "ADC", false, false, AddressingMode::IndirectIndexedX, 0x61);
        insert(&mut map, "ADC", false, false, AddressingMode::YIndexedIndirect, 0x71);

        // AND
        insert(&mut map, "AND", false, false, AddressingMode::Immediate, 0x29);
        insert(&mut map, "AND", false, false, AddressingMode::ZeroPage, 0x25);
        insert(&mut map, "AND", false, false, AddressingMode::ZeroPageX, 0x35);
        insert(&mut map, "AND", false, false, AddressingMode::Absolute, 0x2D);
        insert(&mut map, "AND", false, false, AddressingMode::AbsoluteX, 0x3D);
        insert(&mut map, "AND", false, false, AddressingMode::AbsoluteY, 0x39);
        insert(&mut map, "AND", false, false, AddressingMode::IndirectIndexedX, 0x21);
        insert(&mut map, "AND", false, false, AddressingMode::YIndexedIndirect, 0x31);

        // ASL
        insert(&mut map, "ASL", false, false, AddressingMode::Accumulator, 0x0A);
        insert(&mut map, "ASL", false, false, AddressingMode::ZeroPage, 0x06);
        insert(&mut map, "ASL", false, false, AddressingMode::ZeroPageX, 0x16);
        insert(&mut map, "ASL", false, false, AddressingMode::Absolute, 0x0E);
        insert(&mut map, "ASL", false, false, AddressingMode::AbsoluteX, 0x1E);

        insert(&mut map, "BCC", false, true, AddressingMode::Relative, 0x90);
        insert(&mut map, "BCS", false, true, AddressingMode::Relative, 0xB0);
        insert(&mut map, "BEQ", false, true, AddressingMode::Relative, 0xF0);
        insert(&mut map, "BIT", false, false, AddressingMode::Absolute, 0x2C);
        insert(&mut map, "BMI", false, true, AddressingMode::Relative, 0x30);
        insert(&mut map, "BNE", false, true, AddressingMode::Relative, 0xD0);
        insert(&mut map, "BPL", false, true, AddressingMode::Relative, 0x10);
        insert(&mut map, "BRK", true, false, AddressingMode::Implied, 0x00);
        insert(&mut map, "CLC", true, false, AddressingMode::Implied, 0x18);
        insert(&mut map, "CLD", true, false, AddressingMode::Implied, 0xD8);
        insert(&mut map, "CLI", true, false, AddressingMode::Implied, 0x58);
        insert(&mut map, "CMP", false, false, AddressingMode::Immediate, 0xC9);
        insert(&mut map, "CPX", false, false, AddressingMode::Immediate, 0xE0);
        insert(&mut map, "CPX", false, false, AddressingMode::Absolute, 0xEC);
        insert(&mut map, "DEC", false, false, AddressingMode::ZeroPage, 0xC6);
        insert(&mut map, "DEX", true, false, AddressingMode::Implied, 0xCA);
        insert(&mut map, "DEY", true, false, AddressingMode::Implied, 0x88);
        insert(&mut map, "INC", false, false, AddressingMode::ZeroPage, 0xE6);
        insert(&mut map, "INC", false, false, AddressingMode::Absolute, 0xEE);
        insert(&mut map, "INX", true, false, AddressingMode::Implied, 0xE8);
        insert(&mut map, "INY", true, false, AddressingMode::Implied, 0xC8);
        insert(&mut map, "JMP", false, false, AddressingMode::Absolute, 0x4C);
        insert(&mut map, "JMP", false, false, AddressingMode::Indirect, 0x6C);
        insert(&mut map, "JSR", false, false, AddressingMode::Absolute, 0x20);
        insert(&mut map, "LDA", false, false, AddressingMode::Absolute, 0xAD);
        insert(&mut map, "LDA", false, false, AddressingMode::AbsoluteX, 0xBD);
        insert(&mut map, "LDA", false, false, AddressingMode::Immediate, 0xA9);
        insert(&mut map, "LDA", false, false, AddressingMode::YIndexedIndirect, 0xB1);
        insert(&mut map, "LDA", false, false, AddressingMode::ZeroPage, 0xA5);
        insert(&mut map, "LDX", false, false, AddressingMode::Immediate, 0xA2);
        insert(&mut map, "LDX", false, false, AddressingMode::ZeroPage, 0xA6);
        insert(&mut map, "LDY", false, false, AddressingMode::Immediate, 0xA0);
        insert(&mut map, "LSR", false, false, AddressingMode::Accumulator, 0x4A);
        insert(&mut map, "ORA", false, false, AddressingMode::Immediate, 0x09);
        insert(&mut map, "ORA", false, false, AddressingMode::ZeroPage, 0x0D);
        insert(&mut map, "PHA", true, false, AddressingMode::Implied, 0x48);
        insert(&mut map, "PLA", true, false, AddressingMode::Implied, 0x68);
        insert(&mut map, "ROL", false, false, AddressingMode::Accumulator, 0x2A);
        insert(&mut map, "ROR", false, false, AddressingMode::Accumulator, 0x6A);
        insert(&mut map, "RTS", true, false, AddressingMode::Implied, 0x60);
        insert(&mut map, "SBC", false, false, AddressingMode::Immediate, 0xE9);
        insert(&mut map, "SBC", false, false, AddressingMode::ZeroPageX, 0xF5);
        insert(&mut map, "SEC", true, false, AddressingMode::Implied, 0x38);
        insert(&mut map, "SED", true, false, AddressingMode::Implied, 0xF8);
        insert(&mut map, "SEI", true, false, AddressingMode::Implied, 0x78);
        insert(&mut map, "STA", false, false, AddressingMode::Absolute, 0x8D);
        insert(&mut map, "STA", false, false, AddressingMode::AbsoluteX, 0x9D);
        insert(&mut map, "STA", false, false, AddressingMode::ZeroPage, 0x85);
        insert(&mut map, "STA", false, false, AddressingMode::ZeroPageX, 0x95);
        insert(&mut map, "STX", false, false, AddressingMode::Absolute, 0x8E);
        insert(&mut map, "STX", false, false, AddressingMode::ZeroPage, 0x86);
        insert(&mut map, "STY", false, false, AddressingMode::Absolute, 0x8C);
        insert(&mut map, "STY", false, false, AddressingMode::ZeroPage, 0x84);
        insert(&mut map, "TAX", true, false, AddressingMode::Implied, 0xAA);
        insert(&mut map, "TXA", true, false, AddressingMode::Implied, 0x98);
        insert(&mut map, "TSX", true, false, AddressingMode::Implied, 0xBA);
        insert(&mut map, "TXS", true, false, AddressingMode::Implied, 0x9A);

        AddressingModeMap {
            map
        }
    }

    pub fn find(&self, op_code: &str) -> Result<Instruction, Error> {
        if !self.map.contains_key(op_code) {
            return Err(Error::UnknownOpCode(op_code.to_string()));
        }

        Ok(self.map[op_code].clone())
    }
}


#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn find_brk() {
        assert_instruction("BRK", true);
        assert_instruction("LDA", false);
    }

    #[test]
    fn find_unknown_opcode() {
        let map = AddressingModeMap::new();

        let error = map.find("UNK").unwrap_err();

        assert_that!(error, equal_to(Error::UnknownOpCode("UNK".to_string())));
    }

    fn assert_instruction(op_code: &str, implied: bool) {
        let map = AddressingModeMap::new();
        let instruction = map.find(op_code).unwrap();

        assert_that!(instruction.op_code, equal_to(op_code.to_string()));
        assert_that!(instruction.implied, equal_to(implied));
    }
}