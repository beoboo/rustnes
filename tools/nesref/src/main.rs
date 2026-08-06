//! Reference verdicts: run a blargg-protocol ROM in tetanes and report what it says.
//!
//! Exists to answer one question this project cannot answer alone — whether a known-good emulator
//! passes the ROMs we still fail — rather than to be part of the emulator. A ROM we both fail is
//! obscure; one that tetanes passes is a bug of ours with a readable reference beside it.

use tetanes_core::control_deck::{Config, ControlDeck};
use tetanes_core::input::{JoypadBtn, Player};
use tetanes_core::video::VideoFilter;

/// The NES picture, which is not negotiable.
const WIDTH: usize = 256;
const HEIGHT: usize = 240;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let rom = args.next().expect("usage: nesref <rom> [frames] [--trace N]");
    let frames: usize = args.next().unwrap_or_else(|| "1800".into()).parse()?;

    // `--trace N`: print one line per instruction for N instructions and stop, in the same shape
    // `rom_test trace` prints, so the two can be diffed line for line. The first line that differs
    // is the bug; everything above it is agreement that has been checked rather than assumed.
    // `--frame <file>`: write what tetanes drew, as a PPM, so it can be diffed against ours pixel
    // for pixel. The reason this exists is the ROMs that report visually and say nothing a runner
    // can read — `nmi_sync` is two of them, `dmc_tests` four more. A reference picture is the only
    // verdict they have.
    let mut trace: Option<u64> = None;
    let mut frame: Option<String> = None;
    let mut into_level = false;
    let mut skip_frames: usize = 0;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--trace" => trace = Some(args.next().unwrap_or_else(|| "1000000".into()).parse()?),
            "--frame" => frame = args.next(),
            "--into-level" => into_level = true,
            "--skip-frames" => skip_frames = args.next().unwrap_or_default().parse()?,
            other => return Err(format!("unknown option {other}").into()),
        }
    }

    // Pixellate, not tetanes' default. Its default is an NTSC composite simulation — what a CRT
    // would have shown — which blends neighbouring pixels and shifts every colour. Comparing a
    // filtered picture against this emulator's unfiltered one says every pixel differs and means
    // nothing by it. That mistake was made and published before it was caught: a flat grey backdrop
    // came out 83,83,83 against our 117,117,117 and was written up as differing power-on palette
    // RAM, when it was the filter.
    let mut config = Config::default();
    config.filter = VideoFilter::Pixellate;
    let mut deck = ControlDeck::with_config(config);
    deck.load_rom_path(&rom)?;

    // Run to a scene before tracing. A split partway down a frame a thousand frames in cannot be
    // traced from the start — the trace would be tens of millions of lines — and this is the only
    // way to line two emulators up at it, since neither can read the other's save states.
    for _ in 0..skip_frames {
        deck.clock_frame()?;
    }

    if let Some(instructions) = trace {
        // tetanes gates its per-instruction line on `tracing::enabled!(TRACE)` but prints it with
        // `println!`, so the subscriber only has to answer the question — its own writer is sent to
        // a sink, or the emulator's internal `trace!` calls would bury the instruction stream.
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_writer(std::io::sink)
            .init();

        for _ in 0..instructions {
            deck.clock_instr()?;
        }
        return Ok(());
    }

    // The identical sequence `rom_test frame --into-level` runs: Start, four times, with pauses,
    // which carries Super Mario Bros 3 from its title screen into a level. It exists because a save
    // state reaches that scene in a second and only the other emulator can read one — so the only
    // way to get a reference picture of the scene its status-bar split goes wrong in is to drive
    // both there by the same route.
    if into_level {
        let mut run = |deck: &mut ControlDeck, count: usize| -> Result<(), Box<dyn std::error::Error>> {
            for _ in 0..count {
                deck.clock_frame()?;
            }
            Ok(())
        };
        run(&mut deck, 240)?;
        for _ in 0..4 {
            deck.joypad_mut(Player::One).set_button(JoypadBtn::Start, true);
            run(&mut deck, 8)?;
            deck.joypad_mut(Player::One).set_button(JoypadBtn::Start, false);
            run(&mut deck, 68)?;
        }
        run(&mut deck, 120)?;
    }

    for _ in 0..frames {
        let _ = deck.clock_frame()?;
    }

    if let Some(path) = frame {
        // tetanes hands back RGBA; a PPM is RGB, which is what `rom_test frame --out` writes and
        // so what a diff expects.
        let rgba = deck.frame_buffer();
        let mut ppm = format!("P6\n{WIDTH} {HEIGHT}\n255\n").into_bytes();
        ppm.extend(rgba.chunks_exact(4).flat_map(|px| [px[0], px[1], px[2]]));
        std::fs::write(&path, ppm)?;
        println!("Wrote {path}");
        return Ok(());
    }

    // Blargg's ROMs put their status at $6000 and a message at $6004. Read straight off the bus
    // rather than through `sram`, which only answers for battery-backed carts.
    let bus = deck.bus_mut();
    let status = bus.peek(0x6000);
    let signature = [bus.peek(0x6001), bus.peek(0x6002), bus.peek(0x6003)];
    let mut message = String::new();
    for offset in 0..512u16 {
        match bus.peek(0x6004 + offset) {
            0 => break,
            byte => message.push(byte as char),
        }
    }

    let verdict = match status {
        _ if signature != [0xDE, 0xB0, 0x61] => "NO PROTOCOL".to_string(),
        0x00 => "PASS".to_string(),
        0x80 => "still running".to_string(),
        0x81 => "wants a reset".to_string(),
        code => format!("FAIL({code:02X})"),
    };

    println!("{verdict}  {}", message.trim().replace('\n', " | "));
    Ok(())
}
