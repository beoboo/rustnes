use crate::op_code::OpCode;

pub struct Instruction {
    pub op_code: OpCode,
    pub name: String,
    pub cycles: u8,
}

impl Instruction {
    pub fn new(op_code: OpCode, name: &str, cycles: u8) -> Instruction {
        Instruction {
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
        let instruction = Instruction::new(OpCode::LDA, "NOP", 1);

        assert_that!(instruction.op_code, equal_to(OpCode::LDA));
        assert_that!(instruction.name, equal_to("NOP"));
        assert_that!(instruction.cycles, equal_to(1));
    }
}