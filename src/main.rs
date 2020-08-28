use crate::cpu::Cpu;
use crate::rom::Rom;
use crate::instructions_map::InstructionsMap;
use crate::instruction::Instruction;

#[cfg(test)]
#[macro_use]
extern crate hamcrest2;

mod cpu;
mod rom;
mod instructions_map;
mod instruction;

fn main() {
    let cpu = Cpu::new();
}
