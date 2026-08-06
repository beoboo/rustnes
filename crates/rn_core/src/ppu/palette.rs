//! The NES palette, all 512 entries of it, derived rather than tabulated.
//!
//! The PPU does not hold RGB anywhere. It emits a composite video signal, and the colour you see is
//! whatever a television made of that signal — which is why every emulator's palette differs
//! slightly and why there is no single correct table. What there *is* is the signal, which is
//! documented exactly: a square wave whose phase carries the hue, whose two voltage levels carry
//! the brightness, and which the de-emphasis bits attenuate over part of its cycle.
//!
//! So this decodes the signal instead of listing colours, following the generator described on the
//! NESdev wiki (Bisqwit's). It matters for two reasons beyond fidelity:
//!
//! - **There are 512 colours, not 64.** The three de-emphasis bits are part of the colour, and they
//!   do not attenuate uniformly: they darken the samples that fall in phase with one primary, which
//!   is a different amount for every hue. Approximating that as "multiply the other two channels by
//!   0.746" collapses entries that hardware keeps distinct — `full_palette` drew 223 distinct
//!   colours against a reference's 426 for exactly that reason, and no palette-independent
//!   comparison can succeed while two entries share one RGB value.
//! - **It is the honest answer to "where do these numbers come from".** A table of 64 hex triples
//!   copied from somewhere cannot be explained; twenty lines of signal decoding can.

/// Voltage the signal sits at for black, and for white, relative to the sync level.
const BLACK: f32 = 0.518;
const WHITE: f32 = 1.962;

/// How much a de-emphasis bit darkens the samples it applies to.
const ATTENUATION: f32 = 0.746;

/// The two voltages a colour's square wave alternates between, indexed by its brightness level:
/// the low four are the trough, the high four the crest.
const LEVELS: [f32; 8] = [0.350, 0.518, 0.962, 1.550, 1.094, 1.506, 1.962, 1.962];

/// Whether sample `p` of the twelve falls in the half-cycle where colour `hue` is high.
///
/// Twelve samples make one colour cycle, and a hue is just a phase offset into it — which is why
/// hues 1..=12 are the same brightness in a different order, and why hue 0 (all high) and hues
/// 13..=15 (all low) are grey.
fn in_phase(hue: i32, p: i32) -> bool {
    (hue + p + 8) % 12 < 6
}

/// Decode one of the 512 entries — `hue | level << 4 | emphasis << 6` — into RGB.
fn decode(index: usize) -> [u8; 3] {
    let hue = (index & 0x0F) as i32;
    let mut level = ((index >> 4) & 0x03) as i32;
    let emphasis = (index >> 6) & 0x07;

    // Hues 14 and 15 are black whatever the brightness bits say, so the level is forced.
    if hue > 13 {
        level = 1;
    }

    let mut low = LEVELS[level as usize];
    let mut high = LEVELS[4 + level as usize];
    // Hue 0 is the grey column: the wave never drops, so only the crest is emitted.
    if hue == 0 {
        low = high;
    }
    // Hues 13..=15 are the black column: the wave never rises.
    if hue >= 13 {
        high = low;
    }

    let (mut y, mut i, mut q) = (0.0f32, 0.0f32, 0.0f32);
    for p in 0..12 {
        let mut spot = if in_phase(hue, p) { high } else { low };

        // Each de-emphasis bit darkens the part of the cycle in phase with one primary — red at
        // phase 12, green at 4, blue at 8. It is *not* a uniform dimming, and that unevenness is
        // the whole reason a 512-entry table cannot be folded down into 64 plus a multiplier.
        if (emphasis & 1 != 0 && in_phase(12, p))
            || (emphasis & 2 != 0 && in_phase(4, p))
            || (emphasis & 4 != 0 && in_phase(8, p))
        {
            spot *= ATTENUATION;
        }

        // Normalise the sample to 0..1 against the black and white levels, then run it through an
        // ideal NTSC demodulator: the mean is luma, and the projections onto the colour subcarrier
        // in quadrature are the two chroma components.
        let v = (spot - BLACK) / (WHITE - BLACK) / 12.0;
        let phase = std::f32::consts::PI * (p as f32) / 6.0;
        y += v;
        i += v * phase.cos();
        q += v * phase.sin();
    }

    // The chroma components come out of that loop at half amplitude, because a sine averaged
    // against itself over a whole cycle is a half.
    i *= 2.0;
    q *= 2.0;

    // YIQ to RGB, by the FCC matrix, with a gamma step: the signal is meant for a CRT, whose
    // response is not linear, and skipping it leaves everything washed out.
    let channel = |value: f32| -> u8 {
        let corrected = if value <= 0.0 { 0.0 } else { value.powf(2.2 / 1.8) };
        (corrected * 255.0).clamp(0.0, 255.0) as u8
    };

    [
        channel(y + 0.946_882 * i + 0.623_557 * q),
        channel(y - 0.274_788 * i - 0.635_691 * q),
        channel(y - 1.108_545 * i + 1.709_007 * q),
    ]
}

