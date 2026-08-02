//! Which opcodes execute fewer bus accesses than they take cycles.
//!
//! The 6502 has no bus-idle state: every cycle drives the address bus and performs a read or a
//! write, including the cycles that do nothing useful with the result. An implied instruction
//! reads the byte after its opcode and discards it. A push reads it too, before writing to the
//! stack. A read-modify-write writes the *unmodified* value back before writing the modified one.
//! Those accesses are invisible against RAM, which is why an emulator can omit them for a long
//! time without anything appearing wrong.
//!
//! They stop being invisible for two reasons. Against a register with side effects, a discarded
//! read is not discarded at all — reading $2007 advances the PPU address. And more importantly,
//! while an instruction performs fewer accesses than it takes cycles, there is no way to say *when*
//! within it anything happens: a cycle cannot be named from outside. Interrupt sampling is defined
//! as happening before an instruction's last cycle, so it cannot be placed correctly until the two
//! counts agree.
//!
//! This reports the difference per opcode, which names exactly which addressing modes are missing
//! accesses. Driven by nestest because it exercises every official opcode against a known-good
//! trace, so a gap here is a gap in the emulator rather than in the program being run.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use rn_core::{cartridge::load_rom, memory::Addressable, system::NesSystem};

/// nestest's automated entry point, which needs no PPU.
const AUTOMATED_ENTRY: u16 = 0xC000;

struct Gap {
    missing: i32,
    executions: u32,
}

pub fn report(rom_path: &Path, instructions: usize) -> Result<()> {
    let rom = load_rom(rom_path)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| format!("loading {}", rom_path.display()))?;

    let mut system = NesSystem::new();
    system
        .load_rom(&rom)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("loading the ROM into the system")?;

    system.cpu().set_pc(AUTOMATED_ENTRY);
    let mut registers = system.cpu().registers();
    registers.status = 0x24;
    registers.sp = 0xFD;
    system.cpu().set_registers(registers);

    let mut gaps: BTreeMap<u8, Gap> = BTreeMap::new();
    let mut executed = 0u32;

    for _ in 0..instructions {
        let opcode = system.cpu().read_byte(system.cpu().pc()).unwrap_or(0);
        if system.step().is_err() {
            break;
        }
        executed += 1;

        let (accesses, cycles) = system.last_step_cycles();
        let entry = gaps.entry(opcode).or_insert(Gap {
            missing: 0,
            executions: 0,
        });
        entry.missing = cycles as i32 - accesses as i32;
        entry.executions += 1;
    }

    let short: Vec<(&u8, &Gap)> = gaps.iter().filter(|(_, gap)| gap.missing != 0).collect();

    if short.is_empty() {
        println!("every opcode accounts for all of its cycles ({executed} instructions)");
        return Ok(());
    }

    println!("opcodes whose cycles exceed their bus accesses:");
    for (opcode, gap) in &short {
        println!(
            "  ${opcode:02X}  missing {} access(es), executed {} times",
            gap.missing, gap.executions
        );
    }

    let affected: u32 = short.iter().map(|(_, gap)| gap.executions).sum();
    println!(
        "\n{} of {} distinct opcodes account for every cycle",
        gaps.len() - short.len(),
        gaps.len()
    );
    println!("{affected} of {executed} executed instructions are short of at least one access");

    Ok(())
}
