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
use rn_core::{cartridge::load_rom, cpu::Disassembler, memory::Addressable, system::NesSystem};

use crate::screen;

const STATUS: u16 = 0x6000;
const SIGNATURE: u16 = 0x6001;
const MESSAGE: u16 = 0x6004;

/// The signature blargg's ROMs write to mark the protocol active.
const SIGNATURE_BYTES: [u8; 3] = [0xDE, 0xB0, 0x61];

const STATUS_RUNNING: u8 = 0x80;
/// The ROM is asking to be reset, having first held the status long enough to be seen.
///
/// Part of the protocol rather than a failure: several tests exist precisely to check what
/// survives a reset and what does not, and cannot run at all unless something presses the button.
const STATUS_NEEDS_RESET: u8 = 0x81;

/// How long to wait before resetting, in CPU cycles — roughly 100ms.
///
/// The delay is required, not politeness. A reset that arrives immediately can land before the ROM
/// has finished recording what it expects to survive, and the test then measures the harness
/// rather than the emulator.
const RESET_DELAY_CYCLES: u64 = 180_000;
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

/// Where a verdict came from.
///
/// Recorded and reported rather than flattened away, because the two are not equally trustworthy.
/// The `$6000` protocol is exact; the screen is this runner's reading of what a ROM drew, and if
/// that reading is ever wrong it should be obvious which results to doubt.
#[derive(Debug, PartialEq, Eq)]
pub enum Source {
    Protocol,
    Screen,
}

pub struct Outcome {
    pub status: Status,
    pub message: String,
    pub instructions: usize,
    pub source: Source,

    /// The distinct addresses the CPU was executing at when the budget ran out, in order, each
    /// with the instruction found there.
    ///
    /// A hung ROM is nearly always spinning in a handful of instructions, and knowing *which*
    /// turns "it hangs" into a place to look. Empty unless the run timed out.
    pub spinning_at: Vec<(u16, String)>,
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

    // The per-dot pixel path is off by default; this is how the suites are run against it.
    if std::env::var_os("RN_PER_DOT").is_some() {
        system.ppu().set_per_dot_pixels(true);
    }

    let mut signature_seen = false;
    // When the ROM asked to be reset, so the wait can be measured from that moment.
    let mut reset_requested_at: Option<u64> = None;

    // A rolling window of recent program counters, for describing a hang.
    const WINDOW: usize = 128;
    let mut recent = [0u16; WINDOW];

    for instruction in 0..max_instructions {
        recent[instruction % WINDOW] = system.cpu().pc();

        if let Err(error) = system.step() {
            // The system reports the program counter; the cause is in its error message, and
            // without that a failing access says only where the CPU was, not what it asked for.
            let cause =
                system.error_message().map(str::to_string).unwrap_or_else(|| error.to_string());
            bail!(
                "emulation failed after {instruction} instructions at PC ${:04X}: {cause}",
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
            STATUS_RUNNING => {
                reset_requested_at = None;
                continue;
            },
            STATUS_NEEDS_RESET => {
                let now = system.cpu().cycles();
                match reset_requested_at {
                    None => reset_requested_at = Some(now),
                    Some(asked) if now.saturating_sub(asked) >= RESET_DELAY_CYCLES => {
                        system.reset().map_err(|e| anyhow::anyhow!("{e}")).context("resetting")?;
                        reset_requested_at = None;
                    },
                    Some(_) => {},
                }
                continue;
            },
            STATUS_PASSED => {
                return Ok(Outcome {
                    status: Status::Passed,
                    message: read_message(&system),
                    instructions: instruction,
                    source: Source::Protocol,
                    spinning_at: Vec::new(),
                })
            },
            code => {
                return Ok(Outcome {
                    status: Status::Failed { code },
                    message: read_message(&system),
                    instructions: instruction,
                    source: Source::Protocol,
                    spinning_at: Vec::new(),
                })
            },
        }
    }

    if signature_seen {
        return Ok(Outcome {
            status: Status::TimedOut,
            message: read_message(&system),
            instructions: max_instructions,
            source: Source::Protocol,
            spinning_at: distinct_in_order(&recent, &system),
        });
    }

    // No signature, so the ROM predates the protocol and reports on screen instead. Blargg's
    // earlier suites all end by looping forever on purpose — "disable IRQ and NMI then loop
    // endlessly" — so running out of budget is their normal finish, not a hang, and the answer is
    // sitting in the nametable.
    let drawn = screen::read(&system);
    let outcome = match screen::verdict(&drawn.text()) {
        Some(screen::Verdict::Passed) => Status::Passed,
        Some(screen::Verdict::Failed { code }) => Status::Failed { code },
        // Nothing legible. Left as it was rather than guessed at.
        None => Status::NoProtocol,
    };

    let outcome_is_unread = outcome == Status::NoProtocol;
    let source = if outcome_is_unread { Source::Protocol } else { Source::Screen };

    Ok(Outcome {
        status: outcome,
        message: drawn.text(),
        instructions: max_instructions,
        source,
        spinning_at: if outcome_is_unread { distinct_in_order(&recent, &system) } else { Vec::new() },
    })
}

/// The distinct values of `window`, in the order they first appear, each disassembled.
///
/// A spin loop visits the same few addresses over and over, so the distinct set is short and is
/// the loop itself. Kept in order rather than sorted, because the order is the loop's shape.
///
/// Bounded at sixteen: past that it is not a spin loop and the list stops being an explanation.
fn distinct_in_order(window: &[u16], system: &NesSystem) -> Vec<(u16, String)> {
    let disassembler = Disassembler::new();
    let mut seen: Vec<u16> = Vec::new();

    for &pc in window {
        if !seen.contains(&pc) {
            seen.push(pc);
        }
        if seen.len() > 16 {
            return Vec::new();
        }
    }

    seen.into_iter()
        .map(|pc| {
            // Three bytes is the longest 6502 instruction, so this always has enough to decode.
            let bytes: Vec<u8> =
                (0..3).map(|offset| read(system, pc.wrapping_add(offset))).collect();
            let text = disassembler
                .disassemble_instruction(&bytes, 0)
                .map(|(text, _)| text)
                .unwrap_or_else(|_| format!("{:02X} ?", bytes[0]));
            (pc, text)
        })
        .collect()
}

/// Peek, never read.
///
/// This runner looks at `$6000` after every single instruction. Doing that through a real read put
/// the status byte on the open bus each time, which almost every ROM cannot tell — and
/// `cpu_exec_space` can, because it executes what an undriven read returns. It reported "Failed #2"
/// against an emulator that was getting the answer right; the runner was supplying the wrong one.
fn read(system: &NesSystem, address: u16) -> u8 {
    system.cpu().peek_byte(address).unwrap_or(0)
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

    // Bounded: a corrupt or absent terminator must not read the whole address space. Generous,
    // because several of these ROMs explain themselves at length before saying what went wrong,
    // and a message cut off mid-sentence hides the one line that matters.
    for offset in 0..2048u16 {
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
