//! Saving and restoring a running machine.
//!
//! The test that matters is not that a snapshot round-trips through a file — it is that a restored
//! machine goes on to produce *exactly* what the original would have. A save state that misses one
//! register loads fine, looks fine, and then diverges a few thousand instructions later, which is
//! the hardest kind of fault to trace back to its cause.

use rn_core::{cartridge::load_rom, memory::Addressable, system::NesSystem};

/// Build a minimal iNES image whose program runs a long, state-touching loop.
fn synthesise_rom(prg: &[u8], reset: u16) -> Vec<u8> {
    let mut image = Vec::new();
    image.extend_from_slice(b"NES\x1A");
    image.push(1); // 16 KB of PRG
    image.push(1); // 8 KB of CHR
    image.extend_from_slice(&[0x00, 0x00]); // mapper 0
    image.extend_from_slice(&[0; 8]);

    let mut bank = vec![0u8; 16 * 1024];
    bank[..prg.len()].copy_from_slice(prg);
    let vector = bank.len() - 4;
    bank[vector..vector + 2].copy_from_slice(&reset.to_le_bytes());

    image.extend_from_slice(&bank);
    image.extend_from_slice(&vec![0u8; 8 * 1024]);
    image
}

/// A program that keeps changing memory and registers, so a missed field shows up as a difference.
fn counting_system() -> NesSystem {
    let program = [
        0xA2, 0x00, // LDX #$00
        0xE8, // INX
        0x8A, // TXA
        0x9D, 0x00, 0x03, // STA $0300,X
        0x69, 0x07, // ADC #$07
        0x8D, 0x00, 0x02, // STA $0200
        0x4C, 0x02, 0xC0, // JMP $C002
    ];
    let image = synthesise_rom(&program, 0xC000);
    let path = std::env::temp_dir().join("rn_save_state.nes");
    std::fs::write(&path, image).expect("writing the test ROM");

    let rom = load_rom(&path).expect("loading the ROM");
    let mut system = NesSystem::new();
    system.load_rom(&rom).expect("loading into the system");
    system
}

/// A cheap summary of everything a divergence would show up in.
fn fingerprint(system: &NesSystem) -> Vec<u8> {
    let registers = system.cpu().registers();
    let mut out = vec![registers.a, registers.x, registers.y, registers.status, registers.sp];
    out.extend_from_slice(&registers.pc.to_le_bytes());
    for address in 0x0200..0x0400u16 {
        out.push(system.cpu().read_byte(address).unwrap_or(0));
    }
    out
}

fn run(system: &mut NesSystem, steps: usize) {
    for _ in 0..steps {
        system.step().expect("stepping");
    }
}

#[test]
fn a_restored_machine_continues_identically() {
    let mut system = counting_system();
    run(&mut system, 5_000);

    let snapshot = system.save_state();

    // What the original goes on to do.
    run(&mut system, 5_000);
    let expected = fingerprint(&system);

    // Rewind and do it again. Identical, or some piece of state was not captured.
    system.load_state(&snapshot).expect("restoring");
    run(&mut system, 5_000);

    assert_eq!(
        fingerprint(&system),
        expected,
        "a restored machine diverged from the one it was copied from"
    );
}

/// Restoring into a *different* instance must work too, which is the case a file load exercises.
#[test]
fn a_snapshot_restores_into_a_fresh_machine() {
    let mut original = counting_system();
    run(&mut original, 3_000);
    let snapshot = original.save_state();
    run(&mut original, 2_000);
    let expected = fingerprint(&original);

    let mut restored = counting_system();
    restored.load_state(&snapshot).expect("restoring");
    run(&mut restored, 2_000);

    assert_eq!(fingerprint(&restored), expected, "a fresh machine diverged after restoring");
}

#[test]
fn a_snapshot_survives_serialisation() {
    let mut system = counting_system();
    run(&mut system, 1_000);

    let encoded = serde_json::to_string(&system.save_state()).expect("encoding");
    let decoded: rn_core::system::SaveState = serde_json::from_str(&encoded).expect("decoding");

    run(&mut system, 1_000);
    let expected = fingerprint(&system);

    system.load_state(&decoded).expect("restoring");
    run(&mut system, 1_000);
    assert_eq!(fingerprint(&system), expected, "a snapshot changed by being written out");
}
