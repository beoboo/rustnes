//! Headless frame capture.
//!
//! The video counterpart to `apu_probe`: run a ROM for a number of frames, then look at what the
//! PPU actually drew. Without this, judging the PPU means opening the debugger and squinting —
//! which cannot be scripted, cannot run in CI, and cannot say *how* wrong a frame is.
//!
//! Output is a PPM, which is three lines of header followed by raw RGB. Writing it by hand avoids
//! an image-encoding dependency for something this simple, and every image viewer reads it.

use std::path::Path;

use anyhow::{bail, Context, Result};
use rn_core::{apu::CPU_CLOCK_RATE, cartridge::load_rom, system::NesSystem};

const WIDTH: usize = 256;
const HEIGHT: usize = 240;

/// CPU cycles in one NTSC video frame.
const CYCLES_PER_FRAME: f64 = CPU_CLOCK_RATE / 60.0;

pub struct Capture {
    pub pixels: Vec<u8>,
    pub instructions: usize,
    /// Distinct RGB values in the frame. One means a flat fill — nothing was drawn.
    pub distinct_colours: usize,
    /// Fraction of pixels that are not the most common colour, i.e. actual content.
    pub coverage: f32,
    /// Whether emulation stopped early, and why.
    pub stopped: Option<String>,
    /// What the PPU did while producing the capture.
    pub diagnostics: rn_core::ppu::FrameDiagnostics,
}

/// Run `rom` for `frames` video frames and capture the final framebuffer.
///
/// `state` resumes from a save state first, which is how a scene deep inside a game can be reached
/// without playing it: the split in Super Mario Bros 3's status bar is a hundred frames into a
/// level and cannot be got to by booting and waiting.
///
/// `per_dot` selects the per-dot pixel path over the per-line one, so the two can be compared on a
/// real ROM rather than only on a synthetic scene.
pub fn capture(
    rom_path: &Path,
    frames: usize,
    state: Option<&Path>,
    per_dot: bool,
) -> Result<Capture> {
    let rom = load_rom(rom_path)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| format!("loading {}", rom_path.display()))?;

    let mut system = NesSystem::new();
    system
        .load_rom(&rom)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("loading the ROM into the system")?;

    if let Some(path) = state {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        let saved = serde_json::from_str(&text).context("parsing the save state")?;
        system
            .load_state(&saved)
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("restoring the save state")?;
    }

    system.ppu().set_per_dot_pixels(per_dot);

    let target = (CYCLES_PER_FRAME * frames as f64) as u64;
    let mut cycles = 0u64;
    let mut instructions = 0usize;
    let mut stopped = None;

    while cycles < target {
        match system.step() {
            Ok(step_cycles) => {
                cycles += step_cycles.max(1) as u64;
                instructions += 1;
            },
            Err(error) => {
                // Keep the partial frame: what a ROM managed to draw before dying is usually the
                // most informative thing about where it died.
                stopped = Some(format!("stopped at PC ${:04X}: {error}", system.cpu().pc()));
                break;
            },
        }
    }

    let pixels = system.ppu().frame_buffer();
    if pixels.len() < WIDTH * HEIGHT * 3 {
        bail!("frame buffer is {} bytes, expected {}", pixels.len(), WIDTH * HEIGHT * 3);
    }

    let (distinct_colours, coverage) = analyse(&pixels);
    let diagnostics = system.ppu().diagnostics();

    Ok(Capture {
        pixels,
        instructions,
        distinct_colours,
        coverage,
        stopped,
        diagnostics,
    })
}

/// Count distinct colours and how much of the frame differs from the dominant one.
fn analyse(pixels: &[u8]) -> (usize, f32) {
    use std::collections::HashMap;

    let mut counts: HashMap<[u8; 3], usize> = HashMap::new();
    for pixel in pixels.chunks_exact(3) {
        *counts.entry([pixel[0], pixel[1], pixel[2]]).or_default() += 1;
    }

    let total = (pixels.len() / 3) as f32;
    let dominant = counts.values().copied().max().unwrap_or(0) as f32;
    (counts.len(), (total - dominant) / total)
}

