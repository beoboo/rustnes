# Conformance Plan 🎯

How this emulator gets validated against the NES test ROMs the community uses, and what has to
exist first. Companion to [PLAN.md](PLAN.md) and [TODO.md](TODO.md); the same shape as
[AUDIO_PLAN.md](AUDIO_PLAN.md), which is now largely done.

> **nestest passes in full: all 8991 instructions match the golden log**, official and unofficial
> opcodes alike. blargg's `instr_test-v5` passes both `official_only` and `all_instrs` (16/16
> each), `apu_test` passes, and all four `apu_mixer` ROMs pass.
>
> What remains failing is almost entirely **cycle timing**: interrupts are polled between
> instructions rather than within them, so tests that measure exactly when an interrupt lands
> ("Frame irq is set too late", "Flag first set too late") cannot pass yet. That is the next
> structural piece, and it is a CPU-wide change rather than a bug fix.
>
> **Progress.** P1 (PRG-ROM loading), P2 (the instruction set) and P4 (the runner) are done:
> `.nes` files boot from their reset vector, all 56 official instructions exist, and
> `tools/rom_test` runs both nestest log-diff and blargg's `$6000` protocol. P3 (interrupts) is
> next, and then the ROMs themselves — which need downloading, since they cannot live here.
>
> Building the runner already found one gap: `$6000-$7FFF` cartridge PRG-RAM was unmapped, so
> every blargg ROM would have been unable to report a result at all.

## Why this document

Every audio defect this project had was invisible to its unit tests: each channel behaved
correctly in isolation while the assembled pipeline produced silence, wrong pitch, or a DC level.
Only an end-to-end measurement caught it. The same argument applies to the CPU and PPU, and the
emulator community has already written those end-to-end tests. They are the difference between
"my tests pass" and "it runs Donkey Kong".

## 1. Where we actually are

Measured, not estimated:

| | Was | Now |
| --- | --- | --- |
| Official 6502 instructions | 46 of 56 | **56 of 56** |
| Official opcodes | 108 of 151 | **128 of 151** |
| Interrupts (NMI / IRQ / RTI) | none | `RTI` exists; **no NMI or IRQ delivery** |
| PRG-ROM loading from `.nes` | none — header parsed, PRG *skipped* | **loads and boots from the reset vector** |
| Cartridge PRG-RAM (`$6000-$7FFF`) | unmapped | **mapped** |
| Mappers | none | none (header's mapper field parsed and ignored) |
| Test ROM harness | none | **`tools/rom_test`** |

The remaining 23 opcodes are addressing-mode gaps on instructions that already exist, not missing
instructions.

**What still blocks a full pass:** interrupts (P3), and the mapper layer — cartridge space is
currently backed by RAM, so writes there are accepted where hardware ignores them.

## 2. The blocking prerequisites

In dependency order. None of these is optional — each test ROM below needs all four.

### P1 — Load PRG-ROM from `.nes` files ✅

`cartridge/loader.rs` read the iNES header and then did `file.read_exact(&mut prg_rom)` purely to
skip past it to the CHR data, discarding the program.

- [x] Return PRG-ROM alongside CHR-ROM from the loader (`Rom`, `load_rom`)
- [x] Map PRG-ROM into `$8000-$FFFF`, mirroring a 16 KB image at both `$8000` and `$C000`
- [x] Load the reset vector from `$FFFC` and start execution there
- [ ] Add `--rom file.nes` to the debugger (rom_test takes ROMs directly; the debugger still
      only loads assembly)

### P2 — Complete the official instruction set ✅

- [x] `PHA` `PHP` `PLA` `PLP` — stack push/pull, including the B flag's behaviour in the pushed status byte
- [x] `ROL` `ROR` — accumulator and memory forms
- [x] `CPY`, `CLV`, `TSX`
- [x] `RTI`
- [ ] Fill in missing addressing modes on existing instructions until all 151 official opcodes decode
- [x] Unofficial opcodes: the stable set is implemented — `SLO` `RLA` `SRE` `RRA` `SAX` `LAX`
      `DCP` `ISB`, the unofficial `SBC` at `$EB`, and the multi-byte `NOP`s. 231 of 256 opcodes now
      decode. The remainder are the unstable ones (`ANC`, `ARR`, `XAA`, `AHX`, `TAS`, `SHY`, `SHX`,
      `LAS`, `AXS`) and the `JAM`/`KIL` opcodes that lock a real CPU up; those stay undecodable,
      which is honest — they have no behaviour worth emulating.

### P3 — Interrupts ✅ (functional, not cycle-exact)

- [x] NMI: 7-cycle sequence, vector at `$FFFA`, triggered by PPU vblank when `$2000` bit 7 is set
- [x] IRQ: vector at `$FFFE`, gated on the `InterruptDisable` flag, asserted by the APU frame counter
- [x] `BRK` pushes B set, hardware interrupts push it clear; `RTI` restores state
- [x] `Apu::irq_pending()` wired to the CPU IRQ line
- [ ] Poll interrupts *within* an instruction rather than only between them. Interrupts currently
      take effect at instruction boundaries, which is enough for games and for `apu_test` and
      `vbl_basics`, but not for `cpu_interrupts_v2` or the NMI-timing tests, which measure exactly
      when an interrupt lands relative to a partially executed instruction.

NMI is edge-triggered (latched by the PPU, consumed once) and IRQ is level-triggered (held by the
APU until `$4015` is read) — modelled separately, because conflating them makes a held IRQ fire
once or a single vblank fire repeatedly.

### P4 — A headless test-ROM runner ✅

Blargg's ROMs are designed for exactly this: they write a status byte to `$6000` (`$80` = running,
`$00` = pass, other = fail) and a NUL-terminated message at `$6004`. So a runner needs no screen
and no eyes.

