//! Drive Super Mario Bros 3 into gameplay with synthetic input, then look for per-row flicker.
//!
//! A reported background flicker could not be reproduced from the title screen, which is static.
//! Pressing Start programmatically reaches the state where it was reported, so the frames can be
//! compared rather than guessed about.

use rn_core::{cartridge::load_rom, input::ControllerButton, input::ControllerState, system::NesSystem};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn advance_frames(sys: &mut NesSystem, frames: u64) {
    let target = sys.ppu().frame_count() + frames;
    while sys.ppu().frame_count() < target {
        if sys.step().is_err() { return }
    }
}

fn press(sys: &mut NesSystem, button: ControllerButton, frames: u64) {
    let mut state = ControllerState::new();
    state.set_button(button, true);
    sys.set_controller1_state(state);
    advance_frames(sys, frames);
    sys.set_controller1_state(ControllerState::new());
    advance_frames(sys, frames);
}

/// Gameplay frames must not alternate between two images.
///
/// A row that flips between exactly two values every frame is flickering; one that changes
/// continuously is animation, and one that never changes is static. Distinguishing them is what
/// separates "the emulator is producing bad frames" from "the frames are fine and the display is
/// at fault" — a distinction that otherwise costs several wrong guesses.
#[test]
fn gameplay_frames_do_not_alternate() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../nes-roms/super-mario-3.nes");
    // Commercial ROMs cannot live in this repository, so this has to skip without one — and a
    // skipped test in `cargo test` is indistinguishable from a passing one, which is the whole
    // problem. `println!` here is captured and shown to nobody.
    //
    // So: skipping is the default, and setting `RN_REQUIRE_ROMS` turns a missing ROM into a
    // failure. Anyone with the ROMs can export it once and stop wondering whether this ran. The
    // same fault, in a different shape, cost a frame baseline its meaning: it sat in /tmp
    // disagreeing with the emulator for weeks while still reading as evidence.
    if !path.exists() {
        assert!(
            std::env::var_os("RN_REQUIRE_ROMS").is_none(),
            "RN_REQUIRE_ROMS is set but there is no ROM at {} — this test measures nothing without \
             it, and silently passing would say otherwise",
            path.display()
        );
        eprintln!("SKIP: no ROM at {} (set RN_REQUIRE_ROMS to make this a failure)", path.display());
        return;
    }
    let rom = load_rom(&path).unwrap();
    let mut sys = NesSystem::new();
    sys.load_rom(&rom).unwrap();

    advance_frames(&mut sys, 240);
    for _ in 0..4 { press(&mut sys, ControllerButton::Start, 8); advance_frames(&mut sys, 60); }
    advance_frames(&mut sys, 120);

    // Hash each row of each frame; a row that alternates between two values every frame is
    // flickering, as opposed to changing continuously (animation) or not at all (static).
    let mut rows: Vec<Vec<u64>> = Vec::new();
    for _ in 0..16 {
        let f = sys.ppu().frame_buffer();
        let mut row_hashes = Vec::with_capacity(240);
        for y in 0..240 {
            let mut h = DefaultHasher::new();
            f[y*256*3..(y+1)*256*3].hash(&mut h);
            row_hashes.push(h.finish());
        }
        rows.push(row_hashes);
        advance_frames(&mut sys, 1);
    }

    // Jitter shows up as content moving rather than alternating in place: a row matches its
    // neighbour's position in the next frame. Animation almost never does that for many rows at
    // once, so a high proportion here means scanlines are being drawn a line off, intermittently.
    let (mut moved, mut shifted) = (0usize, 0usize);
    for pair in rows.windows(2) {
        for y in 1..239 {
            if pair[0][y] == pair[1][y] {
                continue;
            }
            moved += 1;
            if pair[0][y] == pair[1][y - 1] || pair[0][y] == pair[1][y + 1] {
                shifted += 1;
            }
        }
    }
    println!("row changes: {moved}, of which shifted by exactly one scanline: {shifted}");

    let mut alternating = 0;
    let mut changing = 0;
    let mut static_rows = 0;
    for y in 0..240 {
        let series: Vec<u64> = rows.iter().map(|r| r[y]).collect();
        let distinct: std::collections::HashSet<_> = series.iter().collect();
        if distinct.len() == 1 { static_rows += 1; }
        else if distinct.len() == 2 && series.windows(2).all(|w| w[0] != w[1]) { alternating += 1; }
        else { changing += 1; }
    }
    println!("{static_rows} static rows, {alternating} alternating, {changing} changing");

    // Some rows changing is expected — that is the game animating. None should alternate.
    assert!(changing > 0, "nothing changed at all; the game may not have started");
    assert_eq!(
        alternating, 0,
        "{alternating} rows flip between two images every frame, which is flicker in the \
         emulator's own output rather than in how it is displayed"
    );
}
