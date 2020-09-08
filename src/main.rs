use crate::cpu::Cpu;

#[cfg(test)]
#[macro_use]
extern crate hamcrest2;

mod assembler;
mod addressing;
mod bus;
mod cpu;
mod error;
mod instructions_map;
mod instruction;
mod lexer;
mod op_code;
mod parser;
mod rom;
mod token;
mod types;

fn main() {
    let _cpu = Cpu::new();
}
