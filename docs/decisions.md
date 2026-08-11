# Decisions

Design decisions that came out of completed plans, kept after the plans themselves were retired
(their full text lives in git history; the *stories* live in [research-log.md](research-log.md)).

## Audio (from the audio repair)

- **Channels emit raw DAC levels** (`u8`, 0..=15 and 0..=127), mixed through NESdev's non-linear
  lookup tables in `apu/mixer.rs`. No linear mixing, no invented auto-gain.
- **The sample pipeline decimates**: emulate at CPU rate, box-average down to the device's real
  rate, then filter as the console does — 90 Hz + 440 Hz high-pass, 14 kHz low-pass
  (`apu/filter.rs`).
- **Pulse and noise clock every second CPU cycle** through an explicit APU divider, never per
  CPU cycle (that octave-sharp bug is what the divider exists to prevent).
- **The sound card is the master clock**: emulation paces itself by refilling the audio buffer
  to 50%, not by UI repaint rate or instruction counts.
- **No `unsafe` in `rn_audio`**: producers are `Send` only; the realtime callback touches
  atomics and a queue pop, nothing else — no printing, no allocation.
- **Audio is verified end to end** (waveform measurements through the assembled pipeline), not
  only per channel: every one of the five original defects was invisible to unit tests and
  audible in the output.
- **macOS windowing workaround**: `objc2 = { version = "0.5", features = ["relax-sign-encoding"] }`
  under `cfg(target_os = "macos")` for every binary that opens a window — an OS update made
  `NSScreen.screens` Swift-backed and objc2's debug sign-check falsely fatal. Removable once
  eframe/winit reach objc2 0.6+.

## The per-cycle CPU (from the cycle-accuracy design)

- **Interrupt lines are sampled at the end of an instruction's second-to-last cycle** — the
  single hardware rule the whole design serves. `CLI` latency, branch delays, and BRK hijacking
  all fall out of it.
- **No computed poll cycle.** The CPU keeps NMI/IRQ shadow flags updated every cycle by the same
  clock that advances the PPU, and polling acts on the shadows; nothing asks how long an
  instruction is.
- **Every cycle drives the bus.** The 6502 has no idle state: implied instructions read and
  discard, read-modify-write writes twice. Dummy accesses are real accesses.
- **A CPU cycle has phases**: two PPU dots, then the bus access, then the third dot — with the
  interrupt lines read at the cycle's *end*, one dot after the access. That one dot is what made
  `05-nmi_timing` pass; Mesen is built the same way.
- **Timing questions are measured, not argued**: instrument the clock, record which dot raised
  the line, line it up against the test's table.

## Conformance (the standing rules)

- **Test ROMs are never committed** — they are freely distributed but not licensed for
  redistribution. The runner takes a path, skips cleanly when ROMs are absent, and CI stays
  green without them. The suites come from the community's
  [nes-test-roms](https://github.com/christopherpow/nes-test-roms) collection; nestest and its
  golden log from the NESdev wiki.
- **End-to-end ROMs outrank unit tests** for accuracy claims; numbers in docs are claims until
  re-run ([TODO.md](TODO.md) keeps the measured table).
