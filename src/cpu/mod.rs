use crate::addressing_mode::AddressingMode;
use crate::bus::Bus as BusTrait;
use crate::cpu::instructions_map::InstructionsMap;
use crate::cpu::op_code::OpCode;
use crate::types::*;

mod instruction;
mod instructions_map;
mod op_code;

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

impl CpuStatus {
    pub fn reset(&mut self) {
        self.C = false;
        self.Z = false;
        self.I = false;
        self.D = false;
        self.B = false;
        self.U = false;
        self.V = false;
        self.N = false;
    }
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
    pub fn new(start_pc: Word) -> Cpu {
        Cpu {
            A: 0,
            X: 0,
            Y: 0,
            SP: 0xFF,
            PC: start_pc,
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

    pub fn reset(&mut self, start_pc: Word) {
        self.A = 0;
        self.X = 0;
        self.Y = 0;
        self.SP = 0xFF;
        self.PC = start_pc;

        self.status.reset();
    }


    pub fn process<Bus: BusTrait>(&mut self, bus: &mut Bus) -> usize {
        let op_id = bus.read_byte(self.PC);

        let instruction = self.instructions_map.find(op_id);
        let mut cycles = instruction.cycles;

        if instruction.op_code == OpCode::NOP {
            println!("NOP, exiting!");
            return 0;
        }

        println!("\n{:#06X}: {:?} ({:#04X}) {:?}", self.PC, instruction.op_code, op_id, instruction.addressing_mode);
        self.PC += 1;

        // self.debug();
        //
        let address = self.fetch_address(bus, &instruction.addressing_mode);

        cycles += match instruction.op_code {
            OpCode::ADC => self.adc(address),
            OpCode::AND => self.and(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::BCC => self.bcc(address),
            OpCode::BIT => self.bit(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::BMI => self.bmi(address),
            OpCode::BNE => self.bne(address),
            OpCode::BPL => self.bpl(address),
            OpCode::BRK => self.brk(),
            OpCode::CLC => self.clc(),
            OpCode::CLD => self.cld(),
            OpCode::CLI => self.cli(),
            OpCode::CMP => self.cmp(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::CPX => self.cpx(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::DEX => self.dex(),
            OpCode::DEY => self.dey(),
            OpCode::INX => self.inx(),
            OpCode::INY => self.iny(),
            OpCode::JMP => self.jmp(address),
            OpCode::JSR => self.jsr(address, bus),
            OpCode::LDA => self.lda(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::LDX => self.ldx(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::LDY => self.ldy(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::ORA => self.ora(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::PHA => self.pha(bus),
            OpCode::PLA => self.pla(bus),
            OpCode::ROL => self.rol(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::ROR => self.ror(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::RTS => self.rts(bus),
            OpCode::SBC => self.sbc(address),
            OpCode::SEC => self.sec(),
            OpCode::SED => self.sed(),
            OpCode::SEI => self.sei(),
            OpCode::STA => self.sta(address, bus),
            OpCode::STX => self.stx(address, bus),
            OpCode::STY => self.sty(address, bus),
            OpCode::TXA => self.txa(),
            OpCode::TXS => self.txs(),
            OpCode::TSX => self.tsx(),
            OpCode::TAX => self.tax(),
            OpCode::NOP => 0,
        };
        self.debug();

        cycles
    }

    fn fetch_address<Bus: BusTrait>(&mut self, bus: &Bus, addressing_mode: &AddressingMode) -> Word {
        let address = match addressing_mode {
            AddressingMode::Absolute => {
                let address = bus.read_word(self.PC);
                self.PC += 2;
                address
            }
            AddressingMode::AbsoluteX => {
                let address = bus.read_word(self.PC) + self.X as Word;
                self.PC += 2;
                address
            }
            AddressingMode::Accumulator => {
                self.A as Word
            }
            AddressingMode::Immediate => {
                let operand = bus.read_byte(self.PC);
                self.PC += 1;
                operand as Word
            }
            AddressingMode::Implied => return 0,
            AddressingMode::Relative => {
                let relative = bus.read_byte(self.PC) as Word;
                let address = if relative > 0x80 {
                    self.PC + relative - 0xFF
                } else {
                    self.PC + relative
                };

                self.PC += 1;
                address as Word
            }
            AddressingMode::YIndexedIndirect => {
                let address = bus.read_byte(self.PC) as Word + self.Y as Word;

                self.PC += 1;
                address
            }
            AddressingMode::ZeroPage => {
                let address = bus.read_byte(self.PC);
                self.PC += 1;
                address as Word
            }
            AddressingMode::ZeroPageX => {
                let address = bus.read_byte(self.PC) + self.X;
                self.PC += 1;
                address as Word
            }
            _ => panic!(format!("[Cpu::read_byte] Unexpected addressing mode: {:?}", addressing_mode))
        };
        println!("Address: {:#06X}", address);
        address
    }

    fn read_operand<Bus: BusTrait>(&self, operand: Word, bus: &Bus, addressing_mode: &AddressingMode) -> Byte {
        let operand = match addressing_mode {
            AddressingMode::Accumulator | AddressingMode::Immediate => operand as Byte,
            AddressingMode::Absolute |
            AddressingMode::AbsoluteX |
            AddressingMode::YIndexedIndirect |
            AddressingMode::ZeroPage
            => bus.read_byte(operand),
            _ => panic!(format!("[Cpu::read_operand] Unexpected addressing mode: {:?}", addressing_mode)),
        };
        println!("Operand: {:#04X}", operand);
        operand
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

    fn and(&mut self, operand: Byte) -> usize {
        self.A = self.A & operand;
        self.status.Z = self.A == 0x00;
        self.status.N = (self.A as SignedByte) < 0;

        0
    }

    fn bcc(&mut self, address: Word) -> usize {
        if !self.status.C {
            self.PC = address;
            1
        } else {
            0
        }
    }

    fn bit(&mut self, operand: Byte) -> usize {
        self.status.Z = self.A & operand == 0x00;
        self.status.N = (operand as SignedByte) < 0;
        self.status.V = operand & 0x40 == 0x40;

        0
    }

    fn bmi(&mut self, address: Word) -> usize {
        if self.status.N {
            self.PC = address;
            1
        } else {
            0
        }
    }

    fn bne(&mut self, address: Word) -> usize {
        if !self.status.Z {
            self.PC = address;
            1
        } else {
            0
        }
    }

    fn bpl(&mut self, address: Word) -> usize {
        if !self.status.N {
            self.PC = address;
            1
        } else {
            0
        }
    }

    fn brk(&mut self) -> usize {
        self.PC = 0xFFFE;
        self.status.B = true;
        self.status.I = true;
        0
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

    fn cmp(&mut self, operand: Byte) -> usize {
        self.compare(self.A, operand);

        0
    }

    fn cpx(&mut self, operand: Byte) -> usize {
        self.compare(self.X, operand);

        0
    }

    fn compare(&mut self, register: Byte, operand: Byte) {
        let computed = register as SignedByte - operand as SignedByte;
        self.status.C = computed >= 0;
        self.status.Z = computed == 0x00;
        self.status.N = (register as SignedByte) < 0;
    }

    fn dex(&mut self) -> usize {
        let computed = self.X as SignedWord - 1;

        self.X = computed as Byte;
        self.status.Z = computed == 0x00;
        self.status.N = computed < 0;

        0
    }

    fn dey(&mut self) -> usize {
        let computed = self.Y as SignedWord - 1;

        self.Y = computed as Byte;
        self.status.Z = computed == 0x00;
        self.status.N = computed < 0;

        0
    }

    fn inx(&mut self) -> usize {
        let computed = self.X as Word + 1;

        self.X = computed as Byte;
        self.status.Z = self.X == 0x00;
        self.status.N = (self.X as SignedByte) < 0;

        0
    }

    fn iny(&mut self) -> usize {
        let computed = self.Y as Word + 1;

        self.Y = computed as Byte;
        self.status.Z = self.Y == 0x00;
        self.status.N = (self.Y as SignedByte) < 0;

        0
    }

    fn jmp(&mut self, address: Word) -> usize {
        println!("Jumping to {:#06X}", address);

        self.PC = address;

        0
    }

    fn jsr<Bus: BusTrait>(&mut self, address: Word, bus: &mut Bus) -> usize {
        bus.write_word((self.SP as Word) - 1 | 0x0100, self.PC - 1);
        self.SP -= 2;
        println!("Jumping to {:#06X}", address);

        self.PC = address;

        0
    }

    fn lda(&mut self, operand: Byte) -> usize {
        self.A = operand;

        let operand = operand as SignedByte;
        self.status.Z = operand == 0x00;
        self.status.N = operand < 0;

        0
    }

    fn ldx(&mut self, operand: Byte) -> usize {
        self.X = operand;
        self.status.Z = self.X == 0x00;
        self.status.N = (self.X & 0x80) == 0x80;

        0
    }

    fn ldy(&mut self, operand: Byte) -> usize {
        self.Y = operand;
        self.status.Z = self.Y == 0x00;
        self.status.N = (self.Y & 0x80) == 0x80;

        0
    }

    fn ora(&mut self, operand: Byte) -> usize {
        self.A = self.A | operand;
        self.status.Z = self.A == 0x00;
        self.status.N = (self.A as SignedByte) < 0;

        0
    }

    fn pha<Bus: BusTrait>(&mut self, bus: &mut Bus) -> usize {
        bus.write_byte((self.SP as Word) | 0x0100, self.A);
        self.SP -= 1;

        0
    }

    fn pla<Bus: BusTrait>(&mut self, bus: &mut Bus) -> usize {
        self.SP += 1;
        self.A = bus.read_byte((self.SP as Word) | 0x0100);
        self.status.Z = self.A == 0x00;
        self.status.N = (self.A as SignedByte) < 0;

        0
    }

    fn rol(&mut self, operand: Byte) -> usize {
        self.A = (operand << 1) + if self.status.C { 1 } else { 0 };
        self.status.Z = self.A == 0x00;
        self.status.N = (self.A as SignedByte) < 0;
        self.status.C = (operand as SignedByte) < 0;

        0
    }

    fn ror(&mut self, operand: Byte) -> usize {
        let acc = self.A;
        self.A = (operand >> 1) + if self.status.C { 0x80 } else { 0 };
        self.status.C = (acc & 0x01) == 0x01;
        self.status.Z = self.A == 0x00;
        self.status.N = (self.A as SignedByte) < 0;

        0
    }

    fn rts<Bus: BusTrait>(&mut self, bus: &mut Bus) -> usize {
        self.SP += 2;
        self.PC = bus.read_word(self.SP as Word - 1 | 0x0100) + 1;

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
        self.status.N = (self.A as SignedByte) < 0;

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

    fn stx<Bus: BusTrait>(&mut self, address: Word, bus: &mut Bus) -> usize {
        bus.write_byte(address, self.X);

        0
    }

    fn sty<Bus: BusTrait>(&mut self, address: Word, bus: &mut Bus) -> usize {
        bus.write_byte(address, self.Y);

        0
    }

    fn tax(&mut self) -> usize {
        self.X = self.A;
        self.status.Z = self.A == 0x00;
        self.status.N = (self.A as SignedByte) < 0;

        0
    }

    fn txa(&mut self) -> usize {
        self.A = self.X;
        self.status.Z = self.A == 0x00;
        self.status.N = (self.A as SignedByte) < 0;

        0
    }

    fn txs(&mut self) -> usize {
        self.SP = self.X;

        0
    }

    fn tsx(&mut self) -> usize {
        self.X = self.SP;
        self.status.Z = self.X == 0x00;
        self.status.N = (self.X as SignedByte) < 0;

        0
    }

    fn debug(&self) {
        println!("A: {:#04X}", self.A);
        println!("X: {:#04X}", self.X);
        println!("Y: {:#04X}", self.Y);
        println!("SP: {:#04X}", self.SP);
        println!("PC: {:#06X}", self.PC);
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

    // use crate::apu::Apu;
    use crate::assembler::Assembler;
    // use crate::bus::BusImpl;
    use crate::parser::Parser;

    use super::*;

// use crate::ppu::Ppu;
    // use crate::ram::Ram;
    // use crate::rom::Rom;

    struct MockBus {
        program: Vec<u8>,
        data: Vec<u8>,
    }

    impl MockBus {
        fn new(program: Vec<u8>) -> MockBus {
            let data = vec![0; 0xFFFF - program.len()];

            MockBus {
                program,
                data,
            }
        }
    }

    impl BusTrait for MockBus {
        fn read_byte(&self, address: Word) -> Byte {
            let address = address as usize;
            let len = self.program.len();

            let data = if address < self.program.len() {
                self.program[address]
            } else {
                self.data[address - len]
            };
            println!("Reading from {:#06X} -> {:#04X}", address, data);
            data
        }

        fn write_byte(&mut self, address: Word, data: Byte) {
            println!("Writing {:#04X} to {:#06X}", data, address);
            let address = address as usize;
            let len = self.program.len();

            if address < self.program.len() {
                self.program[address] = data
            } else {
                self.data[address - len] = data
            }
        }
    }

    #[test]
    fn ctor() {
        let cpu = Cpu::new(0x1234);

        assert_that!(cpu.A, eq(0));
        assert_that!(cpu.X, eq(0));
        assert_that!(cpu.Y, eq(0));
        assert_that!(cpu.PC, eq(0x1234));
    }
    //
    // #[test]
    // fn process_all() {
    //     let rom = Rom::load("roms/nestest.nes", 16384, 8192);
    //     let mut cpu = build_cpu(0, 0, 0, 0, "");
    //     let mut bus = BusImpl::new(Ram::new(0x0800), Apu::new(), Ppu::new(), rom);
    //
    //     let start = bus.read_word(0xFFFC);
    //     println!("Starting address: {:#06X}", start);
    //
    //     cpu.PC = start;
    //     // println!("First op: {:#04X}", bus.read_byte(start));
    //
    //     let cycles = run(&mut cpu, &mut bus);
    //     assert_that!(cycles, geq(1234));
    // }

    #[test]
    fn process_adc() {
        let cpu = build_cpu(1, 0, 0, 0, "");

        assert_instructions(&cpu, "ADC #1", 2, 0, 0, 2, "zncv", 2);

        // 1 + 1 = 2, C = 0, V = 0
        assert_instructions(&cpu, "CLC\nLDA #1\nADC #1", 2, 0, 0, 5, "zncv", 5);

        // 1 + -1 = 0, C = 1, V = 0
        assert_instructions(&cpu, "CLC\nLDA #1\nADC #$FF", 0, 0, 0, 5, "ZnCv", 5);

        // 127 + 1 = 128 (-128), C = 0, V = 1
        assert_instructions(&cpu, "CLC\nLDA #$7F\nADC #$01", 128, 0, 0, 5, "zNcV", 5);

        // -128 + -1 = -129 (127), C = 0, V = 1
        assert_instructions(&cpu, "CLC\nLDA #$80\nADC #$FF", 127, 0, 0, 5, "znCV", 5);
    }

    #[test]
    fn process_and() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "AND #1", 0, 0, 0, 2, "Zn", 2);
        assert_instructions(&cpu, "LDA #$80\nAND #$FF", 0x80, 0, 0, 4, "zN", 4);
    }

    #[test]
    fn process_bcc() {
        let cpu = build_cpu(0, 0, 0, 0, "c");
        assert_instructions(&cpu, "BCC $3\nLDA #3", 0, 0, 0, 4, "", 3);

        let cpu = build_cpu(0, 0, 0, 0, "C");
        assert_instructions(&cpu, "BCC $3\nLDA #3", 3, 0, 0, 4, "", 4);
    }

    #[test]
    fn process_bit() {
        let mut cpu = build_cpu(0, 0, 0, 0, "");
        let mut bus = build_bus("LDA #1\nBIT $8005");
        bus.write_byte(0x8005, 0xFF);
        run(&mut cpu, &mut bus);

        assert_status(cpu.status.clone(), "NVz");

        let mut cpu = build_cpu(0, 0, 0, 0, "");
        let mut bus = build_bus("LDA #0\nBIT $8005");
        bus.write_byte(0x8005, 0x1);
        run(&mut cpu, &mut bus);

        assert_status(cpu.status.clone(), "nvZ");
    }

    #[test]
    fn process_bmi() {
        let cpu = build_cpu(0, 0, 0, 0, "");
        assert_instructions(&cpu, "LDA #0\nBMI $3\nLDA #3", 3, 0, 0, 6, "", 6);
        assert_instructions(&cpu, "LDA #$FF\nBMI $3\nLDA #3", 0xFF, 0, 0, 6, "", 5);
    }

    #[test]
    fn process_bne() {
        let cpu = build_cpu(0, 0, 0, 0, "");
        assert_instructions(&cpu, "LDA #1\nBNE $3\nLDA #3", 1, 0, 0, 6, "", 4);
        assert_instructions(&cpu, "LDA #0\nBNE $3\nLDA #3", 3, 0, 0, 6, "", 5);
        assert_instructions(&cpu, "LDA #1\nBNE $3\nBPL $3\nBNE $FC", 1, 0, 0, 8, "", 9);
    }

    #[test]
    fn process_bpl() {
        let cpu = build_cpu(0, 0, 0, 0, "");
        assert_instructions(&cpu, "LDA #0\nBPL $3\nLDA #3", 0, 0, 0, 6, "", 5);
        assert_instructions(&cpu, "LDA #1\nBPL $3\nLDA #3", 1, 0, 0, 6, "", 5);
        assert_instructions(&cpu, "LDA #$FF\nBPL $3\nLDA #3", 3, 0, 0, 6, "", 6);
    }

    #[test]
    fn process_brk() {
        let cpu = build_cpu(0, 0, 0, 0, "");
        assert_instructions(&cpu, "LDA #$EA\nSTA $FFFE\nBRK", 0xEA, 0, 0, 0xFFFE, "BI", 7);
    }

    #[test]
    fn process_clc() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_registers(&cpu, "CLC", 0, 0, 0, "c");
        assert_registers(&cpu, "SEC\nCLC", 0, 0, 0, "c");
    }

    #[test]
    fn process_cld() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_registers(&cpu, "CLD", 0, 0, 0, "d");
        assert_registers(&cpu, "SED\nCLD", 0, 0, 0, "d");
    }

    #[test]
    fn process_cmp() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_registers(&cpu, "CMP #1", 0, 0, 0, "czn");
        assert_registers(&cpu, "CMP #0", 0, 0, 0, "CZn");
        assert_registers(&cpu, "LDA #$FF\nCMP #1", 0xFF, 0, 0, "czN");
    }

    #[test]
    fn process_cpx() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_registers(&cpu, "CPX #1", 0, 0, 0, "czn");
        assert_registers(&cpu, "CPX #0", 0, 0, 0, "CZn");
        assert_registers(&cpu, "LDX #$FF\nCPX #1", 0, 0xFF, 0, "czN");
    }

    #[test]
    fn process_cli() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_registers(&cpu, "CLI", 0, 0, 0, "i");
        assert_registers(&cpu, "SEI\nCLI", 0, 0, 0, "i");
    }

    #[test]
    fn process_dex() {
        let cpu = build_cpu(0, 1, 0, 0, "");

        assert_instructions(&cpu, "LDX #1\nDEX", 0, 0, 0, 3, "Zn", 4);
        assert_instructions(&cpu, "LDX #0\nDEX", 0, -1i8 as Byte, 0, 3, "zN", 4);
    }

    #[test]
    fn process_dey() {
        let cpu = build_cpu(0, 0, 1, 0, "");

        assert_instructions(&cpu, "LDY #1\nDEY", 0, 0, 0, 3, "Zn", 4);
        assert_instructions(&cpu, "LDY #0\nDEY", 0, 0, -1i8 as Byte, 3, "zN", 4);
    }

    #[test]
    fn process_inx() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "INX", 0, 1, 0, 1, "zn", 2);
        assert_instructions(&cpu, "LDX #$FF\nINX", 0, 0, 0, 3, "Zn", 4);
    }

    #[test]
    fn process_iny() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "INY", 0, 0, 1, 1, "zn", 2);
        assert_instructions(&cpu, "LDY #$FF\nINY", 0, 0, 0, 3, "Zn", 4);
    }

    #[test]
    fn process_jmp() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "JMP $03", 0, 0, 0, 0x0003, "", 3);
    }

    #[test]
    fn process_jsr() {
        let mut cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "JSR $4\nBRK\nLDA #1", 1, 0, 0, 6, "", 8);

        let mut bus = build_bus("JSR $4\nBRK\nLDA #1");
        run(&mut cpu, &mut bus);
        assert_that!(cpu.SP, eq(0xFD));
        assert_that!(bus.read_word(0x01FE), eq(0x0002));
    }

    #[test]
    fn process_lda() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "LDA #0", 0x00, 0, 0, 2, "Zn", 2);
        assert_instructions(&cpu, "LDA #01", 0x01, 0, 0, 2, "zn", 2);
        assert_instructions(&cpu, "LDA #255", 0xFF, 0, 0, 2, "zN", 2);
        assert_instructions(&cpu, "LDA #255", 0xFF, 0, 0, 2, "zN", 2);
        assert_instructions(&cpu, "LDA #$FF\nSTA $1234\nLDA $1234", 0xFF, 0, 0, 8, "zN", 10);

        // Absolute
        let mut bus = build_bus("LDA $8001");
        bus.write_byte(0x8001, 123);

        assert_address(&cpu, &mut bus, 123, 0, 0, 3, "", 4);

        // Absolute, X
        let cpu = build_cpu(0, 1, 0, 0, "");
        let mut bus = build_bus("LDA $0001,X");
        bus.write_byte(0x0001, 0x00);
        bus.write_byte(0x0002, 0x80);
        bus.write_byte(0x8001, 123);
        assert_address(&cpu, &mut bus, 123, 1, 0, 3, "", 4);

        // Indirect, Y
        let cpu = build_cpu(0, 0, 1, 0, "");
        let mut bus = build_bus("LDA ($1F),Y");
        bus.write_byte(0x20, 123);

        assert_address(&cpu, &mut bus, 123, 0, 1, 2, "", 5);

        // Zeropage
        let cpu = build_cpu(0, 0, 0, 0, "");
        let mut bus = build_bus("LDA $1F");
        bus.write_byte(0x1F, 123);

        assert_address(&cpu, &mut bus, 123, 0, 0, 2, "", 3);
    }

