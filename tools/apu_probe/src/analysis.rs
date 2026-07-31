//! Measurements over a captured sample buffer.
//!
//! Deliberately dependency-free and simple enough to read in one sitting: the point is to be able
//! to trust the numbers when the emulator's output is in question.

use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct Analysis {
    pub sample_count: usize,
    pub sample_rate: f64,
    /// Wall-clock length of the capture, in seconds.
    pub duration: f64,
    /// How many samples were expected for the emulated duration, and the relative error.
    pub expected_samples: usize,
    pub rate_error: f64,
    pub peak: f32,
    pub rms: f32,
    /// Mean level. Anything far from zero means the DC blocker is not doing its job.
    pub dc_offset: f32,
    /// Samples at or beyond full scale.
    pub clipped: usize,
    /// Dominant frequency by counting zero crossings — cheap, and exact for a square wave.
    pub zero_crossing_hz: f64,
    /// Dominant frequency by scanning a DFT — slower, but not fooled by harmonics or noise.
    pub spectral_hz: f64,
    /// True if every sample is exactly zero.
    pub silent: bool,
}

/// Portion of the capture to skip so the output filters have settled.
const SETTLE_FRACTION: usize = 4;

pub fn analyse(samples: &[f32], sample_rate: f64, emulated_seconds: f64) -> Analysis {
    let expected_samples = (sample_rate * emulated_seconds) as usize;
    let rate_error = if expected_samples > 0 {
        (samples.len() as f64 - expected_samples as f64).abs() / expected_samples as f64
    } else {
        0.0
    };

    // Measure the settled portion: the high-pass filters need time to remove the mixer's DC
    // offset, and including their startup transient would skew every statistic here.
    let settled = if samples.len() > SETTLE_FRACTION {
        &samples[samples.len() / SETTLE_FRACTION..]
    } else {
        samples
    };

    let peak = settled.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
    let rms = (settled.iter().map(|&s| (s as f64) * (s as f64)).sum::<f64>() / settled.len().max(1) as f64)
        .sqrt() as f32;
    let dc_offset = settled.iter().sum::<f32>() / settled.len().max(1) as f32;
    let clipped = settled.iter().filter(|&&s| s.abs() >= 1.0).count();

    Analysis {
        sample_count: samples.len(),
        sample_rate,
        duration: samples.len() as f64 / sample_rate,
        expected_samples,
        rate_error,
        peak,
        rms,
        dc_offset,
        clipped,
        zero_crossing_hz: zero_crossing_frequency(settled, sample_rate),
        spectral_hz: spectral_peak(settled, sample_rate),
        silent: peak == 0.0,
    }
}

/// Dominant frequency from zero crossings.
///
/// A hysteresis band at 25% of peak keeps filter ripple and noise around zero from being counted,
/// which would otherwise inflate the estimate wildly.
/// Dominant frequency and peak level for each of `count` equal windows.
///
/// A single figure for the whole capture is misleading for anything that changes over time — a
/// melody, an envelope, a sweep — where the interesting question is how pitch and level *move*.
pub fn segments(samples: &[f32], sample_rate: f64, count: usize) -> Vec<(f64, f64, f32)> {
    if count == 0 || samples.len() < count {
        return Vec::new();
    }

    let width = samples.len() / count;
    (0..count)
        .map(|i| {
            let window = &samples[i * width..(i + 1) * width];
            let start = (i * width) as f64 / sample_rate;
            let peak = window.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
            (start, zero_crossing_frequency(window, sample_rate), peak)
        })
        .collect()
}

fn zero_crossing_frequency(samples: &[f32], sample_rate: f64) -> f64 {
    if samples.len() < 2 {
        return 0.0;
    }

    let threshold = samples.iter().fold(0.0f32, |a, &b| a.max(b.abs())) * 0.25;
    if threshold <= 0.0 {
        return 0.0;
    }

    let mut crossings = 0usize;
    let mut above = samples[0] > 0.0;
    for &sample in samples {
        if above && sample < -threshold {
            above = false;
            crossings += 1;
        } else if !above && sample > threshold {
            above = true;
            crossings += 1;
        }
    }

    crossings as f64 / 2.0 / (samples.len() as f64 / sample_rate)
}

