//! Output filters.
//!
//! The NES puts its mixed signal through analogue filtering before it reaches the speaker:
//! two high-pass stages at 90 Hz and 440 Hz, and a low-pass at 14 kHz. Emulating them is not
//! cosmetic — the mixer's output is unipolar (silence is 0.0, not the midpoint), and the 90 Hz
//! high-pass is what removes that DC offset. Without it every sample sits above zero and the
//! result is a thump rather than a tone.
//!
//! Both are one-pole IIR sections, which is all the hardware's single R-C stage amounts to.

use std::f32::consts::PI;

/// One-pole high-pass: passes change, blocks steady offsets.
#[derive(Debug, Clone)]
struct HighPass {
    alpha: f32,
    prev_input: f32,
    prev_output: f32,
}

impl HighPass {
    fn new(sample_rate: f32, cutoff_hz: f32) -> Self {
        let rc = 1.0 / (2.0 * PI * cutoff_hz);
        let dt = 1.0 / sample_rate;
        Self {
            alpha: rc / (rc + dt),
            prev_input: 0.0,
            prev_output: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.alpha * (self.prev_output + input - self.prev_input);
        self.prev_input = input;
        self.prev_output = output;
        output
    }

    fn reset(&mut self) {
        self.prev_input = 0.0;
        self.prev_output = 0.0;
    }
}

/// One-pole low-pass: rounds off the harshest edges of the square waves.
#[derive(Debug, Clone)]
struct LowPass {
    alpha: f32,
    prev_output: f32,
}

impl LowPass {
    fn new(sample_rate: f32, cutoff_hz: f32) -> Self {
        let rc = 1.0 / (2.0 * PI * cutoff_hz);
        let dt = 1.0 / sample_rate;
        Self {
            alpha: dt / (rc + dt),
            prev_output: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        self.prev_output += self.alpha * (input - self.prev_output);
        self.prev_output
    }

    fn reset(&mut self) {
        self.prev_output = 0.0;
    }
}

/// The NES output filter chain: 90 Hz high-pass → 440 Hz high-pass → 14 kHz low-pass.
#[derive(Debug, Clone)]
pub struct OutputFilter {
    high_pass_90: HighPass,
    high_pass_440: HighPass,
    low_pass_14k: LowPass,
}

impl OutputFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            high_pass_90: HighPass::new(sample_rate, 90.0),
            high_pass_440: HighPass::new(sample_rate, 440.0),
            // Guard against a nonsensical rate: a low-pass above Nyquist is meaningless.
            low_pass_14k: LowPass::new(sample_rate, 14_000.0f32.min(sample_rate / 2.0 - 1.0)),
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let sample = self.high_pass_90.process(sample);
        let sample = self.high_pass_440.process(sample);
        self.low_pass_14k.process(sample)
    }

    pub fn reset(&mut self) {
        self.high_pass_90.reset();
        self.high_pass_440.reset();
        self.low_pass_14k.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48_000.0;

    #[test]
    fn removes_dc_offset() {
        let mut filter = OutputFilter::new(SAMPLE_RATE);

        // A constant input is pure DC — exactly what the mixer's unipolar output looks like
        // during a steady tone. After the high-pass stages settle it must decay to ~0.
        let mut last = 0.0;
        for _ in 0..SAMPLE_RATE as usize {
            last = filter.process(0.5);
        }

        assert!(last.abs() < 1e-3, "DC offset survived filtering: {last}");
    }

    #[test]
    fn passes_audible_frequencies() {
        let mut filter = OutputFilter::new(SAMPLE_RATE);

        // A 1 kHz sine sits between the high-pass corners and the low-pass corner, so it should
        // come through with most of its amplitude intact.
        let mut peak: f32 = 0.0;
        for n in 0..SAMPLE_RATE as usize {
            let t = n as f32 / SAMPLE_RATE;
            let out = filter.process((2.0 * PI * 1000.0 * t).sin());
            if n > SAMPLE_RATE as usize / 2 {
                peak = peak.max(out.abs());
            }
        }

        assert!(peak > 0.8, "1 kHz was attenuated to {peak}");
    }

    #[test]
    fn attenuates_below_the_high_pass_corner() {
        let mut filter = OutputFilter::new(SAMPLE_RATE);

        // 20 Hz is well below both high-pass corners and should be heavily attenuated.
        let mut peak: f32 = 0.0;
        for n in 0..(SAMPLE_RATE as usize * 2) {
            let t = n as f32 / SAMPLE_RATE;
            let out = filter.process((2.0 * PI * 20.0 * t).sin());
            if n > SAMPLE_RATE as usize {
                peak = peak.max(out.abs());
            }
        }

        assert!(peak < 0.2, "20 Hz passed through at {peak}");
    }

    #[test]
    fn reset_clears_state() {
        let mut filter = OutputFilter::new(SAMPLE_RATE);
        for _ in 0..100 {
            filter.process(1.0);
        }
        filter.reset();
        // With cleared state the first sample behaves as it did from a fresh filter.
        let fresh = OutputFilter::new(SAMPLE_RATE).process(1.0);
        assert!((filter.process(1.0) - fresh).abs() < 1e-6);
    }
}
