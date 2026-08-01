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

mod blargg;
mod frame;
mod nestest;

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
        Command::Frame { rom, frames, out, ascii } => run_frame(&rom, frames, out.as_deref(), ascii),
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

fn run_frame(rom: &Path, frames: usize, out: Option<&Path>, ascii: bool) -> Result<()> {
    if missing(rom, "ROM") {
        return Ok(());
    }

    println!("Running {} for {frames} frames\n", rom.display());
    let capture = frame::capture(rom, frames)?;

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
    for rom in &roms {
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

    println!("\n{} passed, {failures} failed", roms.len() - failures);
    if failures > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn report(rom: &Path, outcome: &blargg::Outcome) {
    let name = rom.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();

    let label = match outcome.status {
        blargg::Status::Passed => "PASS ".to_string(),
        blargg::Status::Failed { code } => format!("FAIL({code:02X})"),
        blargg::Status::TimedOut => "HUNG ".to_string(),
        blargg::Status::NoProtocol => "NOPROTO".to_string(),
    };

    println!("  {label}  {name}  ({} instructions)", outcome.instructions);

    if !outcome.message.is_empty() {
        for line in outcome.message.lines() {
            println!("          {line}");
        }
    }

    if matches!(outcome.status, blargg::Status::NoProtocol) {
        println!("          ROM never wrote the $6000 signature — it may need a PPU or interrupts");
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
