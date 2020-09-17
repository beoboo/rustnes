use std::iter::Peekable;
use std::str::Chars;

use crate::error::{Error, report_stage_error};
use crate::token::{Token, TokenType};
use crate::types::Word;
use crate::addressing_mode::AddressingMode;

pub struct Parser {}

type PeekableChar<'a> = Peekable<Chars<'a>>;

impl Parser {
    pub fn new() -> Parser {
        Parser {}
    }

    pub fn parse(&self, source: &str) -> Result<Vec<Token>, Error> {
        let mut it = source.chars().peekable();
        let mut line = 1;
        let mut tokens = Vec::new();

        loop {
            let ch = peek(&mut it);

            let token_type = match ch {
                '\0' => break,
                ' ' | '\t' => {
                    advance(&mut it);
                    continue;
                }
                '\n' => {
                    line += 1;
                    advance(&mut it);
                    continue;
                }
                '#' | '*' | '(' | '<' | '>' | '0'..='9' | '%' | '$' => address(&mut it),
                _ => if is_alphanum(peek(&mut it)) {
                    identifier(&mut it)
                } else {
                    return _report_error(format!("Invalid token: '{}'.", advance(&mut it)));
                }
            }?;

            tokens.push(build_token(token_type, line));
        }

        tokens.push(build_token(TokenType::EOF, line));
        Ok(tokens.clone())
    }
}

fn identifier(it: &mut PeekableChar) -> Result<TokenType, Error> {
    let mut keyword = String::new();
    while is_alphanum(peek(it)) {
        keyword.push(advance(it));
    }

    Ok(TokenType::Keyword(keyword))
}

fn address(it: &mut PeekableChar) -> Result<TokenType, Error> {
    let address_type = peek(it);

    match address_type {
        '#' => address_immediate(it),
        '*' => address_zero_page(it),
        '(' => address_indirect(it),
        _ => address_absolute(it),
    }
}

fn address_absolute(it: &mut PeekableChar) -> Result<TokenType, Error> {
    let address = number(it)?;
    let mut addressing_mode = AddressingMode::Absolute;

    if peek(it) == ',' {
        advance(it);
        match peek(it) {
            'X' => addressing_mode = AddressingMode::AbsoluteX,
            'Y' => addressing_mode = AddressingMode::AbsoluteY,
            c => return _report_error(format!("Invalid absolute indirect address: {}", c))
        }
        advance(it);
    }

    Ok(TokenType::Address(addressing_mode, address))
}

fn address_immediate(it: &mut PeekableChar) -> Result<TokenType, Error> {
    advance(it);

    Ok(TokenType::Address(AddressingMode::Immediate, number(it)?))
}

fn address_indirect(it: &mut PeekableChar) -> Result<TokenType, Error> {
    advance(it);

    let address = number(it)?;
    let mut addressing_mode = AddressingMode::Indirect;

    if peek(it) == ',' {
        advance(it);
        match peek(it) {
            'X' => addressing_mode = AddressingMode::IndirectIndexedX,
            c => return _report_error(format!("Invalid indexed indirect address: {}", c))
        }
        advance(it);
    }

    consume(')', it)?;

    if peek(it) == ',' {
        advance(it);
        match peek(it) {
            'Y' => addressing_mode = AddressingMode::YIndexedIndirect,
            c => return _report_error(format!("Invalid indirect indexed address: {}", c))
        }
        advance(it);
    }

    Ok(TokenType::Address(addressing_mode, address))
}

fn address_zero_page(it: &mut PeekableChar) -> Result<TokenType, Error> {
    advance(it);

    let address = number(it)?;
    let mut addressing_mode = AddressingMode::ZeroPage;

    if peek(it) == ',' {
        advance(it);
        match peek(it) {
            'X' => addressing_mode = AddressingMode::ZeroPageX,
            'Y' => addressing_mode = AddressingMode::ZeroPageY,
            c => return _report_error(format!("Invalid zero page indirect address: {}", c))
        }
        advance(it);
    }

    Ok(TokenType::Address(addressing_mode, address))
}

fn number(it: &mut Peekable<Chars>) -> Result<Word, Error> {
    let number_type = peek(it);

    match number_type {
        '<' => low(it),
        '>' => high(it),
        '$' => hex(it),
        '%' => binary(it),
        '0' => octal(it),
        _ => decimal(it),
    }
}

fn hex(it: &mut PeekableChar) -> Result<Word, Error> {
    advance(it);

    let mut number:Word = 0;

    while is_hex(peek(it)) {
        number *= 16;
        number += advance(it).to_digit(16).unwrap() as Word;
    }

    Ok(number)
}

fn low(it: &mut PeekableChar) -> Result<Word, Error> {
    advance(it);

    let number = number(it)?;

    Ok(number & 0x00FF)
}

fn high(it: &mut PeekableChar) -> Result<Word, Error> {
    advance(it);

    let number = number(it)?;

    Ok((number >> 8) & 0x00FF)
}

fn decimal(it: &mut PeekableChar) -> Result<Word, Error> {
    let mut number:Word = 0;

    while is_decimal(peek(it)) {
        number *= 10;
        number += advance(it).to_digit(10).unwrap() as Word;
    }

    Ok(number)
}

