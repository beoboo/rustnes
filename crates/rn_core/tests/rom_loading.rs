//! Loading and running iNES ROM images.
//!
//! Until now the loader parsed the header and then read the PRG data purely to seek past it to the
//! CHR data, discarding the program. That meant no `.nes` file could be run at all, which is why
//! none of the community's test ROMs had ever been tried against this emulator.
//!
//! The ROMs used here are synthesised in-process. Real test ROMs (`nestest`, blargg's suites)
//! cannot be committed to this repository, and a test that silently skips when a file is missing
//! is worse than one that builds its own input.

use rn_core::{cartridge::load_rom, memory::Addressable, system::NesSystem};

/// Build a minimal iNES image: a header, `prg` padded to whole 16 KB banks, and 8 KB of CHR.
///
/// `reset` is written into the vector at `$FFFC` so the CPU knows where to start.
fn synthesise_rom(prg: &[u8], reset: u16, banks: usize) -> Vec<u8> {
    let prg_len = banks * 16 * 1024;
    assert!(prg.len() <= prg_len, "program does not fit in {banks} bank(s)");

    let mut image = Vec::new();
    image.extend_from_slice(b"NES\x1A");
    image.push(banks as u8); // PRG size, in 16 KB units
    image.push(1); // CHR size, in 8 KB units
    image.extend_from_slice(&[0x00, 0x00]); // mapper 0, no flags
    image.extend_from_slice(&[0; 8]); // padding

    let mut prg_bank = vec![0u8; prg_len];
    prg_bank[..prg.len()].copy_from_slice(prg);

    // The reset vector lives at $FFFC, which is 4 bytes from the end of the *last* bank.
    let vector = prg_len - 4;
    prg_bank[vector..vector + 2].copy_from_slice(&reset.to_le_bytes());

    image.extend_from_slice(&prg_bank);
    image.extend_from_slice(&vec![0u8; 8 * 1024]); // CHR
    image
}

/// Distinguishes the ROMs written by tests running side by side.
static NEXT_ROM: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn write_temp_rom(name: &str, image: &[u8]) -> std::path::PathBuf {
    // Unique per call: tests in a file run in parallel threads, and a shared path means one test
    // truncating the ROM while another reads it.
    let path = std::env::temp_dir().join(format!(
        "{}_{}_{}",
        std::process::id(),
        NEXT_ROM.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        name
    ));
    std::fs::write(&path, image).expect("writing the test ROM");
    path
}

#[test]
fn loader_returns_prg_rom_rather_than_discarding_it() {
    let image = synthesise_rom(&[0xEA, 0xEA, 0xEA], 0x8000, 1);
    let path = write_temp_rom("rn_prg_present.nes", &image);

    let rom = load_rom(&path).expect("loading the ROM");

    assert_eq!(rom.prg_rom.len(), 16 * 1024, "one 16 KB PRG bank");
    assert_eq!(rom.chr_rom.len(), 8 * 1024, "one 8 KB CHR bank");
    assert_eq!(&rom.prg_rom[..3], &[0xEA, 0xEA, 0xEA], "the program must survive loading");
    assert_eq!(rom.header.mapper, 0);
}

#[test]
fn a_16k_rom_is_mirrored_into_both_halves_of_cartridge_space() {
    // NROM-128 has a single 16 KB bank that hardware presents at both $8000 and $C000. Without
    // that mirroring the reset vector at $FFFC reads as zero and the CPU starts executing nothing.
    let image = synthesise_rom(&[0xA9, 0x42], 0xC000, 1);
    let path = write_temp_rom("rn_mirroring.nes", &image);

    let rom = load_rom(&path).expect("loading the ROM");

    assert_eq!(rom.read_prg(0x8000), 0xA9);
    assert_eq!(rom.read_prg(0xC000), 0xA9, "the bank must also appear at $C000");
    assert_eq!(rom.reset_vector(), 0xC000, "reset vector should resolve through the mirror");
}

#[test]
fn a_32k_rom_is_not_mirrored() {
    let mut prg = vec![0u8; 32 * 1024];
    prg[0] = 0x11; // at $8000
    prg[0x4000] = 0x22; // at $C000
    let image = synthesise_rom(&prg, 0x8000, 2);
    let path = write_temp_rom("rn_32k.nes", &image);

    let rom = load_rom(&path).expect("loading the ROM");

    assert_eq!(rom.prg_rom.len(), 32 * 1024);
    assert_eq!(rom.read_prg(0x8000), 0x11);
    assert_eq!(rom.read_prg(0xC000), 0x22, "a 32 KB image has distinct halves");
}

