use crate::types::Word;

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub address: Word,
    pub instruction: String,
}

impl<'a> Line {
    pub fn new(address: Word, instruction: &'a str) -> Self {
        Line {
            address,
            instruction: instruction.to_string(),
        }
    }
}