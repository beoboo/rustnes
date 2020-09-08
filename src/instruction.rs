use crate::op_code::OpCode;
use crate::types::Byte;

pub struct Instruction {
    pub id: Byte,
    pub op_code: OpCode,
    pub cycles: usize,
}

impl Instruction {
    pub fn new(id: Byte, op_code: OpCode, cycles: usize) -> Instruction {
        Instruction {
            id,
            op_code,
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
        let instruction = Instruction::new(0xEA, OpCode::NOP, 1);

        assert_that!(instruction.id, equal_to(0xEA));
        assert_that!(instruction.op_code, equal_to(OpCode::NOP));
        assert_that!(instruction.cycles, equal_to(1));
    }
}