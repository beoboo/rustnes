//! `apu_probe` — headless APU harness.
//!
//! Runs a 6502 program with no window and no sound card, captures exactly what the APU produced,
//! and measures it. Every audio defect this project has had was a property of the *signal* — wrong
//! sample rate, wrong pitch, DC offset instead of a waveform, amplitude an order of magnitude too
//! low — and all of them are visible in a few numbers here, whereas in the GUI they all sound
//! vaguely like "broken".
//!
//! ```text
//! apu_probe list
//! apu_probe run pulse
//! apu_probe run triangle --seconds 3 --out /tmp/tri.wav
//! apu_probe run --asm asm/simple_tone_test.asm
//! apu_probe check
//! ```

mod analysis;
mod programs;
mod wav;

use std::{
    path::PathBuf,
    sync::mpsc::{channel, Sender},
};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use rn_core::{apu::CPU_CLOCK_RATE, audio::SampleProducer, cpu::Assembler, system::NesSystem};

const LOAD_ADDRESS: u16 = 0x8000;

#[derive(Parser, Debug)]
#[command(author, version, about = "Run a program headlessly and measure the audio it produces")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// List the built-in test programs
    List,

    /// Run a program and measure its output
    Run {
        /// Name of a built-in preset (see `list`). Omit when using --asm.
        preset: Option<String>,

        /// Run a 6502 source file instead of a preset
        #[arg(long, value_name = "FILE")]
        asm: Option<PathBuf>,

        /// Seconds of emulated time to run
        #[arg(long, default_value_t = 2.0)]
        seconds: f64,

        /// Output sample rate in Hz
        #[arg(long, default_value_t = 48_000.0)]
        rate: f64,

        /// Write the capture to a WAV file
        #[arg(long, value_name = "FILE")]
        out: Option<PathBuf>,

        /// Print the first N samples, for eyeballing the waveform directly
        #[arg(long, value_name = "N")]
        dump: Option<usize>,

        /// Break the capture into N windows and report pitch and level for each.
        /// Use this for anything that changes over time — a melody, an envelope, a sweep.
        #[arg(long, value_name = "N")]
        segments: Option<usize>,
    },

    /// Run every preset and report pass/fail — a quick whole-pipeline health check
    Check {
        /// Seconds of emulated time per preset
        #[arg(long, default_value_t = 1.0)]
        seconds: f64,

        /// Output sample rate in Hz
        #[arg(long, default_value_t = 48_000.0)]
        rate: f64,
    },
}

/// Captures everything the APU emits.
struct Capture(Sender<f32>);

impl SampleProducer<f32> for Capture {
    fn set_volume(&mut self, _volume: f32) {}
    fn set_muted(&mut self, _muted: bool) {}
    fn produce(&mut self, sample: f32) {
        let _ = self.0.send(sample);
    }
}

