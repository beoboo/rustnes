use std::io::{stdout, Write};
use std::time::Duration;

use clap::{App, Arg};
use crossterm::cursor::position;
use crossterm::ErrorKind;
use crossterm::event::{Event, KeyCode, poll, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use log::LevelFilter;
use log::*;

use crate::apu::Apu;
use crate::bus::Bus;
use crate::bus::bus_impl::BusImpl;
use crate::colored_log::formatted_builder;
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
mod colored_log;

fn main() -> Result<(), ErrorKind> {
    formatted_builder()
        // .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter(Some("rustnes"), LevelFilter::Trace).init();

    // trace!("trace");
    // debug!("debug");
    // info!("info");
    // warn!("warning");
    // error!("error");

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

fn run(filename: &str) -> Result<(), ErrorKind> {
    let rom = Rom::load(filename, 16384, 8192);
    let mut bus = BusImpl::new(Ram::new(0x0800), Apu::new(), Ppu::new(), rom);

    let start = bus.read_word(0xFFFC);

    enable_raw_mode()?;

    info!("Starting address: {:#06X}\r", start);

    let mut cpu = Cpu::new(start);

    loop {

        // Wait up to 1s for another event
        if poll(Duration::from_millis(1_000))? {
            // It's guaranteed that read() wont block if `poll` returns `Ok(true)`
            let event = read()?;

            // println!("Event::{:?}\r", event);

            if event == Event::Key(KeyCode::Char('q').into()) {
                break;
            }

            if event == Event::Key(KeyCode::Char(' ').into()) {
                disable_raw_mode()?;
                tick(&mut cpu, &mut bus);
                enable_raw_mode()?;
            }
        }
    }

    disable_raw_mode()?;

    Ok(())
}

fn tick<B: Bus>(cpu: &mut Cpu, bus: &mut B) {
    cpu.process(bus);
}
