use crate::instructions_map::InstructionsMap;
use crate::op_code::OpCode;
use std::io::Read;

#[derive(Debug)]
pub struct Cpu {
    a: u8, // Accumulator
    x: u8, // X register
    y: u8, // Y register
    pc: usize, // Program counter
    instructions_map: InstructionsMap,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            instructions_map: InstructionsMap::new(),
        }
    }

    pub fn process(&mut self, program: &[u8]) -> u8 {
        let op_code = program[self.pc];

        let instruction = self.instructions_map.find(op_code);
        self.pc += 1;

        match instruction.op_code {
            OpCode::LDA => self.lda(program),
            _ => panic!(format!("Unexpected op code: {}", op_code))
        };

        0
    }

    fn lda(&mut self, program: &[u8]) {
        let operand = program[self.pc];
        self.pc += 1;
        self.a = operand;
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;
    use super::*;

    #[test]
    fn ctor() {
        let cpu = Cpu::new();

        assert_that!(cpu.a, eq(0));
        assert_that!(cpu.x, eq(0));
        assert_that!(cpu.y, eq(0));
        assert_that!(cpu.pc, eq(0));
    }

    #[test]
    fn process_single_instruction() {
        let mut cpu = Cpu::new();

        cpu.process(&[0xA9, 0x01]);

        assert_that!(cpu.pc, eq(2));
        assert_that!(cpu.a, eq(1));
    }
}