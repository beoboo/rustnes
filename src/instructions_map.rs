use crate::instruction::Instruction;
use crate::op_code::OpCode;

#[derive(Debug)]
pub struct InstructionsMap {}

impl InstructionsMap {
    pub fn new() -> InstructionsMap {
        InstructionsMap {}
    }

    pub fn find(&self, op_code: u8) -> Instruction {
        match &op_code {
            0x00 => { println!("here"); Instruction::new(OpCode::BRK, "BRK") }
            0xA9 => { println!("here"); Instruction::new(OpCode::LDA, "LDA") }
            _ => panic!(format!("Invalid op code: {:x?}", op_code))
        }
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn find_instruction() {
        let map = InstructionsMap::new();
        let instruction = map.find(0x00);

        assert_that!(instruction, type_of::<Instruction>());
    }
}