use colored::{ColoredString, Colorize};
use log::*;

use crate::bus::Bus as BusTrait;
use crate::cpu::status::Status;
use crate::instructions::addressing_mode::AddressingMode;
use crate::instructions::Instructions;
use crate::instructions::op_code::OpCode;
use crate::types::*;

pub mod status;

#[cfg(test)]
mod test_instructions;
#[cfg(test)]
mod test_timings;
#[cfg(test)]
mod test_interrupts;
#[cfg(test)]
mod test_execution;


fn bool_to_byte(flag: bool) -> Byte {
    if flag { 1 } else { 0 }
}

fn color_flag(flag: bool, text: &str) -> ColoredString {
    if flag { text.to_uppercase().as_str().green() } else { text.to_lowercase().as_str().red() }
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct Cpu {
    // Accumulator
    pub A: Byte,
    // X register
    pub X: Byte,
    // Y register
    pub Y: Byte,
    // Program counter
    pub PC: Word,
    // Stack pointer
    pub SP: Byte,
    // Status
    pub status: Status,
    pub left_cycles: usize,
    instructions: Instructions,
}

impl Cpu {
    const STACK_POINTER_ADDRESS: Byte = 0xFD;
    const INTERRUPT_ADDRESS: Word = 0xFFFE;
    const RESET_ADDRESS: Word = 0xFFFC;

    pub fn new(start_pc: Word) -> Cpu {
        Cpu {
            A: 0,
            X: 0,
            Y: 0,
            SP: Cpu::STACK_POINTER_ADDRESS,
            PC: start_pc,
            status: Status::default(),
            left_cycles: 0,
            instructions: Instructions::default(),
        }
    }

    pub fn reset<Bus: BusTrait>(&mut self, bus: &mut Bus) -> usize {
        self.A = 0;
        self.X = 0;
        self.Y = 0;
        self.SP = Cpu::STACK_POINTER_ADDRESS;
        self.PC = bus.read_word(Cpu::RESET_ADDRESS);

        self.status.reset();
        self.left_cycles = 8;

        self.tick(bus)
    }

    pub fn tick<Bus: BusTrait>(&mut self, bus: &mut Bus) -> usize {
        if self.left_cycles == 0 {
            self.process(bus);
        } else {
            trace!("Nothing to do, just wait for {} cycle{}", self.left_cycles, if self.left_cycles == 1 { "" } else { "s" })
        }

        self.left_cycles -= 1;
        self.left_cycles
    }

    pub fn process<Bus: BusTrait>(&mut self, bus: &mut Bus) -> usize {
        let op_id = bus.read_byte(self.PC);

        let instruction = self.instructions.find(op_id);

        trace!("\n{:#06X}: {:?} ({:#04X}) {:?}", self.PC, instruction.op_code, op_id, instruction.addressing_mode);
        self.PC += 1;

        // self.trace();
        //
        let (address, extra_cycles) = self.fetch_address(bus, &instruction.addressing_mode);
        let mut cycles = instruction.cycles + extra_cycles;

        cycles += match instruction.op_code {
            OpCode::ADC => self.adc(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::AND => self.and(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::ASL => self.asl(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::BCC => self.bcc(address),
            OpCode::BCS => self.bcs(address),
            OpCode::BEQ => self.beq(address),
            OpCode::BIT => self.bit(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::BMI => self.bmi(address),
            OpCode::BNE => self.bne(address),
            OpCode::BPL => self.bpl(address),
            OpCode::BVC => self.bvc(address),
            OpCode::BVS => self.bvs(address),
            OpCode::BRK => self.brk(bus),
            OpCode::CLC => self.clc(),
            OpCode::CLD => self.cld(),
            OpCode::CLI => self.cli(),
            OpCode::CLV => self.clv(),
            OpCode::CMP => self.cmp(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::CPX => self.cpx(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::CPY => self.cpy(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::DEC => self.dec(address, bus, &instruction.addressing_mode),
            OpCode::DEX => self.dex(),
            OpCode::DEY => self.dey(),
            OpCode::EOR => self.eor(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::INC => self.inc(address, bus, &instruction.addressing_mode),
            OpCode::INX => self.inx(),
            OpCode::INY => self.iny(),
            OpCode::JMP => self.jmp(address),
            OpCode::JSR => self.jsr(address, bus),
            OpCode::LDA => self.lda(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::LDX => self.ldx(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::LDY => self.ldy(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::LSR => self.lsr(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::NOP => self.nop(),
            OpCode::ORA => self.ora(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::PHA => self.pha(bus),
            OpCode::PHP => self.php(bus),
            OpCode::PLA => self.pla(bus),
            OpCode::PLP => self.plp(bus),
            OpCode::ROL => self.rol(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::ROR => self.ror(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::RTI => self.rti(bus),
            OpCode::RTS => self.rts(bus),
            OpCode::SBC => self.sbc(self.read_operand(address, bus, &instruction.addressing_mode)),
            OpCode::SEC => self.sec(),
            OpCode::SED => self.sed(),
            OpCode::SEI => self.sei(),
            OpCode::STA => self.sta(address, bus),
            OpCode::STX => self.stx(address, bus),
            OpCode::STY => self.sty(address, bus),
            OpCode::TAX => self.tax(),
            OpCode::TAY => self.tay(),
            OpCode::TXA => self.txa(),
            OpCode::TYA => self.tya(),
            OpCode::TXS => self.txs(),
            OpCode::TSX => self.tsx(),
        };
        self.debug();

        self.left_cycles = cycles;

        cycles
    }

    fn fetch_address<Bus: BusTrait>(&mut self, bus: &mut Bus, addressing_mode: &AddressingMode) -> (Word, usize) {
        let mut extra_cycles = 0;
        let address = match addressing_mode {
            AddressingMode::Absolute => {
                let address = bus.read_word(self.PC);
                self.PC += 2;
                address
            }
            AddressingMode::AbsoluteX => {
                let address = bus.read_word(self.PC);
                let page1 = address & 0xFF00;

                let address = address + self.X as Word;
                let page2 = address & 0xFF00;

                self.PC += 2;

                if page1 != page2 {
                    extra_cycles += 1;
                }

                address
            }
            AddressingMode::AbsoluteY => {
                let address = bus.read_word(self.PC);
                let page1 = address & 0xFF00;

                let address = address + self.Y as Word;
                let page2 = address & 0xFF00;

                self.PC += 2;

                if page1 != page2 {
                    extra_cycles += 1;
                }

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
            AddressingMode::Implied => return (0, 0),
            AddressingMode::Indirect => {
                let address = bus.read_word(self.PC) as Word;
                let address = bus.read_word(address) as Word;

                self.PC += 1;
                address
            }
            AddressingMode::Relative => {
                let mut relative = bus.read_byte(self.PC) as Word;

                self.PC += 1;

                if relative > 0x80 {
                    relative |= 0xFF00;
                }

                relative
            }
            AddressingMode::YIndexedIndirect => {
                let address = bus.read_byte(self.PC) as Word;

                let page1 = address & 0xFF00;
                let address = bus.read_word(address) as Word + self.Y as Word;
                let page2 = address & 0xFF00;

                self.PC += 1;

                if page1 != page2 {
                    extra_cycles += 1;
                }

                address
            }
            AddressingMode::IndirectIndexedX => {
                let address = bus.read_byte(self.PC) as Word + self.X as Word;
                let address = bus.read_word(address) as Word;

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
            AddressingMode::ZeroPageY => {
                let address = bus.read_byte(self.PC) + self.Y;
                self.PC += 1;
                address as Word
            }
        };

        trace!("Address: {:#06X}", address);
        (address, extra_cycles)
    }

    fn read_operand<Bus: BusTrait>(&self, operand: Word, bus: &mut Bus, addressing_mode: &AddressingMode) -> Byte {
        let operand = match addressing_mode {
            AddressingMode::Implied => 0,
            AddressingMode::Accumulator | AddressingMode::Immediate => operand as Byte,
            AddressingMode::Absolute |
            AddressingMode::AbsoluteX | AddressingMode::AbsoluteY |
            AddressingMode::YIndexedIndirect | AddressingMode::IndirectIndexedX |
            AddressingMode::ZeroPage | AddressingMode::ZeroPageX | AddressingMode::ZeroPageY
            => bus.read_byte(operand),
            _ => panic!(format!("[Cpu::read_operand] Unexpected addressing mode: {:?}", addressing_mode)),
        };
        trace!("Operand: {:#04X}", operand);
        operand
    }

    fn adc(&mut self, operand: Byte) -> usize {
        let computed = self.A as Word + operand as Word + bool_to_byte(self.status.C) as Word;
        let acc = self.A;

        trace!("Operand: {}, computed: {}, acc: {}", operand, computed, acc);

        self.A = computed as Byte;
        self.status.Z = self.A == 0x00;
        self.status.V = (!((acc ^ operand as Byte) & 0x80) != 0) && (((acc ^ (computed as Byte)) & 0x80) != 0);
        self.status.C = computed > 0xFF;
        self.status.N = (self.A & 0x80) == 0x80;

        0
    }

    fn and(&mut self, operand: Byte) -> usize {
        self.A &= operand;
        self.status.Z = self.A == 0x00;
        self.status.N = (self.A as SignedByte) < 0;

        0
    }

    fn asl(&mut self, operand: Byte) -> usize {
        self.status.C = operand & 0x80 == 0x80;
        self.A = operand << 1;
        self.status.Z = self.A == 0x00;
        self.status.N = (self.A as SignedByte) < 0;

        0
    }

    fn bcc(&mut self, address: Word) -> usize {
        self._test(!self.status.C, address)
    }

    fn bcs(&mut self, address: Word) -> usize {
        self._test(self.status.C, address)
    }

    fn beq(&mut self, address: Word) -> usize {
        self._test(self.status.Z, address)
    }

    fn bit(&mut self, operand: Byte) -> usize {
        self.status.Z = self.A & operand == 0x00;
        self.status.N = (operand as SignedByte) < 0;
        self.status.V = operand & 0x40 == 0x40;

        0
    }

    fn bmi(&mut self, address: Word) -> usize {
        self._test(self.status.N, address)
    }

    fn bne(&mut self, address: Word) -> usize {
        self._test(!self.status.Z, address)
    }

    fn bpl(&mut self, address: Word) -> usize {
        self._test(!self.status.N, address)
    }

    fn brk<Bus: BusTrait>(&mut self, bus: &mut Bus) -> usize {
        self._push_word(bus, self.PC + 1);
        self._push_byte(bus, self.status.to_byte());

        let address = bus.read_word(Cpu::INTERRUPT_ADDRESS);
        trace!("Jumping to {:#06X}", address);
        self.PC = address;

        self.status.B = true;
        self.status.I = true;

        0
    }

    fn bvc(&mut self, address: Word) -> usize {
        self._test(!self.status.V, address)
    }

    fn bvs(&mut self, address: Word) -> usize {
        self._test(self.status.V, address)
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

    fn clv(&mut self) -> usize {
        self.status.V = false;

        0
    }

    fn cmp(&mut self, operand: Byte) -> usize {
        self._compare(self.A, operand);

        0
    }

    fn cpx(&mut self, operand: Byte) -> usize {
        self._compare(self.X, operand);

        0
    }

    fn cpy(&mut self, operand: Byte) -> usize {
        self._compare(self.Y, operand);

        0
    }

    fn dec<Bus: BusTrait>(&mut self, address: Word, bus: &mut Bus, _addressing_mode: &AddressingMode) -> usize {
        let data = bus.read_byte(address) as SignedWord - 1;
        let data = data as Byte;
        bus.write_byte(address, data);

        self.status.Z = data == 0x00;
        self.status.N = (data as SignedByte) < 0;

        0
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

    fn eor(&mut self, operand: Byte) -> usize {
        self.A ^= operand;
        self.status.Z = self.A == 0x00;
        self.status.N = (self.A as SignedByte) < 0;

        0
    }

    fn inc<Bus: BusTrait>(&mut self, address: Word, bus: &mut Bus, _addressing_mode: &AddressingMode) -> usize {
        let data = bus.read_byte(address) as Word + 1;
        let data = data as Byte;
        bus.write_byte(address, data);

        self.status.Z = data == 0x00;
        self.status.N = (data as SignedByte) < 0;

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
        trace!("Jumping to {:#06X}", address);

        self.PC = address;

        0
    }

    fn jsr<Bus: BusTrait>(&mut self, address: Word, bus: &mut Bus) -> usize {
        self._push_word(bus, self.PC - 1);
        trace!("Jumping to {:#06X}", address);

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

    fn lsr(&mut self, operand: Byte) -> usize {
        self.status.C = operand & 0x01 == 0x01;
        self.A = operand >> 1 & 0x7F;

        0
    }

    fn ora(&mut self, operand: Byte) -> usize {
        self.A |= operand;
        self.status.Z = self.A == 0x00;
        self.status.N = (self.A as SignedByte) < 0;

        0
    }

    fn nop(&mut self) -> usize {
        0
    }

    fn pha<Bus: BusTrait>(&mut self, bus: &mut Bus) -> usize {
        self._push_byte(bus, self.A);

        0
    }

    fn php<Bus: BusTrait>(&mut self, bus: &mut Bus) -> usize {
        self._push_byte(bus, self.status.to_byte());

        0
    }

    fn pla<Bus: BusTrait>(&mut self, bus: &mut Bus) -> usize {
        self.A = self._pop_byte(bus);
        self.status.Z = self.A == 0x00;
        self.status.N = (self.A as SignedByte) < 0;

        0
    }

    fn plp<Bus: BusTrait>(&mut self, bus: &mut Bus) -> usize {
        self.status = Status::from_byte(self._pop_byte(bus));

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

    fn rti<Bus: BusTrait>(&mut self, bus: &mut Bus) -> usize {
        self.status = Status::from_byte(self._pop_byte(bus));
        self.PC = self._pop_word(bus);

        0
    }

    fn rts<Bus: BusTrait>(&mut self, bus: &mut Bus) -> usize {
        self.PC = self._pop_word(bus) + 1;

        0
    }

    fn sbc(&mut self, operand: Byte) -> usize {
        let computed = self.A as SignedWord - operand as SignedWord - bool_to_byte(!self.status.C) as SignedWord;
        let acc = self.A;

        trace!("Operand: {}, computed: {}, acc: {}", operand, computed, acc);

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

    fn tay(&mut self) -> usize {
        self.Y = self.A;
        self.status.Z = self.A == 0x00;
        self.status.N = (self.A as SignedByte) < 0;

        0
    }

    fn tsx(&mut self) -> usize {
        self.X = self.SP;
        self.status.Z = self.X == 0x00;
        self.status.N = (self.X as SignedByte) < 0;

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

    fn tya(&mut self) -> usize {
        self.A = self.Y;
        self.status.Z = self.A == 0x00;
        self.status.N = (self.A as SignedByte) < 0;

        0
    }

    pub fn debug(&self) {
        self.debug_status();

        info!("PC: {:#06X}", self.PC);
        info!("A: {:#04X}", self.A);
        info!("X: {:#04X}", self.X);
        info!("Y: {:#04X}", self.Y);
        info!("SP: {:#04X}", self.SP);
    }

    fn debug_status(&self) {
        info!("Status: {}{}{}{}{}{}{}{}",
              color_flag(self.status.C, "C"),
              color_flag(self.status.Z, "Z"),
              color_flag(self.status.I, "I"),
              color_flag(self.status.D, "D"),
              color_flag(self.status.B, "B"),
              color_flag(self.status.U, "U"),
              color_flag(self.status.V, "V"),
              color_flag(self.status.N, "N"),
        )
    }

    fn _compare(&mut self, register: Byte, operand: Byte) {
        let computed = register.wrapping_sub(operand) as SignedByte;
        self.status.N = (register as SignedByte) < 0;
        self.status.Z = computed == 0x00;
        self.status.C = computed >= 0;
    }

    fn _pop_byte<Bus: BusTrait>(&mut self, bus: &mut Bus) -> Byte {
        trace!("Popping byte from {:#04X}", self.SP.wrapping_add(1));
        self.SP = self.SP.wrapping_add(1);
        bus.read_byte((self.SP as Word) | 0x0100)
    }

    fn _pop_word<Bus: BusTrait>(&mut self, bus: &mut Bus) -> Word {
        trace!("Popping word from {:#04X}", self.SP.wrapping_add(1));
        self.SP = self.SP.wrapping_add(2);
        bus.read_word((self.SP.wrapping_sub(1) as Word) | 0x0100)
    }

    fn _push_byte<Bus: BusTrait>(&mut self, bus: &mut Bus, byte: Byte) {
        trace!("Pushing byte to {:#04X}", self.SP.wrapping_sub(1));
        bus.write_byte((self.SP as Word) | 0x0100, byte);
        self.SP = self.SP.wrapping_sub(1);
    }

    fn _push_word<Bus: BusTrait>(&mut self, bus: &mut Bus, word: Word) {
        trace!("Pushing word to {:#04X}", self.SP.wrapping_sub(1));
        bus.write_word((self.SP.wrapping_sub(1) as Word) | 0x0100, word);
        self.SP = self.SP.wrapping_sub(2);
    }

    fn _test(&mut self, test: bool, address: Word) -> usize {
        trace!("Test is {}", test);
        if test {
            let page1 = self.PC & 0xFF00;

            trace!("Jumping relative to {:#04X}", address as Byte);
            self.PC = self.PC.wrapping_add(address);

            let page2 = self.PC & 0xFF00;

            if page1 != page2 { 2 } else { 1 }
        } else {
            0
        }
    }
}