#[test]
fn system_boots_from_the_reset_vector_and_executes_the_program() {
    // LDA #$42 / STA $0200 / JMP self, placed at the start of the bank and entered via the vector.
    let program = [
        0xA9, 0x42, // LDA #$42
        0x8D, 0x00, 0x02, // STA $0200
        0x4C, 0x05, 0xC0, // JMP $C005 (spin)
    ];
    let image = synthesise_rom(&program, 0xC000, 1);
    let path = write_temp_rom("rn_boot.nes", &image);

    let rom = load_rom(&path).expect("loading the ROM");
    let mut system = NesSystem::new();
    system.load_rom(&rom).expect("loading the ROM into the system");

    assert_eq!(system.cpu().pc(), 0xC000, "execution must start at the reset vector");

    for _ in 0..10 {
        system.step().expect("stepping");
    }

    assert_eq!(
        system.cpu().read_byte(0x0200).expect("reading RAM"),
        0x42,
        "the loaded program should have run"
    );
}

/// A banked mapper's reset vector lives in the *last* PRG bank, which hardware fixes at $E000.
///
/// Mirroring the image from the start instead — as a mapperless loader does — reads the vector
/// from the wrong bank entirely. Super Mario Bros 3 computed $FFFF that way and executed garbage.
#[test]
fn a_banked_rom_boots_from_the_last_bank() {
    const BANKS: usize = 16; // 256 KB, as SMB3 has
    let mut prg = vec![0u8; BANKS * 16 * 1024];

    // A recognisable program at the start of the final bank, entered via the vector.
    let entry = 0xC000u16;
    let last_bank_start = prg.len() - 16 * 1024;
    prg[last_bank_start..last_bank_start + 5].copy_from_slice(&[
        0xA9, 0x37, // LDA #$37
        0x8D, 0x00, 0x02, // STA $0200
    ]);
    let vector = prg.len() - 4;
    prg[vector..vector + 2].copy_from_slice(&entry.to_le_bytes());

    // Byte 0 of the *first* bank differs, so reading the vector from the wrong place is visible.
    prg[0] = 0xFF;

    let mut image = Vec::new();
    image.extend_from_slice(b"NES\x1A");
    image.push(BANKS as u8);
    image.push(1);
    image.extend_from_slice(&[0x40, 0x00]); // mapper 4 (MMC3), horizontal mirroring
    image.extend_from_slice(&[0; 8]);
    image.extend_from_slice(&prg);
    image.extend_from_slice(&vec![0u8; 8 * 1024]);

    let path = write_temp_rom("rn_mmc3_boot.nes", &image);
    let rom = load_rom(&path).expect("loading the ROM");
    assert_eq!(rom.header.mapper, 4);

    let mut system = NesSystem::new();
    system.load_rom(&rom).expect("loading into the system");

    assert_eq!(
        system.cpu().pc(),
        entry,
        "the reset vector must come from the last bank, which MMC3 fixes at $E000"
    );

    for _ in 0..10 {
        system.step().expect("stepping");
    }
    assert_eq!(
        system.cpu().read_byte(0x0200).expect("reading RAM"),
        0x37,
        "the program in the last bank should have run"
    );
}

/// Cartridge space is not RAM: writes there drive the mapper, and must not corrupt the program.
#[test]
fn writes_to_cartridge_space_do_not_overwrite_the_program() {
    let image = synthesise_rom(&[0xEA, 0xEA, 0xEA], 0x8000, 1);
    let path = write_temp_rom("rn_rom_write.nes", &image);
    let rom = load_rom(&path).expect("loading the ROM");

    let mut system = NesSystem::new();
    system.load_rom(&rom).expect("loading into the system");

    let before = system.cpu().read_byte(0x8000).expect("reading");
    system.cpu().write_byte(0x8000, 0x00).ok();
    let after = system.cpu().read_byte(0x8000).expect("reading");

    assert_eq!(before, after, "a write to cartridge space must not modify ROM contents");
}

#[test]
fn rejects_a_file_that_is_not_an_ines_image() {
    let path = write_temp_rom("rn_not_a_rom.nes", b"this is not a ROM at all, not even close");
    assert!(load_rom(&path).is_err(), "a non-iNES file must be rejected");
}

