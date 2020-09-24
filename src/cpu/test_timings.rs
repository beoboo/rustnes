use hamcrest2::core::*;
use hamcrest2::prelude::*;

use crate::assembler::Assembler;
use crate::parser::Parser;

use super::*;

struct MockBus {
    data: Vec<u8>,
}

fn replace_slice<T>(source: &mut [T], from: &[T])
    where
        T: Clone + PartialEq,
{
    source[..from.len()].clone_from_slice(from);
}

impl MockBus {
    fn new() -> MockBus {
        MockBus {
            data: vec![0; 0xFFFF],
        }
    }

    fn load(&mut self, program: Vec<u8>, starting_pos: usize) {
        // let mut data = self.data;
        replace_slice(&mut self.data[starting_pos..], program.as_slice());
    }
}

impl BusTrait for MockBus {
    fn read_byte(&self, address: Word) -> Byte {
        let address = address as usize;

        let data = self.data[address];
        println!("Reading from {:#06X} -> {:#04X}", address, data);
        data
    }

    fn write_byte(&mut self, address: Word, data: Byte) {
        println!("Writing {:#04X} to {:#06X}", data, address);
        let address = address as usize;

        self.data[address] = data
    }
}

#[test]
fn process_adc() {
    assert_instruction("ADC #$44", 0x69, 2, 2);
    assert_instruction("ADC $44", 0x65, 2, 3);
    assert_instruction("ADC $44,X", 0x75, 2, 4);
    assert_instruction("ADC $4400", 0x6D, 3, 4);
    assert_instruction("ADC $4400,X", 0x7D, 3, 4);
    assert_instruction("ADC $4400,Y", 0x79, 3, 4);
    assert_instruction("ADC ($44,X)", 0x61, 2, 6);
    assert_instruction("ADC ($44),Y", 0x71, 2, 5);

    assert_instruction_with_page_cross("ADC $44FF,X", 0xFF, 0, 0x7D, 3, 5);
    assert_instruction_with_page_cross("ADC $44FF,Y", 0, 0xFF, 0x79, 3, 5);
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

fn assert_instruction(source: &str, expected_op_code: Byte, expected_length: usize, expected_cycles: usize) {
    let mut cpu = Cpu::new(0);
    let mut bus = MockBus::new();
    let program = build_program(source);
    let length = program.len();
    let op_code = program[0];

    bus.load(program, 0);

    let total_cycles = cpu.process(&mut bus);
    println!("Cycles: {}", total_cycles);

    assert_that!(op_code, equal_to(expected_op_code));
    assert_that!(length, equal_to(expected_length));
    assert_that!(total_cycles, equal_to(expected_cycles));
}

fn assert_branch(source: &str, status: &str, expected_op_code: Byte, expected_length: usize, expected_cycles: usize) {
    let mut cpu = Cpu::new(0);
    cpu.status = build_status(status);

    let mut bus = MockBus::new();

    let program = build_program(source);
    let length = program.len();
    let op_code = program[0];

    bus.load(program, 0);

    let total_cycles = cpu.process(&mut bus);
    println!("Cycles: {}", total_cycles);

    assert_that!(op_code, equal_to(expected_op_code));
    assert_that!(length, equal_to(expected_length));
    assert_that!(total_cycles, equal_to(expected_cycles));
}

fn assert_branch_with_page_cross(source: &str, pc: Word, status: &str, expected_op_code: Byte, expected_length: usize, expected_cycles: usize) {
    let mut cpu = Cpu::new(0);
    cpu.PC = pc;
    cpu.status = build_status(status);

    let mut bus = MockBus::new();

    let program = build_program(source);
    let length = program.len();
    let op_code = program[0];

    bus.load(program, pc as usize);

    let total_cycles = cpu.process(&mut bus);
    println!("Cycles: {}", total_cycles);

    assert_that!(op_code, equal_to(expected_op_code));
    assert_that!(length, equal_to(expected_length));
    assert_that!(total_cycles, equal_to(expected_cycles));
}

fn assert_instruction_with_page_cross(source: &str, x: Byte, y: Byte, expected_op_code: Byte, expected_length: usize, expected_cycles: usize) {
    let mut cpu = Cpu::new(0);
    cpu.X = x;
    cpu.Y = y;

    let mut bus = MockBus::new();
    let program = build_program(source);
    let length = program.len();
    let op_code = program[0];

    bus.load(program, 0);

    let total_cycles = cpu.process(&mut bus);
    println!("Cycles: {}", total_cycles);

    assert_that!(op_code, equal_to(expected_op_code));
    assert_that!(length, equal_to(expected_length));
    assert_that!(total_cycles, equal_to(expected_cycles));
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

    program.data
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
