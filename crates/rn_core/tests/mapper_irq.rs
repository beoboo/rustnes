//! The MMC3 scanline counter must only advance while the PPU is rendering.
//!
//! A scanline-counting mapper is really counting pattern fetches, which stop when rendering is
//! disabled. Counting regardless fires its IRQ during the rendering-off window a game uses to
//! rewrite nametables — the handler then runs at a moment the game never planned for, which
//! presents as the CPU executing data somewhere unrelated rather than as anything IRQ-shaped.

use rn_core::{cartridge::load_rom, memory::Addressable, system::NesSystem};

/// A minimal MMC3 ROM that spins, so the PPU can be driven independently.
fn spinning_mmc3_rom() -> std::path::PathBuf {
    const BANKS: usize = 4;
    let mut prg = vec![0u8; BANKS * 16 * 1024];

    // JMP to self at the start of the last bank.
    let last = prg.len() - 16 * 1024;
    prg[last..last + 3].copy_from_slice(&[0x4C, 0x00, 0xC0]);
    let vector = prg.len() - 4;
    prg[vector..vector + 2].copy_from_slice(&0xC000u16.to_le_bytes());

    let mut image = Vec::new();
    image.extend_from_slice(b"NES\x1A");
    image.push(BANKS as u8);
    image.push(1);
    image.extend_from_slice(&[0x40, 0x00]); // mapper 4
    image.extend_from_slice(&[0; 8]);
    image.extend_from_slice(&prg);
    image.extend_from_slice(&vec![0u8; 8 * 1024]);

    let path = std::env::temp_dir().join("rn_mmc3_irq.nes");
    std::fs::write(&path, &image).expect("writing the ROM");
    path
}

fn run_frames(system: &mut NesSystem, frames: usize) {
    // ~29,780 CPU cycles per frame.
    let target = 29_780u64 * frames as u64;
    let mut cycles = 0u64;
    while cycles < target {
        match system.step() {
            Ok(step) => cycles += step.max(1) as u64,
            Err(_) => break,
        }
    }
}

#[test]
fn the_scanline_counter_does_not_advance_while_rendering_is_disabled() {
    let path = spinning_mmc3_rom();
    let rom = load_rom(&path).expect("loading");
    let mut system = NesSystem::new();
    system.load_rom(&rom).expect("loading into system");

    // The APU's frame counter also holds the IRQ line, so inhibit it ($4017 bit 6) to leave the
    // mapper as the only possible source. Games do this during setup for the same reason.
    system.cpu().write_byte(0x4017, 0x40).ok();

    // Arm the counter to fire after a single scanline, which is as eager as it gets.
    system.cpu().write_byte(0xC000, 1).ok(); // latch
    system.cpu().write_byte(0xC001, 0).ok(); // request reload
    system.cpu().write_byte(0xE001, 0).ok(); // enable

    // Rendering left disabled ($2001 = 0), as during a level load.
    run_frames(&mut system, 4);

    assert!(
        !system.cpu().irq_line(),
        "a disabled PPU fetches no patterns, so the counter must not run and no IRQ may fire"
    );
}

#[test]
fn the_scanline_counter_advances_once_rendering_is_enabled() {
    let path = spinning_mmc3_rom();
    let rom = load_rom(&path).expect("loading");
    let mut system = NesSystem::new();
    system.load_rom(&rom).expect("loading into system");

    system.cpu().write_byte(0x4017, 0x40).ok(); // inhibit the APU frame IRQ
    system.cpu().write_byte(0xC000, 1).ok();
    system.cpu().write_byte(0xC001, 0).ok();
    system.cpu().write_byte(0xE001, 0).ok();

    // Enable background rendering, which is what makes the PPU fetch patterns.
    system.cpu().write_byte(0x2001, 0x08).ok();
    run_frames(&mut system, 4);

    assert!(
        system.cpu().irq_line(),
        "with rendering on the counter should reach zero and assert the IRQ"
    );
}
