# RustNES

A Nintendo Entertainment System emulator written in Rust, built test-first, with every subsystem
inspectable live through a debugger UI while it runs.

- **[TODO.md](docs/TODO.md)** — the live task list, and only that
- **[docs/research-log.md](docs/research-log.md)** — how every accuracy bug was found and fixed
- **[IDEAS.md](docs/IDEAS.md)** — someday-maybe, explicitly not planned
- **[AUDIO_PLAN.md](docs/AUDIO_PLAN.md)** — diagnosis and repair of the audio pipeline (done)
- **[CONFORMANCE_PLAN.md](docs/CONFORMANCE_PLAN.md)** — validating against the NES test ROMs (done)

## Status

| Subsystem | State |
| --- | --- |
| 6502 CPU | Working — **nestest passes 8991/8991**; 231 of 256 opcodes; NMI/IRQ at instruction granularity |
| Memory / bus | Working — address decoding, component attachment, region map |
| DMA | Working — OAM DMA controller with cycle stealing |
| Input | Working — controllers, remappable key profiles |
| PPU | Working — scanline rendering, scrolling, mirroring, sprites, sprite-zero hit, mask features; not cycle-accurate *within* a scanline |
| Cartridge | Mappers 0, 1, 2, 4 and 7 (NROM, MMC1, UxROM, MMC3, AxROM), including MMC3's scanline IRQ |
| APU | Working — all five channels, hardware non-linear mixing, resampling, output filters |
| Debugger UI | Working — dockable egui workspace with per-subsystem widgets |

**Commercial games run.** Donkey Kong (NROM) and Super Mario Bros 3 (MMC3) both boot and are
playable, and every conformance suite with a verdict to give passes — nestest's 8991 instructions
against the golden log, blargg's instruction, timing, interrupt and reset suites, the PPU
vblank/NMI/sprite suites and the APU suites, on NTSC and PAL. The measured table, kept honest by
re-running rather than remembering, lives in [docs/TODO.md](docs/TODO.md), along with the short
list of open dot-level residuals and the current hypothesis for each. How validation works is
[docs/CONFORMANCE_PLAN.md](docs/CONFORMANCE_PLAN.md).

## Repository layout

```
crates/
  rn_core/     Emulator core: cpu, ppu, apu, memory, cartridge, dma, input, system bus
  rn_audio/    Host audio backend: cpal output, ring buffer, channel, multiplexer, test oscillator
  rn_input/    Controller profiles and key mapping
  rn_ui/       egui widgets: cpu, ppu, memory, pattern table, disasm, audio, waveform, ...
tools/
  apu_probe/        Headless audio harness — run a program, measure/save what the APU produced
  rom_test/         Headless NES test-ROM runner (nestest log-diff, blargg $6000 protocol,
                    frame capture to PPM/ASCII)
  nes_asm/          Command-line 6502 assembler
  nes_debugger/     The main application — full dockable debugger workspace
  waveform_player/  Standalone oscillator + waveform-visualizer playground (no emulation)
asm/           6502 test programs, including one per APU channel
docs/          Requirements, references, development guide
```

`rn_core` has no dependency on any host graphics or audio library. It exposes narrow traits
(`Addressable` for the bus, `SampleProducer`/`SampleConsumer` for audio) that the outer crates
implement, which is what keeps the core testable in isolation and portable to WebAssembly later.

## Building and running

```bash
cargo build --workspace
cargo test  --workspace

cargo run -p nes_debugger                                # main debugger
cargo run -p nes_debugger -- asm/simple_tone_test.asm    # load assembly on startup
cargo run -p nes_debugger -- game.nes                    # or an iNES ROM
cargo run -p nes_asm -- asm/basic_tone_test.asm          # assemble from the command line
cargo run -p waveform_player                             # audio playground, no emulator involved

cargo run -p apu_probe -- list                           # built-in audio test programs
cargo run -p apu_probe -- check                          # measure them all, pass/fail
cargo run -p apu_probe -- run pulse --out /tmp/a.wav     # capture one to a WAV

cargo run -p rom_test -- nestest roms/nestest.nes roms/nestest.log
cargo run -p rom_test -- suite roms/                     # every .nes under a directory
cargo run -p rom_test -- frame game.nes --ascii          # what the PPU drew, in the terminal
```

