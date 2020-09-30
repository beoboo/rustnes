use hamcrest2::core::*;
use hamcrest2::prelude::*;

// use crate::apu::Apu;
use crate::assembler::Assembler;
use crate::bus::simple_bus::SimpleBus;
// use crate::bus::BusImpl;
use crate::parser::Parser;

use super::*;

// use crate::ppu::Ppu;
// use crate::ram::Ram;
// use crate::rom::Rom;

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
//     let mut cpu = Cpu::new(0;
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
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "LDA #1\nADC #1", 2, 0, 0, 4, "zncv");
    assert_absolute_instructions(&mut cpu, "LDA #1\nADC $8000", &[5], 6, 0, 0, 5, "zncv");

    // 1 + 1 = 2, C = 0, V = 0
    assert_instructions(&mut cpu, "CLC\nLDA #1\nADC #1", 2, 0, 0, 5, "zncv");

    // 1 + -1 = 0, C = 1, V = 0
    assert_instructions(&mut cpu, "CLC\nLDA #1\nADC #$FF", 0, 0, 0, 5, "ZnCv");

    // 127 + 1 = 128 (-128), C = 0, V = 1
    assert_instructions(&mut cpu, "CLC\nLDA #$7F\nADC #$01", 128, 0, 0, 5, "zNcV");

    // -128 + -1 = -129 (127), C = 0, V = 1
    assert_instructions(&mut cpu, "CLC\nLDA #$80\nADC #$FF", 127, 0, 0, 5, "znCV");
}

#[test]
fn process_and() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "AND #1", 0, 0, 0, 2, "Zn");
    assert_instructions(&mut cpu, "LDA #$80\nAND #$FF", 0x80, 0, 0, 4, "zN");
    assert_absolute_instructions(&mut cpu, "LDA #1\nAND $8000", &[0xFF], 1, 0, 0, 5, "zn");
}

#[test]
fn process_asl() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "ASL A", 0, 0, 0, 1, "Znc");
    assert_instructions(&mut cpu, "LDA #$C0\nASL A", 0x80, 0, 0, 3, "zNC");
    assert_absolute_instructions(&mut cpu, "ASL $8000", &[1], 2, 0, 0, 3, "zn");
}

#[test]
fn process_bcc() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "BCC $2\nLDA #3", 0, 0, 0, 4, "");
    assert_instructions(&mut cpu, "SEC\nBCC $2\nLDA #3", 3, 0, 0, 5, "");
}

#[test]
fn process_bcs() {
    let mut cpu = Cpu::new(0);
    assert_instructions(&mut cpu, "BCS $2\nLDA #3", 3, 0, 0, 4, "");
    assert_instructions(&mut cpu, "SEC\nBCS $2\nLDA #3", 0, 0, 0, 5, "");
}

#[test]
fn process_beq() {
    let mut cpu = Cpu::new(0);
    assert_instructions(&mut cpu, "LDA #1\nBEQ $2\nLDA #3", 3, 0, 0, 6, "");
    assert_instructions(&mut cpu, "LDA #0\nBEQ $2\nLDA #3", 0, 0, 0, 6, "");
}

#[test]
fn process_bit() {
    let mut cpu = Cpu::new(0);
    let mut bus = build_bus("LDA #1\nBIT $8005");
    bus.write_byte(0x8005, 0xFF);
    run(&mut cpu, &mut bus);

    assert_status(cpu.status.clone(), "NVz");

    let mut cpu = Cpu::new(0);
    let mut bus = build_bus("LDA #0\nBIT $8005");
    bus.write_byte(0x8005, 0x1);
    run(&mut cpu, &mut bus);

    assert_status(cpu.status.clone(), "nvZ");
}

#[test]
fn process_bmi() {
    let mut cpu = Cpu::new(0);
    assert_instructions(&mut cpu, "LDA #0\nBMI $2\nLDA #3", 3, 0, 0, 6, "");
    assert_instructions(&mut cpu, "LDA #$FF\nBMI $2\nLDA #3", 0xFF, 0, 0, 6, "");
}

#[test]
fn process_bne() {
    let mut cpu = Cpu::new(0);
    // assert_instructions(&mut cpu, "LDA #1\nBNE $2\nLDA #3", 1, 0, 0, 6, "");
    // assert_instructions(&mut cpu, "LDA #0\nBNE $2\nLDA #3", 3, 0, 0, 6, "");
    assert_instructions(&mut cpu, "LDA #1\nBNE $2\nBPL $2\nBNE $FC", 1, 0, 0, 8, "");
}

