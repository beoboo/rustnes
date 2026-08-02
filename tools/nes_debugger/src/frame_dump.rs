//! Capture the current frame and the PPU state that produced it.
//!
//! A rendering fault reported from play — "the plant is in front of the pipe instead of inside it"
//! — cannot be chased without knowing what the PPU was actually told. The picture alone does not
//! say whether a sprite was placed wrongly, given the wrong tile, or drawn with the wrong
//! priority; the registers and OAM do. Dumping both together at a moment the player chooses turns
//! a description into a measurement.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rn_core::system::NesSystem;

const WIDTH: usize = 256;
const HEIGHT: usize = 240;

/// Write `frame-NNNN.ppm` and `frame-NNNN.txt` into `directory`, returning the image's path.
pub fn dump(system: &NesSystem, directory: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(directory)
        .with_context(|| format!("creating {}", directory.display()))?;

    let ppu = system.ppu();
    let frame = ppu.frame_count();
    let image_path = directory.join(format!("frame-{frame:04}.ppm"));

    let pixels = ppu.frame_buffer();
    // PPM is a three-line header followed by raw RGB, which every image viewer reads and which
    // needs no encoding dependency.
    let mut image = format!("P6\n{WIDTH} {HEIGHT}\n255\n").into_bytes();
    image.extend_from_slice(&pixels[..(WIDTH * HEIGHT * 3).min(pixels.len())]);
    std::fs::write(&image_path, image)
        .with_context(|| format!("writing {}", image_path.display()))?;

    std::fs::write(directory.join(format!("frame-{frame:04}.txt")), describe(system))?;
    Ok(image_path)
}

fn describe(system: &NesSystem) -> String {
    let ppu = system.ppu();
    let diagnostics = ppu.diagnostics();
    let ctrl = ppu.ctrl();
    let tall = (ctrl & 0x20) != 0;

    let mut out = String::new();
    out.push_str(&format!("frame            {}\n", ppu.frame_count()));
    out.push_str(&format!("PPUCTRL          ${ctrl:02X}\n"));
    out.push_str(&format!("  sprite size    {}\n", if tall { "8x16" } else { "8x8" }));
    out.push_str(&format!(
        "  sprite tiles   {}\n",
        // In 8x16 mode this bit is ignored: each sprite picks its own table from tile bit 0.
        if tall { "per sprite (bit 0 of the tile index)".into() } else { format!("${:04X}", if ctrl & 0x08 != 0 { 0x1000 } else { 0 }) }
    ));
    out.push_str(&format!("PPUMASK          ${:02X}\n", ppu.mask()));
    out.push_str(&format!("mirroring        {:?}\n", ppu.mirroring()));
    out.push_str(&format!("sprite-0 hit     line {}\n", diagnostics.sprite_zero_hit_scanline));
    out.push_str(&format!("scanlines drawn  {}\n", diagnostics.scanlines_rendered));
    out.push_str("scroll (line, v, fine x)\n");
    for (line, v, fine_x) in &diagnostics.scroll_changes {
        out.push_str(&format!(
            "  line {line:3}  v=${v:04X}  nametable {}  coarse ({:2},{:2})  fine ({},{})\n",
            (v >> 10) & 3,
            v & 0x1F,
            (v >> 5) & 0x1F,
            fine_x,
            (v >> 12) & 7
        ));
    }

    out.push_str("\nOAM — sprites with y < 240, in priority order (lower index draws in front)\n");
    out.push_str("  idx    x    y  tile  attr  palette  flip     priority\n");
    let oam = ppu.oam();
    for (index, sprite) in oam.chunks_exact(4).enumerate() {
        let (y, tile, attributes, x) = (sprite[0], sprite[1], sprite[2], sprite[3]);
        // A y of 240 or more is how a game parks a sprite off-screen.
        if y >= 240 {
            continue;
        }
        // OAM stores y one less than where the sprite is drawn.
        let screen_y = y as u16 + 1;
        let flip = match (attributes & 0x40 != 0, attributes & 0x80 != 0) {
            (false, false) => "none",
            (true, false) => "h",
            (false, true) => "v",
            (true, true) => "h+v",
        };
        out.push_str(&format!(
            "  {index:3}  {x:3}  {screen_y:3}   ${tile:02X}   ${attributes:02X}        {}  {flip:4}  {}\n",
            attributes & 3,
            if attributes & 0x20 != 0 { "behind background" } else { "in front" }
        ));
        if tall {
            // In 8x16 mode bit 0 of the tile index selects the pattern table and the rest is the
            // top tile, with the bottom being the one after it. Spelling this out because a plant
            // emerging from a pipe is exactly where an 8x16 mistake shows up.
            out.push_str(&format!(
                "        8x16: table ${:04X}, tiles ${:02X} and ${:02X}\n",
                if tile & 1 != 0 { 0x1000 } else { 0 },
                tile & 0xFE,
                (tile & 0xFE) + 1
            ));
        }
    }
    out
}
