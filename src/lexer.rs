use std::iter::Peekable;
use std::str::Chars;

use crate::error::{Error, report_stage_error};
use crate::token::{Token, TokenType};

pub struct Lexer {}

type PeekableChar<'a> = Peekable<Chars<'a>>;

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {}
    }

    pub fn lex(&self, source: &str) -> Result<Vec<Token>, Error> {
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
                '#' => address(&mut it),
                _ => if is_alphanum(peek(&mut it)) {
                    keyword(&mut it)
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

fn address(it: &mut PeekableChar) -> Result<TokenType, Error> {
    let mut id = String::new();

    let addressing_type = advance(it);

    while is_alphanum(peek(it)) {//} || is(',', peek(it)) {
        id.push(advance(it));
    }

    Ok(TokenType::ImmediateAddress(id.parse::<u8>().unwrap()))
}

fn keyword(it: &mut PeekableChar) -> Result<TokenType, Error> {
    let mut id = String::new();

    while is_alphanum(peek(it)) {
        id.push(advance(it));
    }

    Ok(TokenType::Keyword(id))
}

fn build_token(token_type: TokenType, line: u32) -> Token {
    Token::new(token_type, line)
}

fn is_alphanum(ch: char) -> bool {
    match ch {
        '0'..='9' | 'a'..='z' | 'A'..='Z' | '?' | '_' | '+' | '-' | '*' | '/' | '$' | '=' | '<' | '>' => true,
        _ => false
    }
}

fn advance(it: &mut PeekableChar) -> char {
    match it.next() {
        Some(t) => t,
        None => '\0'
    }
}

fn peek(it: &mut PeekableChar) -> char {
    match it.peek() {
        Some(t) => *t,
        None => '\0'
    }
}

fn _report_error<S: Into<String>, T>(err: S) -> Result<T, Error> {
    report_stage_error(err, "lexer")
}


#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use super::*;
    use crate::addressing::Addressing;

    #[test]
    fn parse_one_byte_instruction() {
        let tokens = lex("BRK").unwrap();

        assert_that!(tokens.len(), eq(2));
        assert_token(&tokens[0], &TokenType::Keyword("BRK".to_string()), 1);
    }

    #[test]
    fn parse_two_bytes_instruction() {
        let tokens = lex("LDA #01").unwrap();

        assert_that!(tokens.len(), eq(3));
        assert_token(&tokens[0], &TokenType::Keyword("LDA".to_string()), 1);
        assert_token(&tokens[1], &TokenType::ImmediateAddress(1), 1);
    }

    fn assert_token(token: &Token, token_type: &TokenType, line: u32) {
        assert_that!(&token.token_type, equal_to(token_type));
        assert_that!(token.line, equal_to(line));
    }


    fn lex(source: &str) -> Result<Vec<Token>, Error> {
        let lexer = Lexer::new();

        lexer.lex(source)
    }

}