pub fn write_ppm(path: &Path, pixels: &[u8]) -> Result<()> {
    let mut data = format!("P6\n{WIDTH} {HEIGHT}\n255\n").into_bytes();
    data.extend_from_slice(&pixels[..WIDTH * HEIGHT * 3]);
    std::fs::write(path, data).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// Render the frame as ASCII, so a capture can be judged in a terminal.
///
/// Downsamples by averaging blocks and maps brightness onto a ramp — enough to tell "a title
/// screen" from "diagonal stripes" from "nothing at all", which is the question being asked.
pub fn to_ascii(pixels: &[u8], columns: usize, rows: usize) -> String {
    const RAMP: &[u8] = b" .:-=+*#%@";

    let mut out = String::new();
    for row in 0..rows {
        for column in 0..columns {
            let x0 = column * WIDTH / columns;
            let x1 = ((column + 1) * WIDTH / columns).max(x0 + 1);
            let y0 = row * HEIGHT / rows;
            let y1 = ((row + 1) * HEIGHT / rows).max(y0 + 1);

            let mut sum = 0u64;
            let mut count = 0u64;
            for y in y0..y1.min(HEIGHT) {
                for x in x0..x1.min(WIDTH) {
                    let i = (y * WIDTH + x) * 3;
                    // Rough luminance; exact weights do not matter for a terminal preview.
                    sum += pixels[i] as u64 + pixels[i + 1] as u64 + pixels[i + 2] as u64;
                    count += 3;
                }
            }

            let brightness = if count == 0 { 0 } else { (sum / count) as usize };
            out.push(RAMP[brightness * (RAMP.len() - 1) / 255] as char);
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(colour: [u8; 3]) -> Vec<u8> {
        colour.iter().copied().cycle().take(WIDTH * HEIGHT * 3).collect()
    }

    #[test]
    fn a_flat_frame_reports_one_colour_and_no_coverage() {
        // This is what "the PPU drew nothing" looks like, and the whole point of the measurement:
        // a blank frame and a rendered one are indistinguishable without it.
        let (colours, coverage) = analyse(&solid([0x1D, 0x1D, 0x1D]));
        assert_eq!(colours, 1);
        assert_eq!(coverage, 0.0);
    }

    #[test]
    fn coverage_counts_pixels_differing_from_the_dominant_colour() {
        let mut pixels = solid([0, 0, 0]);
        // Make a quarter of the frame white.
        for pixel in pixels.chunks_exact_mut(3).take(WIDTH * HEIGHT / 4) {
            pixel.copy_from_slice(&[255, 255, 255]);
        }

        let (colours, coverage) = analyse(&pixels);
        assert_eq!(colours, 2);
        assert!((coverage - 0.25).abs() < 0.001, "coverage was {coverage}");
    }

    #[test]
    fn ascii_maps_brightness_onto_the_ramp() {
        let dark = to_ascii(&solid([0, 0, 0]), 8, 2);
        let bright = to_ascii(&solid([255, 255, 255]), 8, 2);

        assert!(dark.trim_end().chars().all(|c| c == ' ' || c == '\n'), "black should be blank");
        assert!(bright.contains('@'), "white should reach the top of the ramp");
        assert_eq!(dark.lines().count(), 2, "one line per requested row");
        assert_eq!(dark.lines().next().map(str::len), Some(8), "one character per column");
    }

    #[test]
    fn ppm_has_the_expected_header_and_payload_size() {
        let path = std::env::temp_dir().join("rn_frame_test.ppm");
        write_ppm(&path, &solid([1, 2, 3])).expect("writing the image");

        let written = std::fs::read(&path).expect("reading it back");
        assert!(written.starts_with(b"P6\n256 240\n255\n"));
        assert_eq!(written.len(), 15 + WIDTH * HEIGHT * 3);
    }
}
