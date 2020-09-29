use hamcrest2::core::*;
use hamcrest2::prelude::*;

use crate::assembler::Assembler;
use crate::parser::Parser;
use env_logger;

use super::*;
use crate::bus::simple_bus::SimpleBus;

#[test]
fn process_adc() {
    let _ = env_logger::init();
    // assert_instruction("ADC #$44", 0x69, 2, 2);
    // assert_instruction("ADC $44", 0x65, 2, 3);
    // assert_instruction("ADC $44,X", 0x75, 2, 4);
    // assert_instruction("ADC $4400", 0x6D, 3, 4);
    // assert_instruction("ADC $4400,X", 0x7D, 3, 4);
    // assert_instruction("ADC $4400,Y", 0x79, 3, 4);
    // assert_instruction("ADC ($44,X)", 0x61, 2, 6);
    // assert_instruction("ADC ($44),Y", 0x71, 2, 5);
    //
    // assert_instruction_with_page_cross("ADC $44FF,X", 0xFF, 0, 0x7D, 3, 5);
    // assert_instruction_with_page_cross("ADC $44FF,Y", 0, 0xFF, 0x79, 3, 5);
    assert_instruction_with_page_cross("ADC ($44),Y", 0, 0xFF, 0x71, 2, 6);
}

#[test]
fn process_and() {
    assert_instruction("AND #$44", 0x29, 2, 2);
    assert_instruction("AND $44", 0x25, 2, 3);
    assert_instruction("AND $44,X", 0x35, 2, 4);
    assert_instruction("AND $4400", 0x2D, 3, 4);
    assert_instruction("AND $4400,X", 0x3D, 3, 4);
    assert_instruction("AND $4400,Y", 0x39, 3, 4);
    assert_instruction("AND ($44,X)", 0x21, 2, 6);
    assert_instruction("AND ($44),Y", 0x31, 2, 5);

    assert_instruction_with_page_cross("AND $44FF,X", 0xFF, 0, 0x3D, 3, 5);
    assert_instruction_with_page_cross("AND $44FF,Y", 0, 0xFF, 0x39, 3, 5);
    assert_instruction_with_page_cross("AND ($44),Y", 0, 0xFF, 0x31, 2, 6);
}

#[test]
fn process_asl() {
    assert_instruction("ASL A", 0x0A, 1, 2);
    assert_instruction("ASL $44", 0x06, 2, 5);
    assert_instruction("ASL $44,X", 0x16, 2, 6);
    assert_instruction("ASL $4400", 0x0E, 3, 6);
    assert_instruction("ASL $4400,X", 0x1E, 3, 7);
}

#[test]
fn process_bit() {
    assert_instruction("BIT $44", 0x24, 2, 3);
    assert_instruction("BIT $4400", 0x2C, 3, 4);
}

#[test]
fn process_branching_instructions() {
    // Not taken
    assert_branch("BPL $2", "N", 0x10, 2, 2);
    assert_branch("BMI $2", "n", 0x30, 2, 2);
    assert_branch("BVC $2", "V", 0x50, 2, 2);
    assert_branch("BVS $2", "v", 0x70, 2, 2);
    assert_branch("BCC $2", "C", 0x90, 2, 2);
    assert_branch("BCS $2", "c", 0xB0, 2, 2);
    assert_branch("BNE $2", "Z", 0xD0, 2, 2);
    assert_branch("BEQ $2", "z", 0xF0, 2, 2);

    // Taken
    assert_branch("BPL $2", "n", 0x10, 2, 3);
    assert_branch("BMI $2", "N", 0x30, 2, 3);
    assert_branch("BVC $2", "v", 0x50, 2, 3);
    assert_branch("BVS $2", "V", 0x70, 2, 3);
    assert_branch("BCC $2", "c", 0x90, 2, 3);
    assert_branch("BCS $2", "C", 0xB0, 2, 3);
    assert_branch("BNE $2", "z", 0xD0, 2, 3);
    assert_branch("BEQ $2", "Z", 0xF0, 2, 3);

    // Taken with page cross
    assert_branch_with_page_cross("BPL $80", 0x00CF, "n", 0x10, 2, 4);
    assert_branch_with_page_cross("BMI $80", 0x00CF, "N", 0x30, 2, 4);
    assert_branch_with_page_cross("BVC $80", 0x00CF, "v", 0x50, 2, 4);
    assert_branch_with_page_cross("BVS $80", 0x00CF, "V", 0x70, 2, 4);
    assert_branch_with_page_cross("BCC $80", 0x00CF, "c", 0x90, 2, 4);
    assert_branch_with_page_cross("BCS $80", 0x00CF, "C", 0xB0, 2, 4);
    assert_branch_with_page_cross("BNE $80", 0x00CF, "z", 0xD0, 2, 4);
    assert_branch_with_page_cross("BEQ $80", 0x00CF, "Z", 0xF0, 2, 4);
}

