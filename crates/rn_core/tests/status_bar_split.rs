//! The Super Mario Bros 3 status-bar split, at the scene where it happens: inside a level.
//!
//! The split rows (193/194) wobbled by 8-16 pixels between frames for three TODO.md entries
//! running, and the diagnosis closed 2026-08-06 with a region, not a timing fault: the wobbling
//! ROM was `super-mario-3-eu.nes`, a PAL cart, and this emulator clocks everything NTSC. The
//! game's IRQ handler ends in a hard-coded `LDX #$0C` delay tuned so its `$2001` writes land in
//! hblank — *at 3.2 PPU dots per CPU cycle*. At our 3.0 the same 206-cycle handler spans 41 fewer
//! dots, the writes land mid-scanline, and the CPU's genuine 0-3 cycle latency in taking the IRQ
//! (it spins in a 3-cycle loop at `$96F4`) becomes visible as the wobble. tetanes runs the same
//! cart as PAL and its writes jitter identically — at dots 263/266/271 of line 193, all in
//! hblank, where jitter draws nothing.
//!
//! So the NTSC ROM is the one this emulator can be held to, and this test holds it: on
//! `super-mario-3.nes` (US), the split rows' interior must be pixel-identical across consecutive
//! frames. Pointing the same assertion at the EU ROM fails it with rows 193-194 differing — that
//! is the wobble, and the proof this test can see it.

use rn_core::{
    cartridge::load_rom,
    input::{ControllerButton, ControllerState},
    system::NesSystem,
};

fn advance_frames(sys: &mut NesSystem, frames: u64) {
    let target = sys.ppu().frame_count() + frames;
    while sys.ppu().frame_count() < target {
        if sys.step().is_err() {
            return;
        }
    }
}

/// The route into level 1-1, mirroring `into_a_level` in tools/rom_test/src/frame.rs and the
/// identical sequence in tools/nesref — the same taps with the same pauses, so a frame captured
/// here is the frame those tools capture. Four Starts to the world map, Right and Up onto the
/// level 1 panel, A to enter. Every step was read off a frame dump; the route once ended at the
/// world map while claiming to reach a level.
fn into_a_level(sys: &mut NesSystem) {
    let tap = |sys: &mut NesSystem, button: ControllerButton, after: u64| {
        let mut pressed = ControllerState::new();
        pressed.set_button(button, true);
        sys.set_controller1_state(pressed);
        advance_frames(sys, 8);
        sys.set_controller1_state(ControllerState::new());
        advance_frames(sys, after);
    };

    advance_frames(sys, 240);
    for _ in 0..4 {
        tap(sys, ControllerButton::Start, 68);
    }
    advance_frames(sys, 120);
    tap(sys, ControllerButton::Right, 60);
    tap(sys, ControllerButton::Up, 60);
    tap(sys, ControllerButton::A, 240);
    advance_frames(sys, 180);
}

/// In a level, the rows the split serves — the ones above and inside the status bar — must not
/// move between frames.
///
/// The comparison is pixels, not summary statistics, and it is bounded on purpose:
///
/// - Rows 186-200 bracket the forced-blank window (off at 193, on at 194) with margin on both
///   sides, so a drifted write shows up even if it drifts a row.
/// - Columns 24-231 exclude the left edge, because columns 10-23 of row 194 flicker *on the
///   reference too* — tetanes shows 8-14 shimmering pixels there on this same ROM, the well-known
///   garbage above SMB3's status bar, born where the rendering-on write straddles the line 193/194
///   prefetch boundary. Asserting stillness there would be asserting against hardware.
#[test]
fn split_rows_hold_still_in_a_level() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../nes-roms/super-mario-3.nes");
    // Commercial ROMs cannot live in this repository, so this has to skip without one — and
    // `RN_REQUIRE_ROMS` turns the skip into a failure for anyone who has them, because a skipped
    // test in `cargo test` reads exactly like a passing one.
    if !path.exists() {
        assert!(
            std::env::var_os("RN_REQUIRE_ROMS").is_none(),
            "RN_REQUIRE_ROMS is set but there is no ROM at {} — this test measures nothing \
             without it, and silently passing would say otherwise",
            path.display()
        );
        eprintln!(
            "SKIP: no ROM at {} (set RN_REQUIRE_ROMS to make this a failure)",
            path.display()
        );
        return;
    }
    let rom = load_rom(&path).unwrap();
    let mut sys = NesSystem::new();
    sys.load_rom(&rom).unwrap();
    into_a_level(&mut sys);

    const ROWS: std::ops::Range<usize> = 186..201;
    const COLS: std::ops::Range<usize> = 24..232;

    let band = |sys: &NesSystem| -> Vec<u8> {
        let f = sys.ppu().frame_buffer();
        ROWS.flat_map(|y| f[(y * 256 + COLS.start) * 3..(y * 256 + COLS.end) * 3].iter().copied())
            .collect()
    };

    let first_band = band(&sys);
    let first_frame = sys.ppu().frame_buffer().to_vec();
    let mut a_frame_changed = false;
    for n in 2..=10 {
        advance_frames(&mut sys, 1);
        let this_band = band(&sys);
        if let Some(i) = first_band.iter().zip(&this_band).position(|(a, b)| a != b) {
            let y = ROWS.start + i / 3 / COLS.len();
            let x = COLS.start + (i / 3) % COLS.len();
            panic!(
                "frame {n} differs from frame 1 inside the split band, first at row {y} col {x} \
                 — the status-bar split is wobbling again"
            );
        }
        a_frame_changed |= sys.ppu().frame_buffer() != first_frame[..];
    }
    // A band that held still because emulation stalled proves nothing. Somewhere outside it,
    // the game must visibly go on — the timer alone ticks every second.
    assert!(
        a_frame_changed,
        "ten frames were byte-identical everywhere; the game is not running, so the still \
         split band is meaningless"
    );
}
