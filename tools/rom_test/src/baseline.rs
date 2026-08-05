//! Frame baselines that live in the repository rather than in `/tmp`.
//!
//! The rule this exists to serve is that anything touching rendering is gated on a pixel diff
//! against a saved frame, never on a summary statistic — a coverage percentage once hid a bug that
//! shifted the whole picture sixteen pixels sideways. The practice was sound and the *storage* was
//! not: the saved frames sat in `/tmp`, where nothing checks them and nothing keeps them. One of
//! them was found to disagree with an unmodified build, meaning it had quietly stopped being a gate
//! at some earlier commit and no one had been told. A gate that can go stale in silence is worse
//! than no gate, because it reads as evidence.
//!
//! The frames themselves cannot be committed: they are pictures of commercial games. Hashes can.
//! Each baseline stores one hash per scanline as well as one for the whole frame, so a mismatch
//! says *where* — "rows 192 to 194 differ" is most of a diagnosis, and is the part a single
//! whole-frame hash would throw away. To see the pixels, re-render with `rom_test frame --out`.
//!
//! A missing ROM is a skip, never a failure. Nobody else has these files.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Where the baselines live, relative to the repository root.
pub const DEFAULT_PATH: &str = "tools/rom_test/frame_baselines.json";

/// The width of a frame in pixels, three bytes each.
const WIDTH: usize = 256;
const ROWS: usize = 240;

/// One captured frame, as hashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    /// The ROM's file name, looked up inside whichever directory the caller names.
    pub rom: String,
    /// Video frames to run before capturing.
    pub frames: usize,
    /// A save state to resume from first, for scenes that cannot be reached by booting and waiting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Hash of the whole frame.
    pub frame: String,
    /// One hash per scanline, so a mismatch can say which rows moved.
    pub rows: Vec<String>,
}

/// The file as a whole: baselines by name, so the order in the file is stable under editing.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Baselines {
    #[serde(flatten)]
    pub entries: BTreeMap<String, Baseline>,
}