fn octal(it: &mut PeekableChar) -> Result<Word, Error> {
    advance(it);

    let mut number:Word = 0;

    while is_octal(peek(it)) {
        number *= 8;
        number += advance(it).to_digit(8).unwrap() as Word;
    }

    Ok(number)
}

fn binary(it: &mut PeekableChar) -> Result<Word, Error> {
    advance(it);

    let mut number:Word = 0;

    while is_binary(peek(it)) {
        number *= 2;
        number += advance(it).to_digit(2).unwrap() as Word;
    }

    Ok(number)
}

fn build_token(token_type: TokenType, line: u32) -> Token {
    Token::new(token_type, line)
}

fn is_alphanum(ch: char) -> bool {
    match ch {
        '0'..='9' | 'a'..='z' | 'A'..='Z' => true,
        _ => false
    }
}

fn is_hex(ch: char) -> bool {
    match ch {
        '0'..='9' | 'a'..='f' | 'A'..='F'=> true,
        _ => false
    }
}

fn is_decimal(ch: char) -> bool {
    match ch {
        '0'..='9' => true,
        _ => false
    }
}

fn is_octal(ch: char) -> bool {
    match ch {
        '0'..='7' => true,
        _ => false
    }
}

fn is_binary(ch: char) -> bool {
    match ch {
        '0' | '1'=> true,
        _ => false
    }
}

fn advance(it: &mut PeekableChar) -> char {
    match it.next() {
        Some(t) => t,
        None => '\0'
    }
}

fn consume(ch: char, it: &mut PeekableChar) -> Result<(), Error> {
    let next = peek(it);

    if next == ch {
        advance(it);
        Ok(())
    } else {
        _report_error(format!("Expected {}", ch))
    }
}

fn peek(it: &mut PeekableChar) -> char {
    match it.peek() {
        Some(t) => *t,
        None => '\0'
    }
}

fn _report_error<S: Into<String>, T>(err: S) -> Result<T, Error> {
    report_stage_error(err, "parser")
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use crate::addressing_mode::AddressingMode;

    use super::*;

    #[test]
    fn parse_one_byte_instruction() {
        let tokens = parse("BRK").unwrap();

        assert_token(&tokens[0], &TokenType::Keyword("BRK".to_string()), 1);
    }

    #[test]
    fn parse_multiple_instructions() {
        let tokens = parse("LDA $DEAD,X\nBRK").unwrap();

        assert_token(&tokens[0], &TokenType::Keyword("LDA".to_string()), 1);
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::AbsoluteX, 0xDEAD), 1);
        assert_token(&tokens[2], &TokenType::Keyword("BRK".to_string()), 2);
    }

    #[test]
    fn parse_two_bytes_instruction() {
        let tokens = parse("LDA 123").unwrap();

        assert_token(&tokens[0], &TokenType::Keyword("LDA".to_string()), 1);
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::Absolute, 123), 1);
    }

    #[test]
    fn parse_address_numbers() {
        let tokens = parse("LDA $DEAD").unwrap();
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::Absolute, 0xDEAD), 1);

        let tokens = parse("LDA 073").unwrap();
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::Absolute, 59), 1);

        let tokens = parse("LDA %10101010").unwrap();
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::Absolute, 170), 1);

        let tokens = parse("LDA <$DEAD").unwrap();
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::Absolute, 0x00AD), 1);

        let tokens = parse("LDA >$DEAD").unwrap();
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::Absolute, 0x00DE), 1);
    }

    #[test]
    fn parse_address() {
        let tokens = parse("LDA $DEAD").unwrap();
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::Absolute, 0xDEAD), 1);

        let tokens = parse("LDA $DEAD,X").unwrap();
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::AbsoluteX, 0xDEAD), 1);

        let tokens = parse("LDA $DEAD,Y").unwrap();
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::AbsoluteY, 0xDEAD), 1);

        let tokens = parse("LDA #$DEAD").unwrap();
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::Immediate, 0xDEAD), 1);

        let tokens = parse("LDA *$DEAD").unwrap();
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::ZeroPage, 0xDEAD), 1);

        let tokens = parse("LDA *$DEAD,X").unwrap();
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::ZeroPageX, 0xDEAD), 1);

        let tokens = parse("LDA *$DEAD,Y").unwrap();
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::ZeroPageY, 0xDEAD), 1);

        let tokens = parse("LDA ($DEAD)").unwrap();
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::Indirect, 0xDEAD), 1);

        let tokens = parse("LDA ($DEAD,X)").unwrap();
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::IndirectIndexedX, 0xDEAD), 1);

        let tokens = parse("LDA ($DEAD),Y").unwrap();
        assert_token(&tokens[1], &TokenType::Address(AddressingMode::YIndexedIndirect, 0xDEAD), 1);
    }

    fn assert_token(token: &Token, token_type: &TokenType, line: u32) {
        assert_that!(&token.token_type, equal_to(token_type));
        assert_that!(token.line, equal_to(line));
    }

    fn parse(source: &str) -> Result<Vec<Token>, Error> {
        let parser = Parser::new();

        parser.parse(source)
    }

}