#[test]
fn process_bpl() {
    let mut cpu = Cpu::new(0);
    assert_instructions(&mut cpu, "LDA #0\nBPL $2\nLDA #3", 0, 0, 0, 6, "");
    assert_instructions(&mut cpu, "LDA #1\nBPL $2\nLDA #3", 1, 0, 0, 6, "");
    assert_instructions(&mut cpu, "LDA #$FF\nBPL $2\nLDA #3", 3, 0, 0, 6, "");
}

#[test]
fn process_brk() {
    let mut cpu = Cpu::new(0);
    let mut bus = build_bus("LDA #$FF\nADC #1\nSEC\nBRK");

    bus.write_word(Cpu::INTERRUPT_ADDRESS, 0x0006);
    run(&mut cpu, &mut bus);

    assert_that!(cpu.PC, eq(0x0006));

    // PC + 1
    assert_that!(bus.read_word(0x01FC), eq(0x0007));

    let status = Status::from_string("CZidbUVn");
    assert_that!(bus.read_byte(0x01FB), eq(status.to_byte()));
    assert_that!(cpu.SP, eq(0xFA));
    assert_status(cpu.status, "CZIdBUVn");
}

#[test]
fn process_bvc() {
    let mut cpu = Cpu::new(0);
    assert_instructions(&mut cpu, "BVC $2\nLDA #3", 0, 0, 0, 4, "");
    assert_instructions(&mut cpu, "LDA #$FF\nADC #1\nBVC $2\nLDA #3", 3, 0, 0, 8, "");
}

#[test]
fn process_bvs() {
    let mut cpu = Cpu::new(0);
    assert_instructions(&mut cpu, "BVS $2\nLDA #3", 3, 0, 0, 4, "");
    assert_instructions(&mut cpu, "LDA #$FF\nADC #1\nBVS $2\nLDA #3", 0, 0, 0, 8, "");
}

#[test]
fn process_clc() {
    let mut cpu = Cpu::new(0);

    assert_registers(&mut cpu, "CLC", 0, 0, 0, "c");
    assert_registers(&mut cpu, "SEC\nCLC", 0, 0, 0, "c");
}

#[test]
fn process_cld() {
    let mut cpu = Cpu::new(0);

    assert_registers(&mut cpu, "CLD", 0, 0, 0, "d");
    assert_registers(&mut cpu, "SED\nCLD", 0, 0, 0, "d");
}

#[test]
fn process_cli() {
    let mut cpu = Cpu::new(0);

    assert_registers(&mut cpu, "CLI", 0, 0, 0, "i");
    assert_registers(&mut cpu, "SEI\nCLI", 0, 0, 0, "i");
}

#[test]
fn process_clv() {
    let mut cpu = Cpu::new(0);
    assert_registers(&mut cpu, "CLV", 0, 0, 0, "v");

    let mut cpu = Cpu::new(0);
    assert_registers(&mut cpu, "CLV", 0, 0, 0, "v");
}

#[test]
fn process_cmp() {
    let mut cpu = Cpu::new(0);

    assert_registers(&mut cpu, "CMP #1", 0, 0, 0, "czn");
    assert_registers(&mut cpu, "CMP #0", 0, 0, 0, "CZn");
    assert_registers(&mut cpu, "LDA #$FF\nCMP #1", 0xFF, 0, 0, "czN");
    assert_absolute_instructions(&mut cpu, "LDA #1\nCMP $8000", &[1], 1, 0, 0, 5, "CZn");
}

#[test]
fn process_cpx() {
    let mut cpu = Cpu::new(0);

    assert_registers(&mut cpu, "CPX #1", 0, 0, 0, "czn");
    assert_registers(&mut cpu, "CPX #0", 0, 0, 0, "CZn");
    assert_registers(&mut cpu, "LDX #$FF\nCPX #1", 0, 0xFF, 0, "czN");
    assert_absolute_instructions(&mut cpu, "LDX #1\nCPX $8000", &[1], 0, 1, 0, 5, "CZn");
}

#[test]
fn process_cpy() {
    let mut cpu = Cpu::new(0);

    assert_registers(&mut cpu, "CPY #1", 0, 0, 0, "czn");
    assert_registers(&mut cpu, "CPY #0", 0, 0, 0, "CZn");
    assert_registers(&mut cpu, "LDY #$FF\nCPY #1", 0, 0, 0xFF, "czN");
    assert_absolute_instructions(&mut cpu, "LDY #1\nCPY $8000", &[1], 0, 0, 1, 5, "CZn");
}

