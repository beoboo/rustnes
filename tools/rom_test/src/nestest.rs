//! `nestest` in log-diff mode.
//!
//! `nestest.nes` has an automated mode: start at `$C000` instead of the reset vector and it walks
//! every official opcode, then the unofficial ones, with no PPU involvement. Its author published
//! a cycle-exact log of the correct run, so instead of "something is wrong somewhere" a failure
//! becomes "line 4711, register X differs" — which is the whole reason to start here.
//!
//! A log line looks like:
//!
//! ```text
//! C000  4C F5 C5  JMP $C5F5                       A:00 X:00 Y:00 P:24 SP:FD PPU:  0, 21 CYC:7
//! ```
//!
//! Only the address and the register block are compared. The disassembly text is the log author's
//! formatting, not emulator state, and PPU/cycle columns need cycle-accurate timing this emulator
//! does not claim yet — comparing them would report thousands of failures that say nothing about
//! CPU correctness.

use std::{fmt, path::Path};

use anyhow::{bail, Context, Result};
use rn_core::{cartridge::load_rom, cpu::CpuRegisters, system::NesSystem};

/// nestest's automated entry point, bypassing the menu that needs a PPU.
const AUTOMATED_ENTRY: u16 = 0xC000;

/// The CPU state at the start of one instruction, as the log records it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub status: u8,
    pub sp: u8,
}

impl State {
    fn from_registers(registers: &CpuRegisters) -> Self {
        Self {
            pc: registers.pc,
            a: registers.a,
            x: registers.x,
            y: registers.y,
            status: registers.status,
            sp: registers.sp,
        }
    }

    /// Parse one line of `nestest.log`.
    fn parse(line: &str) -> Option<Self> {
        let field = |name: &str| -> Option<u8> {
            let start = line.find(name)? + name.len();
            u8::from_str_radix(line.get(start..start + 2)?, 16).ok()
        };

        Some(Self {
            pc: u16::from_str_radix(line.get(0..4)?, 16).ok()?,
            a: field("A:")?,
            x: field("X:")?,
            y: field("Y:")?,
            status: field("P:")?,
            sp: field("SP:")?,
        })
    }

    /// Names of the fields that differ, for a failure message that points at the cause.
    fn differences(&self, other: &Self) -> Vec<&'static str> {
        let mut fields = Vec::new();
        if self.pc != other.pc {
            fields.push("PC");
        }
        if self.a != other.a {
            fields.push("A");
        }
        if self.x != other.x {
            fields.push("X");
        }
        if self.y != other.y {
            fields.push("Y");
        }
        if self.status != other.status {
            fields.push("P");
        }
        if self.sp != other.sp {
            fields.push("SP");
        }
        fields
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04X}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.pc, self.a, self.x, self.y, self.status, self.sp
        )
    }
}

pub struct Outcome {
    pub instructions: usize,
    pub divergence: Option<Divergence>,
}

pub struct Divergence {
    pub line: usize,
    pub expected: State,
    pub actual: State,
    pub fields: Vec<&'static str>,
}

/// Run nestest against its golden log, stopping at the first divergence.
///
/// Stopping at the first difference is deliberate: once the CPU state is wrong, every subsequent
/// line is wrong too, and a thousand cascading failures hide the one that matters.
pub fn run(rom_path: &Path, log_path: &Path, max_instructions: usize) -> Result<Outcome> {
    let rom = load_rom(rom_path).map_err(|e| anyhow::anyhow!("{e:?}")).context("loading nestest.nes")?;

    let log = std::fs::read_to_string(log_path).context("reading nestest.log")?;
    let expected: Vec<(usize, State)> = log
        .lines()
        .enumerate()
        .filter_map(|(i, line)| State::parse(line).map(|state| (i + 1, state)))
        .collect();

    if expected.is_empty() {
        bail!("no parseable lines in the log — is this really nestest.log?");
    }

    let mut system = NesSystem::new();
    system
        .load_rom(&rom)
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .context("loading the ROM into the system")?;

    // Automated mode, and the documented initial state the log was captured with.
    system.cpu().set_pc(AUTOMATED_ENTRY);
    let mut registers = system.cpu().registers();
    registers.status = 0x24;
    registers.sp = 0xFD;
    system.cpu().set_registers(registers);

    let limit = max_instructions.min(expected.len());

    for (index, &(line, want)) in expected.iter().take(limit).enumerate() {
        let got = State::from_registers(&system.cpu().registers());

        if got != want {
            return Ok(Outcome {
                instructions: index,
                divergence: Some(Divergence {
                    line,
                    expected: want,
                    actual: got,
                    fields: want.differences(&got),
                }),
            });
        }

        if let Err(error) = system.step() {
            bail!("emulation failed at log line {line} (PC ${:04X}): {error}", got.pc);
        }
    }

    Ok(Outcome {
        instructions: limit,
        divergence: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real line from nestest.log, to pin the parser against the actual format.
    const SAMPLE: &str =
        "C000  4C F5 C5  JMP $C5F5                       A:00 X:00 Y:00 P:24 SP:FD PPU:  0, 21 CYC:7";

    #[test]
    fn parses_a_log_line() {
        let state = State::parse(SAMPLE).expect("the sample line should parse");
        assert_eq!(state.pc, 0xC000);
        assert_eq!(state.a, 0x00);
        assert_eq!(state.status, 0x24);
        assert_eq!(state.sp, 0xFD);
    }

    #[test]
    fn ignores_lines_that_are_not_log_entries() {
        assert!(State::parse("").is_none());
        assert!(State::parse("not a log line").is_none());
    }

    #[test]
    fn reports_which_fields_differ() {
        let a = State::parse(SAMPLE).expect("parse");
        let mut b = a;
        b.x = 0x05;
        b.status = 0x25;

        assert_eq!(a.differences(&b), vec!["X", "P"]);
    }
}