    #[test]
    fn process_ldx() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "LDX #1", 0, 1, 0, 2, "zn", 2);

        // Zeropage
        let mut bus = build_bus("LDX $1F");
        bus.write_byte(0x1F, 123);

        assert_address(&cpu, &mut bus, 0, 123, 0, 2, "", 3);
    }

    #[test]
    fn process_ldy() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "LDY #1", 0, 0, 1, 2, "zn", 2);
    }

    #[test]
    fn process_nop() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "", 0, 0, 0, 0, "zncv", 0);
    }

    #[test]
    fn process_ora() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "ORA #0", 0, 0, 0, 2, "Zn", 2);
        assert_instructions(&cpu, "LDA #$80\nORA #$FF", 0xFF, 0, 0, 4, "zN", 4);
    }

    #[test]
    fn process_pha() {
        let mut cpu = build_cpu(0, 0, 0, 0, "");

        let mut bus = build_bus("LDA $1\nPHA");
        run(&mut cpu, &mut bus);

        assert_that!(cpu.SP, eq(0xFE));
        assert_that!(bus.read_byte(0x01FF), eq(0x01));
    }

    #[test]
    fn process_pla() {
        let mut cpu = build_cpu(0, 0, 0, 0, "");

        let mut bus = build_bus("LDA #1\nPHA\nLDA #0\nPLA");
        bus.write_byte(0x01FF, 0x01);
        run(&mut cpu, &mut bus);

        assert_that!(cpu.SP, eq(0xFF));
        assert_that!(cpu.A, eq(0x01));

        let cpu = build_cpu(0, 0, 0, 0, "");
        assert_instructions(&cpu, "LDA #0\nPHA\nLDA #1\nPLA", 0, 0, 0, 6, "Zn", 11);

        let cpu = build_cpu(0, 0, 0, 0, "");
        assert_instructions(&cpu, "LDA #$FF\nPHA\nLDA #0\nPLA", 0xFF, 0, 0, 6, "zN", 11);
    }

    #[test]
    fn process_rol() {
        let cpu = build_cpu(0, 0, 0, 0, "");
        assert_instructions(&cpu, "LDA #1\nROL A", 2, 0, 0, 3, "nzc", 4);
        assert_instructions(&cpu, "LDA #$80\nROL A", 0, 0, 0, 3, "nZC", 4);

        let cpu = build_cpu(0, 0, 0, 0, "C");
        assert_instructions(&cpu, "ROL A", 1, 0, 0, 1, "nzc", 2);
    }

    #[test]
    fn process_ror() {
        let cpu = build_cpu(0, 0, 0, 0, "");
        assert_instructions(&cpu, "LDA #1\nROR A", 0, 0, 0, 3, "nZC", 4);
        assert_instructions(&cpu, "LDA #$80\nROR A", 0x40, 0, 0, 3, "nzc", 4);

        let cpu = build_cpu(0, 0, 0, 0, "C");
        assert_instructions(&cpu, "ROR A", 0x80, 0, 0, 1, "Nzc", 2);
    }

    #[test]
    fn process_rts() {
        let mut cpu = build_cpu(0, 0, 0, 0, "");
        assert_instructions(&cpu, "JSR $6\nJMP $7\nRTS", 0, 0, 0, 7, "", 15);

        let mut bus = build_bus("JSR $6\nJMP $7\nRTS");
        run(&mut cpu, &mut bus);

        assert_that!(cpu.SP, eq(0xFF));
    }

    #[test]
    fn process_sbc() {
        let cpu = build_cpu(1, 0, 0, 0, "C");

        assert_instructions(&cpu, "SBC #$1", 0, 0, 0, 2, "ZnCv", 2);

        let cpu = build_cpu(0, 0, 0, 0, "");

        // 0 - 1 = -1 (255), C = 1, V = 1
        assert_instructions(&cpu, "SEC\nLDA #0\nSBC #1", 255, 0, 0, 5, "zNcV", 5);
        // -128 - 1 = -129 (127), C = 1, V = 1
        assert_instructions(&cpu, "SEC\nLDA #$80\nSBC #1", 127, 0, 0, 5, "znCV", 5);
        // 127 - -1 = 128 (-128), C = 0, V = 1
        assert_instructions(&cpu, "SEC\nLDA #$7F\nSBC #$FF", -128i8 as u8, 0, 0, 5, "zNcV", 5);
    }

    #[test]
    fn process_sec() {
        let cpu = build_cpu(0, 0, 0, 0, "C");

        assert_registers(&cpu, "SEC", 0, 0, 0, "C");
        assert_registers(&cpu, "CLC\nSEC", 0, 0, 0, "C");
    }

    #[test]
    fn process_sed() {
        let cpu = build_cpu(0, 0, 0, 0, "D");

        assert_registers(&cpu, "SED", 0, 0, 0, "D");
        assert_registers(&cpu, "CLD\nSED", 0, 0, 0, "D");
    }

    #[test]
    fn process_sei() {
        let cpu = build_cpu(0, 0, 0, 0, "I");

        assert_registers(&cpu, "SEI", 0, 0, 0, "I");
        assert_registers(&cpu, "CLI\nSEI", 0, 0, 0, "I");
    }

    #[test]
    fn process_sta() {
        // Absolute
        let mut cpu = build_cpu(0, 0, 0, 0, "");

        let mut bus = build_bus("LDA #1\nSTA $1234");
        run(&mut cpu, &mut bus);

        assert_that!(bus.read_byte(0x1234), equal_to(0x01));

        // Absolute X
        let mut cpu = build_cpu(0, 1, 0, 0, "");
        let mut bus = build_bus("LDA #1\nSTA $1233,X");
        run(&mut cpu, &mut bus);

        assert_that!(bus.read_byte(0x1234), equal_to(0x01));

        // ZeroPage, X
        let mut cpu = build_cpu(0, 1, 0, 0, "");
        let mut bus = build_bus("LDA #1\nSTA $12,X");
        run(&mut cpu, &mut bus);

        assert_that!(bus.read_byte(0x13), equal_to(0x01));

        let cpu = build_cpu(0, 1, 0, 0, "");
        assert_instructions(&cpu, "LDA #1\nSTA $1234,X", 1, 1, 0, 5, "", 7);

        let cpu = build_cpu(0, 1, 0, 0, "");
        assert_instructions(&cpu, "LDA #1\nSTA $12,X", 1, 1, 0, 4, "", 6);
    }

    #[test]
    fn process_stx() {
        let mut cpu = build_cpu(0, 0, 0, 0, "");

        let mut bus = build_bus("LDX #1\nSTX $1234");

        run(&mut cpu, &mut bus);

        assert_that!(bus.read_byte(0x1234), equal_to(0x01));
    }

    #[test]
    fn process_tax() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "LDA #1\nTAX", 1, 1, 0, 3, "nz", 4);
        assert_instructions(&cpu, "LDA #0\nTAX", 0, 0, 0, 3, "nZ", 4);
        assert_instructions(&cpu, "LDA #$FF\nTAX", 0xFF, 0xFF, 0, 3, "Nz", 4);
    }

    #[test]
    fn process_txa() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "LDX #1\nTXA", 1, 1, 0, 3, "nz", 4);
        assert_instructions(&cpu, "LDX #0\nTXA", 0, 0, 0, 3, "nZ", 4);
        assert_instructions(&cpu, "LDX #$FF\nTXA", 0xFF, 0xFF, 0, 3, "Nz", 4);
    }

    #[test]
    fn process_txs() {
        let mut cpu = build_cpu(0, 0, 0, 0, "");

        let mut bus = build_bus("LDX #1\nTXS");

        run(&mut cpu, &mut bus);

        assert_that!(cpu.SP, eq(1));
    }

    #[test]
    fn process_tsx() {
        let cpu = build_cpu(0, 0, 0, 0, "");

        assert_instructions(&cpu, "TSX", 0, 0xFF, 0, 1, "Nz", 2);
    }

    fn build_cpu(a: Byte, x: Byte, y: Byte, pc: Word, status: &str) -> Cpu {
        let mut cpu = Cpu::new(pc);
        cpu.reset(0);

        cpu.A = a;
        cpu.X = x;
        cpu.Y = y;
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

    fn assert_instructions(cpu: &Cpu, source: &str, a: Byte, x: Byte, y: Byte, pc: Word, expected_status: &str, expected_cycles: usize) {
        let cpu = &mut cpu.clone();

        let total_cycles = process_source(cpu, source);
        println!("Cycles: {}", total_cycles);

        assert_that!(cpu.A, eq(a));
        assert_that!(cpu.X, eq(x));
        assert_that!(cpu.Y, eq(y));
        assert_that!(cpu.PC, eq(pc));
        assert_status(cpu.status.clone(), expected_status);
        assert_that!(total_cycles, eq(expected_cycles));
    }

    fn assert_address<Bus: BusTrait>(cpu: &Cpu, bus: &mut Bus, a: Byte, x: Byte, y: Byte, pc: Word, expected_status: &str, expected_cycles: usize) {
        let cpu = &mut cpu.clone();

        let total_cycles = run(cpu, bus);

        println!("Cycles: {}", total_cycles);

        assert_that!(cpu.A, eq(a));
        assert_that!(cpu.X, eq(x));
        assert_that!(cpu.Y, eq(y));
        assert_that!(cpu.PC, eq(pc));
        assert_status(cpu.status.clone(), expected_status);
        assert_that!(total_cycles, eq(expected_cycles));
    }

    fn assert_registers(cpu: &Cpu, source: &str, a: Byte, x: Byte, y: Byte, expected_status: &str) {
        let cpu = &mut cpu.clone();

        process_source(cpu, source);

        assert_that!(cpu.A, eq(a));
        assert_that!(cpu.X, eq(x));
        assert_that!(cpu.Y, eq(y));
        assert_status(cpu.status.clone(), expected_status);
    }

    fn build_program(source: &str) -> Vec<Byte> {
        println!("Processing:\n {}", source);
        let assembler = Assembler::new();
        let parser = Parser::new();
        let tokens = parser.parse(source).unwrap();
        // println!("Tokens: {:?}", tokens);

        let program = match assembler.assemble(tokens) {
            Ok(program) => program,
            Err(e) => panic!("Assembler error: {}", e)
        };
        println!("Program: {:x?}", program);

        let mut data = program.data;

        // Append NOP
        data.push(0xEA);

        data
    }

    fn build_bus(source: &str) -> MockBus {
        let program = build_program(source);

        MockBus::new(program)
    }

    fn process_source(cpu: &mut Cpu, source: &str) -> usize {
        let mut bus = build_bus(source);

        run(cpu, &mut bus)
    }

    fn run<Bus: BusTrait>(cpu: &mut Cpu, bus: &mut Bus) -> usize {
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