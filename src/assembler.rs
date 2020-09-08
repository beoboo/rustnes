struct Assembler {}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {}
    }

    pub fn assemble(tokens: Vec<Token>) -> Vec<u8> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn assemble_simple() {
        let assembler = Assembler::new();

        assert_that!(assembler.assemble(["BRK"]), eq(&[0x00]));
    }
}