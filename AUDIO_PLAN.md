# Audio Repair Plan 🔊

> **Status: Stages A–F implemented.** 265 tests pass, including end-to-end regression tests for
> each of the five blocking defects, and the debugger runs.

Companion to [PLAN.md](PLAN.md) and [TODO.md](TODO.md), scoped to one thing: getting the APU to
produce correct, listenable audio. Tracks 6 and 7 are marked 100% complete in
[TODO.md](TODO.md), and per-task that is nearly true — the registers, channel state machines and
cpal backend really were built. What was never built is the layer that joins them, so the boxes are
ticked and the emulator still makes no usable sound. The first outcome of this plan is honest
checkboxes.

## 1. Diagnosis

### What already works

- **Register decoding.** `$4000–$4013`, `$4015`, `$4017` are decoded and routed to the right
  channel in [apu/mod.rs](crates/rn_core/src/apu/mod.rs). `$4015` reads reconstruct the status byte
  from live length-counter state.
- **Channel state machines.** Pulse ×2 (duty, envelope, sweep, length counter), triangle (linear
  counter, 32-step sequence), noise (15-bit LFSR, both tap modes), DMC (delta decoder, shift
  register). 63 of 64 unit tests pass.
- **Frame sequencer rate.** Quarter frames every 7457 CPU cycles ≈ 240 Hz, half frames ≈ 120 Hz.
  These numbers are correct.
- **The cpal backend.** Device selection, format negotiation across every `SampleFormat`, stream
  build/play/pause, atomic volume and mute shared with the callback.
- **The trait design.** `SampleProducer`/`SampleConsumer` in
  [rn_core/src/audio/mod.rs](crates/rn_core/src/audio/mod.rs) keep `rn_core` free of host-audio
  dependencies, and `rn_audio` offers a `ringbuf` pair, a crossbeam-channel pair, a multiplexer and
  a test oscillator against them. This is the right shape and should survive the repair unchanged.

### The five defects that break output

**D1 — No resampling.** `Apu::generate_sample` is called from `Apu::tick`, which
`NesSystem::step` runs once per CPU cycle. The APU therefore emits ~1.79 M samples/sec into a buffer
the sound card drains at 44.1/48 kHz. The constants meant to do the decimation — `CPU_CLOCK_RATE`,
`DEFAULT_SAMPLE_RATE` — and the `sample_counter` accumulator field are declared and **never read**;
the compiler reports all three as dead. The ring buffer sits permanently full, `try_push` silently
drops ~97% of samples, and the callback plays a contiguous 1.79 MHz-rate slice at 48 kHz: roughly
37× slow motion. This alone makes the output unrecognisable.

**D2 — The mixer is non-physical and DC-offset.** Pulse, triangle and noise all return unipolar
`0.0..=1.0` while DMC returns bipolar `-1.0..=1.0` — already inconsistent. The mix therefore never
crosses zero: it is a fluctuating DC level, not a waveform. On top of that `generate_sample` applies
an invented AGC (`5.0 / active_channels`, no hardware basis) and then divides by a magic `400.0`,
leaving peak amplitude near 0.1. The NESdev constants `95.88` and `159.79` are present but applied
as if the mix were linear, when they are numerators of a non-linear formula. The single failing test,
`apu::tests::test_all_channels_mixing` ("Pulse 1 scaling incorrect"), is this defect.

**D3 — Pulse and noise run an octave sharp.** On hardware their timers decrement once per *APU*
cycle, i.e. every *second* CPU cycle. `PulseChannel::tick` and `NoiseChannel::tick` are driven once
per CPU cycle from `NesSystem::step`, so every pulse tone is an octave too high. Triangle *is*
CPU-rate on hardware, so it is the only channel with correct pitch today.

**D4 — `println!` in the realtime callback.** `CpalAudioConsumer::process_frame` in
[cpal_audio.rs](crates/rn_audio/src/cpal_audio.rs) prints every sample: a locked stdout write tens
of thousands of times per second on the audio thread. Guaranteed underruns even after D1–D3 are
fixed.

**D5 — Emulation is not clocked against wall time.** `AsmWidget::run_continuous`
([asm_widget.rs](crates/rn_ui/src/widgets/asm_widget.rs)) runs a batch per *repaint*, and counts
`system.step()` calls — instructions — against a budget named `cycles_per_frame` (29 780 *cycles*).
At ~3 cycles per average instruction a "frame" executes roughly 3× too much work, at whatever rate
the compositor repaints. Production rate is untethered from consumption rate, so even perfect
resampling would drift and eventually under- or overrun.

### Secondary problems

