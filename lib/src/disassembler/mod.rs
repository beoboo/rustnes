use std::collections::BTreeMap;
use std::slice::Iter;

use crate::disassembler::line::Line;
use crate::instructions::addressing_mode::AddressingMode;
use crate::instructions::Instructions;
use crate::types::{Byte, Word};

// use log::trace;
pub mod line;

pub type DisassembledCode = BTreeMap<Word, Line>;

#[derive(Clone, Debug, Default)]
pub struct Disassembler {}

fn decode(it: &mut Iter<Byte>) -> Byte {
    *(it.next().unwrap_or_else(|| panic!("Unexpected end of instructions")))
}

fn append_byte(source_code: String, it: &mut Iter<Byte>, prefix: &str, postfix: &str) -> (Byte, Word, String) {
    let address = decode(it) as Word;

    (1, address, source_code + format!("{}{:02X?}", prefix, address).as_str() + postfix)
}

fn append_word(source_code: String, it: &mut Iter<Byte>, prefix: &str, postfix: &str) -> (Byte, Word, String) {
    let lo = decode(it) as Word;
    let hi = decode(it) as Word;

    let address = (hi << 8) + lo;

    (2, address, source_code + format!("{}{:04X?}", prefix, address).as_str() + postfix)
}

impl Disassembler {
    pub fn disassemble(&mut self, source: &[Byte]) -> DisassembledCode {
        let map = Instructions::new();
        let mut instructions = DisassembledCode::new();

        let mut it = source.iter();
        let mut pc: Word = 0;

        while let Some(byte) = it.next() {
            let instruction = map.find(*byte);

            let source_code = format!("{:?}", instruction.op_code);

            let (bytes, address, source_code) = match instruction.addressing_mode {
                AddressingMode::Implied => { (0, 0, source_code) }
                AddressingMode::Accumulator => { (0, 0, source_code + " A") }
                AddressingMode::ZeroPage => { append_byte(source_code, &mut it, " $", "") }
                AddressingMode::ZeroPageX => { append_byte(source_code, &mut it, " $", ",X") }
                AddressingMode::ZeroPageY => { append_byte(source_code, &mut it, " $", ",Y") }
                AddressingMode::Relative => { append_byte(source_code, &mut it, " $", "") }
                AddressingMode::Absolute => { append_word(source_code, &mut it, " $", "") }
                AddressingMode::AbsoluteX => { append_word(source_code, &mut it, " $", ",X") }
                AddressingMode::AbsoluteY => { append_word(source_code, &mut it, " $", ",Y") }
                AddressingMode::Immediate => { append_byte(source_code, &mut it, " #$", "") }
                AddressingMode::Indirect => { append_word(source_code, &mut it, " ($", ")") }
                AddressingMode::IndirectIndexedX => { append_byte(source_code, &mut it, " ($", ",X)") }
                AddressingMode::YIndexedIndirect => { append_byte(source_code, &mut it, " ($", "),Y") }
            };

            // trace!("Decoding: {:#06X}: {:#04X} -> {} [{:?}]", count, *byte, source_code, instruction.addressing_mode);

            instructions.insert(pc, Line::new(source_code.as_str(), instruction.addressing_mode, address));
            pc += 1 + bytes as Word;
        }

        instructions
    }
}

#[cfg(test)]
mod tests {
    // use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use crate::assembler::Assembler;
    use crate::parser::Parser;

    use super::*;

    #[test]
    fn one_byte_instruction() {
        let instructions = disassemble("BRK");

        assert_line(&instructions[&0x0000], "BRK")
    }

    #[test]
    fn two_bytes_instruction() {
        let instructions = disassemble("LDA #1");

        assert_line(&instructions[&0x0000], "LDA #$01");
    }

    #[test]
    fn three_bytes_instruction() {
        let instructions = disassemble("LDA 1");

        assert_line(&instructions[&0x0000], "LDA $01");
    }

    #[test]
    fn addressing_modes() {
        let instructions = disassemble("LDA #$44\nLDA $44\nLDA $44,X\nLDA $4400\nLDA $4400,X\nLDA $4400,Y\nLDA ($44,X)\nLDA ($44),Y\nBPL $2\nSTX $2,Y\nASL A\nJMP ($1)");
        assert_line(&instructions[&0], "LDA #$44");
        assert_line(&instructions[&2], "LDA $44");
        assert_line(&instructions[&4], "LDA $44,X");
        assert_line(&instructions[&6], "LDA $4400");
        assert_line(&instructions[&9], "LDA $4400,X");
        assert_line(&instructions[&12], "LDA $4400,Y");
        assert_line(&instructions[&15], "LDA ($44,X)");
        assert_line(&instructions[&17], "LDA ($44),Y");
        assert_line(&instructions[&19], "BPL $02");
        assert_line(&instructions[&21], "STX $02,Y");
        assert_line(&instructions[&23], "ASL A");
        assert_line(&instructions[&24], "JMP ($0001)");
    }

    fn disassemble(source_code: &str) -> DisassembledCode {
        let mut disassembler = Disassembler::default();
        disassembler.disassemble(build_program(source_code).as_slice())
    }

    fn assert_line(line: &Line, instruction: &str) {
        assert_that!(line.instruction.as_str(), eq(instruction));
    }

    fn build_program(source: &str) -> Vec<Byte> {
        let assembler = Assembler::default();
        let parser = Parser::default();
        let tokens = parser.parse(source).unwrap();

        let program = match assembler.assemble(tokens) {
            Ok(program) => program,
            Err(e) => panic!("Assembler error: {}", e)
        };

        program.data
    }
}
