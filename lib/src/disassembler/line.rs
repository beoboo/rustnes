use crate::types::Word;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line<'a> {
    address: Word,
    instruction: &'a str,
}

impl<'a> Line<'a> {
    pub fn new(address: Word, instruction: &'a str) -> Self {
        Line {
            address,
            instruction,
        }
    }
}