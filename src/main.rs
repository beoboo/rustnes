use crate::cpu::Cpu;

#[cfg(test)]
#[macro_use]
extern crate hamcrest2;

mod assembler;
mod addressing_mode;
mod bus;
mod cpu;
mod error;
mod parser;
mod rom;
mod token;
mod types;
mod ppu;
mod ram;

fn main() {
    let _cpu = Cpu::new();
}
