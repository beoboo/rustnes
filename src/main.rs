use clap::{App, Arg};

use crate::apu::Apu;
use crate::bus::{Bus, BusImpl};
use crate::cpu::Cpu;
use crate::error::Error;
use crate::ppu::Ppu;
use crate::ram::Ram;
use crate::rom::Rom;

mod addressing_mode;
mod apu;
mod assembler;
mod bus;
mod cpu;
mod nes;
mod error;
mod parser;
mod ppu;
mod ram;
mod rom;
mod token;
mod types;

fn main() -> Result<(), Error> {
    let matches = App::new("rustnes")
        .arg(Arg::with_name("filename")
            .takes_value(true)
            .help("File to run")
        ).get_matches();

    if let Some(filename) = matches.value_of("filename") {
        run(filename)?;
    }

    Ok(())
}

fn run(filename: &str) -> Result<usize, Error> {
    let rom = Rom::load(filename, 16384, 8192);
    let mut bus = BusImpl::new(Ram::new(0x0800), Apu::new(), Ppu::new(), rom);

    let start = bus.read_word(0xFFFC);
    println!("Starting address: {:#06X}", start);

    let mut cpu = Cpu::new(start);

    let mut cycles = cpu.process(&mut bus);
    let mut total_cycles = cycles;

    while cycles != 0 {
        cycles = cpu.process(&mut bus);
        total_cycles += cycles;
    }

    Ok(total_cycles)
}

