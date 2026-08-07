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
use rn_core::{
    apu::CPU_CLOCK_RATE,
    cartridge::load_rom,
    input::{ControllerButton, ControllerState},
    system::NesSystem,
};

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
/// Carry Super Mario Bros 3 from its title screen into a level: Start, four times, with pauses.
///
/// Hard-coded rather than made scriptable, because it is not a general facility — it is the one
/// sequence needed to reach the one scene two emulators have to be compared at. `nesref` carries
/// the identical sequence, which is the whole point: a save state reaches this scene in a second
/// but only this emulator can read one, and that is exactly why no reference picture of it existed.
pub fn into_a_level(system: &mut NesSystem) {
    let run = |system: &mut NesSystem, frames: u64| {
        let target = system.ppu().frame_count() + frames;
        while system.ppu().frame_count() < target {
            if system.step().is_err() {
                return;
            }
        }
    };

    let tap = |system: &mut NesSystem, button: ControllerButton, after: u64| {
        let mut pressed = ControllerState::new();
        pressed.set_button(button, true);
        system.set_controller1_state(pressed);
        run(system, 8);
        system.set_controller1_state(ControllerState::new());
        run(system, after);
    };

    run(system, 240);
    // Four Starts carry the title screen through to the world map...
    for _ in 0..4 {
        tap(system, ControllerButton::Start, 68);
    }
    run(system, 120);
    // ...where Mario stands on the START panel, on which A does nothing — measured, not assumed:
    // this sequence used to tap A alone here and its frame dump was still the world map. Right
    // and then Up walk him onto the level 1 panel — each step read off a frame dump, not recalled
    // from the game...
    tap(system, ControllerButton::Right, 60);
    tap(system, ControllerButton::Up, 60);
    // ...and A enters it, which is the scene wanted: its status bar is split with `$2006` writes,
    // where the map's is split with `$2005`, and only the first goes wrong.
    tap(system, ControllerButton::A, 240);
    run(system, 180);
}

/// What a capture should do beyond "run the ROM": which scene to reach, how to draw it, and which
/// console to be. Grouped rather than passed as six positional flags, all of which are `bool`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Options<'a> {
    pub frames: usize,
    /// Resume from a save state before running, to reach a scene inside a game.
    pub state: Option<&'a Path>,
    /// Draw from the per-dot path rather than the per-line one.
    pub per_dot: bool,
    /// Drive Super Mario Bros 3 into a level first.
    pub into_level: bool,
    /// Force PAL, for the cartridges whose header claims NTSC and lies.
    pub force_pal: bool,
}

pub fn capture(rom_path: &Path, options: Options<'_>) -> Result<Capture> {
    let Options { frames, state, per_dot, into_level, force_pal } = options;
    let rom = load_rom(rom_path)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| format!("loading {}", rom_path.display()))?;

    let mut system = NesSystem::new();

    // The header first, then the override. Byte 9's TV-system bit is right for most cartridges and
    // wrong for a good many European ones — `super-mario-3-eu.nes` claims NTSC and is PAL — which
    // is why reference emulators keep a database of cartridge hashes and why this takes a flag.
    let region = if force_pal { rn_core::region::Region::Pal } else { rom.header.region };
    system.set_region(region);

    system
        .load_rom(&rom)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("loading the ROM into the system")?;

    if into_level {
        into_a_level(&mut system);
    }

    if let Some(path) = state {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        let saved = serde_json::from_str(&text).context("parsing the save state")?;
        system
            .load_state(&saved)
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("restoring the save state")?;
    }

    if per_dot || std::env::var_os("RN_PER_DOT").is_some() {
        system.ppu().set_per_dot_pixels(true);
    }

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

