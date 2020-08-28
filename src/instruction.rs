use crate::op_code::OpCode;

pub struct Instruction {
    pub op_code: OpCode,
    pub name: String,
}

impl Instruction {
    pub fn new(op_code: OpCode, name: &str) -> Instruction {
        Instruction {
            op_code,
            name: name.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn simple_instruction() {
        let instruction = Instruction::new(OpCode::LDA, "NOP");

        assert_that!(instruction.op_code, equal_to(OpCode::LDA));
        assert_that!(instruction.name, equal_to("NOP"));
    }
}