/// Dominant frequency by scanning a naive DFT across the audible range.
///
/// A full FFT would be overkill: this only needs the single strongest bin, over a few hundred
/// candidate frequencies, on one windowed block. Independent of the zero-crossing estimate, so
/// agreement between the two is good evidence the reading is real.
fn spectral_peak(samples: &[f32], sample_rate: f64) -> f64 {
    const BLOCK: usize = 8192;
    const MIN_HZ: f64 = 20.0;
    const MAX_HZ: f64 = 8_000.0;
    const BINS: usize = 1024;

    if samples.len() < BLOCK {
        return 0.0;
    }

    // A Hann window keeps the block's edges from smearing energy across the spectrum.
    let block: Vec<f64> = samples[..BLOCK]
        .iter()
        .enumerate()
        .map(|(n, &s)| {
            let w = 0.5 - 0.5 * (2.0 * PI * n as f64 / BLOCK as f64).cos();
            s as f64 * w
        })
        .collect();

    let mut best_hz = 0.0;
    let mut best_magnitude = 0.0;

    for bin in 0..BINS {
        // Logarithmic spacing: pitch is perceived logarithmically, and it gives fine resolution
        // down where the NES actually plays without wasting bins up at 8 kHz.
        let hz = MIN_HZ * (MAX_HZ / MIN_HZ).powf(bin as f64 / (BINS - 1) as f64);
        let step = 2.0 * PI * hz / sample_rate;

        let (mut real, mut imag) = (0.0, 0.0);
        for (n, &sample) in block.iter().enumerate() {
            let phase = step * n as f64;
            real += sample * phase.cos();
            imag -= sample * phase.sin();
        }

        let magnitude = (real * real + imag * imag).sqrt();
        if magnitude > best_magnitude {
            best_magnitude = magnitude;
            best_hz = hz;
        }
    }

    best_hz
}

impl Analysis {
    pub fn report(&self) {
        println!("Capture");
        println!("  samples          {}", self.sample_count);
        println!("  sample rate      {:.0} Hz", self.sample_rate);
        println!("  duration         {:.3} s", self.duration);
        println!(
            "  rate error       {:.2}%  (expected {} samples){}",
            self.rate_error * 100.0,
            self.expected_samples,
            if self.rate_error > 0.01 { "   <-- WRONG" } else { "" }
        );

        println!("\nLevel");
        if self.silent {
            println!("  SILENT — no channel produced any output");
        } else {
            println!("  peak             {:.4}", self.peak);
            println!("  rms              {:.4}", self.rms);
            println!(
                "  dc offset        {:+.5}{}",
                self.dc_offset,
                if self.dc_offset.abs() > 0.01 {
                    "   <-- not a waveform, the DC blocker is not working"
                } else {
                    ""
                }
            );
            println!(
                "  clipped samples  {}{}",
                self.clipped,
                if self.clipped > 0 { "   <-- clipping" } else { "" }
            );
        }

        if !self.silent {
            println!("\nPitch");
            println!("  zero crossings   {:.1} Hz", self.zero_crossing_hz);
            println!("  spectral peak    {:.1} Hz", self.spectral_hz);

            // The two methods measure different things badly in different ways; when they disagree
            // sharply, neither reading should be trusted.
            let (lo, hi) = if self.zero_crossing_hz < self.spectral_hz {
                (self.zero_crossing_hz, self.spectral_hz)
            } else {
                (self.spectral_hz, self.zero_crossing_hz)
            };
            if lo > 0.0 && hi / lo > 1.2 {
                println!("  (estimates disagree — the signal may be noisy or inharmonic)");
            }
        }
    }

    /// Compare against an expected pitch, reporting the ratio — an octave error shows up as 2.00.
    pub fn report_expected_pitch(&self, expected_hz: f64) {
        if self.silent {
            return;
        }

        let measured = self.zero_crossing_hz;
        let ratio = if expected_hz > 0.0 { measured / expected_hz } else { 0.0 };

        println!("  expected         {expected_hz:.1} Hz");
        println!(
            "  ratio            {ratio:.3}x{}",
            match ratio {
                r if (r - 1.0).abs() < 0.05 => "   OK",
                r if (r - 2.0).abs() < 0.1 => "   <-- ONE OCTAVE SHARP (clocked at CPU rate, not APU rate?)",
                r if (r - 0.5).abs() < 0.05 => "   <-- ONE OCTAVE FLAT",
                _ => "   <-- WRONG",
            }
        );
    }
}
