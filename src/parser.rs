use crate::token::Token;

struct Parser {}

impl Parser {
    pub fn new() -> Parser {
        Parser {}
    }

    pub fn parse(&self, instructions: Vec<Token>) -> Vec<u8> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn parse_empty() {
        // let parser = Parser::new();
        //
        // assert_that!(parser.parse(vec![Token::Identifier("BRK".to_string())]), eq(vec![0x00]));
    }
}