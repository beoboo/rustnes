//! Advancing the emulator a whole frame at a time.
//!
//! The debugger draws one repaint per frame the PPU completes. If a single advance can span two
//! frames, one is published and replaced before it is ever displayed — an animation visibly
//! skipping — and if it spans none, the previous frame is shown twice. Both were visible as
//! flicker, so the unit of advance has to be exactly one frame.

use rn_core::{cartridge::load_rom, system::NesSystem};

fn spinning_rom() -> std::path::PathBuf {
    const BANK: usize = 16 * 1024;
    let mut prg = vec![0u8; BANK];

    // Enable background rendering, then spin. $2001 = $08.
    prg[..8].copy_from_slice(&[
        0xA9, 0x08, // LDA #$08
        0x8D, 0x01, 0x20, // STA $2001
        0x4C, 0x05, 0xC0, // JMP $C005
    ]);
    prg[BANK - 4..BANK - 2].copy_from_slice(&0xC000u16.to_le_bytes());

    let mut image = Vec::new();
    image.extend_from_slice(b"NES\x1A");
    image.extend_from_slice(&[1, 1, 0x00, 0x00]);
    image.extend_from_slice(&[0; 8]);
    image.extend_from_slice(&prg);
    image.extend_from_slice(&vec![0u8; 8 * 1024]);

    let path = std::env::temp_dir().join("rn_frame_pacing.nes");
    std::fs::write(&path, &image).expect("writing the ROM");
    path
}

/// Step until the PPU reports a new frame, as the debugger's per-repaint advance does.
fn advance_one_frame(system: &mut NesSystem) -> u64 {
    let start = system.ppu().frame_count();
    let mut cycles = 0u64;

    while system.ppu().frame_count() == start && cycles < 200_000 {
        match system.step() {
            Ok(step) => cycles += step.max(1) as u64,
            Err(_) => break,
        }
    }
    cycles
}

#[test]
fn advancing_one_frame_completes_exactly_one_frame() {
    let rom = load_rom(&spinning_rom()).expect("loading");
    let mut system = NesSystem::new();
    system.load_rom(&rom).expect("loading into system");

    // Let the program enable rendering first.
    for _ in 0..100 {
        system.step().expect("stepping");
    }

    for _ in 0..10 {
        let before = system.ppu().frame_count();
        advance_one_frame(&mut system);
        let after = system.ppu().frame_count();

        assert_eq!(
            after - before,
            1,
            "each advance must complete exactly one frame, not {} ",
            after - before
        );
    }
}

/// A frame should cost roughly the cycles a real NTSC frame does. Far fewer would mean frames are
/// being counted without work; far more would mean the advance overshoots.
#[test]
fn a_frame_costs_about_one_ntsc_frame_of_cycles() {
    let rom = load_rom(&spinning_rom()).expect("loading");
    let mut system = NesSystem::new();
    system.load_rom(&rom).expect("loading into system");

    for _ in 0..100 {
        system.step().expect("stepping");
    }

    // Discard the first, which starts partway through a frame.
    advance_one_frame(&mut system);

    let cycles = advance_one_frame(&mut system);
    assert!(
        (28_000..=32_000).contains(&cycles),
        "a frame took {cycles} cycles, expected about 29,780"
    );
}
