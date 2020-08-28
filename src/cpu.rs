use crate::instructions_map::InstructionsMap;
use crate::op_code::OpCode;
use std::io::Read;

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct CpuStatus {
    C: bool, // Carry
    Z: bool, // Zero
    I: bool, // Enable/Disable Interrupts
    D: bool, // Decimal Mode
    B: bool, // Break
    U: bool, // Unused
    V: bool, // Overflow
    S: bool, // Sign
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct Cpu {
    A: u8, // Accumulator
    X: u8, // X register
    Y: u8, // Y register
    PC: usize, // Program counter
    status: CpuStatus, // Status
    instructions_map: InstructionsMap,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            A: 0,
            X: 0,
            Y: 0,
            PC: 0,
            status: CpuStatus {
                C: false,
                Z: false,
                I: false,
                D: false,
                B: false,
                U: false,
                V: false,
                S: false,
            },
            instructions_map: InstructionsMap::new(),
        }
    }

    pub fn process(&mut self, program: &[u8]) -> u8 {
        let op_code = program[self.PC];

        let instruction = self.instructions_map.find(op_code);
        self.PC += 1;

        match instruction.op_code {
            OpCode::LDA => self.lda(program),
            _ => panic!(format!("Unexpected op code: {}", op_code))
        };

        0
    }

    fn lda(&mut self, program: &[u8]) {
        let operand = program[self.PC];
        self.PC += 1;
        self.A = operand;
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;
    use super::*;

    #[test]
    fn ctor() {
        let cpu = Cpu::new();

        assert_that!(cpu.A, eq(0));
        assert_that!(cpu.X, eq(0));
        assert_that!(cpu.Y, eq(0));
        assert_that!(cpu.PC, eq(0));
    }

    #[test]
    fn process_single_instruction() {
        let mut cpu = Cpu::new();

        cpu.process(&[0xA9, 0x01]);

        assert_that!(cpu.PC, eq(2));
        assert_that!(cpu.A, eq(1));
        assert_status(cpu.status, "zn");
    }

    fn assert_status(status: CpuStatus, flags: &str) {
        for flag in flags.chars() {
            match flag {
                'C' | 'c' => assert_flag(status.C, flag),
                'Z' | 'z' => assert_flag(status.Z, flag),
                'I' | 'i' => assert_flag(status.I, flag),
                'D' | 'd' => assert_flag(status.D, flag),
                'B' | 'b' => assert_flag(status.B, flag),
                'U' | 'u' => assert_flag(status.U, flag),
                'V' | 'v' => assert_flag(status.V, flag),
                'S' | 's' => assert_flag(status.S, flag),
                _ => {}
            }
        }
    }

    fn assert_flag(status: bool, flag: char) {
        assert_that!(status, is(flag.is_uppercase()));
    }
}