/// Assemble `source`, run it for `seconds` of emulated time, return every sample it played.
fn capture(source: &str, seconds: f64, sample_rate: f64) -> Result<Vec<f32>> {
    let mut assembler = Assembler::new(LOAD_ADDRESS).with_nes_segments();
    let segments = assembler
        .assemble_program(source)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("assembling the program")?;

    let code = segments
        .get("STARTUP")
        .context("program has no STARTUP segment")?;

    let mut system = NesSystem::new();
    let (sender, receiver) = channel();
    system.connect_audio_output(Box::new(Capture(sender)), sample_rate);
    system
        .load_program(code, LOAD_ADDRESS)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("loading the program")?;

    let target = (CPU_CLOCK_RATE * seconds) as u64;
    let mut cycles = 0u64;
    while cycles < target {
        match system.step() {
            Ok(step_cycles) => cycles += step_cycles.max(1) as u64,
            Err(error) => {
                // Report where it died rather than discarding the capture: partial output is often
                // exactly what identifies the fault.
                eprintln!(
                    "warning: emulation stopped after {cycles} cycles at PC ${:04X}: {error}",
                    system.cpu().pc()
                );
                break;
            },
        }
    }

    Ok(receiver.try_iter().collect())
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::List => {
            println!("Built-in programs:\n");
            for preset in programs::all() {
                println!("  {:<12} {}", preset.name, preset.description);
                if let Some(hz) = preset.expected_hz {
                    println!("  {:<12} expected pitch: {hz:.1} Hz", "");
                }
            }
            println!("\nRun one with:  apu_probe run <name>");
        },

        Command::Run {
            preset,
            asm,
            seconds,
            rate,
            out,
            dump,
            segments,
        } => {
            let (label, source, expected_hz) = resolve(preset, asm)?;

            println!("Running {label} for {seconds}s at {rate:.0} Hz\n");
            let samples = capture(&source, seconds, rate)?;

            let result = analysis::analyse(&samples, rate, seconds);
            result.report();
            if let Some(hz) = expected_hz {
                result.report_expected_pitch(hz);
            }

            if let Some(count) = segments {
                println!("\nOver time ({count} windows)");
                println!("  {:>8}  {:>10}  {:>7}", "start", "pitch", "peak");
                for (start, hz, peak) in analysis::segments(&samples, rate, count) {
                    println!("  {start:7.2}s  {hz:9.1} Hz  {peak:7.4}");
                }
            }

            if let Some(count) = dump {
                println!("\nFirst {count} samples");
                for (i, sample) in samples.iter().take(count).enumerate() {
                    println!("  {i:>6}  {sample:+.6}");
                }
            }

            if let Some(path) = out {
                wav::write_mono_16bit(&path, &samples, rate as u32)?;
                println!("\nWrote {} samples to {}", samples.len(), path.display());
            }
        },

        Command::Check { seconds, rate } => {
            let mut failures = 0;

            for preset in programs::all() {
                let samples = capture(preset.source, seconds, rate)?;
                let result = analysis::analyse(&samples, rate, seconds);
                let mut problems = Vec::new();

                if result.rate_error > 0.01 {
                    problems.push(format!("sample rate off by {:.1}%", result.rate_error * 100.0));
                }

                if preset.name == "silence" {
                    if !result.silent {
                        problems.push(format!("expected silence, got peak {:.4}", result.peak));
                    }
                } else {
                    if result.silent {
                        problems.push("no output".to_string());
                    }
                    if result.dc_offset.abs() > 0.01 {
                        problems.push(format!("DC offset {:+.4}", result.dc_offset));
                    }
                    if result.clipped > 0 {
                        problems.push(format!("{} clipped samples", result.clipped));
                    }
                    if !result.silent && result.peak < 0.02 {
                        problems.push(format!("inaudibly quiet (peak {:.4})", result.peak));
                    }
                    if let Some(expected) = preset.expected_hz {
                        let ratio = result.zero_crossing_hz / expected;
                        if (ratio - 1.0).abs() > 0.05 {
                            problems.push(format!(
                                "pitch {:.1} Hz, expected {expected:.1} Hz ({ratio:.2}x)",
                                result.zero_crossing_hz
                            ));
                        }
                    }
                }

                if problems.is_empty() {
                    println!("  PASS  {}", preset.name);
                } else {
                    failures += 1;
                    println!("  FAIL  {}  — {}", preset.name, problems.join("; "));
                }
            }

            println!();
            if failures > 0 {
                bail!("{failures} preset(s) failed");
            }
            println!("All presets passed.");
        },
    }

    Ok(())
}

/// Work out what to run: a named preset, or a source file.
fn resolve(preset: Option<String>, asm: Option<PathBuf>) -> Result<(String, String, Option<f64>)> {
    match (preset, asm) {
        (Some(_), Some(_)) => bail!("give either a preset name or --asm, not both"),

        (Some(name), None) => {
            let preset = programs::find(&name).with_context(|| {
                format!(
                    "unknown preset '{name}' (available: {})",
                    programs::all()
                        .iter()
                        .map(|p| p.name)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
            Ok((
                format!("preset '{}'", preset.name),
                preset.source.to_string(),
                preset.expected_hz,
            ))
        },

        (None, Some(path)) => {
            let source = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            Ok((format!("{}", path.display()), source, None))
        },

        (None, None) => bail!("give a preset name or --asm (see `apu_probe list`)"),
    }
}
