pub struct Instruction {
    op_code: u8,
    name: String,
}

impl Instruction {
    pub fn new(op_code: u8, name: &str) -> Instruction {
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
        let instruction = Instruction::new(0x69, "NOP");

        assert_that!(instruction.op_code, equal_to(0x69));
        assert_that!(instruction.name, equal_to("NOP"));
    }
}