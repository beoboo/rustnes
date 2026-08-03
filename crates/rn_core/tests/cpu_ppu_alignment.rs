//! How many CPU cycles the PPU takes to complete a frame.
//!
//! A frame is 89342 dots and the PPU runs at three times the CPU's rate, so a frame must cost
//! 29780.67 CPU cycles. Any other number means the two are not advancing together, and everything
//! that depends on where the beam is when a given instruction runs — a mid-frame scroll change, a
//! mapper IRQ handler that counts cycles to reach hblank — lands in the wrong place by exactly
//! that much.
//!
//! This is the measurement the alignment work is judged by, so it is a test rather than a tool.
//! It runs on a synthesised ROM, since no commercial ROM can be committed here.

use rn_core::{cartridge::load_rom, memory::Addressable, system::NesSystem};

/// CPU cycles in one NTSC frame: 341 dots x 262 lines / 3.
const CYCLES_PER_FRAME: f64 = (341.0 * 262.0) / 3.0;

/// A ROM that enables rendering, then spins on a mix of instructions and a sprite DMA.
///
/// The mix matters: an emulator can be exact on one addressing mode and wrong on another, and a
/// spin loop of `JMP` alone would exercise almost nothing. The sprite DMA is here because it is
/// 513 cycles during which the CPU is stalled rather than idle — those cycles still belong to it,
/// and leaving them out of the count made a frame appear 518 cycles short.
fn drifting_rom() -> std::path::PathBuf {
    let mut prg = vec![0u8; 32 * 1024];

    let program: &[u8] = &[
        0xA9, 0x1E, // LDA #$1E
        0x8D, 0x01, 0x20, // STA $2001   enable rendering
        0xA9, 0x00, // LDA #$00
        0x85, 0x10, // STA $10
        // The loop: a spread of addressing modes, a read-modify-write, and a sprite DMA.
        0xA9, 0x02, // LDA #$02
        0x8D, 0x14, 0x40, // STA $4014   sprite DMA, 513 cycles
        0xE6, 0x10, // INC $10     read-modify-write, zero page
        0xEE, 0x00, 0x03, // INC $0300   read-modify-write, absolute
        0xA0, 0x05, // LDY #$05
        0xB1, 0x10, // LDA ($10),Y indirect indexed
        0xB9, 0x00, 0x03, // LDA $0300,Y absolute indexed
        0xBD, 0x00, 0x03, // LDA $0300,X absolute indexed
        0xEA, // NOP
        0x4C, 0x09, 0x80, // JMP back to the loop
    ];
    prg[..program.len()].copy_from_slice(program);

    // Reset vector at $8000.
    let vectors = prg.len() - 6;
    prg[vectors..vectors + 2].copy_from_slice(&0x8000u16.to_le_bytes()); // NMI
    prg[vectors + 2..vectors + 4].copy_from_slice(&0x8000u16.to_le_bytes()); // reset
    prg[vectors + 4..vectors + 6].copy_from_slice(&0x8000u16.to_le_bytes()); // IRQ

    let mut image = Vec::new();
    image.extend_from_slice(b"NES\x1A");
    image.push(2); // 32 KB PRG
    image.push(1); // 8 KB CHR
    image.extend_from_slice(&[0x00, 0x00]); // mapper 0
    image.extend_from_slice(&[0; 8]);
    image.extend_from_slice(&prg);
    image.extend_from_slice(&vec![0u8; 8 * 1024]);

    let path = std::env::temp_dir().join("rn_alignment.nes");
    std::fs::write(&path, &image).expect("writing the ROM");
    path
}

#[test]
fn a_frame_costs_the_cpu_the_cycles_it_should() {
    let rom = load_rom(&drifting_rom()).expect("loading");
    let mut system = NesSystem::new();
    system.load_rom(&rom).expect("loading into system");

    // Quiet the APU frame IRQ, so the measurement is of the CPU and PPU alone.
    system.cpu().write_byte(0x4017, 0x40).ok();

    // Settle first: the first frames include reset and the run-up to rendering being enabled.
    while system.ppu().frame_count() < 4 {
        if system.step().is_err() {
            panic!("the ROM stopped before rendering began");
        }
    }

    let frames = 20u64;
    let start_frame = system.ppu().frame_count();
    let start_cycles = system.cpu().cycles();
    while system.ppu().frame_count() < start_frame + frames {
        system.step().expect("stepping");
    }

    let per_frame = (system.cpu().cycles() - start_cycles) as f64 / frames as f64;
    let drift = per_frame - CYCLES_PER_FRAME;

    // A tolerance rather than an equality, because a handful of cycles a frame are still
    // unaccounted for — the internal cycles of a few addressing modes, which `rom_test cycles`
    // lists per opcode. The bound is here to stop it growing, and to be tightened as they land.
    assert!(
        drift.abs() < 8.0,
        "a frame cost {per_frame:.2} CPU cycles, {drift:+.2} against hardware's {CYCLES_PER_FRAME:.2}"
    );
}
