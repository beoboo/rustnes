#[derive(Debug)]
pub struct Cpu {
    pc: u8,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            pc: 0
        }
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;
    use super::*;

    #[test]
    fn ctor() {
        let cpu = Cpu::new();

        assert_that!(cpu.pc, eq(0));
    }
    //
    // #[test]
    // fn process_single_instruction() {
    //     let cpu = Cpu::new();
    //
    //     let instructions = vec![Instruction::LDA, 1];
    //
    //     cpu.process(instructions);
    //     assert_that!(cpu.pc, eq(1));
    // }
}