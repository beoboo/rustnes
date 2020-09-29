use std::slice::Iter;

use crate::disassembler::line::Line;
use crate::instructions::addressing_mode::AddressingMode;
use crate::instructions::Instructions;
use crate::types::{Byte, Word};

mod line;

#[derive(Debug, Default)]
pub struct Disassembler {
    instructions: Vec<Line>
}

fn decode(it: &mut Iter<Byte>) -> Byte {
    *(it.next().unwrap_or_else(|| panic!("Unexpected end of instructions")))
}

fn append_byte(source_code: String, it: &mut Iter<Byte>, prefix: &str, postfix: &str) -> String {
    source_code + format!("{}{:02X?}", prefix, decode(it)).as_str() + postfix
}

fn append_word(source_code: String, it: &mut Iter<Byte>, prefix: &str, postfix: &str) -> String {
    let lo = decode(it) as Word;
    let hi = decode(it) as Word;

    source_code + format!("{}{:04X?}", prefix, (hi << 8) + lo).as_str() + postfix
}

impl Disassembler {
    pub fn disassemble(&mut self, source: &[Byte]) {
        let map = Instructions::new();
        let mut instructions = vec![];

        let mut it = source.iter();
        let mut line = 0x00;

        while let Some(byte) = it.next() {
            let instruction = map.find(*byte);

            let mut source_code = format!("{:?}", instruction.op_code);

            match instruction.addressing_mode {
                AddressingMode::Implied => {}
                AddressingMode::Accumulator => { source_code += " A" }
                AddressingMode::ZeroPage => { source_code = append_byte(source_code, &mut it, " $", ""); }
                AddressingMode::ZeroPageX => { source_code = append_byte(source_code, &mut it, " $", ",X"); }
                AddressingMode::ZeroPageY => { source_code = append_byte(source_code, &mut it, " $", ",Y"); }
                AddressingMode::Relative => { source_code = append_byte(source_code, &mut it, " $", ""); }
                AddressingMode::Absolute => { source_code = append_word(source_code, &mut it, " $", ""); }
                AddressingMode::AbsoluteX => { source_code = append_word(source_code, &mut it, " $", ",X"); }
                AddressingMode::AbsoluteY => { source_code = append_word(source_code, &mut it, " $", ",Y"); }
                AddressingMode::Immediate => { source_code = append_byte(source_code, &mut it, " #$", ""); }
                AddressingMode::Indirect => { source_code = append_word(source_code, &mut it, " ($", ")"); }
                AddressingMode::IndirectIndexedX => { source_code = append_byte(source_code, &mut it, " ($", ",X)"); }
                AddressingMode::YIndexedIndirect => { source_code = append_byte(source_code, &mut it, " ($", "),Y"); }
            }

            instructions.push(Line::new(line, source_code.as_str()));

            line += 1;
        }

        self.instructions = instructions
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

        assert_line(&instructions[0], 0x0000, "BRK")
    }

    #[test]
    fn two_bytes_instruction() {
        let instructions = disassemble("LDA #1");

        assert_line(&instructions[0], 0x0000, "LDA #$01");
    }

    #[test]
    fn three_bytes_instruction() {
        let instructions = disassemble("LDA 1");

        assert_line(&instructions[0], 0x0000, "LDA $01");
    }

    #[test]
    fn addressing_modes() {
        env_logger::init();

        let instructions = disassemble("LDA #$44\nLDA $44\nLDA $44,X\nLDA $4400\nLDA $4400,X\nLDA $4400,Y\nLDA ($44,X)\nLDA ($44),Y\nBPL $2\nSTX $2,Y\nASL A\nJMP ($1)");
        assert_line(&instructions[0], 0x0000, "LDA #$44");
        assert_line(&instructions[1], 0x0001, "LDA $44");
        assert_line(&instructions[2], 0x0002, "LDA $44,X");
        assert_line(&instructions[3], 0x0003, "LDA $4400");
        assert_line(&instructions[4], 0x0004, "LDA $4400,X");
        assert_line(&instructions[5], 0x0005, "LDA $4400,Y");
        assert_line(&instructions[6], 0x0006, "LDA ($44,X)");
        assert_line(&instructions[7], 0x0007, "LDA ($44),Y");
        assert_line(&instructions[8], 0x0008, "BPL $02");
        assert_line(&instructions[9], 0x0009, "STX $02,Y");
        assert_line(&instructions[10], 0x000A, "ASL A");
        assert_line(&instructions[11], 0x000B, "JMP ($0001)");
    }

    fn disassemble(source_code: &str) -> Vec<Line> {
        let mut disassembler = Disassembler::default();
        disassembler.disassemble(build_program(source_code).as_slice());
        disassembler.instructions
    }

    fn assert_line(line: &Line, address: Word, instruction: &str) {
        assert_that!(line.address, eq(address));
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
