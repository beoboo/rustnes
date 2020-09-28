use crate::addressing_mode::AddressingMode;
use crate::cpu::op_code::OpCode;

pub struct Instruction {
    pub op_code: OpCode,
    pub addressing_mode: AddressingMode,
    pub cycles: usize,
}

impl Instruction {
    pub fn new(op_code: OpCode, addressing_mode: AddressingMode, cycles: usize) -> Instruction {
        Instruction {
            op_code,
            addressing_mode,
            cycles
        }
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn simple_instruction() {
        let instruction = Instruction::new(OpCode::NOP, AddressingMode::Implied, 1);

        assert_that!(instruction.op_code, equal_to(OpCode::NOP));
        assert_that!(instruction.addressing_mode, equal_to(AddressingMode::Implied));
        assert_that!(instruction.cycles, equal_to(1));
    }
}