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
    // Commercial ROMs cannot live in this repository, so skip cleanly without one.
    if !path.exists() {
        println!("SKIP: no ROM at {}", path.display());
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
