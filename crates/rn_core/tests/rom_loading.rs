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

fn write_temp_rom(name: &str, image: &[u8]) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(name);
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

#[test]
fn rejects_a_file_that_is_not_an_ines_image() {
    let path = write_temp_rom("rn_not_a_rom.nes", b"this is not a ROM at all, not even close");
    assert!(load_rom(&path).is_err(), "a non-iNES file must be rejected");
}