| # | Problem | Location |
| --- | --- | --- |
| S1 | Waveform widget receives nothing — the `Multiplexer` is constructed with both `add_producer` calls commented out, the local is dropped at the end of `new()`, and `Multiplexer::tick` is never called anywhere in the workspace | [nes_debugger/main.rs](tools/nes_debugger/src/main.rs) |
| S2 | Duplicated volume/mute atomics — `CpalAudioBuilder::build` creates one pair inside `RingBufferBuilder` and a second shared with the callback; the first is dead | [cpal_audio.rs](crates/rn_audio/src/cpal_audio.rs) |
| S3 | Gratuitous `unsafe impl Send + Sync` on types whose fields are already `Send + Sync` — unnecessary, and it disables the check that would catch a real mistake later | [ring_buffer.rs](crates/rn_audio/src/ring_buffer.rs), [channel_buffer.rs](crates/rn_audio/src/channel_buffer.rs) |
| S4 | DMC has no bus access — `load_next_byte` carries a `TODO` and substitutes the direct-load register, so DPCM playback is a stub | [dmc_channel.rs](crates/rn_core/src/apu/dmc_channel.rs) |
| S5 | Frame counter incomplete — `$4017` stores the mode bit but 5-step mode is unimplemented, and the frame IRQ is entirely absent (no `$4015` flag, no inhibit, no CPU IRQ) | [apu/mod.rs](crates/rn_core/src/apu/mod.rs) |
| S6 | No output filtering — hardware applies two high-pass filters (90 Hz, 440 Hz) and a low-pass (14 kHz) | — |
| S7 | Naming leftovers — `Apu::audio_output2`, `NesSystem::connect_audio_output2`; the `2` suffixes are from an abandoned migration and there is no `1` | [apu/mod.rs](crates/rn_core/src/apu/mod.rs), [nes_system.rs](crates/rn_core/src/system/nes_system.rs) |
| S8 | Dead per-channel `volume` field shadowing `Envelope::get_volume()` | [pulse_channel.rs](crates/rn_core/src/apu/pulse_channel.rs) |

## 2. Repair plan

Six stages, ordered so each one produces an audible or measurable change rather than a batch of
edits you cannot evaluate until the end.

Throughout, [tools/waveform_player](tools/waveform_player) is the isolation harness: it drives the
same `rn_audio` stack from a clean `Oscillator` with no emulation in the loop, so it tells you
immediately whether a glitch comes from the backend or from the APU.

---

### Stage A — Make the backend trustworthy

*Goal: `waveform_player` produces a clean 440 Hz sine with no crackle. Nothing in `rn_core` changes.*

- [x] Delete the per-sample `println!` in `CpalAudioConsumer::process_frame` (**D4**)
- [x] Audit the whole callback for allocation, locking and I/O; it must only read atomics and pop
      from the ring buffer
- [x] Collapse the duplicated volume/mute atomics into one pair owned by the builder and shared by
      producer, consumer and callback (**S2**)
- [x] Remove the `unsafe impl Send`/`Sync` blocks; if anything then fails to compile, that is a real
      bug the assertions were hiding (**S3**)
- [x] Reduce the ring buffer from 250 ms × 4 to ~100 ms total, and expose the configured latency
- [x] Add an underrun counter incremented when `consume()` returns `None`, readable from the UI

**Verify:** `cargo run -p waveform_player`, cycle through all four waveforms, listen for clicks;
underrun counter stays at zero after startup.

---

### Stage B — Fix the mixer

*Goal: a correct, hardware-shaped mix. Still no resampling, so it will not sound right yet — but the
unit tests will be truthful.*

- [x] Change every `generate_sample` to return the channel's **raw DAC level** as an integer, not a
      pre-scaled float: pulse/triangle/noise `0..=15`, DMC `0..=127`. This removes the
      unipolar/bipolar inconsistency at the source (**D2**)
- [x] Replace `Apu::generate_sample`'s body with the NESdev non-linear mix:

      pulse_out = 95.88 / (8128 / (pulse1 + pulse2) + 100)          // 0 when both are 0
      tnd_out   = 159.79 / (1 / (tri/8227 + noise/12241 + dmc/22638) + 100)
      output    = pulse_out + tnd_out                              // 0.0 ..= ~1.0

      Precompute both as lookup tables (`[f32; 31]` and `[f32; 203]`) built once — the standard
      approach, and it keeps the hot path to two array indexes and an add
- [x] Delete the `dynamic_gain` AGC and the magic `/ 400.0`
- [x] Rewrite `test_all_channels_mixing` against values derived from the formula rather than from the
      old scaling, and add a test asserting silence maps to exactly 0.0
- [x] Remove the dead per-channel `volume` field (**S8**)

**Verify:** `cargo test -p rn_core apu` — 64/64 passing, with the mixing test asserting real numbers.

---

### Stage C — Correct the clock domains

