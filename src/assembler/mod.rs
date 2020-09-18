use std::iter::Peekable;
use std::slice::Iter;

use crate::addressing_mode::AddressingMode;
use crate::assembler::addressing_mode_map::AddressingModeMap;
use crate::error::{Error, report_stage_error};
use crate::token::{Token, TokenType};
use crate::types::{Byte, Word};

mod instruction;
mod addressing_mode_map;

pub struct Assembler {
    map: AddressingModeMap
}

#[derive(Clone, Debug)]
pub struct Instructions {
    pub(crate) data: Vec<Byte>
}

impl Instructions {
    fn new() -> Instructions {
        Instructions {
            data: vec![]
        }
    }

    fn push_byte(&mut self, byte: Byte) {
        self.data.push(byte);
    }

    fn push_word(&mut self, word: Word) {
        self.data.push(word as Byte);
        self.data.push((word >> 8) as Byte);
    }
}

type IterToken<'a> = Iter<'a, Token>;
type PeekableToken<'a> = Peekable<IterToken<'a>>;

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
            map: AddressingModeMap::new()
        }
    }

    pub fn assemble(&self, tokens: Vec<Token>) -> Result<Instructions, Error> {
        let mut it = tokens.iter().peekable();
        let mut instructions = Instructions::new();

        loop {
            if peek(&mut it) == TokenType::EOF {
                break;
            }

            self.instruction(&mut instructions, &mut it)?;
        }

        Ok(instructions)
    }

    fn instruction(&self, instructions: &mut Instructions, it: &mut PeekableToken) -> Result<(), Error> {
        match advance(it)?.token_type {
            TokenType::Identifier(k) => self.keyword(k, instructions, it),

            t => Err(Error::Assembler(format!("[Assembler::instruction] Undefined token type: '{:?}'", t)))
        }
    }

    fn keyword(&self, k: String, instructions: &mut Instructions, it: &mut PeekableToken) -> Result<(), Error> {
        let instruction = match self.map.find(k.as_str()) {
            Ok(i) => i,
            Err(e) => return _report_error(format!("[Assembler::keyword] {}", e))
        };

        if instruction.implied {
            println!("Implied");
            instructions.push_byte(instruction.find(AddressingMode::Implied).unwrap())
        } else {
            let address = advance(it)?;

            match address.token_type {
                TokenType::Identifier(a) => {
                    match a.as_str() {
                        "A" => {
                            let op_code = instruction.find(AddressingMode::Accumulator)?;
                            instructions.push_byte(op_code);
                        }
                        _ => return _report_error(format!("[Assembler::keyword] Expected 'A' for accumulator address"))
                    }
                },
                TokenType::Address(mode, address) => {
                    println!("{:?}", mode);
                    let mode = if instruction.relative { AddressingMode::Relative } else { mode };
                    let mode = if mode == AddressingMode::Absolute && address <= 0xFF && instruction.contains(AddressingMode::ZeroPage) {
                        AddressingMode::ZeroPage
                    } else {
                        mode
                    };

                    let op_code = instruction.find(mode.clone())?;

                    match mode {
                        AddressingMode::Absolute | AddressingMode::AbsoluteX => {
                            instructions.push_byte(op_code);
                            instructions.push_word(address);
                        }
                        AddressingMode::Immediate | AddressingMode::YIndexedIndirect => {
                            instructions.push_byte(op_code);
                            instructions.push_byte(address as Byte);
                        }
                        AddressingMode::Relative | AddressingMode::ZeroPage => {
                            instructions.push_byte(op_code);
                            instructions.push_byte(address as Byte);
                        }
                        _ => return _report_error(format!("[Assembler::keyword] Invalid addressing mode: '{:?}'", mode))
                    }
                }
                t => return _report_error(format!("[Assembler::keyword] Expected address token for '{}' instruction (found {:?})", instruction.op_code, t))
            }
        }

        Ok(())
    }
}

pub fn advance(it: &mut PeekableToken) -> Result<Token, Error> {
    match it.next() {
        Some(token) => {
            Ok(token.clone())
        }
        None => Err(Error::Assembler(format!("Token not found.")))
    }
}

pub fn peek(it: &mut PeekableToken) -> TokenType {
    match it.peek() {
        Some(token) => token.token_type.clone(),
        None => TokenType::EOF,
    }
}

fn _report_error<S: Into<String>, T>(err: S) -> Result<T, Error> {
    report_stage_error(err, "assembler")
}


#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use crate::parser::Parser;

    use super::*;

    #[test]
    fn assemble_brk() {
        assert_assemble("BRK", &[0x00]);
    }

    #[test]
    fn assemble_adc() {
        assert_assemble("ADC #$1", &[0x69, 0x01]);
    }

    #[test]
    fn assemble_bpl() {
        assert_assemble("BPL $1", &[0x10, 0x01]);
    }

    #[test]
    fn assemble_lda() {
        assert_assemble("LDA #$1", &[0xA9, 0x01]);
    }

    fn assert_assemble(source: &str, expected: &[Byte]) {
        let instructions = assemble(source);

        assert_that!(instructions.as_slice(), eq(expected));
    }

    fn assemble(source: &str) -> Vec<Byte> {
        let assembler = Assembler::new();
        let parser = Parser::new();
        let tokens = parser.parse(source).unwrap();

        assembler.assemble(tokens).unwrap().data
    }
}