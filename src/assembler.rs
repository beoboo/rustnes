use std::iter::Peekable;
use std::slice::Iter;

use crate::error::{Error, report_stage_error};
use crate::token::{Token, TokenType};
use crate::types::Byte;

struct Assembler {}

type IterToken<'a> = Iter<'a, Token>;
type PeekableToken<'a> = Peekable<IterToken<'a>>;
type Instructions = Vec<Byte>;

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {}
    }

    pub fn assemble(&self, tokens: Vec<Token>) -> Result<Instructions, Error> {
        let mut it = tokens.iter().peekable();
        let mut instructions = vec![];

        loop {
            if peek(&mut it) == TokenType::EOF {
                break;
            }

            instruction(&mut instructions, &mut it)?;
        }

        Ok(instructions)
    }
}

fn instruction(instructions: &mut Vec<Byte>, it: &mut PeekableToken) -> Result<(), Error> {
    match advance(it)?.token_type {
        TokenType::Keyword(k) => keyword(k, instructions, it),

        t => Err(Error::Assembler(format!("Undefined token type: '{:?}'", t)))
    }
}

fn keyword(k: String, instructions: &mut Instructions, it: &mut PeekableToken) -> Result<(), Error> {
    match k.as_str() {
        "BRK" => instructions.push(0x00),
        "LDA" => return lda(instructions, it),
        _ => return _report_error(format!("Undefined instruction: '{}'", k))
    }

    Ok(())
}

fn lda(instructions: &mut Instructions, it: &mut PeekableToken) -> Result<(), Error> {
    let addressing = advance(it)?;

    match addressing.token_type {
        TokenType::ImmediateAddress(address) => { instructions.push(0xA9); instructions.push(address); }
        _ => return _report_error(format!("Invalid addressing: '{:?}'", addressing.token_type))
    }

    Ok(())
}


pub fn advance(it: &mut PeekableToken) -> Result<Token, Error> {
    match it.next() {
        Some(token) => {
            Ok(token.clone())
        }
        None => Err(Error::Parser(format!("Token not found.")))
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

    use crate::lexer::Lexer;

    use super::*;

    #[test]
    fn assemble_brk() {
        assert_assemble("BRK", &[0x00]);
    }


    #[test]
    fn assemble_lda() {
        assert_assemble("LDA #01", &[0xA9, 0x01]);
    }

    fn assert_assemble(source: &str, expected: &[Byte]) {
        let instructions = assemble(source);

        assert_that!(instructions.as_slice(), eq(expected));
    }

    fn assemble(source: &str) -> Vec<Byte> {
        let assembler = Assembler::new();
        let lexer = Lexer::new();
        let tokens = lexer.lex(source).unwrap();

        assembler.assemble(tokens).unwrap()
    }
}