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
            0x00 => Instruction::new(OpCode::BRK, 1),
            0x18 => Instruction::new(OpCode::CLC, 1),
            0x38 => Instruction::new(OpCode::SEC, 1),
            0x4C => Instruction::new(OpCode::JMP, 3),
            0x58 => Instruction::new(OpCode::CLI, 2),
            0x69 => Instruction::new(OpCode::ADC, 2),
            0x78 => Instruction::new(OpCode::SEI, 2),
            0x9A => Instruction::new(OpCode::TXS, 2),
            0xA2 => Instruction::new(OpCode::LDX, 2),
            0xA9 => Instruction::new(OpCode::LDA, 2),
            0xEA => Instruction::new(OpCode::NOP, 0),
            0xD8 => Instruction::new(OpCode::CLD, 1),
            0xF8 => Instruction::new(OpCode::SED, 1),
            0xE9 => Instruction::new(OpCode::SBC, 2),
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