/// Compare two captured frames, ignoring which RGB each emulator gives a palette entry.
///
/// Two emulators ship different NES palette tables. This one renders black as `0,0,0` and tetanes
/// as `3,3,3`, so a byte-for-byte comparison of the same picture reports all 61,440 pixels as
/// differing and means nothing by it. That mistake was made and published before it was caught: a
/// pair of flat backdrops differing only in shade was written up as differing power-on palette RAM.
///
/// What matters is whether the same regions got the same palette *entry*, which a consistent
/// one-to-one mapping between the two pictures' colour sets captures exactly. A pixel is counted as
/// differing when it breaks a mapping an earlier pixel established — in either direction, so that
/// two of our colours cannot both map onto one of theirs.
/// Where the two frames disagree about *edges* — the positions at which the colour changes from
/// the pixel to the left, or from the one above.
///
/// Returns the number of disagreeing horizontal positions, then vertical, then how many positions
/// of each were examined.
///
/// This exists because [`structural_diff`] asks for something two emulators cannot always give.
/// It requires a bijection between the two colour sets, which fails the moment one side's palette
/// quantises a pair of entries to the same RGB where the other keeps them apart — and two
/// independently derived palettes will always differ somewhere. That is a difference in the
/// *table*, not in the emulation, and reporting it as "the frame does not match" hides whether the
/// picture is right.
///
/// An edge map does not care what the colours are, only where they change, so it answers the
/// question actually being asked: did the same regions come out in the same places? On
/// `blargg_litewall`'s demos it reads 0.00% against a reference whose exact RGB differs in almost
/// every pixel.
pub fn edge_disagreement(
    ours: &[u8],
    reference: &[u8],
    width: usize,
    height: usize,
) -> (usize, usize, usize, usize) {
    let at = |data: &[u8], x: usize, y: usize| {
        let i = (y * width + x) * 3;
        [data[i], data[i + 1], data[i + 2]]
    };

    let (mut horizontal, mut vertical) = (0, 0);
    for y in 0..height {
        for x in 1..width {
            let ours_edge = at(ours, x, y) != at(ours, x - 1, y);
            let reference_edge = at(reference, x, y) != at(reference, x - 1, y);
            horizontal += usize::from(ours_edge != reference_edge);
        }
    }
    for y in 1..height {
        for x in 0..width {
            let ours_edge = at(ours, x, y) != at(ours, x, y - 1);
            let reference_edge = at(reference, x, y) != at(reference, x, y - 1);
            vertical += usize::from(ours_edge != reference_edge);
        }
    }

    (horizontal, vertical, height * (width - 1), width * (height - 1))
}

pub fn structural_diff(ours: &[u8], reference: &[u8]) -> Vec<usize> {
    use std::collections::HashMap;

    let mut forward: HashMap<[u8; 3], [u8; 3]> = HashMap::new();
    let mut backward: HashMap<[u8; 3], [u8; 3]> = HashMap::new();
    let mut differing = Vec::new();

    for (pixel, (a, b)) in ours.chunks_exact(3).zip(reference.chunks_exact(3)).enumerate() {
        let (a, b) = ([a[0], a[1], a[2]], [b[0], b[1], b[2]]);
        let consistent =
            *forward.entry(b).or_insert(a) == a && *backward.entry(a).or_insert(b) == b;
        if !consistent {
            differing.push(pixel);
        }
    }

    differing
}

/// Read a PPM written by [`write_ppm`] or by `nesref --frame`, returning its pixels.
pub fn read_ppm(path: &Path) -> Result<Vec<u8>> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;

    // Three header lines — magic, dimensions, maximum value — then the pixels.
    let mut start = 0;
    for _ in 0..3 {
        start += bytes[start..]
            .iter()
            .position(|&b| b == b'\n')
            .context("truncated PPM header")?
            + 1;
    }

    Ok(bytes[start..].to_vec())
}

#[cfg(test)]
mod palette_independent_diff {
    use super::*;

    /// The same picture in two palettes is the same picture.
    #[test]
    fn a_different_palette_for_the_same_regions_is_not_a_difference() {
        let ours = [0, 0, 0, 255, 255, 255, 0, 0, 0];
        let theirs = [3, 3, 3, 250, 250, 250, 3, 3, 3];
        assert!(structural_diff(&ours, &theirs).is_empty());
    }

    /// But a region that changed colour is, even if both palettes are otherwise consistent.
    #[test]
    fn a_region_that_took_a_different_entry_is_a_difference() {
        let ours = [0, 0, 0, 255, 255, 255, 0, 0, 0];
        let theirs = [3, 3, 3, 250, 250, 250, 250, 250, 250];
        assert_eq!(structural_diff(&ours, &theirs), vec![2]);
    }

    /// And two of ours mapping onto one of theirs is a difference, not a match. Without checking
    /// both directions, a picture that lost a colour entirely would compare as identical.
    #[test]
    fn two_colours_collapsing_into_one_is_a_difference() {
        let ours = [0, 0, 0, 128, 128, 128];
        let theirs = [3, 3, 3, 3, 3, 3];
        assert_eq!(structural_diff(&ours, &theirs), vec![1]);
    }
}