*Goal: every channel advances at its true rate.*

- [x] Give `Apu` an explicit CPU→APU divider: an `odd_cycle: bool` toggled every `tick()`
- [x] Clock pulse ×2 and noise only on alternate CPU cycles; keep triangle on every CPU cycle
      (**D3**)
- [x] Leave the frame sequencer where it is — 7457 CPU cycles is already correct — but express the
      constant in terms of APU cycles so the intent is legible
- [x] Add a test that a pulse channel programmed to a known timer value completes one duty period in
      the expected number of CPU cycles

**Verify:** compare a pulse tone against a reference frequency; A440 programmed via the timer should
measure A440, not A880.

---

### Stage D — Resample

*Goal: real sound. This is the stage that makes the emulator audible.*

- [x] Add a `sample_rate: f64` to `Apu`, set from the actual cpal device rather than the hardcoded
      44 100 — plumb it through `connect_audio_output` (**D1**)
- [x] Put `sample_counter` to work: accumulate `sample_rate / CPU_CLOCK_RATE` each tick and emit one
      mixed sample only when it crosses 1.0, subtracting 1.0 on emit. Delete the now-unused dead
      constants warning
- [x] Apply the hardware output filters to the emitted stream — high-pass 90 Hz, high-pass 440 Hz,
      low-pass 14 kHz, as one-pole IIR sections. The first high-pass also removes the DC offset the
      non-linear mix leaves behind (**S6**)
- [x] Consider a simple box-average of the discarded intermediate samples as cheap anti-aliasing
      before decimation; measure whether it is worth the cost

**Verify:** `cargo run -p nes_debugger -- asm/simple_tone_test.asm`, press Run — a clean, correctly
pitched tone. Then each of `pulse_channel_test.asm`, `simple_triangle_test.asm`,
`noise_channel_test.asm` in turn.

---

### Stage E — Pace the emulator

*Goal: production rate matches consumption rate indefinitely, with no drift.*

- [x] Fix the unit confusion in `run_continuous`: accumulate the cycle count `system.step()`
      returns, rather than counting calls, so `cycles_per_frame` means what it says (**D5**)
- [x] Drive execution from the audio clock instead of from repaints — run the CPU until the ring
      buffer holds a target number of samples, so the sound card becomes the master clock. This is
      the standard fix and it removes the dependence on compositor timing entirely
- [x] Keep a wall-clock fallback for when audio is muted or the stream is paused, so stepping and
      frame-advance still behave
- [x] Surface buffer fill level and underrun count in the audio widget — cheap, and it turns the
      next timing bug into something you can see instead of something you have to infer

**Verify:** run a tone program for several minutes; no drift, no underruns, fill level stable.

---

### Stage F — Reconnect the visualiser and finish accuracy work

*Goal: the debugger shows what it plays, and the remaining hardware features land.*

- [x] Wire the `Multiplexer` properly — APU → multiplexer → {cpal producer, waveform producer} — and
      call `Multiplexer::tick` from the debugger's update loop, or restructure so the split happens
      inside the audio path (**S1**)
- [x] Have the waveform widget decimate for display rather than consume every sample, so it cannot
      starve the speaker path
- [x] Implement the frame counter's 5-step mode and the frame IRQ: `$4015` bit 6, the `$4017`
      inhibit flag, and the CPU IRQ line (**S5**)
- [x] Give the DMC bus access so `load_next_byte` reads real memory, including the CPU stall cycles
      it causes (**S4**)
- [x] Rename `audio_output2` → `audio_output` and `connect_audio_output2` → `connect_audio_output`
      (**S7**)
- [x] Add `crates/rn_audio` to `members` in the root `Cargo.toml`
- [x] Update the Track 6/7 checkboxes in [TODO.md](TODO.md) to reflect reality, and add the missing
      tasks this plan uncovered

**Verify:** the waveform widget shows the same signal that is playing; `dmc_channel_test.asm`
produces real DPCM output.

## 3. What changed

Summary of the implementation, for anyone reading the diff:

| Area | Before | After |
| --- | --- | --- |
| Sample rate | one sample per CPU cycle (~1.79 MHz) | decimated to the device's real rate, with box-averaging as anti-aliasing |
| Mixer | linear, unipolar, invented AGC, `/400.0` | NESdev non-linear lookup tables in `apu/mixer.rs` |
| Channel outputs | `f32`, mixed unipolar/bipolar conventions | raw DAC levels (`u8`), 0..=15 and 0..=127 |
| Output filtering | none | 90 Hz + 440 Hz high-pass, 14 kHz low-pass in `apu/filter.rs` |
| Pulse/noise clock | every CPU cycle (an octave sharp) | every second CPU cycle, via an explicit APU divider |
| Master clock | UI repaint rate | the sound card — emulation refills the buffer to 50% |
| `run_continuous` | counted instructions against a cycle budget | counts real cycles; `static mut` replaced by a field |
| Multiplexer | consumer-driven, needed an external `tick`, never wired | a `SampleProducer` that fans out inline; wired to speakers + visualiser |
| Realtime callback | `println!` per sample | atomics and a queue pop, nothing else |
| Volume/mute state | two independent atomic pairs, one dead | one shared `AudioControls`, plus fill level and error counters |
| `SampleProducer`/`Consumer` | `Send + Sync`, forcing `unsafe impl` | `Send` only; no `unsafe` anywhere in `rn_audio` |