/// The whole palette, built once on first use.
///
/// Computed rather than written down, and computed lazily rather than in a `const` because the
/// decoding needs `cos`, `sin` and `powf`, none of which are available in a constant.
pub fn table() -> &'static [[u8; 3]; 512] {
    static TABLE: std::sync::OnceLock<[[u8; 3]; 512]> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [[0u8; 3]; 512];
        for (index, entry) in table.iter_mut().enumerate() {
            *entry = decode(index);
        }
        table
    })
}

/// The RGB for a palette entry under a `$2001` mask, which supplies greyscale and de-emphasis.
///
/// `entry` is the six-bit value out of palette memory; `mask` is `$2001` as written.
pub fn rgb(entry: u8, mask: u8) -> [u8; 3] {
    // Greyscale ($2001 bit 0) forces every colour onto the grey column before anything else sees
    // it. Games use it for fades and flashes.
    let entry = if mask & 0x01 != 0 { entry & 0x30 } else { entry & 0x3F };
    let emphasis = (mask as usize >> 5) & 0x07;
    table()[(emphasis << 6) | entry as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// De-emphasis must not collapse entries that hardware keeps apart.
    ///
    /// This is the property the old 64-entry table plus a uniform multiplier failed, and the
    /// reason it failed silently: the picture looked right and only a palette-independent
    /// comparison against another emulator could see it. `full_palette` drew 223 distinct colours
    /// where the reference drew 426.
    #[test]
    fn every_emphasis_combination_is_a_distinct_palette() {
        let table = table();
        for emphasis in 1..8usize {
            let plain: Vec<_> = (0..64).map(|entry| table[entry]).collect();
            let emphasised: Vec<_> = (0..64).map(|entry| table[(emphasis << 6) | entry]).collect();
            assert_ne!(
                plain, emphasised,
                "emphasis {emphasis} produced the same 64 colours as no emphasis at all"
            );
        }
    }

    /// The table has to be *mostly* distinct, or a structural frame comparison cannot work: two
    /// palette entries sharing an RGB value make two different regions indistinguishable.
    #[test]
    fn the_table_is_overwhelmingly_distinct() {
        let unique: std::collections::HashSet<_> = table().iter().collect();
        assert!(
            unique.len() > 400,
            "only {} of 512 entries are distinct; a uniform attenuation would give far fewer",
            unique.len()
        );
    }

    /// The colourless columns come out colourless — and one of them is not black.
    ///
    /// Hues 14 and 15 have their brightness forced, so they are black at every level. Hue 13 does
    /// not: its wave is flat at its own level, which makes `$0D` and `$1D` black but `$2D` and
    /// `$3D` genuine greys. That is the "blacker than black" column, and it is why a game that
    /// uses `$0D` as a backdrop can push a television below its black level.
    #[test]
    fn the_colourless_columns_have_no_colour() {
        for level in 0..4usize {
            for hue in [0x0Eusize, 0x0F] {
                let [r, g, b] = table()[(level << 4) | hue];
                assert_eq!(
                    [r, g, b],
                    [0, 0, 0],
                    "hue {hue:X} has its level forced, so it is black at every brightness"
                );
            }
            for hue in [0x00usize, 0x0D] {
                let [r, g, b] = table()[(level << 4) | hue];
                let spread = r.abs_diff(g).max(g.abs_diff(b)).max(r.abs_diff(b));
                assert!(spread < 24, "hue {hue:X} level {level} should be grey, got {r},{g},{b}");
            }
        }
        // And the two halves of that column really do differ, which is the point of the test.
        assert_eq!(table()[0x0D], [0, 0, 0], "$0D is black");
        assert!(table()[0x3D][0] > 100, "$3D is a light grey, not black");
    }

    /// Greyscale collapses the hue and keeps the brightness, and the mask's emphasis bits still
    /// apply on top of it.
    #[test]
    fn greyscale_uses_the_grey_column() {
        assert_eq!(rgb(0x21, 0x01), rgb(0x20, 0x01), "greyscale must drop the hue");
        assert_ne!(rgb(0x21, 0x00), rgb(0x20, 0x00), "without it, the hue must survive");
        assert_ne!(rgb(0x21, 0x21), rgb(0x21, 0x01), "emphasis applies with greyscale too");
    }
}
