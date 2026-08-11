//! `rom_test` — headless runner for the NES test ROMs.
//!
//! The community's test ROMs are the independent check on an emulator: unit tests confirm that the
//! code does what its author intended, these confirm it does what the hardware does. Both blargg's
//! suites and `nestest` are designed to run without a screen, so this needs no window.
//!
//! ```text
//! rom_test nestest roms/nestest.nes roms/nestest.log
//! rom_test run roms/instr_test-v5/01-basics.nes
//! rom_test suite roms/
//! ```
//!
//! The ROMs themselves cannot live in this repository — `nestest` and blargg's suites are freely
//! distributed but not licensed for redistribution, and commercial games obviously not. Every
//! command therefore exits cleanly with a clear message when its input is missing, so a checkout
//! without ROMs stays green.

mod baseline;
mod blargg;
mod cycles;
mod frame;
mod nestest;
mod screen;
mod trace;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

/// Instruction budget before a ROM is declared hung.
///
/// Generous: blargg's longer suites legitimately run for tens of millions of instructions.
const DEFAULT_BUDGET: usize = 30_000_000;

#[derive(Parser, Debug)]
#[command(author, version, about = "Run NES test ROMs headlessly and report pass/fail")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run nestest against its golden log, stopping at the first divergence
    Nestest {
        /// Path to nestest.nes
        rom: PathBuf,

        /// Path to nestest.log
        log: PathBuf,

        /// Stop after this many instructions
        #[arg(long, default_value_t = usize::MAX)]
        limit: usize,
    },

    /// Run a single blargg-style ROM and report what it says
    Run {
        /// Path to the .nes file
        rom: PathBuf,

        /// Instruction budget before declaring the ROM hung
        #[arg(long, default_value_t = DEFAULT_BUDGET)]
        budget: usize,
    },

    /// Report which opcodes execute fewer bus accesses than they take cycles
    Cycles {
        /// Path to nestest.nes, which exercises every official opcode
        rom: PathBuf,

        /// Instructions to execute
        #[arg(long, default_value_t = 8991)]
        instructions: usize,
    },

    /// Run a ROM and capture what the PPU drew
    Frame {
        /// Path to the .nes file
        rom: PathBuf,

        /// Video frames to run before capturing
        #[arg(long, default_value_t = 60)]
        frames: usize,

        /// Write the captured frame as a PPM image
        #[arg(long, value_name = "FILE")]
        out: Option<PathBuf>,

        /// Print the frame as ASCII, for judging it in a terminal
        #[arg(long)]
        ascii: bool,

        /// Run as a PAL console: 3.2 dots to a CPU cycle and 312 scanlines. The iNES header is
        /// consulted first, but plenty of European releases claim NTSC in it.
        #[arg(long)]
        pal: bool,

        /// Resume from a save state before running, to reach a scene inside a game
        #[arg(long, value_name = "FILE")]
        state: Option<PathBuf>,

        /// Draw pixels from the per-dot path rather than the per-line one
        #[arg(long)]
        per_dot: bool,

        /// Press Start into a Super Mario Bros 3 level first, matching `nesref --into-level`
        #[arg(long)]
        into_level: bool,

        /// Tap a button on controller 1: BUTTON@FRAME, e.g. `start@130`. Held ten frames;
        /// repeatable. With presses, --frames counts from power-on
        #[arg(long, value_name = "BUTTON@FRAME")]
        press: Vec<String>,
    },

    /// Print one line per instruction, for diffing against another emulator
    Trace {
        /// Path to the .nes file
        rom: PathBuf,

        /// Instructions to trace
        #[arg(long, default_value_t = 200_000)]
        instructions: usize,

        /// Resume from a save state first, to reach a scene inside a game
        #[arg(long, value_name = "FILE")]
        state: Option<PathBuf>,

        /// Run this many frames before tracing, for a scene too far in to trace from the start
        #[arg(long, default_value_t = 0)]
        skip_frames: usize,

        /// Press Start into a Super Mario Bros 3 level first, matching `nesref --into-level`
        #[arg(long)]
        into_level: bool,
    },

    /// Print the text a ROM has drawn on screen, for ROMs that report no other way
    Screen {
        /// Path to the .nes file
        rom: PathBuf,

        /// Video frames to run before reading the screen
        #[arg(long, default_value_t = 240)]
        frames: usize,

        /// Print raw tile indices instead of decoded text
        #[arg(long)]
        raw: bool,
    },

    /// Check rendered frames against the hashes committed beside this tool
    Baselines {
        /// Directory holding the games the baselines name
        roms: PathBuf,

        /// Rewrite the hashes from what the emulator draws now, instead of checking them
        #[arg(long)]
        update: bool,

        /// The baselines file, if not the one beside this tool
        #[arg(long, value_name = "FILE")]
        file: Option<PathBuf>,
    },

    /// Compare a captured frame against a reference one, ignoring palette differences
    Compare {
        /// The frame this emulator drew, from `rom_test frame --out`
        ours: PathBuf,

        /// The reference frame, from `nesref --frame`
        reference: PathBuf,
    },

    /// Run every .nes file under a directory and summarise
    Suite {
        /// Directory to search, recursively
        directory: PathBuf,

        /// Instruction budget per ROM
        #[arg(long, default_value_t = DEFAULT_BUDGET)]
        budget: usize,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Nestest { rom, log, limit } => run_nestest(&rom, &log, limit),
        Command::Run { rom, budget } => run_one(&rom, budget),
        Command::Cycles { rom, instructions } => cycles::report(&rom, instructions),
        Command::Frame { rom, frames, out, ascii, state, per_dot, into_level, pal, press } => {
            let presses = press
                .iter()
                .map(|spec| frame::parse_press(spec))
                .collect::<Result<Vec<_>>>()?;
            run_frame(
                &rom,
                out.as_deref(),
                ascii,
                frame::Options {
                    frames,
                    state: state.as_deref(),
                    per_dot,
                    into_level,
                    force_pal: pal,
                    presses: &presses,
                },
            )
        },
        Command::Trace {
            rom,
            instructions,
            state,
            skip_frames,
            into_level,
        } => trace::report(&rom, instructions, state.as_deref(), skip_frames, into_level),
        Command::Screen { rom, frames, raw } => screen::report(&rom, frames, raw),
        Command::Baselines { roms, update, file } => {
            let path = file.unwrap_or_else(baseline::default_path);
            if missing(&path, "the baselines file") {
                return Ok(());
            }
            if update {
                baseline::update(&path, &roms)
            } else if baseline::report(&path, &roms)? {
                Ok(())
            } else {
                // A non-zero exit, so this can gate a change rather than only inform about one.
                anyhow::bail!("a rendered frame no longer matches its committed baseline")
            }
        },
        Command::Compare { ours, reference } => {
            if missing(&ours, "the captured frame") || missing(&reference, "the reference frame") {
                return Ok(());
            }
            let ours_pixels = frame::read_ppm(&ours)?;
            let reference_pixels = frame::read_ppm(&reference)?;
            let differing = frame::structural_diff(&ours_pixels, &reference_pixels);

            if differing.is_empty() {
                println!("IDENTICAL in structure — the same regions took the same palette entries");
                return Ok(());
            }

            // No bijection between the two colour sets. That is not the same as a wrong picture:
            // two emulators derive their palettes differently, so one will quantise a pair of
            // entries together where the other keeps them apart, and every pixel of both entries
            // then reads as "differing". Ask the weaker question before reporting a failure — do
            // the regions line up? — because that is the one about emulation rather than tables.
            let (horizontal, vertical, across, down) =
                frame::edge_disagreement(&ours_pixels, &reference_pixels, 256, 240);

            if horizontal == 0 && vertical == 0 {
                println!(
                    "SAME LAYOUT — every region begins and ends where the reference's does, so the \
                     picture agrees and only the two palette tables differ"
                );
                return Ok(());
            }

            let rows: std::collections::BTreeSet<usize> =
                differing.iter().map(|pixel| pixel / 256).collect();
            println!("{} of {} pixels differ, across {} row(s)", differing.len(), ours_pixels.len() / 3, rows.len());
            println!("  rows: {:?}", rows.iter().take(20).collect::<Vec<_>>());
            println!(
                "  edges disagree at {horizontal} of {across} positions across ({:.2}%) and \
                 {vertical} of {down} down ({:.2}%)",
                100.0 * horizontal as f64 / across as f64,
                100.0 * vertical as f64 / down as f64,
            );
            anyhow::bail!("the frame does not match its reference")
        },
        Command::Suite { directory, budget } => run_suite(&directory, budget),
    }
}