/// FNV-1a, 64-bit.
///
/// Written out rather than pulled in, because a dependency for eleven lines of arithmetic is a
/// dependency to keep up to date forever. It is not a cryptographic hash and does not need to be:
/// nothing here is adversarial, and the only question asked of it is whether two renderings of the
/// same scene are byte for byte the same.
fn hash(bytes: &[u8]) -> String {
    let mut value: u64 = 0xcbf2_9ce4_8422_2325;
    for &byte in bytes {
        value ^= u64::from(byte);
        value = value.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{value:016x}")
}

/// Hash a frame whole and by row.
pub fn hashes(pixels: &[u8]) -> (String, Vec<String>) {
    let stride = WIDTH * 3;
    let rows = (0..ROWS)
        .map(|row| {
            let start = row * stride;
            hash(pixels.get(start..start + stride).unwrap_or(&[]))
        })
        .collect();
    (hash(pixels), rows)
}

/// What checking one baseline produced.
pub enum Outcome {
    Matched,
    /// The ROM is not in this checkout. Nothing is wrong; there is simply nothing to compare.
    Skipped,
    /// The rows that differ, and how many there are in total.
    Differed {
        rows: Vec<usize>,
    },
}

pub fn load(path: &Path) -> Result<Baselines> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

pub fn save(path: &Path, baselines: &Baselines) -> Result<()> {
    let text = serde_json::to_string_pretty(baselines).context("serialising the baselines")?;
    std::fs::write(path, format!("{text}\n")).with_context(|| format!("writing {}", path.display()))
}

/// Render a baseline's scene and compare it, or report that its ROM is absent.
pub fn check(baseline: &Baseline, roms: &Path) -> Result<Outcome> {
    let Some(capture) = render(baseline, roms)? else {
        return Ok(Outcome::Skipped);
    };

    let (frame, rows) = hashes(&capture);
    if frame == baseline.frame {
        return Ok(Outcome::Matched);
    }

    // Which rows moved. A whole-frame mismatch with no differing row means the frame is the wrong
    // *size*, which is worth saying rather than reporting as "no rows differ".
    let differing = rows
        .iter()
        .zip(&baseline.rows)
        .enumerate()
        .filter(|(_, (now, before))| now != before)
        .map(|(row, _)| row)
        .collect();

    Ok(Outcome::Differed { rows: differing })
}

/// Re-render a baseline's scene, or `None` if its ROM is not here.
fn render(baseline: &Baseline, roms: &Path) -> Result<Option<Vec<u8>>> {
    let rom = roms.join(&baseline.rom);
    if !rom.exists() {
        return Ok(None);
    }

    let state = baseline.state.as_ref().map(|name| roms.join(name));
    if let Some(path) = &state {
        if !path.exists() {
            return Ok(None);
        }
    }

    let capture = crate::frame::capture(&rom, baseline.frames, state.as_deref(), false)?;
    Ok(Some(capture.pixels))
}

/// Recompute a baseline's hashes from what the emulator draws now.
pub fn refresh(baseline: &mut Baseline, roms: &Path) -> Result<bool> {
    let Some(capture) = render(baseline, roms)? else {
        return Ok(false);
    };

    let (frame, rows) = hashes(&capture);
    baseline.frame = frame;
    baseline.rows = rows;
    Ok(true)
}

/// Check every baseline, printing a line each, and report whether all of them held.
pub fn report(path: &Path, roms: &Path) -> Result<bool> {
    let baselines = load(path)?;
    let mut all_held = true;
    let mut checked = 0;

    for (name, baseline) in &baselines.entries {
        match check(baseline, roms)? {
            Outcome::Matched => {
                checked += 1;
                println!("  MATCH    {name}");
            },
            Outcome::Skipped => println!("  SKIP     {name}  ({} is not in {})", baseline.rom, roms.display()),
            Outcome::Differed { rows } => {
                checked += 1;
                all_held = false;
                println!("  DIFFERS  {name}  {}", describe(&rows));
                println!("           re-render with: rom_test frame <rom> --frames {} --out /tmp/{name}.ppm", baseline.frames);
            },
        }
    }

    if checked == 0 {
        println!("\nNo baselines could be checked: none of their ROMs are in {}.", roms.display());
        println!("They are commercial games and cannot be committed; see CONFORMANCE_PLAN.md.");
    }

    Ok(all_held)
}

/// Say which rows moved, in ranges, because "192-194" reads and "192, 193, 194" does not.
fn describe(rows: &[usize]) -> String {
    if rows.is_empty() {
        return "the frame differs but no row does — it is the wrong size".to_string();
    }

    let mut ranges: Vec<String> = Vec::new();
    let mut start = rows[0];
    let mut previous = rows[0];

    for &row in &rows[1..] {
        if row != previous + 1 {
            ranges.push(range(start, previous));
            start = row;
        }
        previous = row;
    }
    ranges.push(range(start, previous));

    format!("{} of {ROWS} rows differ: {}", rows.len(), ranges.join(", "))
}

fn range(start: usize, end: usize) -> String {
    if start == end {
        format!("{start}")
    } else {
        format!("{start}-{end}")
    }
}

/// Rewrite every baseline whose ROM is present, leaving the rest alone.
pub fn update(path: &Path, roms: &Path) -> Result<()> {
    let mut baselines = load(path)?;

    for (name, baseline) in &mut baselines.entries {
        if refresh(baseline, roms)? {
            println!("  updated  {name}");
        } else {
            println!("  skipped  {name}  (no ROM)");
        }
    }

    save(path, &baselines)
}

/// The path to the baselines file, wherever the tool is run from.
pub fn default_path() -> PathBuf {
    PathBuf::from(DEFAULT_PATH)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The same pixels hash the same and different ones do not. Not profound, but a hash that
    /// ignored its input would make every baseline pass for ever.
    #[test]
    fn a_changed_pixel_changes_the_hash() {
        let mut pixels = vec![0u8; WIDTH * 3 * ROWS];
        let (frame, rows) = hashes(&pixels);

        pixels[WIDTH * 3 * 100 + 30] = 1;
        let (changed_frame, changed_rows) = hashes(&pixels);

        assert_ne!(frame, changed_frame);
        assert_ne!(rows[100], changed_rows[100], "the row holding the pixel");
        assert_eq!(rows[99], changed_rows[99], "and no other row");
        assert_eq!(rows[101], changed_rows[101]);
    }

    /// The row report is the whole point of storing rows: it has to name them.
    #[test]
    fn differing_rows_are_reported_as_ranges() {
        assert_eq!(describe(&[5]), "1 of 240 rows differ: 5");
        assert_eq!(describe(&[5, 6, 7]), "3 of 240 rows differ: 5-7");
        assert_eq!(describe(&[5, 6, 9, 20, 21]), "5 of 240 rows differ: 5-6, 9, 20-21");
    }

    /// A whole-frame mismatch with no differing row is not "nothing differs".
    #[test]
    fn a_frame_that_differs_in_no_row_says_so() {
        assert!(describe(&[]).contains("wrong size"));
    }
}