#[test]
fn process_brk() {
    assert_instruction("BRK", 0x00, 1, 7);
}

#[test]
fn process_cmp() {
    assert_instruction("CMP #$44", 0xC9, 2, 2);
    assert_instruction("CMP $44", 0xC5, 2, 3);
    assert_instruction("CMP $44,X", 0xD5, 2, 4);
    assert_instruction("CMP $4400", 0xCD, 3, 4);
    assert_instruction("CMP $4400,X", 0xDD, 3, 4);
    assert_instruction("CMP $4400,Y", 0xD9, 3, 4);
    assert_instruction("CMP ($44,X)", 0xC1, 2, 6);
    assert_instruction("CMP ($44),Y", 0xD1, 2, 5);

    assert_instruction_with_page_cross("CMP $44FF,X", 0xFF, 0, 0xDD, 3, 5);
    assert_instruction_with_page_cross("CMP $44FF,Y", 0, 0xFF, 0xD9, 3, 5);
    assert_instruction_with_page_cross("CMP ($44),Y", 0, 0xFF, 0xD1, 2, 6);
}

#[test]
fn process_cpx() {
    assert_instruction("CPX #$44", 0xE0, 2, 2);
    assert_instruction("CPX $44", 0xE4, 2, 3);
    assert_instruction("CPX $4400", 0xEC, 3, 4);
}

#[test]
fn process_cpy() {
    assert_instruction("CPY #$44", 0xC0, 2, 2);
    assert_instruction("CPY $44", 0xC4, 2, 3);
    assert_instruction("CPY $4400", 0xCC, 3, 4);
}

#[test]
fn process_dec() {
    assert_instruction("DEC $44", 0xC6, 2, 5);
    assert_instruction("DEC $44,X", 0xD6, 2, 6);
    assert_instruction("DEC $4400", 0xCE, 3, 6);
    assert_instruction("DEC $4400,X", 0xDE, 3, 7);
}

#[test]
fn process_eor() {
    assert_instruction("EOR #$44", 0x49, 2, 2);
    assert_instruction("EOR $44", 0x45, 2, 3);
    assert_instruction("EOR $44,X", 0x55, 2, 4);
    assert_instruction("EOR $4400", 0x4D, 3, 4);
    assert_instruction("EOR $4400,X", 0x5D, 3, 4);
    assert_instruction("EOR $4400,Y", 0x59, 3, 4);
    assert_instruction("EOR ($44,X)", 0x41, 2, 6);
    assert_instruction("EOR ($44),Y", 0x51, 2, 5);

    assert_instruction_with_page_cross("EOR $44FF,X", 0xFF, 0, 0x5D, 3, 5);
    assert_instruction_with_page_cross("EOR $44FF,Y", 0, 0xFF, 0x59, 3, 5);
    assert_instruction_with_page_cross("EOR ($44),Y", 0, 0xFF, 0x51, 2, 6);
}

#[test]
fn process_flags() {
    assert_instruction("CLC", 0x18, 1, 2);
    assert_instruction("SEC", 0x38, 1, 2);
    assert_instruction("CLI", 0x58, 1, 2);
    assert_instruction("SEI", 0x78, 1, 2);
    assert_instruction("CLV", 0xB8, 1, 2);
    assert_instruction("CLD", 0xD8, 1, 2);
    assert_instruction("SED", 0xF8, 1, 2);
}

#[test]
fn process_inc() {
    assert_instruction("INC $44", 0xE6, 2, 5);
    assert_instruction("INC $44,X", 0xF6, 2, 6);
    assert_instruction("INC $4400", 0xEE, 3, 6);
    assert_instruction("INC $4400,X", 0xFE, 3, 7);
}

#[test]
fn process_jmp() {
    assert_instruction("JMP $5597", 0x4C, 3, 3);
    assert_instruction("JMP ($5597)", 0x6C, 3, 5);
}

#[test]
fn process_jsr() {
    assert_instruction("JSR $5597", 0x20, 3, 6);
}

#[test]
fn process_lda() {
    assert_instruction("LDA #$44", 0xA9, 2, 2);
    assert_instruction("LDA $44", 0xA5, 2, 3);
    assert_instruction("LDA $44,X", 0xB5, 2, 4);
    assert_instruction("LDA $4400", 0xAD, 3, 4);
    assert_instruction("LDA $4400,X", 0xBD, 3, 4);
    assert_instruction("LDA $4400,Y", 0xB9, 3, 4);
    assert_instruction("LDA ($44,X)", 0xA1, 2, 6);
    assert_instruction("LDA ($44),Y", 0xB1, 2, 5);

    assert_instruction_with_page_cross("LDA $44FF,X", 0xFF, 0, 0xBD, 3, 5);
    assert_instruction_with_page_cross("LDA $44FF,Y", 0, 0xFF, 0xB9, 3, 5);
    assert_instruction_with_page_cross("LDA ($44),Y", 0, 0xFF, 0xB1, 2, 6);
}

