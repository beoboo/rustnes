// use std::iter::Peekable;
// use std::str::Chars;
//
// use crate::error::{Error, report_stage_error};
// use crate::token::{Token, TokenType};
//
// pub struct Lexer {}
//
// type PeekableChar<'a> = Peekable<Chars<'a>>;
//
// impl Lexer {
//     pub fn new() -> Lexer {
//         Lexer {}
//     }
//
//     pub fn lex(&self, source: &str) -> Result<Vec<Token>, Error> {
//         let mut it = source.chars().peekable();
//         let mut line = 1;
//         let mut tokens = Vec::new();
//
//         loop {
//             let ch = peek(&mut it);
//
//             let token_type = match ch {
//                 '\0' => break,
//                 ' ' | '\t' => {
//                     advance(&mut it);
//                     continue;
//                 }
//                 '\n' => {
//                     line += 1;
//                     advance(&mut it);
//                     continue;
//                 }
//                 '#' | '*' | '<' | '>' => {
//                     tokens.push(build_token(TokenType::AddressOperator(ch), line));
//                     number(&mut it)
//                 },
//                 '(' => {
//                     tokens.push(build_token(TokenType::AddressOperator(ch), line));
//                     number(&mut it)
//                 },
//                 '0'..='9' | '%' | '$' => number(&mut it),
//                 _ => if is_alphanum(peek(&mut it)) {
//                     identifier(&mut it)
//                 } else {
//                     return _report_error(format!("Invalid token: '{}'.", advance(&mut it)));
//                 }
//             }?;
//
//             tokens.push(build_token(token_type, line));
//         }
//
//         tokens.push(build_token(TokenType::EOF, line));
//         Ok(tokens.clone())
//     }
// }
//
// fn address_operator(it: &mut PeekableChar, operator: char) -> Result<TokenType, Error> {
//     advance(it);
//
//     Ok(TokenType::AddressOperator(operator))
// }
//
// fn identifier(it: &mut PeekableChar) -> Result<TokenType, Error> {
//     let mut id = String::new();
//
//     while is_alphanum(peek(it)) {
//         id.push(advance(it));
//     }
//
//     Ok(TokenType::Identifier(id))
// }
//
// fn number(it: &mut PeekableChar) -> Result<TokenType, Error> {
//     let mut id = String::new();
//
//     id.push(advance(it));
//
//     while is_digit(peek(it)) {
//         id.push(advance(it));
//     }
//
//     Ok(TokenType::Number(id))
// }
//
// fn build_token(token_type: TokenType, line: u32) -> Token {
//     Token::new(token_type, line)
// }
//
// fn is_alphanum(ch: char) -> bool {
//     match ch {
//         '0'..='9' | 'a'..='z' | 'A'..='Z' => true,
//         _ => false
//     }
// }
//
// fn is_digit(ch: char) -> bool {
//     match ch {
//         '0'..='9' | 'a'..='f' | 'A'..='F'=> true,
//         _ => false
//     }
// }
//
// fn advance(it: &mut PeekableChar) -> char {
//     match it.next() {
//         Some(t) => t,
//         None => '\0'
//     }
// }
//
// fn peek(it: &mut PeekableChar) -> char {
//     match it.peek() {
//         Some(t) => *t,
//         None => '\0'
//     }
// }
//
// fn _report_error<S: Into<String>, T>(err: S) -> Result<T, Error> {
//     report_stage_error(err, "lexer")
// }
//
// #[cfg(test)]
// mod tests {
//     use hamcrest2::prelude::*;
//
//     use super::*;
//     use crate::addressing_mode::Addressing;
//
//     #[test]
//     fn parse_tokens() {
//         let tokens = lex("LDA #*<>() 0123456789 $DEADBEEF,X %0101").unwrap();
//
//         // assert_that!(tokens.len(), eq(10));
//         assert_token(&tokens[0], &TokenType::Identifier("LDA".to_string()), 1);
//         assert_token(&tokens[1], &TokenType::AddressOperator('#'), 1);
//         assert_token(&tokens[2], &TokenType::AddressOperator('*'), 1);
//         assert_token(&tokens[3], &TokenType::AddressOperator('<'), 1);
//         assert_token(&tokens[4], &TokenType::AddressOperator('>'), 1);
//         assert_token(&tokens[5], &TokenType::Parens('('), 1);
//         assert_token(&tokens[6], &TokenType::Parens(')'), 1);
//         assert_token(&tokens[7], &TokenType::Number("0123456789".to_string()), 1);
//         assert_token(&tokens[8], &TokenType::Number("$DEADBEEF".to_string()), 1);
//         assert_token(&tokens[9], &TokenType::Number("%0101".to_string()), 1);
//     }
//     //
//     // #[test]
//     // fn parse_numbers() {
//     //     let tokens = lex("LDA 0123456789 $DEADBEEF %0101 ,").unwrap();
//     //
//     //     // assert_that!(tokens.len(), eq(10));
//     //     assert_token(&tokens[0], &TokenType::Identifier("LDA".to_string()), 1);
//     //     assert_token(&tokens[1], &TokenType::AddressOperator('#'), 1);
//     //     assert_token(&tokens[2], &TokenType::AddressOperator('*'), 1);
//     //     assert_token(&tokens[3], &TokenType::AddressOperator('<'), 1);
//     //     assert_token(&tokens[4], &TokenType::AddressOperator('>'), 1);
//     //     assert_token(&tokens[5], &TokenType::Number("0123456789".to_string()), 1);
//     //     assert_token(&tokens[6], &TokenType::Number("$DEADBEEF".to_string()), 1);
//     //     assert_token(&tokens[7], &TokenType::Number("%0101".to_string()), 1);
//     // }
//
//     #[test]
//     fn parse_keywords() {
//         assert_keyword("ADC", &TokenType::Identifier("ADC".to_string()));
//         assert_keyword("CLC", &TokenType::Identifier("CLC".to_string()));
//         assert_keyword("LDA", &TokenType::Identifier("LDA".to_string()));
//     }
//     //
//     // #[test]
//     // fn parse_numbers() {
//     //     assert_addressing("OPC #$FF",  &TokenType::ImmediateAddress(255));     // Hex
//     //     assert_addressing("OPC %010",  &TokenType::ImmediateAddress(2));       // Binary
//     //     assert_addressing("OPC #077",  &TokenType::ImmediateAddress(63));      // Octal
//     //     assert_addressing("OPC #1",    &TokenType::ImmediateAddress(1));       // Decimal
//     //     assert_addressing("OPC <1000", &TokenType::ImmediateAddress(0x0003));  // Low byte
//     //     assert_addressing("OPC >1000", &TokenType::ImmediateAddress(0x00E8));  // High byte
//     // }
//     //
//     // #[test]
//     // fn parse_address() {
//     //     assert_addressing("OPC A",    &TokenType::AccumulatorAddress);
//     //     assert_addressing("OPC #1",    &TokenType::ImmediateAddress(1));
//     //     assert_addressing("OPC 1234",    &TokenType::AbsoluteAddress(1234));
//     //     assert_addressing("OPC 1234,X",    &TokenType::AbsoluteXAddress(1234));
//     //     assert_addressing("OPC 1234,Y",    &TokenType::AbsoluteYAddress(1234));
//     //     assert_addressing("OPC 1234,Y",    &TokenType::AbsoluteYAddress(1234));
//     //     assert_addressing("OPC *1234",    &TokenType::ZeroPageAddress(1234));
//     //     assert_addressing("OPC *1234,X",    &TokenType::ZeroPageXAddress(1234));
//     //     assert_addressing("OPC *1234,Y",    &TokenType::ZeroPageYAddress(1234));
//     // }
//
//     fn assert_keyword(source: &str, token_type: &TokenType) {
//         let tokens = lex(source).unwrap();
//
//         assert_token(&tokens[0], token_type, 1);
//     }
//
//     fn assert_addressing(source: &str, token_type: &TokenType) {
//         let tokens = lex(source).unwrap();
//
//         assert_token(&tokens[1], token_type, 1);
//     }
//
//     fn assert_token(token: &Token, token_type: &TokenType, line: u32) {
//         assert_that!(&token.token_type, equal_to(token_type));
//         assert_that!(token.line, equal_to(line));
//     }
//
//
//     fn lex(source: &str) -> Result<Vec<Token>, Error> {
//         let lexer = Lexer::new();
//
//         lexer.lex(source)
//     }
//
// }