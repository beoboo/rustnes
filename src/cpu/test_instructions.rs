use hamcrest2::prelude::*;

// use crate::apu::Apu;
use crate::assembler::Assembler;
// use crate::bus::BusImpl;
use crate::parser::Parser;

use super::*;
use hamcrest2::core::*;

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

    // Page boundary cross
    let mut cpu = build_cpu(0, 0, 0, 0, "c");
    let mut bus = build_bus("JMP $FF");
    // BCC in address 0xFF
    bus.write_byte(0xFF, 0x90);
    bus.write_byte(0x100, 0x01);
    bus.write_byte(0x101, 0xEA);
    let cycles = run(&mut cpu, &mut bus);

    assert_that!(cpu.PC, eq(0x101));
    assert_that!(cycles, eq(7));
}

#[test]
fn process_bcs() {
    let cpu = build_cpu(0, 0, 0, 0, "c");
    assert_instructions(&cpu, "BCS $3\nLDA #3", 3, 0, 0, 4, "", 4);

    let cpu = build_cpu(0, 0, 0, 0, "C");
    assert_instructions(&cpu, "BCS $3\nLDA #3", 0, 0, 0, 4, "", 3);
}

#[test]
fn process_beq() {
    let cpu = build_cpu(0, 0, 0, 0, "");
    assert_instructions(&cpu, "LDA #1\nBEQ $3\nLDA #3", 3, 0, 0, 6, "", 6);
    assert_instructions(&cpu, "LDA #0\nBEQ $3\nLDA #3", 0, 0, 0, 6, "", 5);
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
    assert_instructions(&cpu, "LDA #1\nBNE $3\nLDA #3", 1, 0, 0, 6, "", 5);
    assert_instructions(&cpu, "LDA #0\nBNE $3\nLDA #3", 3, 0, 0, 6, "", 6);
    assert_instructions(&cpu, "LDA #1\nBNE $3\nBPL $3\nBNE $FC", 1, 0, 0, 8, "", 11);
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
fn process_dec() {
    // // Zeropage
    let mut cpu = build_cpu(0, 0, 0, 0, "");
    let mut bus = build_bus("DEC $10");

    bus.write_byte(0x10, 123);
    run(&mut cpu, &mut bus);

    assert_that!(bus.read_byte(0x10), eq(122));

    let cpu = build_cpu(0, 0, 0, 0, "");
    assert_instructions(&cpu, "DEC $10", 0, 0, 0, 2, "zN", 5);
    assert_instructions(&cpu, "LDA #$80\nSTA $10\nDEC $10", 0x80, 0, 0, 6, "zn", 10);
    assert_instructions(&cpu, "LDA #1\nSTA $10\nDEC $10", 1, 0, 0, 6, "Zn", 10);
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
fn process_inc() {
    // // Zeropage
    let mut cpu = build_cpu(0, 0, 0, 0, "");
    let mut bus = build_bus("INC $10");

    bus.write_byte(0x10, 123);
    run(&mut cpu, &mut bus);

    assert_that!(bus.read_byte(0x10), eq(124));

    let cpu = build_cpu(0, 0, 0, 0, "");
    assert_instructions(&cpu, "INC $1234", 0, 0, 0, 3, "zn", 6);
    assert_instructions(&cpu, "INC $10", 0, 0, 0, 2, "zn", 5);
    assert_instructions(&cpu, "LDA #$80\nSTA $10\nINC $10", 128, 0, 0, 6, "zN", 10);
    assert_instructions(&cpu, "LDA #$FF\nSTA $10\nINC $10", -1i8 as Byte, 0, 0, 6, "Zn", 10);
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

    assert_instructions(&cpu, "JMP $03", 0, 0, 0, 3, "", 3);

    let mut cpu = build_cpu(0, 0, 0, 0, "");
    let mut bus = build_bus("JMP ($10)\nLDA #1");

    bus.write_word(0x10, 0x05);
    run(&mut cpu, &mut bus);

    assert_that!(cpu.A, eq(0));
    assert_that!(cpu.PC, eq(5));
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
    bus.write_word(0x20, 0x0022);
    bus.write_byte(0x22, 123);

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
fn process_lsr() {
    let cpu = build_cpu(0, 0, 0, 0, "");

    assert_instructions(&cpu, "LDA #%10000001\nLSR A", 0b01000000, 0, 0, 3, "C", 4);
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
fn process_sty() {
    let mut cpu = build_cpu(0, 0, 0, 0, "");

    let mut bus = build_bus("LDY #1\nSTY $1234");

    run(&mut cpu, &mut bus);

    assert_that!(bus.read_byte(0x1234), equal_to(0x01));

    // Timing
    let cpu = build_cpu(0, 0, 0, 0, "");
    assert_instructions(&cpu, "LDY #1\nSTY $1234", 0, 0, 1, 5, "", 6);
    assert_instructions(&cpu, "LDY #1\nSTY $12", 0, 0, 1, 4, "", 5);
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
    let mut next_op_code = bus.read_byte(cpu.PC);
    let mut total_cycles = 0;

    while next_op_code != 0xEA {
        let cycles = cpu.process(bus);
        total_cycles += cycles;
        next_op_code = bus.read_byte(cpu.PC);
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