#[test]
fn process_dec() {
    // // Zeropage
    let mut cpu = Cpu::new(0);
    let mut bus = build_bus("DEC $10");

    bus.write_byte(0x10, 123);
    run(&mut cpu, &mut bus);

    assert_that!(bus.read_byte(0x10), eq(122));

    let mut cpu = Cpu::new(0);
    assert_instructions(&mut cpu, "DEC $10", 0, 0, 0, 2, "zN");
    assert_instructions(&mut cpu, "LDA #$80\nSTA $10\nDEC $10", 0x80, 0, 0, 6, "zn");
    assert_instructions(&mut cpu, "LDA #1\nSTA $10\nDEC $10", 1, 0, 0, 6, "Zn");
}

#[test]
fn process_dex() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "LDX #1\nDEX", 0, 0, 0, 3, "Zn");
    assert_instructions(&mut cpu, "LDX #0\nDEX", 0, -1i8 as Byte, 0, 3, "zN");
}

#[test]
fn process_dey() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "LDY #1\nDEY", 0, 0, 0, 3, "Zn");
    assert_instructions(&mut cpu, "LDY #0\nDEY", 0, 0, -1i8 as Byte, 3, "zN");
}

#[test]
fn process_eor() {
    let mut cpu = Cpu::new(0);

    assert_registers(&mut cpu, "EOR #0", 0, 0, 0, "Zn");
    assert_registers(&mut cpu, "LDA #$FF\nEOR #1", 0xFE, 0, 0, "zN");
    assert_absolute_instructions(&mut cpu, "EOR $8000", &[1], 1, 0, 0, 3, "zn");
}

#[test]
fn process_inc() {
    // // Zeropage
    let mut cpu = Cpu::new(0);
    let mut bus = build_bus("INC $10");

    bus.write_byte(0x10, 123);
    run(&mut cpu, &mut bus);

    assert_that!(bus.read_byte(0x10), eq(124));

    let mut cpu = Cpu::new(0);
    assert_instructions(&mut cpu, "INC $1234", 0, 0, 0, 3, "zn");
    assert_instructions(&mut cpu, "INC $10", 0, 0, 0, 2, "zn");
    assert_instructions(&mut cpu, "LDA #$80\nSTA $10\nINC $10", 128, 0, 0, 6, "zN");
    assert_instructions(&mut cpu, "LDA #$FF\nSTA $10\nINC $10", -1i8 as Byte, 0, 0, 6, "Zn");
}

#[test]
fn process_inx() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "INX", 0, 1, 0, 1, "zn");
    assert_instructions(&mut cpu, "LDX #$FF\nINX", 0, 0, 0, 3, "Zn");
}

#[test]
fn process_iny() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "INY", 0, 0, 1, 1, "zn");
    assert_instructions(&mut cpu, "LDY #$FF\nINY", 0, 0, 0, 3, "Zn");
}

#[test]
fn process_jmp() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "JMP $03", 0, 0, 0, 3, "");

    let mut cpu = Cpu::new(0);
    let mut bus = build_bus("JMP ($10)\nLDA #1");

    bus.write_word(0x10, 0x05);
    run(&mut cpu, &mut bus);

    assert_that!(cpu.A, eq(0));
    assert_that!(cpu.PC, eq(5));
}

#[test]
fn process_jsr() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "JSR $4\nBRK\nLDA #1", 1, 0, 0, 6, "");

    let mut bus = build_bus("JSR $4\nBRK\nLDA #1");
    run(&mut cpu, &mut bus);

    assert_that!(cpu.SP, eq(0xFB));
    assert_that!(bus.read_word(0x01FC), eq(0x0002));
}

#[test]
fn process_lda() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "LDA #0", 0x00, 0, 0, 2, "Zn");
    assert_instructions(&mut cpu, "LDA #01", 0x01, 0, 0, 2, "zn");
    assert_instructions(&mut cpu, "LDA #255", 0xFF, 0, 0, 2, "zN");
    assert_instructions(&mut cpu, "LDA #255", 0xFF, 0, 0, 2, "zN");
    assert_instructions(&mut cpu, "LDA #$FF\nSTA $1234\nLDA $1234", 0xFF, 0, 0, 8, "zN");
    assert_absolute_instructions(&mut cpu, "LDA $8000", &[1], 1, 0, 0, 3, "zn");
}

