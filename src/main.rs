use std::time::Duration;

use clap::{App, Arg};
use crossterm::ErrorKind;
use crossterm::event::{Event, KeyCode, poll, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use log::LevelFilter;
use log::*;

use crate::colored_log::formatted_builder;
use rustnes_lib::rom::Rom;
use rustnes_lib::bus::bus_impl::BusImpl;
use rustnes_lib::ram::Ram;
use rustnes_lib::apu::Apu;
use rustnes_lib::ppu::Ppu;
use rustnes_lib::bus::Bus;
use rustnes_lib::cpu::Cpu;

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
    let mut bus = BusImpl::new(Ram::new(0x0800), Apu::default(), Ppu::default(), rom);

    let mut cpu = Cpu::new(0);
    cpu.reset(&mut bus);

    let mut paused = true;

    enable_raw_mode()?;
    loop {
        // Wait up to 1oms for another event
        if poll(Duration::from_millis(10))? {
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

            if event == Event::Key(KeyCode::Char('p').into()) {
                paused = true;
            }

            if event == Event::Key(KeyCode::Char('c').into()) {
                paused = false;
            }
        } else {
            if !paused {
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
