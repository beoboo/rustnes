use std::convert::TryFrom;

use crate::instructions_map::InstructionsMap;
use crate::op_code::OpCode;
use crate::types::*;
use crate::bus::Bus as BusTrait;

fn bool_to_byte(flag: bool) -> Byte {
    if flag { 1 } else { 0 }
}

#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct CpuStatus {
    C: bool,
    // Carry
    Z: bool,
    // Zero
    I: bool,
    // Enable/Disable Interrupts
    D: bool,
    // Decimal Mode
    B: bool,
    // Break
    U: bool,
    // Unused
    V: bool,
    // Overflow
    N: bool, // Negative
}

#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct Cpu {
    A: Byte,
    // Accumulator
    X: Byte,
    // X register
    Y: Byte,
    // Y register
    PC: Word,
    // Program counter
    status: CpuStatus,
    // Status
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

    pub fn process<Bus: BusTrait>(&mut self, bus: &Bus) -> usize {
        let op_id = bus.read(self.PC);

        let instruction = self.instructions_map.find(op_id);
        let mut cycles = instruction.cycles;

        self.PC += if instruction.op_code != OpCode::NOP { 1 } else { 0 };

        println!("Next op code: {:?}", instruction.op_code);

        cycles += match instruction.op_code {
            OpCode::ADC => self.adc(bus),
            OpCode::CLC => self.clc(),
            OpCode::JMP => self.jmp(bus),
            OpCode::LDA => self.lda(bus),
            OpCode::SBC => self.sbc(bus),
            OpCode::SEC => self.sec(),
            OpCode::NOP => 0,
            op_code => panic!(format!("[Cpu::process] Unexpected op code: {:?}", op_code))
        };

        cycles
    }

    fn adc<Bus: BusTrait>(&mut self, bus: &Bus) -> usize {
        let operand = bus.read(self.PC);
        let computed = self.A as Word + operand as Word + bool_to_byte(self.status.C) as Word;
        let acc = self.A;

        println!("Operand: {}, computed: {}, acc: {}", operand, computed, acc);

        self.PC += 1;
        self.A = computed as Byte;
        self.status.Z = self.A == 0x00;
        self.status.V = (!((acc ^ operand) & 0x80) != 0) && (((acc ^ (computed as Byte)) & 0x80) != 0);
        self.status.C = computed > 0xFF;
        self.status.N = (self.A & 0x80) == 0x80;

        0
    }

    fn clc(&mut self) -> usize {
        self.status.C = false;

        0
    }

    fn jmp<Bus: BusTrait>(&mut self, bus: &Bus) -> usize {
        let low = bus.read(self.PC) as Word;
        let high = bus.read(self.PC + 1) as Word;
        println!("Jumping to 0x{:X?}{:X?}", high, low);

        self.PC = (high << 8) + low;

        0
    }

    fn lda<Bus: BusTrait>(&mut self, bus: &Bus) -> usize {
        let operand = bus.read(self.PC);
        self.PC += 1;
        self.A = operand;
        self.status.Z = operand == 0x00;
        self.status.N = (operand & 0x80) == 0x80;

        0
    }

    fn sbc<Bus: BusTrait>(&mut self, bus: &Bus) -> usize {
        let operand = bus.read(self.PC);
        let computed = self.A as SignedWord - operand as SignedWord - bool_to_byte(!self.status.C) as SignedWord;
        let acc = self.A;

        println!("Operand: {}, computed: {}, acc: {}", operand, computed, acc);

        self.PC += 1;
        self.A = computed as Byte;
        self.status.Z = self.A == 0x00;
        self.status.V = (!((acc ^ operand) & 0x80) != 0) && (((acc ^ (computed as Byte)) & 0x80) != 0);
        self.status.C = computed >= 0;
        self.status.N = (self.A & 0x80) == 0x80;

        0
    }

    fn sec(&mut self) -> usize {
        self.status.C = true;

        0
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use crate::rom::Rom;

    use super::*;
    use crate::bus::BusImpl;

    struct MockBus {
        data: Vec<u8>,
    }

    impl MockBus {
        fn new(data: Vec<u8>) -> MockBus {
            MockBus {
                data
            }
        }
    }

    impl BusTrait for MockBus {
        fn read(&self, address: Word) -> Byte {
            self.data[address as usize]
        }

        fn write(&mut self, address: u16, data: u8) {
            self.data[address as usize] = data;
        }
    }

    #[test]
    fn ctor() {
        let cpu = Cpu::new();

        assert_that!(cpu.A, eq(0));
        assert_that!(cpu.X, eq(0));
        assert_that!(cpu.Y, eq(0));
        assert_that!(cpu.PC, eq(0));
    }

    #[test]
    fn process_all() {
        let rom = Rom::load("roms/nestest.nes", 16384, 8192);
        let mut cpu = build_cpu(0, 0, 0, 0, "");
        let bus = BusImpl::new(rom);
        println!("FFFC: {:#X}, FFFD: {:#X}", bus.read(0xFFFC), bus.read(0xFFFD));

        let cycles = process_bus(&mut cpu, &bus);
        assert_that!(cycles, geq(1234));
    }

    #[test]
    fn process_adc() {
        let cpu = build_cpu(1, 0, 0, 0, "");

        assert_registers(&cpu, &[0x69, 0x01], 2, 0, 0, 2, "zncv", 2);
        assert_registers(&cpu, &[0x69, 0x01], 2, 0, 0, 2, "zncv", 2);
    }

    #[test]
    fn process_adc_flags() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        // 1 + 1 = 2, C = 0, V = 0
        assert_status_flags(&cpu, &[0x18, 0xA9, 0x01, 0x69, 0x01], 2, 0, 0, "zncv");
        // 1 + -1 = 0, C = 1, V = 0
        assert_status_flags(&cpu, &[0x18, 0xA9, 0x01, 0x69, 0xFF], 0, 0, 0, "ZnCv");
        // 127 + 1 = 128 (-128), C = 0, V = 1
        assert_status_flags(&cpu, &[0x18, 0xA9, 0x7F, 0x69, 0x01], 128, 0, 0, "zNcV");
        // -128 + -1 = -129 (127), C = 0, V = 1
        assert_status_flags(&cpu, &[0x18, 0xA9, 0x80, 0x69, 0xFF], 127, 0, 0, "znCV");
    }

    #[test]
    fn process_clc() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_status_flags(&cpu, &[0x18], 0, 0, 0, "c");
        assert_status_flags(&cpu, &[0x38, 0x18], 0, 0, 0, "c");
    }

    #[test]
    fn process_lda() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_registers(&cpu, &[0xA9, 0x00], 0x00, 0, 0, 2, "Zn", 2);
        assert_registers(&cpu, &[0xA9, 0x01], 0x01, 0, 0, 2, "zn", 2);
        assert_registers(&cpu, &[0xA9, 0xFF], 0xFF, 0, 0, 2, "zN", 2);
    }

    #[test]
    fn process_nop() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_registers(&cpu, &[], 0, 0, 0, 0, "zncv", 0);
    }

    #[test]
    fn process_jmp() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_registers(&cpu, &[0x4C, 0x03, 0x00], 0, 0, 0, 0x0003, "", 3);
    }

    #[test]
    fn process_sbc() {
        let cpu = build_cpu(1, 0, 0, 0, "C");

        assert_registers(&cpu, &[0xE9, 0x01], 0, 0, 0, 2, "ZnCv", 2);
    }

    #[test]
    fn process_sbc_flags() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        // 0 - 1 = -1 (255), C = 1, V = 1
        assert_status_flags(&cpu, &[0x38, 0xA9, 0x00, 0xE9, 0x01], 255, 0, 0, "zNcV");
        // -128 - 1 = -129 (127), C = 1, V = 1
        assert_status_flags(&cpu, &[0x38, 0xA9, 0x80, 0xE9, 0x01], 127, 0, 0, "znCV");
        // 127 - -1 = 128 (-128), C = 0, V = 1
        assert_status_flags(&cpu, &[0x38, 0xA9, 0x7F, 0xE9, 0xFF], -128i8 as u8, 0, 0, "zNcV");
    }

    #[test]
    fn process_sec() {
        let cpu = build_cpu(0, 0, 0, 0, "C");

        assert_status_flags(&cpu, &[0x38], 0, 0, 0, "C");
        assert_status_flags(&cpu, &[0x18, 0x38], 0, 0, 0, "C");
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

    fn build_cpu(a: Byte, x: Byte, y: Byte, pc: Word, status: &str) -> Cpu {
        let mut cpu = Cpu::new();

        cpu.A = a;
        cpu.X = x;
        cpu.Y = y;
        cpu.PC = pc;
        cpu.status = build_status(status);

        cpu
    }

    fn build_status(flags: &str) -> CpuStatus {
        CpuStatus {
            C: build_status_flag(flags, 'C'),
            Z: build_status_flag(flags, 'Z'),
            I: build_status_flag(flags, 'I'),
            D: build_status_flag(flags, 'D'),
            B: build_status_flag(flags, 'B'),
            U: build_status_flag(flags, 'U'),
            V: build_status_flag(flags, 'V'),
            N: build_status_flag(flags, 'N'),
        }
    }

    fn build_status_flag(flags: &str, flag: char) -> bool {
        flags.contains(flag)
    }

    fn assert_registers(cpu: &Cpu, program: &[Byte], a: Byte, x: Byte, y: Byte, pc: Word, expected_status: &str, expected_cycles: usize) {
        println!("Program: {:x?}", program);
        let cpu = &mut cpu.clone();

        let total_cycles = process(cpu, program);
        // let total_cycles = process(cpu, program);

        assert_that!(cpu.A, eq(a));
        assert_that!(cpu.X, eq(x));
        assert_that!(cpu.Y, eq(y));
        assert_that!(cpu.PC, eq(pc));
        assert_status(cpu.status.clone(), expected_status);
        assert_that!(total_cycles, geq(expected_cycles));
    }

    fn assert_status_flags(cpu: &Cpu, program: &[Byte], a: Byte, x: Byte, y: Byte, expected_status: &str) {
        println!("Program: {:x?}", program);
        let cpu = &mut cpu.clone();

        process(cpu, program);

        assert_that!(cpu.A, eq(a));
        assert_that!(cpu.X, eq(x));
        assert_that!(cpu.Y, eq(y));
        assert_status(cpu.status.clone(), expected_status);
    }

    fn process(cpu: &mut Cpu, program: &[Byte]) -> usize {
        let mut program = program.to_vec();
        program.push(0xEA);

        let bus = MockBus::new(program);

        process_bus(cpu, &bus)
    }

    fn process_bus<Bus: BusTrait>(cpu: &mut Cpu, bus: &Bus) -> usize {
        let mut cycles = cpu.process(bus);
        let mut total_cycles = cycles;

        while cycles != 0 {
            println!("A: {:#04X}", cpu.A);
            println!("X: {:#04X}", cpu.X);
            println!("Y: {:#04X}", cpu.Y);
            println!("PC: {:#06X}", cpu.PC);
            cycles = cpu.process(bus);
            total_cycles += cycles;
        }
        total_cycles
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