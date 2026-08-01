//! Blargg's test ROMs.
//!
//! These are built to be run headlessly, which is why no screen is needed here: the ROM writes a
//! status byte to `$6000` and a NUL-terminated message at `$6004`. A magic signature at
//! `$6001..=$6003` marks the protocol as active, so a runner can tell "still starting up" from
//! "this ROM does not use the protocol at all".
//!
//! ```text
//! $6000  $80        still running
//!        $81        needs a reset (some multi-stage ROMs)
//!        $00        passed
//!        other      failed, and the byte is the error code
//! ```

use std::path::Path;

use anyhow::{bail, Context, Result};
use rn_core::{cartridge::load_rom, memory::Addressable, system::NesSystem};

const STATUS: u16 = 0x6000;
const SIGNATURE: u16 = 0x6001;
const MESSAGE: u16 = 0x6004;

/// The signature blargg's ROMs write to mark the protocol active.
const SIGNATURE_BYTES: [u8; 3] = [0xDE, 0xB0, 0x61];

const STATUS_RUNNING: u8 = 0x80;
const STATUS_NEEDS_RESET: u8 = 0x81;
const STATUS_PASSED: u8 = 0x00;

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Passed,
    Failed { code: u8 },
    /// Ran out of budget without the ROM reporting a result.
    TimedOut,
    /// The ROM never wrote the signature, so it does not use this protocol.
    NoProtocol,
}

pub struct Outcome {
    pub status: Status,
    pub message: String,
    pub instructions: usize,
}

/// Run a blargg-style ROM until it reports a result or the budget runs out.
pub fn run(rom_path: &Path, max_instructions: usize) -> Result<Outcome> {
    let rom = load_rom(rom_path)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| format!("loading {}", rom_path.display()))?;

    let mut system = NesSystem::new();
    system
        .load_rom(&rom)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("loading the ROM into the system")?;

    let mut signature_seen = false;

    for instruction in 0..max_instructions {
        if let Err(error) = system.step() {
            bail!(
                "emulation failed after {instruction} instructions at PC ${:04X}: {error}",
                system.cpu().pc()
            );
        }

        // Only trust $6000 once the signature is present: before that the location holds whatever
        // the RAM powered up with.
        if !signature_seen {
            signature_seen = read_signature(&system) == SIGNATURE_BYTES;
            continue;
        }

        match read(&system, STATUS) {
            STATUS_RUNNING | STATUS_NEEDS_RESET => continue,
            STATUS_PASSED => {
                return Ok(Outcome {
                    status: Status::Passed,
                    message: read_message(&system),
                    instructions: instruction,
                })
            },
            code => {
                return Ok(Outcome {
                    status: Status::Failed { code },
                    message: read_message(&system),
                    instructions: instruction,
                })
            },
        }
    }

    Ok(Outcome {
        status: if signature_seen { Status::TimedOut } else { Status::NoProtocol },
        message: if signature_seen { read_message(&system) } else { String::new() },
        instructions: max_instructions,
    })
}

fn read(system: &NesSystem, address: u16) -> u8 {
    system.cpu().read_byte(address).unwrap_or(0)
}

fn read_signature(system: &NesSystem) -> [u8; 3] {
    [
        read(system, SIGNATURE),
        read(system, SIGNATURE + 1),
        read(system, SIGNATURE + 2),
    ]
}

