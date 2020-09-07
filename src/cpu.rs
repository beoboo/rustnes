use crate::instructions_map::InstructionsMap;
use crate::op_code::OpCode;

#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct CpuStatus {
    C: bool, // Carry
    Z: bool, // Zero
    I: bool, // Enable/Disable Interrupts
    D: bool, // Decimal Mode
    B: bool, // Break
    U: bool, // Unused
    V: bool, // Overflow
    N: bool, // Negative
}

#[allow(non_snake_case)]
#[derive(Clone, Debug)]
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
                N: false,
            },
            instructions_map: InstructionsMap::new(),
        }
    }

    pub fn process(&mut self, program: &[u8]) -> u8 {
        let op_code = program[self.PC];

        let instruction = self.instructions_map.find(op_code);
        let mut cycles = instruction.cycles;

        self.PC += 1;

        cycles += match instruction.op_code {
            OpCode::ADC => self.adc(program),
            OpCode::LDA => self.lda(program),
            _ => panic!(format!("Unexpected op code: {:x?}", op_code))
        };

        cycles
    }

    fn adc(&mut self, program: &[u8]) -> u8 {
        let operand = program[self.PC];
        self.PC += 1;
        self.A += operand;
        0
    }

    fn lda(&mut self, program: &[u8]) -> u8 {
        let operand = program[self.PC];
        self.PC += 1;
        self.A = operand;
        self.status.Z = operand == 0x00;
        self.status.N = (operand & 0x80) == 0x80;

        0
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use super::*;

    // struct MockBus {
    //
    // }
    //
    // impl Bus for MockBus {
    //     fn read(address: u16) -> u8 {
    //
    //     }
    // }

    #[test]
    fn ctor() {
        let cpu = Cpu::new();

        assert_that!(cpu.A, eq(0));
        assert_that!(cpu.X, eq(0));
        assert_that!(cpu.Y, eq(0));
        assert_that!(cpu.PC, eq(0));
    }

    #[test]
    fn process_adc() {
        let mut cpu = Cpu::new();
        cpu.A = 1;

        assert_registers(&cpu, &[0x69, 0x01], 2, 0, 0, 2, "zncv", 2);
        // assert_registers(&cpu, &[0x69, 0xFF], 2, 0, 0, 2, "znCv", 2);
    }

    #[test]
    fn process_lda() {
        let cpu = Cpu::new();

        assert_registers(&cpu, &[0xA9, 0x00], 0x00, 0, 0, 2, "Zn", 2);
        assert_registers(&cpu, &[0xA9, 0x01], 0x01, 0, 0, 2, "zn", 2);
        assert_registers(&cpu, &[0xA9, 0xFF], 0xFF, 0, 0, 2, "zN", 2);
    }

    // #[test]
    // fn process_sta() {
    //     let mut bus = Bus::new(0x0000, 0x02ff);
    //     let mut cpu = Cpu::new(&mut bus);
    //     cpu.A = 1;
    //
    //     assert_registers(&cpu, &[0x8D, 0x00, 0x02], 1, 0, 0, 2, "zn", 1);
    //     assert_bus(&bus, 0x2000, 0x01);
    // }

    fn assert_registers(cpu: &Cpu, program: &[u8], a: u8, x: u8, y: u8, pc: usize, status: &str, expected_cycles: u8) {
        println!("{:x?}", program);
        let cpu = &mut cpu.clone();

        let cycles = cpu.process(program);

        assert_that!(cpu.A, eq(a));
        assert_that!(cpu.X, eq(x));
        assert_that!(cpu.Y, eq(y));
        assert_that!(cpu.PC, eq(pc));
        assert_status(cpu.status.clone(), status);
        assert_that!(cycles, geq(expected_cycles));
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
                'N' | 'n' => assert_flag(status.N, flag),
                _ => panic!(format!("Undefined flag: {}", flag))
            }
        }
    }

    fn assert_flag(status: bool, flag: char) {
        println!("{}: {}", flag, status);
        assert_that!(status, is(flag.is_uppercase()));
    }
}