/// Report a missing input as a clear message rather than an error, so a checkout without ROMs
/// stays green. Returns true when the caller should stop.
fn missing(path: &Path, what: &str) -> bool {
    if path.exists() {
        return false;
    }

    println!("SKIP  {what} not found at {}", path.display());
    println!("      Test ROMs are not distributed with this repository; see CONFORMANCE_PLAN.md.");
    true
}

fn run_nestest(rom: &Path, log: &Path, limit: usize) -> Result<()> {
    if missing(rom, "nestest.nes") || missing(log, "nestest.log") {
        return Ok(());
    }

    println!("Running nestest against {}\n", log.display());
    let outcome = nestest::run(rom, log, limit)?;

    match outcome.divergence {
        None => {
            println!("PASS  {} instructions matched the log", outcome.instructions);
            Ok(())
        },
        Some(divergence) => {
            println!("FAIL  diverged at log line {}", divergence.line);
            println!("      after {} matching instructions", outcome.instructions);
            println!("      expected  {}", divergence.expected);
            println!("      actual    {}", divergence.actual);
            println!("      differing fields: {}", divergence.fields.join(", "));
            std::process::exit(1);
        },
    }
}

fn run_one(rom: &Path, budget: usize) -> Result<()> {
    if missing(rom, "ROM") {
        return Ok(());
    }

    let outcome = blargg::run(rom, budget)?;
    report(rom, &outcome);

    if matches!(outcome.status, blargg::Status::Passed) {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

fn run_frame(
    rom: &Path,
    out: Option<&Path>,
    ascii: bool,
    options: frame::Options<'_>,
) -> Result<()> {
    let (frames, state) = (options.frames, options.state);
    if missing(rom, "ROM") {
        return Ok(());
    }
    if state.is_some_and(|path| missing(path, "save state")) {
        return Ok(());
    }

    println!("Running {} for {frames} frames\n", rom.display());
    let capture = frame::capture(rom, options)?;

    if let Some(reason) = &capture.stopped {
        println!("  WARNING  {reason}");
    }

    println!("  instructions      {}", capture.instructions);
    println!("  frames            {}", capture.diagnostics.frames);
    println!(
        "  scanlines drawn   {} / 240{}",
        capture.diagnostics.scanlines_rendered,
        if capture.diagnostics.scanlines_rendered == 240 { "" } else { "   <-- partial frame" }
    );
    println!(
        "  mid-frame toggles {}{}",
        capture.diagnostics.mid_frame_toggles,
        if capture.diagnostics.mid_frame_toggles > 0 {
            format!("   (last at scanline {})", capture.diagnostics.last_toggle_scanline)
        } else {
            String::new()
        }
    );
    println!("  blank frames      {}", capture.diagnostics.blank_frames);
    println!("  distinct colours  {}", capture.distinct_colours);
    println!("  coverage          {:.1}%", capture.coverage * 100.0);

    if capture.distinct_colours <= 1 {
        println!("  BLANK — the PPU drew a single flat colour, so nothing was rendered");
    }

    // Where the game turned rendering off and on, which is a visible edge when it happens partway
    // down a line.
    let masks = &capture.diagnostics.mask_writes;
    let toggles: Vec<_> = masks
        .iter()
        .filter(|&&(scanline, _, _)| (0..240).contains(&scanline))
        .collect();
    if !toggles.is_empty() {
        println!("\n  $2001 writes during the visible picture ({} of {} in the frame)", toggles.len(), masks.len());
        for &&(scanline, dot, value) in toggles.iter().take(12) {
            let showing = if value & 0b0001_1000 != 0 { "rendering ON " } else { "rendering OFF" };
            println!("    scanline {scanline:3}  dot {dot:3}  -> ${value:02X}  {showing}");
        }
    }

    // Where the game moved the scroll, and — the part that matters — at which dot. A `$2006` write
    // in hblank moves the split cleanly; the same write inside the visible line redraws the rest of
    // that line from the new address. Only the dot tells the two apart.
    let writes = &capture.diagnostics.address_writes;
    if !writes.is_empty() {
        println!("\n  $2006 writes ({} in the frame)", writes.len());
        for &(scanline, dot, address) in writes.iter().take(16) {
            // Only scanlines 0-239 draw anything, so a write during vblank is harmless whatever
            // dot it lands on — most of a game's video memory is written there. Within a visible
            // line, dots 1-256 are the picture and everything after is hblank.
            let where_ = if !(0..240).contains(&scanline) {
                "vblank"
            } else if dot <= 256 {
                "VISIBLE — redraws the rest of the line from here"
            } else {
                "hblank"
            };
            println!("    scanline {scanline:3}  dot {dot:3}  -> ${address:04X}   {where_}");
        }
        if writes.len() > 16 {
            println!("    ... and {} more", writes.len() - 16);
        }
    }

    if ascii {
        println!("\n{}", frame::to_ascii(&capture.pixels, 64, 30));
    }

    if let Some(path) = out {
        frame::write_ppm(path, &capture.pixels)?;
        println!("\nWrote {}", path.display());
    }

    Ok(())
}

fn run_suite(directory: &Path, budget: usize) -> Result<()> {
    if missing(directory, "ROM directory") {
        return Ok(());
    }

    let mut roms = Vec::new();
    collect_roms(directory, &mut roms).context("scanning for ROMs")?;
    roms.sort();

    if roms.is_empty() {
        println!("SKIP  no .nes files under {}", directory.display());
        return Ok(());
    }

    println!("Running {} ROM(s) from {}\n", roms.len(), directory.display());

    let mut failures = 0;
    let mut skipped = 0;
    for rom in &roms {
        if let Some(reason) = not_for_this_machine(rom) {
            skipped += 1;
            println!("  SKIP     {}  ({reason})", rom.file_name().unwrap_or_default().to_string_lossy());
            continue;
        }

        match blargg::run(rom, budget) {
            Ok(outcome) => {
                if !matches!(outcome.status, blargg::Status::Passed) {
                    failures += 1;
                }
                report(rom, &outcome);
            },
            Err(error) => {
                failures += 1;
                println!("ERROR {}  {error}", rom.display());
            },
        }
    }

    let run = roms.len() - skipped;
    print!("\n{} passed, {failures} failed", run - failures);
    if skipped > 0 {
        print!(", {skipped} skipped");
    }
    println!();
    if failures > 0 {
        std::process::exit(1);
    }
    Ok(())
}

/// Why this ROM cannot be judged on this emulator, if it cannot.
///
/// `pal_apu_tests` is the case that exists: the same suite as `blargg_apu_2005.07.30` with the
/// timings a PAL console has. This emulator is NTSC, so every one of them fails by design, and
/// counting them as failures put eleven permanent red marks in every sweep — the kind of noise that
/// teaches people to skim a failure list rather than read it.
///
/// Recognised by path and not by header, which is worth saying plainly because it is a heuristic
/// where one would rather have a fact: the PAL ROMs' iNES headers are byte-identical to their NTSC
/// siblings', flags 9 and 10 both zero. Only the directory says which is which.
fn not_for_this_machine(rom: &Path) -> Option<&'static str> {
    rom.components()
        .any(|part| part.as_os_str().to_string_lossy().starts_with("pal_"))
        .then_some("PAL timings; this emulator is NTSC")
}

fn report(rom: &Path, outcome: &blargg::Outcome) {
    let name = rom.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();

    let label = match outcome.status {
        blargg::Status::Passed => "PASS ".to_string(),
        blargg::Status::Failed { code } => format!("FAIL({code:02X})"),
        blargg::Status::TimedOut => "HUNG ".to_string(),
        blargg::Status::NoProtocol => "NOPROTO".to_string(),
    };

    // Marked, not hidden. A verdict read off the screen is this runner's interpretation of what a
    // ROM drew, where a $6000 result is the ROM's own word for it, and anyone doubting a result
    // should be able to see at a glance which kind it is.
    let source = match outcome.source {
        blargg::Source::Protocol => "",
        blargg::Source::Screen => " [screen]",
    };

    println!("  {label}{source}  {name}  ({} instructions)", outcome.instructions);

    if !outcome.message.is_empty() {
        for line in outcome.message.lines() {
            println!("          {line}");
        }
    }

    if !outcome.spinning_at.is_empty() {
        println!("          spinning in:");
        for (pc, text) in &outcome.spinning_at {
            println!("            ${pc:04X}  {text}");
        }
    }

    if matches!(outcome.status, blargg::Status::NoProtocol) {
        println!("          no $6000 signature, and nothing legible on screen either");
    }
}

fn collect_roms(directory: &Path, into: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_roms(&path, into)?;
        } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("nes")) {
            into.push(path);
        }
    }
    Ok(())
}
