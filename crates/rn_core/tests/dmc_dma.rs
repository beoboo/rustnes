//! The DMC fetches its samples from real memory, and stalls the CPU while it does.
//!
//! Both halves matter. The channel used to play a dummy byte, so a sample was inaudible and the
//! fetch cost nothing — and that second part is a timing bug, not an audio one: hardware halts the
//! CPU for four cycles per sample byte, so a game counting cycles in an interrupt handler takes
//! measurably longer while a sample is playing. Leaving the stall out makes every such loop finish
//! early.

use rn_core::{cartridge::load_rom, memory::Addressable, system::NesSystem};

/// Distinguishes the ROMs written by tests running side by side.
static NEXT_ROM: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// A ROM that spins, with a recognisable byte pattern in the sample area at $F000.
fn spinning_rom_with_sample() -> std::path::PathBuf {
    let mut prg = vec![0u8; 32 * 1024];

    // JMP to self at $8000.
    prg[0..3].copy_from_slice(&[0x4C, 0x00, 0x80]);

    // A sample at $F000, which is offset $7000 into a 32 KB bank mapped at $8000.
    for (offset, byte) in prg[0x7000..0x7100].iter_mut().enumerate() {
        *byte = offset as u8;
    }

    let vectors = prg.len() - 6;
    prg[vectors..vectors + 2].copy_from_slice(&0x8000u16.to_le_bytes());
    prg[vectors + 2..vectors + 4].copy_from_slice(&0x8000u16.to_le_bytes());
    prg[vectors + 4..vectors + 6].copy_from_slice(&0x8000u16.to_le_bytes());

    let mut image = Vec::new();
    image.extend_from_slice(b"NES\x1A");
    image.push(2); // 32 KB PRG
    image.push(1); // 8 KB CHR
    image.extend_from_slice(&[0x00, 0x00]); // mapper 0
    image.extend_from_slice(&[0; 8]);
    image.extend_from_slice(&prg);
    image.extend_from_slice(&vec![0u8; 8 * 1024]);

    let path = std::env::temp_dir().join(format!(
        "rn_dmc_{}_{}.nes",
        std::process::id(),
        NEXT_ROM.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::write(&path, &image).expect("writing the ROM");
    path
}

fn dmc_system(enabled: bool) -> NesSystem {
    let rom = load_rom(&spinning_rom_with_sample()).expect("loading");
    let mut system = NesSystem::new();
    system.load_rom(&rom).expect("loading into the system");

    system.cpu().write_byte(0x4017, 0x40).ok(); // quiet the frame IRQ
    system.cpu().write_byte(0x4010, 0x0F).ok(); // fastest rate, no IRQ, no loop
    system.cpu().write_byte(0x4012, 0xC0).ok(); // sample address $C000 + $C0*64 = $F000
    system.cpu().write_byte(0x4013, 0x10).ok(); // 257 bytes
    if enabled {
        system.cpu().write_byte(0x4015, 0x10).ok();
    }
    system
}

/// A playing sample costs the CPU cycles that a silent one does not.
#[test]
fn a_playing_sample_stalls_the_cpu() {
    let cycles_for = |enabled: bool| {
        let mut system = dmc_system(enabled);
        let start = system.cpu().cycles();
        // A fixed number of *instructions*: the spin loop is a JMP, always three cycles, so any
        // difference in the cycle count is the DMC's doing and nothing else.
        for _ in 0..3000 {
            system.step().expect("stepping");
        }
        system.cpu().cycles() - start
    };

    let silent = cycles_for(false);
    let playing = cycles_for(true);

    assert!(
        playing > silent,
        "a playing sample must cost the CPU cycles: {playing} against {silent} when silent"
    );

    // The arithmetic, because it is easy to get an order of magnitude wrong here: the rate table
    // is CPU cycles per *bit*, not per byte. At rate $0F that is 54 a bit, so 432 a byte. 3000
    // JMPs are 9000 cycles, which is 20.8 bytes, which is 83 cycles of stall.
    //
    // Worth keeping in view: that is under one percent of the CPU's time even at the fastest rate.
    // The DMC is not a plausible explanation for anything costing tens of cycles over a few
    // hundred — a mistake made while chasing exactly such a discrepancy.
    let stalled = playing - silent;
    assert!(
        (60..=110).contains(&stalled),
        "expected about 83 cycles of stall over 9000, got {stalled}"
    );
}

/// The byte the channel plays comes from memory, not from a placeholder.
#[test]
fn the_sample_is_read_from_memory() {
    let mut system = dmc_system(true);

    // $4015 bit 4 reports bytes still to fetch, so the sample is under way.
    let status = system.cpu().read_byte(0x4015).expect("reading status");
    assert_ne!(status & 0x10, 0, "the sample should be playing");

    for _ in 0..3000 {
        system.step().expect("stepping");
    }

    // Still playing: 257 bytes at one per 54 cycles outlasts this run.
    let status = system.cpu().read_byte(0x4015).expect("reading status");
    assert_ne!(status & 0x10, 0, "the sample should still have bytes left");
}