#[test]
fn process_ldx() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "LDX #1", 0, 1, 0, 2, "zn");

    // Zeropage
    let mut bus = build_bus("LDX $1F");
    bus.write_byte(0x1F, 123);

    assert_address(&mut cpu, &mut bus, 0, 123, 0, 2, "", 3);
    assert_absolute_instructions(&mut cpu, "LDX $8000", &[1], 0, 1, 0, 3, "zn");
}

#[test]
fn process_ldy() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "LDY #1", 0, 0, 1, 2, "zn");
    assert_absolute_instructions(&mut cpu, "LDY $8000", &[1], 0, 0, 1, 3, "zn");
}

#[test]
fn process_lsr() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "LDA #%10000001\nLSR A", 0b01000000, 0, 0, 3, "C");
    assert_absolute_instructions(&mut cpu, "LSR $8000", &[2], 1, 0, 0, 3, "zn");
}

#[test]
fn process_nop() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "", 0, 0, 0, 0, "zncv");
}

#[test]
fn process_ora() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "ORA #0", 0, 0, 0, 2, "Zn");
    assert_instructions(&mut cpu, "LDA #$80\nORA #$FF", 0xFF, 0, 0, 4, "zN");
    assert_absolute_instructions(&mut cpu, "ORA $8000", &[1], 1, 0, 0, 3, "zn");
}

#[test]
fn process_pha() {
    let mut cpu = Cpu::new(0);

    let mut bus = build_bus("LDA $1\nPHA");
    run(&mut cpu, &mut bus);

    assert_that!(cpu.SP, eq(0xFC));
    assert_that!(bus.read_byte(0x01FD), eq(0x01));
}

#[test]
fn process_php() {
    let mut cpu = Cpu::new(0);
    let mut bus = build_bus("SEC\nSED\nSEI\nLDA #$FF\nADC #1\nPHP");
    run(&mut cpu, &mut bus);

    assert_that!(cpu.status, eq(Status::from_string("CzIDbUVn")));
    assert_that!(cpu.SP, eq(0xFC));
    assert_that!(bus.read_byte(0x01FD), eq(0b01101101));
}

#[test]
fn process_pla() {
    let mut cpu = Cpu::new(0);

    let mut bus = build_bus("LDA #1\nPHA\nLDA #0\nPLA");
    bus.write_byte(0x01FD, 0x01);
    run(&mut cpu, &mut bus);

    assert_that!(cpu.SP, eq(0xFD));
    assert_that!(cpu.A, eq(0x01));

    assert_instructions(&mut cpu, "LDA #0\nPHA\nLDA #1\nPLA", 0, 0, 0, 6, "Zn");
    assert_instructions(&mut cpu, "LDA #$FF\nPHA\nLDA #0\nPLA", 0xFF, 0, 0, 6, "zN");
}

#[test]
fn process_plp() {
    let mut cpu = Cpu::new(0);
    let mut bus = build_bus("LDX #$FC\nTXS\nPLP");
    bus.write_byte(0x01FD, 0xFF);

    run(&mut cpu, &mut bus);

    assert_that!(cpu.SP, eq(0xFD));
    assert_status(cpu.status, "CZIDBUVN");
}

#[test]
fn process_rol() {
    let mut cpu = Cpu::new(0);
    assert_instructions(&mut cpu, "LDA #1\nROL A", 2, 0, 0, 3, "nzc");
    assert_instructions(&mut cpu, "LDA #$80\nROL A", 0, 0, 0, 3, "nZC");
    assert_instructions(&mut cpu, "LDA #$FF\nADC #1\nROL A", 1, 0, 0, 5, "nzc");
    assert_absolute_instructions(&mut cpu, "ROL $8000", &[1], 2, 0, 0, 3, "zn");
}

#[test]
fn process_ror() {
    let mut cpu = Cpu::new(0);
    assert_instructions(&mut cpu, "LDA #1\nROR A", 0, 0, 0, 3, "nZC");
    assert_instructions(&mut cpu, "LDA #$80\nROR A", 0x40, 0, 0, 3, "nzc");
    assert_instructions(&mut cpu, "LDA #$FF\nADC #1\nROR A", 0x80, 0, 0, 5, "Nzc");
    assert_absolute_instructions(&mut cpu, "ROR $8000", &[2], 1, 0, 0, 3, "zn");
}