Three further defects surfaced while working, each hidden by the ones above:

- **`PulseChannel::output` had a `#[cfg(not(test))]` bypass** around sweep muting, so tests
  exercised a code path that did not exist in release builds. Removing it exposed the next item.
- **`Sweep::calculate_target_period` used `wrapping_sub`.** In negate mode an underflow became
  ~0xFFFF, tripping the "target > $7FF" mute rule and silencing the channel. Hardware does not mute
  on a negative target; now `saturating_sub`.
- **`TriangleChannel::tick_linear_counter` always cleared the reload flag.** With the control flag
  set, hardware keeps reloading and the note sustains; clearing unconditionally silenced the
  triangle after ~60 ms regardless of what the program wrote.

## 4. The macOS startup crash

Not an audio problem, but it blocked verifying audio by ear, so it is recorded here.

`cargo run -p nes_debugger` aborted before its window opened:

```
invalid message send to -[_TtGCs23_ContiguousArrayStorageCSo8NSScreen_$
  countByEnumeratingWithState:objects:count:]:
expected return to have type code 'q', but found 'Q'
```

The class name is the tell: `_TtGCs23_ContiguousArrayStorage...` is a **Swift** array bridged to
Objective-C. A macOS update changed `NSScreen.screens` to return a Swift-backed array, whose
`countByEnumeratingWithState:` is declared `NSUInteger` where objc2 expects `NSInteger`. objc2's
debug-only encoding verification treats that sign mismatch as fatal. Nothing in this repository
changed — the OS did, which is why it ran correctly before.

Signedness does not affect the ABI for this call, so the message send was always correct; only the
assertion was wrong. objc2 0.5.2 already ships the fix as a feature — its own test suite asserts
this exact error string as the thing `relax-sign-encoding` suppresses. Enabling it needs no
version bumps at all, just a workspace dependency declaration so Cargo's feature unification
reaches the transitive copy:

```toml
objc2 = { version = "0.5", features = ["relax-sign-encoding"] }
```

referenced from each binary that opens a window, under
`[target.'cfg(target_os = "macos")'.dependencies]`. The lockfile gains three lines and no version
changes. Removable once eframe/winit move to objc2 0.6+, which fixed the encoding upstream.

## 5. Test strategy

Four layers, all of which now exist:

1. **Unit** — per-channel timing and level tests in `rn_core`, plus mixer tests against values
   computed from the non-linear formula. Cheap and already the strongest part of the suite.
2. **Integration** — `crates/rn_core/tests/audio_pipeline.rs`: an offline `SampleProducer` that
   captures to a `Vec<f32>`, driven by assembled 6502 programs. Asserts on measurable properties
   rather than exact bytes — sample count, dominant frequency by zero-crossing count, DC offset,
   peak level. This is what would have caught D1, D2 and D3 the day they were introduced.

   The programs are inline rather than drawn from [asm/](asm/): the files there call
   `WaitForVBlank` before touching the APU, and the PPU never raises the vblank flag, so they spin
   forever without programming a channel. That is a separate PPU gap, worth its own task — it
   currently makes every `asm/` audio demo silent in the debugger too.
3. **Interactive** — `tools/apu_probe`, a headless CLI that runs a program, measures the capture
   and optionally writes a WAV. Same measurements as the integration tests, but explorable: it
   takes `--asm <file>` as well as built-in presets, so it can be pointed at any program to answer
   "is this silent, and if not, at what pitch and level?" in one command. `apu_probe check` runs
   every preset as a pass/fail sweep.

   It pays for itself immediately on the demo ROMs: `apu_probe run --asm asm/simple_tone_test.asm`
   reports `SILENT — no channel produced any output`, which is the vblank problem above, visible
   without opening a window.

4. **Manual** — the debugger with the waveform widget, for the things a measurement cannot judge.

A regression test worth writing first, before any other change: run
`asm/simple_tone_test.asm` for one emulated second into a capture buffer and assert the sample count
is within 1% of the sample rate. It fails loudly today (it will be off by ~37×) and passing it is
precisely the definition of Stage D being done.
