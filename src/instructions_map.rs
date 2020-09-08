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
            OpCode::JMP => Instruction::new(OpCode::JMP, "JMP", 3),
            OpCode::LDA => Instruction::new(OpCode::LDA, "LDA", 2),
            OpCode::NOP => Instruction::new(OpCode::NOP, "NOP", 0),
            OpCode::SBC => Instruction::new(OpCode::SBC, "SBC", 2),
            OpCode::SEC => Instruction::new(OpCode::SEC, "SEC", 1),
            _ => panic!(format!("Unexpected op code: {:#X?}", op_code))
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