#[test]
fn process_rti() {
    let mut cpu = Cpu::new(0);
    cpu.SP = 0xFC;

    let mut bus = build_bus("RTI");
    bus.write_word(0x01FE, 0x0001); // PC
    bus.write_byte(0x01FD, 0xFF); // Status

    run(&mut cpu, &mut bus);

    assert_that!(cpu.SP, eq(0xFF));
    assert_that!(cpu.PC, eq(0x0001));
    assert_status(cpu.status, "CZIDBUVN");
}

#[test]
fn process_rts() {
    let mut cpu = Cpu::new(0);
    cpu.SP = 0xFD;

    let mut bus = build_bus("RTS");
    bus.write_word(0x01FE, 0x0000); // PC

    run(&mut cpu, &mut bus);

    assert_that!(cpu.SP, eq(0xFF));
    assert_that!(cpu.PC, eq(0x0001));
}

#[test]
fn process_sbc() {
    env_logger::init();
    let mut cpu = Cpu::new(0);

    // 1 - 1 = 0, C = 1, V = 0
    assert_instructions(&mut cpu, "SEC\nLDA #1\nSBC #1", 0, 0, 0, 5, "ZnCv");

    // 0 - 1 = -1 (255), C = 1, V = 1
    assert_instructions(&mut cpu, "SEC\nLDA #0\nSBC #1", 255, 0, 0, 5, "zNcV");
    // -128 - 1 = -129 (127), C = 1, V = 1
    assert_instructions(&mut cpu, "SEC\nLDA #$80\nSBC #1", 127, 0, 0, 5, "znCV");
    // 127 - -1 = 128 (-128), C = 0, V = 1
    assert_instructions(&mut cpu, "SEC\nLDA #$7F\nSBC #$FF", -128i8 as u8, 0, 0, 5, "zNcV");

    assert_absolute_instructions(&mut cpu, "LDA #2\nSBC $8000", &[1], 0, 0, 0, 5, "ZnCv");
}

#[test]
fn process_sec() {
    let mut cpu = Cpu::new(0);

    assert_registers(&mut cpu, "SEC", 0, 0, 0, "C");
    assert_registers(&mut cpu, "CLC\nSEC", 0, 0, 0, "C");
}

#[test]
fn process_sed() {
    let mut cpu = Cpu::new(0);

    assert_registers(&mut cpu, "SED", 0, 0, 0, "D");
    assert_registers(&mut cpu, "CLD\nSED", 0, 0, 0, "D");
}

#[test]
fn process_sei() {
    let mut cpu = Cpu::new(0);

    assert_registers(&mut cpu, "SEI", 0, 0, 0, "I");
    assert_registers(&mut cpu, "CLI\nSEI", 0, 0, 0, "I");
}

#[test]
fn process_sta() {
    let mut cpu = Cpu::new(0);

    let mut bus = build_bus("LDA #1\nSTA $1234");
    run(&mut cpu, &mut bus);

    assert_that!(bus.read_byte(0x1234), equal_to(0x01));
}

#[test]
fn process_stx() {
    let mut cpu = Cpu::new(0);

    let mut bus = build_bus("LDX #1\nSTX $1234");

    run(&mut cpu, &mut bus);

    assert_that!(bus.read_byte(0x1234), equal_to(0x01));
}

#[test]
fn process_sty() {
    let mut cpu = Cpu::new(0);

    let mut bus = build_bus("LDY #1\nSTY $1234");

    run(&mut cpu, &mut bus);

    assert_that!(bus.read_byte(0x1234), equal_to(0x01));

    // Timing
    let mut cpu = Cpu::new(0);
    assert_instructions(&mut cpu, "LDY #1\nSTY $1234", 0, 0, 1, 5, "");
    assert_instructions(&mut cpu, "LDY #1\nSTY $12", 0, 0, 1, 4, "");
}

#[test]
fn process_tax() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "LDA #1\nTAX", 1, 1, 0, 3, "nz");
    assert_instructions(&mut cpu, "LDA #0\nTAX", 0, 0, 0, 3, "nZ");
    assert_instructions(&mut cpu, "LDA #$FF\nTAX", 0xFF, 0xFF, 0, 3, "Nz");
}

