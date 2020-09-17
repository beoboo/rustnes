use crate::cpu::Cpu;

#[cfg(test)]
#[macro_use]
extern crate hamcrest2;

mod addressing_mode;
mod apu;
mod assembler;
mod bus;
mod cpu;
mod error;
mod parser;
mod ppu;
mod ram;
mod rom;
mod token;
mod types;

fn main() {
    let _cpu = Cpu::new();
}
