# RustNES

A Nintendo Entertainment System emulator in Rust that you can **watch working**: every subsystem
inspectable live in a dockable debugger while a game runs, and every subsystem measurable
headless when you'd rather have numbers than impressions.

**It plays.** Donkey Kong and Super Mario Bros 3 boot and play, on NTSC and PAL — and every
community conformance suite with a verdict to give passes: nestest's 8991 instructions against
the golden log, blargg's instruction, timing, interrupt and reset suites, the PPU
vblank/NMI/sprite suites, the APU suites. The measured table lives in
[docs/TODO.md](docs/TODO.md), kept honest by re-running rather than remembering, alongside the
short list of open dot-level residuals — each with a current hypothesis, because "it's probably
timing" is not a bug report.

## What's inside

| Subsystem | State |
| --- | --- |
| 6502 CPU | All 256 opcodes, official and unofficial — nestest passes 8991/8991. Interrupt lines sampled per cycle at the cycle's true phase: CLI latency, branch delays and BRK hijacking all behave |
| PPU | Scanline renderer: scrolling, mirroring, sprites, sprite-zero hit, sprite overflow, mask features, per-scanline sprite evaluation |
| APU | All five channels through the hardware's own non-linear mixer, decimated and filtered like the console, paced by the sound card |
| Cartridge | NROM, MMC1, UxROM, MMC3 (scanline IRQ included), AxROM |
| DMA | OAM DMA with cycle stealing |
| Input | Two controllers, remappable profiles, two keyboard layouts live at once |
| Debugger | Dockable egui workspace: CPU, PPU, memory, pattern tables, disassembly, DMA, controllers, audio, waveform |

## Repository layout

```
crates/
  rn_core/     The emulator: cpu, ppu, apu, memory, cartridge, dma, input, system bus
  rn_audio/    Host audio: cpal output, ring buffer, multiplexer, test oscillator
  rn_input/    Controller profiles and key mapping
  rn_ui/       egui widgets for every subsystem
tools/
  nes_debugger/     The main application — the full dockable workspace
  rom_test/         Headless test-ROM runner: nestest log-diff, blargg's $6000 protocol,
                    screen reading, frame capture, committed frame baselines
  apu_probe/        Headless audio harness — run a program, measure what the APU produced
  nes_asm/          Command-line 6502 assembler
  waveform_player/  Oscillator + waveform playground, no emulation involved
  nesref/           Reference-emulator comparison shims (excluded from the workspace;
                    needs an external checkout — see its README)
asm/           6502 test programs, including one per APU channel
docs/          The task list, the research log, the decisions
```

`rn_core` knows nothing about the host: no graphics, no audio, no windowing. It exposes narrow
traits (`Addressable` for the bus, `SampleProducer`/`SampleConsumer` for audio) that the outer
crates implement — which is what keeps the core testable in isolation.

## Building and running

```bash
cargo build --workspace
cargo test  --workspace

cargo run -p nes_debugger                                # the debugger
cargo run -p nes_debugger -- game.nes                    # ...straight into an iNES ROM
cargo run -p nes_debugger -- asm/simple_tone_test.asm    # ...or a 6502 source file
cargo run -p nes_asm -- asm/basic_tone_test.asm          # assemble from the command line
cargo run -p waveform_player                             # audio playground

cargo run -p rom_test -- nestest roms/nestest.nes roms/nestest.log
cargo run -p rom_test -- suite roms/                     # every .nes under a directory
cargo run -p rom_test -- frame game.nes --ascii          # what the PPU drew, in the terminal
cargo run -p rom_test -- frame game.nes --press start@130 --out shot.ppm

cargo run -p apu_probe -- check                          # measure every audio test program
cargo run -p apu_probe -- run pulse --out /tmp/a.wav     # capture one to a WAV
```

Test ROMs are not distributed here — they're freely available (the community's
[nes-test-roms](https://github.com/christopherpow/nes-test-roms) collection; nestest and its
golden log from the NESdev wiki) but not licensed for redistribution. `rom_test` skips cleanly
when they're absent, so a fresh checkout stays green.

## The debugger

**Assemble** builds the Assembly tab's source into memory, **Run** starts continuous execution
(and the audio stream), **Step** advances one instruction, **Next Frame** one video frame. The
dock rearranges freely; tabs cover CPU, PPU, memory, pattern tables, DMA, controllers,
disassembly, audio controls and the output waveform.

Controller 1 accepts both common layouts at once; the Controller tab shows the live mapping:

| Button | Keys |
| --- | --- |
| D-pad | Arrow keys, or `W` `A` `S` `D` |
| A | `Z` or `K` |
| B | `X` or `L` |
| Start | `Enter` or `Space` |
| Select | `Tab` or `Right Shift` |

## Audio you can measure

The signal path, per CPU cycle:

```
Apu::tick
  |-- APU divider: pulse + noise at CPU/2, triangle at CPU rate
  |-- non-linear mixer (NESdev lookup tables) -> 0.0..=1.0
  |-- decimate to the device's real sample rate, averaging the discarded cycles
  |-- 90 Hz + 440 Hz high-pass, 14 kHz low-pass
  `-- Multiplexer --> ring buffer --> cpal callback --> speakers
                  `-> bounded channel --> waveform widget
```

The sound card is the master clock: emulation runs exactly as many cycles as it takes to keep
the ring buffer half full, so production cannot drift from consumption. Fill level, underruns
and dropped samples are live in the debugger's Audio tab.

Audio bugs are properties of the *signal* — wrong pitch, DC offset, an amplitude an order of
magnitude off — and in a GUI they all just sound "broken". So `apu_probe` runs a program with no
window and no sound card, and turns the output into numbers:

```
$ cargo run -p apu_probe -- run pulse

Pitch
  zero crossings   438.7 Hz
  spectral peak    438.0 Hz
  expected         438.7 Hz
  ratio            1.000x   OK
```

Pitch is measured two independent ways — zero crossings and a DFT scan — so agreement is
evidence the reading is real. A wrong ratio is *named*, not just flagged: `2.000x` prints
"ONE OCTAVE SHARP (clocked at CPU rate, not APU rate?)". The same measurements run as tests in
`crates/rn_core/tests/audio_pipeline.rs`.

## Test programs

[asm/](asm/) holds 6502 sources used as end-to-end tests, written for this project's assembler.
They cover pixel output, pattern tables, sprites, animation, controller input, and one program
per APU channel.

## More to read

- **[docs/TODO.md](docs/TODO.md)** — the live list: measured results, open items, hypotheses
- **[docs/research-log.md](docs/research-log.md)** — how each accuracy bug was found; the
  diagnoses are the part the code can't show
- **[docs/decisions.md](docs/decisions.md)** — the design decisions, in one place
- **[docs/ideas.md](docs/ideas.md)** — someday-maybe, explicitly not planned

## License

MIT OR Apache-2.0