Test ROMs are not distributed here (see [CONFORMANCE_PLAN.md](docs/CONFORMANCE_PLAN.md)); `rom_test`
skips cleanly with a message when they are absent, so a fresh checkout stays green.

In the debugger: **Assemble** builds the source in the Assembly tab into system memory, **Run**
starts continuous execution (and starts the audio stream), **Step** advances one instruction,
**Next Frame** advances one video frame.

Controller 1 accepts both common layouts at once, and the Controller tab lists the live mapping:

| Button | Keys |
| --- | --- |
| D-pad | Arrow keys, or `W` `A` `S` `D` |
| A | `Z` or `K` |
| B | `X` or `L` |
| Start | `Enter` or `Space` |
| Select | `Tab` or `Right Shift` | The dock can be rearranged freely; tabs cover CPU, PPU,
memory, pattern tables, DMA, controller, disassembly, audio controls and the output waveform.

## Test programs

[asm/](asm/) holds 6502 sources used as end-to-end tests, written for this project's assembler
syntax (segments `STARTUP` and `CHARS`). They cover pixel output, pattern tables, sprites,
animation, controller input, and one program per APU channel — `pulse_channel_test.asm`,
`simple_triangle_test.asm`, `noise_channel_test.asm`, `dmc_channel_test.asm`.

## Audio

The audio path was rebuilt; **[AUDIO_PLAN.md](docs/AUDIO_PLAN.md)** has the full diagnosis of what was
wrong, what changed, and what is left.

```
Apu::tick (per CPU cycle)
  |-- APU divider: pulse + noise at CPU/2, triangle at CPU rate
  |-- non-linear mixer (NESdev lookup tables) -> 0.0..=1.0
  |-- decimate to the device's real sample rate, averaging the discarded cycles
  |-- 90 Hz + 440 Hz high-pass, 14 kHz low-pass
  `-- Multiplexer --> ring buffer --> cpal callback --> speakers
                  `-> bounded channel --> waveform widget
```

The sound card is the master clock: emulation runs however many cycles are needed to refill the
ring buffer to ~50%, so production cannot drift from consumption. Buffer fill level, underruns and
dropped samples are shown in the debugger's Audio tab.

`rn_core` still knows nothing about the host — `rn_audio` implements two traits and everything
else is behind them.

### Measuring it

`apu_probe` runs a program with no window and no sound card, captures exactly what the APU
produced, and measures it. Every audio defect this project has had was a property of the signal —
wrong sample rate, wrong pitch, DC offset instead of a waveform, amplitude an order of magnitude
too low — and each shows up as a number here, where in the GUI they all just sound "broken".

```
$ cargo run -p apu_probe -- run pulse

Capture
  samples          95999
  rate error       0.00%  (expected 96000 samples)

Level
  peak             0.1239
  dc offset        -0.00000
  clipped samples  0

Pitch
  zero crossings   438.7 Hz
  spectral peak    438.0 Hz
  expected         438.7 Hz
  ratio            1.000x   OK
```

Pitch is measured two independent ways — zero crossings and a DFT scan — so agreement is evidence
the reading is real. A wrong ratio is named rather than just flagged: `2.000x` prints "ONE OCTAVE
SHARP (clocked at CPU rate, not APU rate?)". `--out file.wav` saves the capture for listening or
for an external analyser.

The same measurements run as tests in `crates/rn_core/tests/audio_pipeline.rs`.

## Known gaps outside audio

- `src/main.rs` at the repository root is orphaned — the root `Cargo.toml` is a pure workspace
  manifest with no `[package]`, so that file is never compiled. It predates the split into crates.
- `crates/rn_audio` is missing from the `members` list in the root `Cargo.toml`. It is pulled in
  anyway as a path dependency of the tools, but should be listed explicitly.
- Mapper support beyond the raw iNES header parse is not implemented.
- The CPU has no interrupt delivery: NMI, IRQ and `RTI` are all absent, so the PPU's vblank NMI
  and the APU's frame IRQ (both maintained correctly) have nothing to assert.
- The PPU never raises its vblank flag, so every demo in [asm/](asm/) spins forever in
  `WaitForVBlank` without reaching its initialisation code.

## License

MIT OR Apache-2.0