#[test]
fn process_ldx() {
    assert_instruction("LDX #$44", 0xA2, 2, 2);
    assert_instruction("LDX $44", 0xA6, 2, 3);
    assert_instruction("LDX $44,Y", 0xB6, 2, 4);
    assert_instruction("LDX $4400", 0xAE, 3, 4);
    assert_instruction("LDX $4400,Y", 0xBE, 3, 4);

    assert_instruction_with_page_cross("LDX $44FF,Y", 0, 0xFF, 0xBE, 3, 5);
}

#[test]
fn process_ldy() {
    assert_instruction("LDY #$44", 0xA0, 2, 2);
    assert_instruction("LDY $44", 0xA4, 2, 3);
    assert_instruction("LDY $44,X", 0xB4, 2, 4);
    assert_instruction("LDY $4400", 0xAC, 3, 4);
    assert_instruction("LDY $4400,X", 0xBC, 3, 4);

    assert_instruction_with_page_cross("LDY $44FF,X", 0xFF, 0, 0xBC, 3, 5);
}

#[test]
fn process_lsr() {
    assert_instruction("LSR A", 0x4A, 1, 2);
    assert_instruction("LSR $44", 0x46, 2, 5);
    assert_instruction("LSR $44,X", 0x56, 2, 6);
    assert_instruction("LSR $4400", 0x4E, 3, 6);
    assert_instruction("LSR $4400,X", 0x5E, 3, 7);
    //
    // assert_instruction_with_page_cross("LSR $44FF,X", 0xFF, 0, 0x5E, 3, 8);
}

#[test]
fn process_nop() {
    assert_instruction("NOP", 0xEA, 1, 2);
}

#[test]
fn process_ora() {
    assert_instruction("ORA #$44", 0x09, 2, 2);
    assert_instruction("ORA $44", 0x05, 2, 3);
    assert_instruction("ORA $44,X", 0x15, 2, 4);
    assert_instruction("ORA $4400", 0x0D, 3, 4);
    assert_instruction("ORA $4400,X", 0x1D, 3, 4);
    assert_instruction("ORA $4400,Y", 0x19, 3, 4);
    assert_instruction("ORA ($44,X)", 0x01, 2, 6);
    assert_instruction("ORA ($44),Y", 0x11, 2, 5);

    assert_instruction_with_page_cross("ORA $44FF,X", 0xFF, 0, 0x1D, 3, 5);
    assert_instruction_with_page_cross("ORA $44FF,Y", 0, 0xFF, 0x19, 3, 5);
    assert_instruction_with_page_cross("ORA ($44),Y", 0, 0xFF, 0x11, 2, 6);
}

#[test]
fn process_register() {
    assert_instruction("TAX", 0xAA, 1, 2);
    assert_instruction("TXA", 0x8A, 1, 2);
    assert_instruction("DEX", 0xCA, 1, 2);
    assert_instruction("INX", 0xE8, 1, 2);
    assert_instruction("TAY", 0xA8, 1, 2);
    assert_instruction("TYA", 0x98, 1, 2);
    assert_instruction("DEY", 0x88, 1, 2);
    assert_instruction("INY", 0xC8, 1, 2);
}

#[test]
fn process_rol() {
    assert_instruction("ROL A", 0x2A, 1, 2);
    assert_instruction("ROL $44", 0x26, 2, 5);
    assert_instruction("ROL $44,X", 0x36, 2, 6);
    assert_instruction("ROL $4400", 0x2E, 3, 6);
    assert_instruction("ROL $4400,X", 0x3E, 3, 7);
}

#[test]
fn process_ror() {
    assert_instruction("ROR A", 0x6A, 1, 2);
    assert_instruction("ROR $44", 0x66, 2, 5);
    assert_instruction("ROR $44,X", 0x76, 2, 6);
    assert_instruction("ROR $4400", 0x6E, 3, 6);
    assert_instruction("ROR $4400,X", 0x7E, 3, 7);
}

#[test]
fn process_rti() {
    assert_instruction("RTI", 0x40, 1, 6);
}

#[test]
fn process_rts() {
    assert_instruction("RTS", 0x60, 1, 6);
}

