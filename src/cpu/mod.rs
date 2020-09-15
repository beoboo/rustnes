use crate::addressing_mode::AddressingMode;
use crate::bus::Bus as BusTrait;
use crate::cpu::instructions_map::InstructionsMap;
use crate::op_code::OpCode;
use crate::types::*;

mod instruction;
mod instructions_map;

fn bool_to_byte(flag: bool) -> Byte {
    if flag { 1 } else { 0 }
}

#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct CpuStatus {
    // Carry
    C: bool,
    // Zero
    Z: bool,
    // Enable/Disable Interrupts
    I: bool,
    // Decimal Mode
    D: bool,
    // Break
    B: bool,
    // Unused
    U: bool,
    // Overflow
    V: bool,
    // Negative
    N: bool,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct Cpu {
    // Accumulator
    A: Byte,
    // X register
    X: Byte,
    // Y register
    Y: Byte,
    // Program counter
    PC: Word,
    // Stack pointer
    SP: Byte,
    // Status
    status: CpuStatus,
    instructions_map: InstructionsMap,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            A: 0,
            X: 0,
            Y: 0,
            SP: 0,
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

    pub fn process<Bus: BusTrait>(&mut self, bus: &mut Bus) -> usize {
        let op_id = bus.read_byte(self.PC);

        let instruction = self.instructions_map.find(op_id);
        let mut cycles = instruction.cycles;

        if instruction.op_code == OpCode::NOP {
            println!("NOP, exiting!");
            return 0;
        }

        self.PC += 1;

        println!("Next op code: {:?}", instruction.op_code);
        // self.debug();

        let address = self.read_address(bus, &instruction.addressing_mode);

        cycles += match instruction.op_code {
            OpCode::ADC => self.adc(address),
            OpCode::BPL => self.bpl(address),
            OpCode::CLC => self.clc(),
            OpCode::CLD => self.cld(),
            OpCode::CLI => self.cli(),
            OpCode::JMP => self.jmp(address),
            OpCode::LDA => self.lda(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::LDX => self.ldx(address),
            OpCode::SBC => self.sbc(address),
            OpCode::SEC => self.sec(),
            OpCode::SED => self.sed(),
            OpCode::SEI => self.sei(),
            OpCode::STA => self.sta(address, bus),
            OpCode::TXS => self.txs(),
            OpCode::NOP => 0,
            op_code => panic!(format!("[Cpu::process] Unexpected op code: {:?}", op_code))
        };
        // self.debug();

        cycles
    }

    fn read_address<Bus: BusTrait>(&mut self, bus: &Bus, addressing_mode: &AddressingMode) -> Word {
        match addressing_mode {
            AddressingMode::Absolute => {
                let address = bus.read_word(self.PC);
                self.PC += 2;
                address
            }
            AddressingMode::Immediate => {
                let operand = bus.read_byte(self.PC);
                self.PC += 1;
                operand as Word
            }
            AddressingMode::Implied => 0,
            AddressingMode::Relative => {
                let address = bus.read_byte(self.PC);
                self.PC += 1;
                address as Word
            }
            _ => panic!(format!("[Cpu::read_byte] Unexpected addressing mode: {:?}", addressing_mode))
        }
    }

    fn read_operand<Bus: BusTrait>(&self, operand: Word, bus: &Bus, addressing_mode: &AddressingMode) -> Byte {
        match addressing_mode {
            AddressingMode::Immediate => operand as Byte,
            AddressingMode::Absolute => bus.read_byte(operand),
            _ => panic!(format!("[Cpu::read_byte] Unexpected addressing mode: {:?}", addressing_mode)),
        }
    }

    fn adc(&mut self, operand: Word) -> usize {
        let computed = self.A as Word + operand + bool_to_byte(self.status.C) as Word;
        let acc = self.A;

        println!("Operand: {}, computed: {}, acc: {}", operand, computed, acc);

        self.A = computed as Byte;
        self.status.Z = self.A == 0x00;
        self.status.V = (!((acc ^ operand as Byte) & 0x80) != 0) && (((acc ^ (computed as Byte)) & 0x80) != 0);
        self.status.C = computed > 0xFF;
        self.status.N = (self.A & 0x80) == 0x80;

        0
    }

    fn bpl(&mut self, address: Word) -> usize {
        if !self.status.N {
            self.PC = address;
            1
        } else {
            0
        }
    }

    fn clc(&mut self) -> usize {
        self.status.C = false;

        0
    }

    fn cld(&mut self) -> usize {
        self.status.D = false;

        0
    }

    fn cli(&mut self) -> usize {
        self.status.I = false;

        0
    }

    fn jmp(&mut self, address: Word) -> usize {
        println!("Jumping to {:#06X}", address);

        self.PC = address;

        0
    }

    fn lda(&mut self, operand: Byte) -> usize {
        self.A = operand;
        self.status.Z = self.A == 0x00;
        self.status.N = (self.A & 0x80) == 0x80;

        0
    }

    fn ldx(&mut self, operand: Word) -> usize {
        self.X = operand as Byte;
        self.status.Z = self.X == 0x00;
        self.status.N = (self.X & 0x80) == 0x80;

        0
    }

    fn sbc(&mut self, operand: Word) -> usize {
        let computed = self.A as SignedWord - operand as SignedWord - bool_to_byte(!self.status.C) as SignedWord;
        let acc = self.A;

        println!("Operand: {}, computed: {}, acc: {}", operand, computed, acc);

        self.A = computed as Byte;
        self.status.Z = self.A == 0x00;
        self.status.V = (!((acc ^ operand as Byte) & 0x80) != 0) && (((acc ^ (computed as Byte)) & 0x80) != 0);
        self.status.C = computed >= 0;
        self.status.N = (self.A & 0x80) == 0x80;

        0
    }

    fn sec(&mut self) -> usize {
        self.status.C = true;

        0
    }

    fn sed(&mut self) -> usize {
        self.status.D = true;

        0
    }

    fn sei(&mut self) -> usize {
        self.status.I = true;

        0
    }

    fn sta<Bus: BusTrait>(&mut self, address: Word, bus: &mut Bus) -> usize {
        bus.write_byte(address, self.A);

        0
    }

    fn txs(&mut self) -> usize {
        self.SP = self.X;

        0
    }

    fn debug(&self) {
        println!("A: {:#04X}", self.A);
        println!("X: {:#04X}", self.X);
        println!("Y: {:#04X}", self.X);
        println!("SP: {}", self.SP);
        println!("PC: {}", self.PC);
        self.debug_status();
    }

    fn debug_status(&self) {
        println!("Status: {}{}{}{}{}{}{}{}",
                 if self.status.C { "C" } else { "c" },
                 if self.status.Z { "Z" } else { "z" },
                 if self.status.I { "I" } else { "i" },
                 if self.status.D { "D" } else { "d" },
                 if self.status.B { "B" } else { "b" },
                 if self.status.U { "U" } else { "u" },
                 if self.status.V { "V" } else { "v" },
                 if self.status.N { "N" } else { "n" }
        )
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use crate::assembler::{Assembler, Instructions};
    use crate::bus::BusImpl;
    use crate::parser::Parser;
    use crate::ppu::Ppu;
    use crate::ram::Ram;
    use crate::rom::Rom;

    use super::*;

    struct MockBus {
        program: Vec<u8>,
        data: Vec<u8>,
    }

    impl MockBus {
        fn new(program: Vec<u8>) -> MockBus {
            let data = vec![0; 0xFFFF - 0x0FFF];

            MockBus {
                program,
                data,
            }
        }
    }

    impl BusTrait for MockBus {
        fn read_byte(&self, address: Word) -> Byte {
            println!("Reading from {:#06X}", address);

            match address {
                0..=0x0FFF => self.program[address as usize],
                _ => self.data[address as usize]
            }
        }

        fn write_byte(&mut self, address: Word, data: Byte) {
            println!("Writing to {:#X}", address);

            match address {
                0..=0x0FFF => self.program[address as usize] = data,
                _ => self.data[address as usize] = data
            }
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
        let mut bus = BusImpl::new(Ram::new(16), Ppu::new(), rom);

        let start = bus.read_word(0xFFFC);
        println!("Starting address: {:#06X}", start);

        cpu.PC = start;
        // println!("First op: {:#04X}", bus.read_byte(start));

        let cycles = process_bus(&mut cpu, &mut bus);
        assert_that!(cycles, geq(1234));
    }

    #[test]
    fn process_adc() {
        let cpu = build_cpu(1, 0, 0, 0, "");

        assert_instructions(&cpu, "ADC #1", 2, 0, 0, 0, 2, "zncv", 2);

        // 1 + 1 = 2, C = 0, V = 0
        assert_instructions(&cpu, "CLC\nLDA #1\nADC #1", 2, 0, 0, 0, 5, "zncv", 5);

        // 1 + -1 = 0, C = 1, V = 0
        assert_instructions(&cpu, "CLC\nLDA #1\nADC #$FF", 0, 0, 0, 0, 5, "ZnCv", 5);

        // 127 + 1 = 128 (-128), C = 0, V = 1
        assert_instructions(&cpu, "CLC\nLDA #$7F\nADC #$01", 128, 0, 0, 0, 5, "zNcV", 5);

        // -128 + -1 = -129 (127), C = 0, V = 1
        assert_instructions(&cpu, "CLC\nLDA #$80\nADC #$FF", 127, 0, 0, 0, 5, "znCV", 5);
    }

    #[test]
    fn process_bpl() {
        let cpu = build_cpu(0, 0, 0, 0, "");
        assert_instructions(&cpu, "BPL $4\nLDA #2\nLDA #3", 3, 0, 0, 0, 6, "", 5);

        let cpu = build_cpu(0, 0, 0, 0, "N");
        assert_instructions(&cpu, "BPL $4", 0, 0, 0, 0, 2, "", 2);
    }

    #[test]
    fn process_clc() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_status_flags(&cpu, "CLC", 0, 0, 0, "c");
        assert_status_flags(&cpu, "SEC\nCLC", 0, 0, 0, "c");
    }

    #[test]
    fn process_cld() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_status_flags(&cpu, "CLD", 0, 0, 0, "d");
        assert_status_flags(&cpu, "SED\nCLD", 0, 0, 0, "d");
    }

    #[test]
    fn process_cli() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_status_flags(&cpu, "CLI", 0, 0, 0, "i");
        assert_status_flags(&cpu, "SEI\nCLI", 0, 0, 0, "i");
    }

    #[test]
    fn process_jmp() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "JMP $03", 0, 0, 0, 0, 0x0003, "", 3);
    }

    #[test]
    fn process_lda() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "LDA #0", 0x00, 0, 0, 0, 2, "Zn", 2);
        assert_instructions(&cpu, "LDA #01", 0x01, 0, 0, 0, 2, "zn", 2);
        assert_instructions(&cpu, "LDA #255", 0xFF, 0, 0, 0, 2, "zN", 2);
        assert_instructions(&cpu, "LDA #$FF\nSTA $1234\nLDA $1234", 0xFF, 0, 0, 0, 8, "zN", 9);
    }

    #[test]
    fn process_ldx() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "LDX #0", 0, 0x00, 0, 0, 2, "Zn", 2);
        assert_instructions(&cpu, "LDX #01", 0, 0x01, 0, 0, 2, "zn", 2);
        assert_instructions(&cpu, "LDX #255", 0, 0xFF, 0, 0, 2, "zN", 2);
    }

    #[test]
    fn process_nop() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "", 0, 0, 0, 0, 0, "zncv", 0);
    }

    #[test]
    fn process_sbc() {
        let cpu = build_cpu(1, 0, 0, 0, "C");

        assert_instructions(&cpu, "SBC #$1", 0, 0, 0, 0, 2, "ZnCv", 2);

        let cpu = build_cpu(0, 0, 0, 0, "");

        // 0 - 1 = -1 (255), C = 1, V = 1
        assert_instructions(&cpu, "SEC\nLDA #0\nSBC #1", 255, 0, 0, 0, 5, "zNcV", 5);
        // -128 - 1 = -129 (127), C = 1, V = 1
        assert_instructions(&cpu, "SEC\nLDA #$80\nSBC #1", 127, 0, 0, 0, 5, "znCV", 5);
        // 127 - -1 = 128 (-128), C = 0, V = 1
        assert_instructions(&cpu, "SEC\nLDA #$7F\nSBC #$FF", -128i8 as u8, 0, 0, 0, 5, "zNcV", 5);
    }

    #[test]
    fn process_sec() {
        let cpu = build_cpu(0, 0, 0, 0, "C");

        assert_status_flags(&cpu, "SEC", 0, 0, 0, "C");
        assert_status_flags(&cpu, "CLC\nSEC", 0, 0, 0, "C");
    }

    #[test]
    fn process_sed() {
        let cpu = build_cpu(0, 0, 0, 0, "D");

        assert_status_flags(&cpu, "SED", 0, 0, 0, "D");
        assert_status_flags(&cpu, "CLD\nSED", 0, 0, 0, "D");
    }

    #[test]
    fn process_sei() {
        let cpu = build_cpu(0, 0, 0, 0, "I");

        assert_status_flags(&cpu, "SEI", 0, 0, 0, "I");
        assert_status_flags(&cpu, "CLI\nSEI", 0, 0, 0, "I");
    }

    #[test]
    fn process_sta() {
        let mut cpu = build_cpu(0, 0, 0, 0, "");

        let program = _build_program("LDA #1\nSTA $1234");

        let mut program = program.data;
        program.push(0xEA);

        let mut bus = MockBus::new(program);

        process_bus(&mut cpu, &mut bus);

        assert_that!(bus.read_byte(0x1234), equal_to(0x01))
    }

    #[test]
    fn process_txs() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        // 127 - -1 = 128 (-128), C = 0, V = 1
        assert_instructions(&cpu, "LDX #1\nTXS", 0, 1, 0, 1, 3, "", 4);
    }

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

    fn assert_instructions(cpu: &Cpu, source: &str, a: Byte, x: Byte, y: Byte, sp: Byte, pc: Word, expected_status: &str, expected_cycles: usize) {
        let cpu = &mut cpu.clone();

        let total_cycles = process(cpu, source);
        println!("Cycles: {}", total_cycles);
        // let total_cycles = process(cpu, program);

        assert_that!(cpu.A, eq(a));
        assert_that!(cpu.X, eq(x));
        assert_that!(cpu.Y, eq(y));
        assert_that!(cpu.SP, eq(sp));
        assert_that!(cpu.PC, eq(pc));
        assert_status(cpu.status.clone(), expected_status);
        assert_that!(total_cycles, geq(expected_cycles));
    }

    fn assert_status_flags(cpu: &Cpu, source: &str, a: Byte, x: Byte, y: Byte, expected_status: &str) {
        let cpu = &mut cpu.clone();

        process(cpu, source);

        assert_that!(cpu.A, eq(a));
        assert_that!(cpu.X, eq(x));
        assert_that!(cpu.Y, eq(y));
        assert_status(cpu.status.clone(), expected_status);
    }

    fn _build_program(source: &str) -> Instructions {
        println!("Processing: {}", source);
        let assembler = Assembler::new();
        let parser = Parser::new();
        let tokens = parser.parse(source).unwrap();

        let program = assembler.assemble(tokens).unwrap();
        println!("Program: {:x?}", program);
        program
    }

    fn process(cpu: &mut Cpu, source: &str) -> usize {
        let program = _build_program(source);

        let mut program = program.data;
        program.push(0xEA);

        let mut bus = MockBus::new(program);

        process_bus(cpu, &mut bus)
    }

    fn process_bus<Bus: BusTrait>(cpu: &mut Cpu, bus: &mut Bus) -> usize {
        let mut cycles = cpu.process(bus);
        let mut total_cycles = cycles;

        while cycles != 0 {
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