//! End-to-end tests for the audio path: 6502 source in, audio samples out.
//!
//! These exist because every defect that made the emulator's audio unusable was invisible to the
//! unit tests. Each channel behaved correctly in isolation while the assembled pipeline produced
//! ~37x too many samples, at the wrong pitch, with a DC offset instead of a waveform. The
//! assertions here are therefore about measurable properties of the output signal — rate, pitch,
//! DC offset, amplitude — rather than about exact sample values.
//!
//! The programs are inline rather than taken from `asm/`, because the ones there
//! (`simple_tone_test.asm` and friends) call `WaitForVBlank` before touching the APU and the PPU
//! does not currently raise the vblank flag, so they spin forever without ever programming a
//! channel. Keeping these programs self-contained means an audio test fails only for audio
//! reasons.

use std::sync::mpsc::{channel, Sender};

use rn_core::{apu::CPU_CLOCK_RATE, audio::SampleProducer, cpu::Assembler, system::NesSystem};

const SAMPLE_RATE: f64 = 48_000.0;
const LOAD_ADDRESS: u16 = 0x8000;

/// Pulse period written by the test programs.
///
/// A pulse channel's frequency is `CPU / (16 * (period + 1))`, so 254 gives ~438.7 Hz — the note
/// `asm/simple_tone_test.asm` describes as "middle A".
const PULSE_PERIOD: u16 = 254;

fn expected_pulse_frequency() -> f64 {
    CPU_CLOCK_RATE / (16.0 * (PULSE_PERIOD as f64 + 1.0))
}

/// Enable pulse 1 at full constant volume, 25% duty, period 254, then loop forever.
const PULSE_TONE: &str = r#"
.segment "STARTUP"
RESET:
  LDA #$01
  STA $4015       ; enable pulse 1
  LDA #%01011111  ; 25% duty, constant volume, volume 15
  STA $4000
  LDA #%00000000  ; sweep off
  STA $4001
  LDA #$FE        ; period low byte (254)
  STA $4002
  LDA #%00001000  ; period high bits + length counter load
  STA $4003
Loop:
  JMP Loop
"#;

/// Enable the triangle channel and loop forever.
const TRIANGLE_TONE: &str = r#"
.segment "STARTUP"
RESET:
  LDA #$04
  STA $4015       ; enable triangle
  LDA #%10001111  ; linear counter: control set, reload 15
  STA $4008
  LDA #$FE        ; period low byte
  STA $400A
  LDA #%00001000  ; period high bits + length counter load
  STA $400B
Loop:
  JMP Loop
"#;

/// Captures the emulator's audio output for offline analysis.
struct Capture(Sender<f32>);

impl SampleProducer<f32> for Capture {
    fn set_volume(&mut self, _volume: f32) {}
    fn set_muted(&mut self, _muted: bool) {}
    fn produce(&mut self, sample: f32) {
        let _ = self.0.send(sample);
    }
}

/// Assemble `source`, run it for `seconds` of emulated time, and return everything it played.
fn run(source: &str, seconds: f64) -> Vec<f32> {
    let mut assembler = Assembler::new(LOAD_ADDRESS).with_nes_segments();
    let segments = assembler.assemble_program(source).expect("program should assemble");
    let code = segments.get("STARTUP").expect("program should have a STARTUP segment");

    let mut system = NesSystem::new();
    let (sender, receiver) = channel();
    system.connect_audio_output(Box::new(Capture(sender)), SAMPLE_RATE);
    system.load_program(code, LOAD_ADDRESS).expect("program should load");

    let target_cycles = (CPU_CLOCK_RATE * seconds) as u64;
    let mut cycles = 0u64;
    while cycles < target_cycles {
        match system.step() {
            Ok(step_cycles) => cycles += step_cycles.max(1) as u64,
            Err(error) => panic!("emulation failed after {cycles} cycles: {error}"),
        }
    }

    receiver.try_iter().collect()
}

/// Everything after the first quarter, so the output filters have settled.
fn settled(samples: &[f32]) -> &[f32] {
    &samples[samples.len() / 4..]
}

