use crate::cpu::Cpu;
use crate::rom::Rom;
use crate::instructions_map::InstructionsMap;
use crate::instruction::Instruction;
use crate::op_code::OpCode;

#[cfg(test)]
#[macro_use]
extern crate hamcrest2;

mod cpu;
mod rom;
mod instructions_map;
mod instruction;
mod op_code;

fn main() {
    let cpu = Cpu::new();
}
