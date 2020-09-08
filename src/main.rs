use crate::cpu::Cpu;

#[cfg(test)]
#[macro_use]
extern crate hamcrest2;

mod assembler;
mod bus;
mod cpu;
mod rom;
mod instructions_map;
mod instruction;
mod op_code;
mod types;

fn main() {
    let _cpu = Cpu::new();
}