/// Estimate the dominant frequency by counting zero crossings.
///
/// Adequate here because the output filters remove the DC offset, so a periodic waveform crosses
/// zero exactly twice per cycle. The hysteresis band keeps filter ripple near zero from being
/// counted as crossings, which would otherwise inflate the estimate.
fn dominant_frequency(samples: &[f32], sample_rate: f64) -> f64 {
    if samples.len() < 2 {
        return 0.0;
    }

    let threshold = peak(samples) * 0.25;
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

    let duration = samples.len() as f64 / sample_rate;
    crossings as f64 / 2.0 / duration
}

fn mean(samples: &[f32]) -> f32 {
    samples.iter().sum::<f32>() / samples.len() as f32
}

fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()))
}

/// The regression test for the resampling defect.
///
/// The APU is evaluated once per CPU cycle (~1.79 MHz); without decimation this over-produced by
/// ~37x, the buffer stayed permanently full, and playback became slow-motion noise.
#[test]
fn emits_at_the_device_sample_rate() {
    let samples = run(PULSE_TONE, 1.0);

    let error = (samples.len() as f64 - SAMPLE_RATE).abs() / SAMPLE_RATE;
    assert!(
        error < 0.01,
        "one emulated second should yield ~{SAMPLE_RATE} samples, got {}",
        samples.len()
    );
}

/// The regression test for the clock-domain defect.
#[test]
fn plays_the_programmed_pitch() {
    let samples = run(PULSE_TONE, 1.0);
    let expected = expected_pulse_frequency();
    let measured = dominant_frequency(settled(&samples), SAMPLE_RATE);

    assert!(
        (measured - expected).abs() / expected < 0.05,
        "expected ~{expected:.1} Hz, measured {measured:.1} Hz \
         (clocking pulse at CPU rate instead of APU rate would read ~{:.1} Hz)",
        expected * 2.0
    );
}

/// The regression test for the DC-offset defect.
#[test]
fn output_is_a_waveform_not_a_dc_level() {
    let samples = run(PULSE_TONE, 1.0);
    let settled = settled(&samples);

    let offset = mean(settled);
    assert!(offset.abs() < 0.01, "output has a DC offset of {offset}");

    assert!(
        settled.iter().any(|&s| s > 0.0) && settled.iter().any(|&s| s < 0.0),
        "output never crosses zero, so it is not a waveform"
    );
}

/// The regression test for the mixer-scaling defect, which left peaks near 0.1.
#[test]
fn output_is_audible_and_unclipped() {
    let samples = run(PULSE_TONE, 1.0);
    let level = peak(settled(&samples));

    assert!(level > 0.05, "output is inaudibly quiet: peak {level}");
    assert!(level <= 1.0, "output clips: peak {level}");
}

#[test]
fn silence_when_no_channel_is_enabled() {
    let samples = run(
        r#"
.segment "STARTUP"
RESET:
  LDA #$00
  STA $4015
Loop:
  JMP Loop
"#,
        0.2,
    );

    assert!(!samples.is_empty(), "the APU should still emit samples while silent");
    assert_eq!(peak(&samples), 0.0, "disabled channels must produce true silence");
}

/// The triangle channel is clocked at CPU rate, not APU rate, so it must *not* pick up the
/// divider that pulse and noise use.
#[test]
fn triangle_plays_an_octave_above_a_pulse_at_the_same_period() {
    let samples = run(TRIANGLE_TONE, 1.0);
    let settled = settled(&samples);

    assert!(peak(settled) > 0.01, "triangle channel produced no audible output");

    // Same period register as PULSE_TONE, but the triangle's 32-step sequence and CPU-rate clock
    // put it an octave below the pulse channel's frequency... which is to say
    // CPU / (32 * (period + 1)).
    let expected = CPU_CLOCK_RATE / (32.0 * (PULSE_PERIOD as f64 + 1.0));
    let measured = dominant_frequency(settled, SAMPLE_RATE);

    assert!(
        (measured - expected).abs() / expected < 0.05,
        "expected ~{expected:.1} Hz, measured {measured:.1} Hz"
    );
}
