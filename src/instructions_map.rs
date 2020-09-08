use crate::instruction::Instruction;
use crate::op_code::OpCode;
use crate::types::Byte;

#[derive(Clone, Debug)]
pub struct InstructionsMap {}

impl InstructionsMap {
    pub fn new() -> InstructionsMap {
        InstructionsMap {}
    }

    pub fn find(&self, op_id: Byte) -> Instruction {
        match &op_id {
            0x00 => Instruction::new(0x00, OpCode::BRK, 1),
            0x18 => Instruction::new(0x18, OpCode::CLC, 1),
            0x38 => Instruction::new(0x38, OpCode::SEC, 1),
            0x4C => Instruction::new(0x4C, OpCode::JMP, 3),
            0x58 => Instruction::new(0x58, OpCode::CLI, 2),
            0x69 => Instruction::new(0x69, OpCode::ADC, 2),
            0x78 => Instruction::new(0x78, OpCode::SEI, 2),
            0xA9 => Instruction::new(0xA9, OpCode::LDA, 2),
            0xEA => Instruction::new(0xEA, OpCode::NOP, 0),
            0xE9 => Instruction::new(0xE9, OpCode::SBC, 2),
            _ => panic!(format!("Unexpected op code: {:#04X}", op_id))
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