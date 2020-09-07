use crate::instruction::Instruction;
use crate::op_code::OpCode;

#[derive(Clone, Debug)]
pub struct InstructionsMap {}

impl InstructionsMap {
    pub fn new() -> InstructionsMap {
        InstructionsMap {}
    }

    pub fn find(&self, op_code: OpCode) -> Instruction {
        match &op_code {
            OpCode::BRK => Instruction::new(OpCode::BRK, "BRK", 1),
            OpCode::ADC => Instruction::new(OpCode::ADC, "ADC", 2),
            OpCode::CLC => Instruction::new(OpCode::CLC, "CLC", 1),
            OpCode::LDA => Instruction::new(OpCode::LDA, "LDA", 2),
            OpCode::SEC => Instruction::new(OpCode::SEC, "SEC", 1),
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
        let instruction = map.find(OpCode::BRK);

        assert_that!(instruction, type_of::<Instruction>());
    }
}