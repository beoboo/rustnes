use crate::instruction::Instruction;
use crate::op_code::OpCode;

#[derive(Clone, Debug)]
pub struct InstructionsMap {}

impl InstructionsMap {
    pub fn new() -> InstructionsMap {
        InstructionsMap {}
    }

    pub fn find(&self, op_code: u8) -> Instruction {
        match &op_code {
            0x00 => Instruction::new(OpCode::BRK, "BRK", 1),
            0x69 => Instruction::new(OpCode::ADC, "ADC", 2),
            0xA9 => Instruction::new(OpCode::LDA, "LDA", 2),
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