/// Most of an instruction's cycles should be run from its bus accesses, not after it finishes.
///
/// This is the whole point of clocking the system from inside the CPU: an access that reads $2002
/// should see the PPU where it stands at that cycle, not where it will be once the instruction
/// ends. The assertion is loose because not every 6502 cycle is modelled as an access — the
/// internal ones are not — so a majority is what success looks like, not all of them.
///
/// Worth pinning because the obvious way to check this from outside reads zero: `step` consumes
/// the per-instruction counter itself, so asking afterwards always says none, and the mechanism
/// looks dead when it is working.
#[test]
fn most_cycles_are_run_from_bus_accesses() {
    let program = [
        0xA9, 0x42, // LDA #$42
        0x8D, 0x00, 0x02, // STA $0200
        0x4C, 0x00, 0xC0, // JMP $C000
    ];
    let image = synthesise_rom(&program, 0xC000, 1);
    let path = write_temp_rom("rn_bus_clock.nes", &image);

    let rom = load_rom(&path).expect("loading the ROM");
    let mut system = NesSystem::new();
    system.load_rom(&rom).expect("loading into the system");

    let mut total = 0u64;
    for _ in 0..500 {
        total += system.step().expect("stepping") as u64;
    }

    let clocked = system.cpu().total_clocked_cycles();
    assert!(total > 0, "the program should have run");
    assert!(
        clocked * 2 > total,
        "only {clocked} of {total} cycles ran from bus accesses; the clock is not driving them"
    );
    assert!(clocked <= total, "a bus access cannot run more cycles than the instruction took");
}


/// A cartridge must not stop when it reaches a `BRK`.
///
/// `BRK` on a cartridge is an ordinary instruction with a handler behind it. The system used to
/// peek at the next opcode after every step and, on seeing `$00`, declare the program Finished and
/// halt — a convenience for the debugger, where a hand-assembled snippet really does end with
/// `BRK`, and fatal for a ROM that uses it.
///
/// It cost more than it looks. `instr_test-v5/15-brk` and `16-special` were recorded as hangs for a
/// long time; so were `all_instrs` and `official_only`, which run them; so was
/// `cpu_interrupts_v2/2-nmi_and_brk`, the test CYCLE_ACCURACY.md names as having hung on two
/// previous attempts at interrupt timing. All five were the machine having switched itself off,
/// with the program counter parked on the `BRK` for the rest of the run.
#[test]
fn a_cartridge_runs_through_a_brk_instead_of_halting_on_it() {
    // A NOP, then a BRK, with a handler that stores a marker and spins.
    //
    //   $8000  NOP
    //   $8001  BRK            ; through the IRQ vector
    //   $8010  LDA #$5A       ; the handler
    //   $8012  STA $0200
    //   $8015  JMP $8015
    //
    // The NOP is load-bearing. The halt fired on the opcode the CPU was *about* to run, checked
    // after each step, so a BRK reached as the first instruction of all was never looked at. It
    // had to be arrived at from somewhere.
    let mut prg = vec![0xEA; 32 * 1024];
    prg[0x0000] = 0xEA; // NOP
    prg[0x0001] = 0x00; // BRK
    prg[0x0002] = 0x00; // its padding byte

    prg[0x0010] = 0xA9; // LDA #$5A
    prg[0x0011] = 0x5A;
    prg[0x0012] = 0x8D; // STA $0200
    prg[0x0013] = 0x00;
    prg[0x0014] = 0x02;
    prg[0x0015] = 0x4C; // JMP $8015
    prg[0x0016] = 0x15;
    prg[0x0017] = 0x80;

    // Vectors, at the end of the 32 KB image: NMI, reset, IRQ.
    let end = prg.len();
    prg[end - 6..end - 4].copy_from_slice(&0x8000u16.to_le_bytes());
    prg[end - 4..end - 2].copy_from_slice(&0x8000u16.to_le_bytes());
    prg[end - 2..end].copy_from_slice(&0x8010u16.to_le_bytes());

    let mut image = Vec::new();
    image.extend_from_slice(b"NES\x1A");
    image.extend_from_slice(&[2, 1, 0x00, 0x00]);
    image.extend_from_slice(&[0; 8]);
    image.extend_from_slice(&prg);
    image.extend_from_slice(&vec![0u8; 8 * 1024]);

    let path = write_temp_rom("brk_does_not_halt.nes", &image);
    let rom = load_rom(&path).expect("loading the ROM");
    let mut system = NesSystem::new();
    system.load_rom(&rom).expect("loading into the system");

    for _ in 0..64 {
        system.step().expect("stepping");
    }

    assert_eq!(
        system.cpu().read_byte(0x0200).expect("reading the marker"),
        0x5A,
        "the BRK should have entered its handler and the machine should have kept running"
    );
}
