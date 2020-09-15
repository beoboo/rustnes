use crate::op_code::OpCode;

pub struct Instruction {
    pub op_code: OpCode,
    pub cycles: usize,
}

impl Instruction {
    pub fn new(op_code: OpCode, cycles: usize) -> Instruction {
        Instruction {
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
        let instruction = Instruction::new(OpCode::NOP, 1);

        assert_that!(instruction.op_code, equal_to(OpCode::NOP));
        assert_that!(instruction.cycles, equal_to(1));
    }
}