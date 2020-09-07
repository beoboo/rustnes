use crate::cpu::Cpu;

#[cfg(test)]
extern crate hamcrest2;

mod bus;
mod cpu;
mod rom;
mod instructions_map;
mod instruction;
mod op_code;

fn main() {
    let _cpu = Cpu::new();
}
