//! The APU mixer.
//!
//! The NES does not sum its channels linearly. Each group of channels drives a resistor ladder
//! into a common node, so adding a second voice raises the output by less than the first did. The
//! NESdev wiki gives a closed-form approximation:
//!
//! ```text
//! pulse_out = 95.88 / (8128 / (pulse1 + pulse2) + 100)
//! tnd_out   = 159.79 / (1 / (triangle/8227 + noise/12241 + dmc/22638) + 100)
//! output    = pulse_out + tnd_out
//! ```
//!
//! and an equivalent pair of lookup tables, which is what this module implements:
//!
//! ```text
//! pulse_table[n] = 95.88  / (8128  / n + 100)     n = pulse1 + pulse2
//! tnd_table[n]   = 163.67 / (24329 / n + 100)     n = 3*triangle + 2*noise + dmc
//! ```
//!
//! The table form replaces the TND term's three exact weights with the integer weights 3/2/1,
//! which is what lets a single index stand in for all three channels; the difference from the
//! closed form is inaudible. Evaluating either per sample would cost two divisions on the hot
//! path, and the inputs are small integers, so precomputing is both faster and tiny: 31 entries
//! for the pulse sum (max 15+15) and 203 for the TND sum (max 3·15 + 2·15 + 127 = 202).
//!
//! Both terms are 0 when their inputs are all 0, and `output` lands in roughly 0.0..=1.0.
//!
//! Note that the result is unipolar — silence is 0.0, not the midpoint. That DC offset is real,
//! and it is removed downstream by the same high-pass filters the hardware uses, rather than by
//! fudging the mix here.

/// `PULSE_TABLE[n]` is the mixer output for `pulse1 + pulse2 == n`.
const PULSE_TABLE_LEN: usize = 31;

/// `TND_TABLE[n]` is the mixer output for `3*triangle + 2*noise + dmc == n`.
const TND_TABLE_LEN: usize = 203;

pub struct Mixer {
    pulse_table: [f32; PULSE_TABLE_LEN],
    tnd_table: [f32; TND_TABLE_LEN],
}

impl std::fmt::Debug for Mixer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The tables are constant; printing 234 floats helps nobody.
        f.write_str("Mixer { .. }")
    }
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
    }
}

impl Mixer {
    pub fn new() -> Self {
        let mut pulse_table = [0.0f32; PULSE_TABLE_LEN];
        for (n, entry) in pulse_table.iter_mut().enumerate().skip(1) {
            *entry = 95.88 / (8128.0 / n as f32 + 100.0);
        }

        let mut tnd_table = [0.0f32; TND_TABLE_LEN];
        for (n, entry) in tnd_table.iter_mut().enumerate().skip(1) {
            *entry = 163.67 / (24329.0 / n as f32 + 100.0);
        }

        Self { pulse_table, tnd_table }
    }

    /// Mix five raw DAC levels into one sample in roughly 0.0..=1.0.
    ///
    /// `pulse1`, `pulse2`, `triangle` and `noise` are 0..=15; `dmc` is 0..=127.
    pub fn mix(&self, pulse1: u8, pulse2: u8, triangle: u8, noise: u8, dmc: u8) -> f32 {
        let pulse_index = (pulse1 as usize + pulse2 as usize).min(PULSE_TABLE_LEN - 1);
        let tnd_index =
            (3 * triangle as usize + 2 * noise as usize + dmc as usize).min(TND_TABLE_LEN - 1);

        self.pulse_table[pulse_index] + self.tnd_table[tnd_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tolerance for comparing against the closed-form reference.
    const EPS: f32 = 1e-6;

    #[test]
    fn silence_is_exactly_zero() {
        let mixer = Mixer::new();
        assert_eq!(mixer.mix(0, 0, 0, 0, 0), 0.0);
    }

    #[test]
    fn pulse_matches_the_reference_formula() {
        let mixer = Mixer::new();

        for sum in 1..=30usize {
            let expected = 95.88 / (8128.0 / sum as f32 + 100.0);
            // Split the sum across both channels to prove only the sum matters.
            let p1 = (sum.min(15)) as u8;
            let p2 = (sum - p1 as usize) as u8;
            assert!(
                (mixer.mix(p1, p2, 0, 0, 0) - expected).abs() < EPS,
                "pulse sum {sum}: got {}, want {expected}",
                mixer.mix(p1, p2, 0, 0, 0)
            );
        }
    }

    #[test]
    fn tnd_matches_the_reference_formula() {
        let mixer = Mixer::new();

        for (triangle, noise, dmc) in [(15u8, 0u8, 0u8), (0, 15, 0), (0, 0, 127), (15, 15, 127), (7, 3, 64)] {
            let n = 3 * triangle as usize + 2 * noise as usize + dmc as usize;
            let expected = 163.67 / (24329.0 / n as f32 + 100.0);
            let got = mixer.mix(0, 0, triangle, noise, dmc);
            assert!(
                (got - expected).abs() < EPS,
                "tnd({triangle},{noise},{dmc}) index {n}: got {got}, want {expected}"
            );
        }
    }

    #[test]
    fn mixing_is_non_linear() {
        let mixer = Mixer::new();

        // Two pulse channels at level 8 must be quieter than twice one at level 8 — that
        // compression is the whole point of the resistor-ladder model.
        let one = mixer.mix(8, 0, 0, 0, 0);
        let two = mixer.mix(8, 8, 0, 0, 0);
        assert!(two > one, "adding a voice must increase output");
        assert!(two < 2.0 * one, "mixing must compress, got {two} vs linear {}", 2.0 * one);
    }

    #[test]
    fn full_scale_stays_in_range() {
        let mixer = Mixer::new();
        let peak = mixer.mix(15, 15, 15, 15, 127);
        assert!(peak > 0.5, "peak {peak} is implausibly quiet");
        // The hardware peak sits a hair above 1.0; anything much beyond that means broken scaling.
        assert!(peak < 1.05, "peak {peak} would clip badly");
    }

    #[test]
    fn output_is_monotonic_in_each_channel() {
        let mixer = Mixer::new();

        for level in 1..=15u8 {
            assert!(mixer.mix(level, 0, 0, 0, 0) > mixer.mix(level - 1, 0, 0, 0, 0));
            assert!(mixer.mix(0, 0, level, 0, 0) > mixer.mix(0, 0, level - 1, 0, 0));
        }
    }
}