/// Read the NUL-terminated message the ROM leaves at `$6004`.
fn read_message(system: &NesSystem) -> String {
    let mut text = String::new();

    // Bounded: a corrupt or absent terminator must not read the whole address space.
    for offset in 0..512u16 {
        match read(system, MESSAGE + offset) {
            0 => break,
            byte => text.push(byte as char),
        }
    }

    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an iNES image whose program runs `prg`, entered from the reset vector.
    ///
    /// `name` keeps temp files distinct, since tests run in parallel.
    fn synthesise(name: &str, prg: &[u8]) -> std::path::PathBuf {
        const BANK: usize = 16 * 1024;
        let mut image = Vec::new();
        image.extend_from_slice(b"NES\x1A");
        image.extend_from_slice(&[1, 1, 0x00, 0x00]);
        image.extend_from_slice(&[0; 8]);

        let mut bank = vec![0u8; BANK];
        bank[..prg.len()].copy_from_slice(prg);
        // Reset vector at $FFFC points at $C000, the start of the mirrored bank.
        bank[BANK - 4..BANK - 2].copy_from_slice(&0xC000u16.to_le_bytes());

        image.extend_from_slice(&bank);
        image.extend_from_slice(&vec![0u8; 8 * 1024]);

        let path = std::env::temp_dir().join(format!("rn_blargg_{name}.nes"));
        std::fs::write(&path, &image).expect("writing the test ROM");
        path
    }

    /// Emit `LDA #value / STA address`.
    fn store(program: &mut Vec<u8>, value: u8, address: u16) {
        let [low, high] = address.to_le_bytes();
        program.extend_from_slice(&[0xA9, value, 0x8D, low, high]);
    }

    /// A program following the real protocol's ordering, then spinning forever.
    ///
    /// The order matters and mirrors what blargg's ROMs actually do: mark "running" *before*
    /// publishing the signature, so a runner that starts sampling $6000 the moment the signature
    /// appears cannot read a stale byte and mistake uninitialised RAM (zero) for a pass.
    fn protocol_program(status: u8, message: &str) -> Vec<u8> {
        let mut program = Vec::new();
        store(&mut program, STATUS_RUNNING, STATUS);

        store(&mut program, SIGNATURE_BYTES[0], SIGNATURE);
        store(&mut program, SIGNATURE_BYTES[1], SIGNATURE + 1);
        store(&mut program, SIGNATURE_BYTES[2], SIGNATURE + 2);

        for (offset, byte) in message.bytes().enumerate() {
            store(&mut program, byte, MESSAGE + offset as u16);
        }
        store(&mut program, 0, MESSAGE + message.len() as u16); // NUL terminator

        store(&mut program, status, STATUS);

        // JMP to self. The target is this instruction's own address.
        let spin = 0xC000 + program.len() as u16;
        let [low, high] = spin.to_le_bytes();
        program.extend_from_slice(&[0x4C, low, high]);
        program
    }

    #[test]
    fn reports_a_passing_rom() {
        let path = synthesise("pass", &protocol_program(STATUS_PASSED, "01-basics\n\nPassed"));
        let outcome = run(&path, 100_000).expect("running the ROM");

        assert_eq!(outcome.status, Status::Passed);
        assert!(outcome.message.contains("Passed"), "message was {:?}", outcome.message);
    }

    #[test]
    fn reports_a_failing_rom_with_its_error_code() {
        let path = synthesise("fail", &protocol_program(0x03, "05-branches\n\nFailed #3"));
        let outcome = run(&path, 100_000).expect("running the ROM");

        assert_eq!(outcome.status, Status::Failed { code: 0x03 });
        assert!(outcome.message.contains("Failed"), "message was {:?}", outcome.message);
    }

    /// A ROM that never writes the signature must not be reported as passing just because $6000
    /// happens to hold zero — which is exactly what uninitialised RAM looks like.
    #[test]
    fn a_rom_without_the_signature_is_not_mistaken_for_a_pass() {
        let mut program = Vec::new();
        store(&mut program, 0x00, STATUS); // status byte, but no signature
        let spin = 0xC000 + program.len() as u16;
        let [low, high] = spin.to_le_bytes();
        program.extend_from_slice(&[0x4C, low, high]);

        let path = synthesise("nosig", &program);
        let outcome = run(&path, 10_000).expect("running the ROM");

        assert_eq!(outcome.status, Status::NoProtocol);
    }

    #[test]
    fn a_rom_that_never_reports_times_out() {
        // Signature written, but the status stays at "still running" forever.
        let path = synthesise("running", &protocol_program(STATUS_RUNNING, "still going"));
        let outcome = run(&path, 20_000).expect("running the ROM");

        assert_eq!(outcome.status, Status::TimedOut);
    }
}