#[test]
fn process_tay() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "LDA #1\nTAY", 1, 0, 1, 3, "nz");
    assert_instructions(&mut cpu, "LDA #0\nTAY", 0, 0, 0, 3, "nZ");
    assert_instructions(&mut cpu, "LDA #$FF\nTAY", 0xFF, 0, 0xFF, 3, "Nz");
}

#[test]
fn process_tsx() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "TSX", 0, 0xFD, 0, 1, "Nz");
}

#[test]
fn process_txa() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "LDX #1\nTXA", 1, 1, 0, 3, "nz");
    assert_instructions(&mut cpu, "LDX #0\nTXA", 0, 0, 0, 3, "nZ");
    assert_instructions(&mut cpu, "LDX #$FF\nTXA", 0xFF, 0xFF, 0, 3, "Nz");
}

#[test]
fn process_txs() {
    let mut cpu = Cpu::new(0);

    let mut bus = build_bus("LDX #1\nTXS");

    run(&mut cpu, &mut bus);

    assert_that!(cpu.SP, eq(1));
}

#[test]
fn process_tya() {
    let mut cpu = Cpu::new(0);

    assert_instructions(&mut cpu, "LDY #1\nTYA", 1, 0, 1, 3, "nz");
    assert_instructions(&mut cpu, "LDY #0\nTYA", 0, 0, 0, 3, "nZ");
    assert_instructions(&mut cpu, "LDY #$FF\nTYA", 0xFF, 0, 0xFF, 3, "Nz");
}

fn assert_address<Bus: BusTrait>(cpu: &mut Cpu, bus: &mut Bus, a: Byte, x: Byte, y: Byte, pc: Word, expected_status: &str, expected_cycles: usize) {
    cpu.reset(bus);

    let total_cycles = run(cpu, bus);

    println!("Cycles: {}", total_cycles);

    assert_that!(cpu.A, eq(a));
    assert_that!(cpu.X, eq(x));
    assert_that!(cpu.Y, eq(y));
    assert_that!(cpu.PC, eq(pc));
    assert_status(cpu.status.clone(), expected_status);
    assert_that!(total_cycles, eq(expected_cycles));
}

fn assert_registers(cpu: &mut Cpu, source: &str, a: Byte, x: Byte, y: Byte, expected_status: &str) {
    process_source(cpu, source, &[]);

    assert_that!(cpu.A, eq(a));
    assert_that!(cpu.X, eq(x));
    assert_that!(cpu.Y, eq(y));
    assert_status(cpu.status.clone(), expected_status);
}

fn assert_status(status: Status, flags: &str) {
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

fn assert_instructions(cpu: &mut Cpu, source: &str, a: Byte, x: Byte, y: Byte, pc: Word, expected_status: &str) {
    let _total_cycles = process_source(cpu, source, &[]);

    assert_that!(cpu.A, eq(a));
    assert_that!(cpu.X, eq(x));
    assert_that!(cpu.Y, eq(y));
    assert_that!(cpu.PC, eq(pc));
    assert_status(cpu.status.clone(), expected_status);
}

fn assert_absolute_instructions(cpu: &mut Cpu, source: &str, ram: &[u8], a: Byte, x: Byte, y: Byte, pc: Word, expected_status: &str) {
    let _total_cycles = process_source(cpu, source, ram);

    assert_that!(cpu.A, eq(a));
    assert_that!(cpu.X, eq(x));
    assert_that!(cpu.Y, eq(y));
    assert_that!(cpu.PC, eq(pc));
    assert_status(cpu.status.clone(), expected_status);
}

fn build_bus(source: &str) -> SimpleBus {
    let program = build_program(source);

    let mut bus = SimpleBus::default();
    bus.load(program.as_slice(), 0);

    bus
}

fn build_program(source: &str) -> Vec<Byte> {
    println!("Processing:\n{}\n", source);
    let assembler = Assembler::default();
    let parser = Parser::default();
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

fn process_source(cpu: &mut Cpu, source: &str, data: &[u8]) -> usize {
    let mut bus = build_bus(source);
    bus.load(data, 0x8000);

    run(cpu, &mut bus)
}

fn run<Bus: BusTrait>(cpu: &mut Cpu, bus: &mut Bus) -> usize {
    cpu.reset(bus);

    let mut next_op_code = bus.read_byte(cpu.PC);
    let mut total_cycles = 0;

    while next_op_code != 0xEA {
        let cycles = cpu.process(bus);
        total_cycles += cycles;
        next_op_code = bus.read_byte(cpu.PC);
    }
    total_cycles
}