#[test]
fn process_sbc() {
    assert_instruction("SBC #$44", 0xE9, 2, 2);
    assert_instruction("SBC $44", 0xE5, 2, 3);
    assert_instruction("SBC $44,X", 0xF5, 2, 4);
    assert_instruction("SBC $4400", 0xED, 3, 4);
    assert_instruction("SBC $4400,X", 0xFD, 3, 4);
    assert_instruction("SBC $4400,Y", 0xF9, 3, 4);
    assert_instruction("SBC ($44,X)", 0xE1, 2, 6);
    assert_instruction("SBC ($44),Y", 0xF1, 2, 5);

    assert_instruction_with_page_cross("SBC $44FF,X", 0xFF, 0, 0xFD, 3, 5);
    assert_instruction_with_page_cross("SBC $44FF,Y", 0, 0xFF, 0xF9, 3, 5);
    assert_instruction_with_page_cross("SBC ($44),Y", 0, 0xFF, 0xF1, 2, 6);
}

#[test]
fn process_sta() {
    assert_instruction("STA $44", 0x85, 2, 3);
    assert_instruction("STA $44,X", 0x95, 2, 4);
    assert_instruction("STA $4400", 0x8D, 3, 4);
    assert_instruction("STA $4400,X", 0x9D, 3, 5);
    assert_instruction("STA $4400,Y", 0x99, 3, 5);
    assert_instruction("STA ($44,X)", 0x81, 2, 6);
    assert_instruction("STA ($44),Y", 0x91, 2, 6);
}

#[test]
fn process_stacks() {
    assert_instruction("TXS", 0x9A, 1, 2);
    assert_instruction("TSX", 0xBA, 1, 2);
    assert_instruction("PHA", 0x48, 1, 3);
    assert_instruction("PLA", 0x68, 1, 4);
    assert_instruction("PHP", 0x08, 1, 3);
    assert_instruction("PLP", 0x28, 1, 4);
}

#[test]
fn process_stx() {
    assert_instruction("STX $44", 0x86, 2, 3);
    assert_instruction("STX $44,Y", 0x96, 2, 4);
    assert_instruction("STX $4400", 0x8E, 3, 4);
}

#[test]
fn process_sty() {
    assert_instruction("STY $44", 0x84, 2, 3);
    assert_instruction("STY $44,X", 0x94, 2, 4);
    assert_instruction("STY $4400", 0x8C, 3, 4);
}

fn assert_instruction(source: &str, expected_op_code: Byte, expected_length: usize, expected_cycles: usize) {
    let mut cpu = Cpu::new(0);

    let mut bus = SimpleBus::default();

    _assert_instruction(&mut cpu, &mut bus, source, 0, expected_op_code, expected_length, expected_cycles);
}

fn assert_branch(source: &str, status: &str, expected_op_code: Byte, expected_length: usize, expected_cycles: usize) {
    let mut cpu = Cpu::new(0);
    cpu.status = Status::from_string(status);

    let mut bus = SimpleBus::default();

    _assert_instruction(&mut cpu, &mut bus, source, 0, expected_op_code, expected_length, expected_cycles);
}

fn assert_branch_with_page_cross(source: &str, pc: Word, status: &str, expected_op_code: Byte, expected_length: usize, expected_cycles: usize) {
    let mut cpu = Cpu::new(pc);
    cpu.status = Status::from_string(status);

    let mut bus = SimpleBus::default();

    _assert_instruction(&mut cpu, &mut bus, source, pc, expected_op_code, expected_length, expected_cycles);
}

fn assert_instruction_with_page_cross(source: &str, x: Byte, y: Byte, expected_op_code: Byte, expected_length: usize, expected_cycles: usize) {
    let mut cpu = Cpu::new(0);
    cpu.X = x;
    cpu.Y = y;

    let mut bus = SimpleBus::default();
    bus.write_word(0x0044, 0x00AB);

    _assert_instruction(&mut cpu, &mut bus, source, 0, expected_op_code, expected_length, expected_cycles);
}

fn _assert_instruction(cpu: &mut Cpu, bus: &mut SimpleBus, source: &str, pc: Word, expected_op_code: Byte, expected_length: usize, expected_cycles: usize) {
    let program = build_program(source);
    let length = program.len();
    let op_code = program[0];

    bus.load(program, pc as usize);

    let total_cycles = cpu.process(bus);
    println!("Cycles: {}", total_cycles);

    assert_that!(op_code, equal_to(expected_op_code));
    assert_that!(length, equal_to(expected_length));
    assert_that!(total_cycles, equal_to(expected_cycles));
}

fn build_program(source: &str) -> Vec<Byte> {
    println!("Processing:\n {}", source);
    let assembler = Assembler::default();
    let parser = Parser::default();
    let tokens = parser.parse(source).unwrap();
    // println!("Tokens: {:?}", tokens);

    let program = match assembler.assemble(tokens) {
        Ok(program) => program,
        Err(e) => panic!("Assembler error: {}", e)
    };
    println!("Program: {:x?}", program);

    program.data
}