- [x] `tools/rom_test`: load a `.nes`, run to completion or timeout, read `$6000`, print `$6004`
- [x] Non-zero exit on failure so it can gate CI
- [x] `nestest` log-diff mode: run from `$C000` with the documented initial state and diff the CPU
      trace against `nestest.log`, stopping at the first divergence and naming the differing fields
- [x] Skip cleanly when ROMs are absent, so a fresh checkout stays green

`apu_probe` is the model here — headless, measures rather than displays, reports the specific
failure. It found three real bugs in a day.

## 3. The test ROMs, in the order they should pass

Roughly easiest-first, and each one's prerequisites are already met by the time it appears.

### Results so far

Run with `cargo run -p rom_test -- suite <dir>` against a clone of
[nes-test-roms](https://github.com/christopherpow/nes-test-roms). `nestest` lives in its `other/`
directory.

| ROM | Result |
| --- | --- |
| `nestest` | **PASS** — 8991/8991 instructions, official *and* unofficial |
| `instr_test-v5/official_only` | **PASS** — 16/16 |
| `instr_test-v5/all_instrs` | **PASS** — 16/16 |
| `instr_test-v5` sub-ROMs | 14 pass; `15-brk` and `16-special` hang |
| `instr_timing/2-branch_timing` | **PASS** |
| `instr_timing/1-instr_timing` | hangs — needs cycle-accurate timing |
| `apu_mixer` (dmc, noise, square, triangle) | **PASS** — independent check on the audio rebuild |
| `apu_test` | **PASS** — all 8 tests |
| `apu_test` sub-ROMs | 4 pass; `4-jitter`, `5-len_timing`, `6-irq_flag_timing` fail on timing |
| `ppu_vbl_nmi/01-vbl_basics`, `03-vbl_clear_time` | **PASS** — needed working NMI |
| `ppu_vbl_nmi` (the other 7) | fail on cycle-exact vblank/NMI timing |
| `cpu_interrupts_v2` | fails — needs interrupt polling *within* an instruction, not between |
| `cpu_reset`, `cpu_dummy_reads` | hang or report nothing — need reset semantics |
| `branch_timing_tests` | older ROMs that report on screen, not through `$6000` |

Bugs these found, none of which the unit tests could see:

1. **Zero-page wrap in indexed indirect.** `LDA ($FF,X)` read its pointer from `$00FF`/`$0100`
   instead of `$00FF`/`$0000`.
2. **ASL and LSR ignored their addressing mode when writing back**, always storing to the
   accumulator — so `LSR $78` corrupted A and left memory untouched.
3. **23 official opcodes were unregistered** — mostly the indirect forms of the arithmetic group.
4. **The APU panicked on writes to unused registers** (`$4009`, `$400D`), taking the whole emulator
   down. blargg's suite does this, so it could not even start.
5. **`$4017` writes never reached the APU.** It is a direction-split register — writes set the
   frame counter, reads return controller 2 — and the bus mapped one component per address, so the
   controller swallowed both halves. Programs therefore could not inhibit the frame IRQ, and
   `apu_test`'s `2-len_table` failed because the frame counter could not be configured at all.
   Fixed by adding `Addressable::handles_write`, which defaults to `handles_address`.

### Tier 1 — CPU correctness

| ROM | What it proves | Needs |
| --- | --- | --- |
| `nestest.nes` (automated mode) | Every official opcode, and then the unofficial ones, against a cycle-exact golden log | P1, P2, P4 |
| `blargg` `instr_test-v5` | Official instruction behaviour, one sub-ROM per group | P1, P2, P3, P4 |
| `blargg` `instr_timing` | Cycle counts, including page-crossing penalties | as above |
| `blargg` `cpu_interrupts_v2` | NMI/IRQ timing, nested interrupts, `RTI` | P3 |

`nestest` is the right first target: it is self-contained, needs no PPU, and its golden log turns
"something is wrong somewhere" into "line 4711, register X differs".

### Where the PPU actually stands

`rom_test frame` runs a ROM headlessly and reports what the PPU drew — distinct colours, coverage,
and an optional ASCII or PPM rendering. Running it against real ROMs gives a clearer answer than
the test suites do:

| ROM | Result |
| --- | --- |
| `nestest` | renders its results screen — text is legible in the ASCII view |
| `blargg_litewall` | 4 colours, 52% coverage — draws real content |
| `full_palette`, `240pee`, `scanline` | **blank**, despite enabling rendering |

The pattern is consistent with the architecture: **the PPU renders a whole frame at once**, at the
frame boundary, rather than scanline by scanline. A static screen therefore comes out right, and
anything that changes state mid-frame does not. `full_palette` displays all 64 colours by
rewriting the palette every scanline; sampled once per frame it can only ever be flat — which is
exactly what it produces.

That makes scanline-based rendering, not any individual PPU bug, the next structural piece of work,
and it is the same change the timing test ROMs below need.

### Tier 2 — PPU

| ROM | What it proves |
| --- | --- |
| `blargg` `ppu_vbl_nmi` | Vblank flag timing and NMI generation — currently entirely untested |
| `blargg` `sprite_hit_tests` | Sprite-zero hit, which games use for split-screen |
| `blargg` `sprite_overflow_tests` | The 8-sprite limit and its hardware quirks |
| `blargg` `oam_read` / `oam_stress` | OAM behaviour under DMA |

### Tier 3 — APU

| ROM | What it proves |
| --- | --- |
| `blargg` `apu_test` | Length counters, frame counter, IRQ, `$4015` semantics |
| `blargg` `apu_mixer` | The non-linear mixer, against reference values |
| `blargg` `dmc_dma` | DMC memory reads and the CPU stalls they cause |

The APU work in [AUDIO_PLAN.md](AUDIO_PLAN.md) was verified by measurement rather than by these
ROMs. They are the independent check on it, and `apu_test` will exercise the frame IRQ that
nothing currently consumes.

### Tier 4 — Whole system

`nestest` in full, then a simple commercial game (Donkey Kong is the usual first, being NROM),
then mapper-dependent titles as mappers land.

## 4. Legal note

None of these ROMs can be committed to this repository — `nestest.nes` and blargg's suites are
freely *distributed* but not licensed for redistribution here, and commercial games obviously not.
The runner should therefore:

- take a path or a directory, and skip cleanly with a clear message when ROMs are absent
- keep CI green without them, running the suite only when a `roms/` directory is present
- document in the README where to obtain them

## 5. Suggested order of work

1. ~~**P1** — PRG loading.~~ Done.
2. ~~**P2** — the ten missing instructions.~~ Done.
3. ~~**P4** — the runner.~~ Done.
4. **Obtain the ROMs and run them.** They cannot be committed here, so drop `nestest.nes` and
   `nestest.log` into a `roms/` directory and run
   `cargo run -p rom_test -- nestest roms/nestest.nes roms/nestest.log`.
5. Fix whatever `nestest` finds. Expect this to take longer than writing the runner did.
6. **P3** — interrupts, then `cpu_interrupts_v2`.
7. Tier 2 and 3 ROMs, which will also validate the PPU and the APU rebuild independently.

Steps 1–3 are worth doing together: they are the smallest change that turns "we think it works"
into "here is the line where it stops working".
