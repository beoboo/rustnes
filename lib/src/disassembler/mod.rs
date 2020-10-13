use std::slice::Iter;

// use log::trace;

use crate::disassembler::assembly::Assembly;
use crate::instructions::addressing_mode::AddressingMode;
use crate::instructions::Instructions;
use crate::types::{Byte, Word};

pub mod assembly;
pub mod assembly_line;


#[derive(Clone, Debug, Default)]
pub struct Disassembler {}

fn decode(it: &mut Iter<Byte>) -> Result<Word, String> {
    match it.next() {
        Some(b) => Ok(*b as Word),
        None => Err("Unexpected end of instructions at position".to_string())
    }
}

fn append_byte(source_code: String, it: &mut Iter<Byte>, prefix: &str, postfix: &str) -> Result<(Byte, Word, String), String> {
    let address = decode(it)?;

    Ok((1, address, source_code + format!("{}{:02X?}", prefix, address).as_str() + postfix))
}

fn append_word(source_code: String, it: &mut Iter<Byte>, prefix: &str, postfix: &str) -> Result<(Byte, Word, String), String> {
    let lo = decode(it)?;
    let hi = decode(it)?;

    let address = (hi << 8) + lo;

    Ok((2, address, source_code + format!("{}{:04X?}", prefix, address).as_str() + postfix))
}

impl Disassembler {
    pub fn disassemble(&self, source: &[Byte], starting_address: Word) -> Assembly {
        let map = Instructions::default();
        let mut assembly = Assembly::default();

        let mut it = source.iter();
        let mut pc = starting_address;

        while let Some(byte) = it.next() {
            let instruction = map.find(*byte);

            let source_code = format!("{:?}", instruction.op_code);

            let decoded = match instruction.addressing_mode {
                AddressingMode::Implied => { Ok((0, 0, source_code)) }
                AddressingMode::Accumulator => { Ok((0, 0, source_code + " A")) }
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

            match decoded {
                Ok((bytes, address, source_code)) => {
                    assembly.add(pc, source_code.as_str(), instruction.addressing_mode, address);
                    pc = pc.wrapping_add(1 + bytes as Word);
                }
                Err(e) => panic!("{} at position {:#06X}, while disassembling {:?}", e, pc, instruction.op_code)
            }

            // trace!("Decoding: {:#06X}: {:#04X} -> {:?} [{:?}]", pc, *byte, instruction.op_code, instruction.addressing_mode);
            // println!("Decoding: {:#06X}: {:#04X} -> {:?} [{:?}]", pc, *byte, instruction.op_code, instruction.addressing_mode);
        }

        assembly
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use crate::assembler::Assembler;
    use crate::disassembler::assembly_line::AssemblyLine;
    use crate::parser::Parser;

    use super::*;

    #[test]
    fn one_byte_instruction() {
        let instructions = disassemble("BRK", 0);

        assert_line(&instructions.at(0x0000), "BRK")
    }

    #[test]
    fn two_bytes_instruction() {
        let instructions = disassemble("LDA #1", 0);

        assert_line(&instructions.at(0x0000), "LDA #$01");
    }

    #[test]
    fn three_bytes_instruction() {
        let instructions = disassemble("LDA 1", 0);

        assert_line(&instructions.at(0x0000), "LDA $01");
    }

    #[test]
    fn addressing_modes() {
        let instructions = disassemble("LDA #$44\nLDA $44\nLDA $44,X\nLDA $4400\nLDA $4400,X\nLDA $4400,Y\nLDA ($44,X)\nLDA ($44),Y\nBPL $2\nSTX $2,Y\nASL A\nJMP ($1)", 0);
        assert_line(&instructions.at(0), "LDA #$44");
        assert_line(&instructions.at(2), "LDA $44");
        assert_line(&instructions.at(4), "LDA $44,X");
        assert_line(&instructions.at(6), "LDA $4400");
        assert_line(&instructions.at(9), "LDA $4400,X");
        assert_line(&instructions.at(12), "LDA $4400,Y");
        assert_line(&instructions.at(15), "LDA ($44,X)");
        assert_line(&instructions.at(17), "LDA ($44),Y");
        assert_line(&instructions.at(19), "BPL $02");
        assert_line(&instructions.at(21), "STX $02,Y");
        assert_line(&instructions.at(23), "ASL A");
        assert_line(&instructions.at(24), "JMP ($0001)");
    }

    #[test]
    fn disassemble_range() {
        let instructions = disassemble("LDA #$44\nLDA $44\nLDA $44,X\nLDA $4400\nLDA $4400,X\nLDA $4400,Y\nLDA ($44,X)\nLDA ($44),Y\nBPL $2\nSTX $2,Y\nASL A\nJMP ($1)", 0);
        let previous = instructions.before(0x0004, 10);
        assert_that!(previous.len(), eq(2));
        assert_line(&previous[0], "LDA #$44");
        assert_line(&previous[1], "LDA $44");

        let current = instructions.at(0x0004);
        assert_line(&current, "LDA $44,X");

        let next = instructions.after(0x0004, 10);
        assert_that!(next.len(), eq(9));
        assert_line(&next[0], "LDA $4400");
    }

    #[test]
    fn starting_address() {
        let instructions = disassemble("LDA #$44", 0x8000);
        let current = instructions.at(0x8000);
        assert_line(&current, "LDA #$44");
    }

    fn disassemble(source_code: &str, starting_address: Word) -> Assembly {
        let disassembler = Disassembler::default();
        disassembler.disassemble(build_program(source_code).as_slice(), starting_address)
    }

    fn assert_line(line: &AssemblyLine, instruction: &str) {
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
