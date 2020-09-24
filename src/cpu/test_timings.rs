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

    fn load(&mut self, program: Vec<u8>) {
        // let mut data = self.data;
        replace_slice(&mut self.data[..], program.as_slice());
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

fn assert_instruction(source: &str, expected_op_code: Byte, expected_length: usize, expected_cycles: usize) {
    let mut cpu = Cpu::new(0);
    let mut bus = MockBus::new();
    let program = build_program(source);
    let length = program.len();
    let op_code = program[0];

    bus.load(program);

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

    bus.load(program);

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
