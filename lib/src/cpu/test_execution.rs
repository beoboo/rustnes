use hamcrest2::core::*;
use hamcrest2::prelude::*;

use crate::assembler::Assembler;
use crate::bus::simple_bus::SimpleBus;
use crate::parser::Parser;

use super::*;

#[test]
fn simple_tick() {
    let mut cpu = Cpu::new(0);
    let mut bus = SimpleBus::default();

    let program = build_program("NOP");
    bus.load(program.as_slice(), 0);
    cpu.tick(&mut bus);
    assert_that!(cpu.PC, eq(0x01));
    assert_that!(cpu.left_cycles, eq(1));

    cpu.tick(&mut bus);
    assert_that!(cpu.PC, eq(0x01));
    assert_that!(cpu.left_cycles, eq(0));
}

fn build_program(source: &str) -> Vec<Byte> {
    println!("Processing:\n {}", source);
    let assembler = Assembler::default();
    let parser = Parser::default();
    let tokens = parser.parse(source).unwrap();

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
