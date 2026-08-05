//! A per-instruction trace, for diffing against another emulator.
//!
//! Reading two implementations side by side answers "what does theirs do"; running both and
//! diffing answers "where do they first disagree", which is a far shorter question. `tools/nesref`
//! produces the same fields from tetanes, so:
//!
//! ```sh
//! cargo run -p rom_test -- trace rom.nes --instructions 200000 > /tmp/ours.txt
//! cargo run --manifest-path tools/nesref/Cargo.toml -- rom.nes 40 --trace > /tmp/theirs.txt
//! diff <(cut -c1-60 /tmp/ours.txt) <(cut -c1-60 /tmp/theirs.txt) | head
//! ```
//!
//! The first differing line is the bug, and everything before it is agreement rather than a
//! guess.

use std::path::Path;

use anyhow::{Context, Result};
use rn_core::{cartridge::load_rom, system::NesSystem};

/// Emit one line per instruction: where it was, the registers, and the cost so far.
///
/// Deliberately close to nestest's format, which every emulator's trace resembles, so the columns
/// line up under `diff` without post-processing.
pub fn report(rom_path: &Path, instructions: usize) -> Result<()> {
    let rom = load_rom(rom_path)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| format!("loading {}", rom_path.display()))?;

    let mut system = NesSystem::new();
    system
        .load_rom(&rom)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("loading the ROM into the system")?;

    for _ in 0..instructions {
        let registers = system.cpu().registers();
        let (scanline, dot) = system.ppu().scanline_cycle();

        println!(
            "${:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:3},{:3} CYC:{}",
            registers.pc,
            registers.a,
            registers.x,
            registers.y,
            registers.status,
            registers.sp,
            dot,
            scanline,
            system.cpu().cycles()
        );

        if system.step().is_err() {
            break;
        }
    }

    Ok(())
}
