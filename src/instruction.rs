use crate::op_code::OpCode;
use crate::types::Byte;

pub struct Instruction {
    pub id: Byte,
    pub op_code: OpCode,
    pub name: String,
    pub cycles: usize,
}

impl Instruction {
    pub fn new(id: Byte, op_code: OpCode, name: &str, cycles: usize) -> Instruction {
        Instruction {
            id,
            op_code,
            name: name.to_string(),
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
        let instruction = Instruction::new(0xEA, OpCode::LDA, "NOP", 1);

        assert_that!(instruction.id, equal_to(0xEA));
        assert_that!(instruction.op_code, equal_to(OpCode::LDA));
        assert_that!(instruction.name, equal_to("NOP"));
        assert_that!(instruction.cycles, equal_to(1));
    }
}