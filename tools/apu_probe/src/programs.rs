//! Built-in test programs.
//!
//! Self-contained on purpose: the demos in `asm/` call `WaitForVBlank` before touching the APU,
//! and the PPU does not currently raise the vblank flag, so they spin forever without ever
//! programming a channel. A probe whose job is to isolate audio faults must not fail for reasons
//! outside the audio path.

/// Period written by the pulse, triangle and DMC presets.
///
/// A pulse channel plays `CPU / (16 * (period + 1))`, so 254 gives ~438.7 Hz.
pub const PERIOD: u16 = 254;

pub struct Preset {
    pub name: &'static str,
    pub description: &'static str,
    pub source: &'static str,
    /// The pitch this program should produce, if it is a tone.
    pub expected_hz: Option<f64>,
}

pub fn all() -> Vec<Preset> {
    vec![
        Preset {
            name: "pulse",
            description: "Pulse 1, 25% duty, constant volume 15, period 254",
            source: PULSE,
            expected_hz: Some(rn_core::apu::CPU_CLOCK_RATE / (16.0 * (PERIOD as f64 + 1.0))),
        },
        Preset {
            name: "pulse-both",
            description: "Both pulse channels, slightly detuned, to exercise the mixer",
            source: PULSE_BOTH,
            expected_hz: None,
        },
        Preset {
            name: "triangle",
            description: "Triangle channel, period 254 (an octave below the pulse preset)",
            source: TRIANGLE,
            expected_hz: Some(rn_core::apu::CPU_CLOCK_RATE / (32.0 * (PERIOD as f64 + 1.0))),
        },
        Preset {
            name: "noise",
            description: "Noise channel, period index 4, constant volume 15",
            source: NOISE,
            expected_hz: None,
        },
        Preset {
            name: "sweep",
            description: "Pulse 1 with the sweep unit enabled, to hear the pitch slide",
            source: SWEEP,
            expected_hz: None,
        },
        Preset {
            name: "silence",
            description: "All channels disabled — output must be exactly zero",
            source: SILENCE,
            expected_hz: None,
        },
    ]
}

pub fn find(name: &str) -> Option<Preset> {
    all().into_iter().find(|p| p.name == name)
}

const PULSE: &str = r#"
.segment "STARTUP"
RESET:
  LDA #$01
  STA $4015       ; enable pulse 1
  LDA #%01011111  ; 25% duty, constant volume, volume 15
  STA $4000
  LDA #%00000000  ; sweep off
  STA $4001
  LDA #$FE        ; period low (254)
  STA $4002
  LDA #%00001000  ; period high + length counter load
  STA $4003
Loop:
  JMP Loop
"#;

const PULSE_BOTH: &str = r#"
.segment "STARTUP"
RESET:
  LDA #$03
  STA $4015       ; enable both pulse channels
  LDA #%01011111
  STA $4000
  LDA #%00000000
  STA $4001
  LDA #$FE
  STA $4002
  LDA #%00001000
  STA $4003
  LDA #%10011111  ; 50% duty on pulse 2
  STA $4004
  LDA #%00000000
  STA $4005
  LDA #$A9        ; a different period, so the two beat against each other
  STA $4006
  LDA #%00001000
  STA $4007
Loop:
  JMP Loop
"#;

const TRIANGLE: &str = r#"
.segment "STARTUP"
RESET:
  LDA #$04
  STA $4015       ; enable triangle
  LDA #%10001111  ; linear counter: control set (sustain), reload 15
  STA $4008
  LDA #$FE
  STA $400A
  LDA #%00001000
  STA $400B
Loop:
  JMP Loop
"#;

const NOISE: &str = r#"
.segment "STARTUP"
RESET:
  LDA #$08
  STA $4015       ; enable noise
  LDA #%00111111  ; halt length counter, constant volume 15
  STA $400C
  LDA #$04        ; period index 4
  STA $400E
  LDA #%00001000  ; length counter load
  STA $400F
Loop:
  JMP Loop
"#;

const SWEEP: &str = r#"
.segment "STARTUP"
RESET:
  LDA #$01
  STA $4015
  LDA #%00111111  ; halt length counter, constant volume 15
  STA $4000
  LDA #%10010011  ; sweep enabled, period 1, shift 3, increasing
  STA $4001
  LDA #$FE
  STA $4002
  LDA #%00001000
  STA $4003
Loop:
  JMP Loop
"#;

const SILENCE: &str = r#"
.segment "STARTUP"
RESET:
  LDA #$00
  STA $4015
Loop:
  JMP Loop
"#;
