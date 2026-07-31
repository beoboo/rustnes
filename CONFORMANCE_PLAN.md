# Conformance Plan 🎯

How this emulator gets validated against the NES test ROMs the community uses, and what has to
exist first. Companion to [PLAN.md](PLAN.md) and [TODO.md](TODO.md); the same shape as
[AUDIO_PLAN.md](AUDIO_PLAN.md), which is now largely done.

## Why this document

Every audio defect this project had was invisible to its unit tests: each channel behaved
correctly in isolation while the assembled pipeline produced silence, wrong pitch, or a DC level.
Only an end-to-end measurement caught it. The same argument applies to the CPU and PPU, and the
emulator community has already written those end-to-end tests. They are the difference between
"my tests pass" and "it runs Donkey Kong".

## 1. Where we actually are

Measured, not estimated:

| | State |
| --- | --- |
| Official 6502 instructions | **46 of 56** |
| Official opcodes | **108 of 151** |
| Interrupts (NMI / IRQ / RTI) | **none** |
| PRG-ROM loading from `.nes` | **none** — the loader parses the header, then *skips* the PRG data |
| Mappers | none (header's mapper field is parsed and ignored) |
| Test ROM harness | none |

**Missing instructions:** `CLV` `CPY` `PHA` `PHP` `PLA` `PLP` `ROL` `ROR` `RTI` `TSX`

That set is not arbitrary — it is every stack operation, both rotates, and the interrupt return.
Nothing that pushes or pulls the stack can run, which rules out most real programs. Addressing-mode
coverage is also incomplete on instructions that do exist (`LDA` was missing five modes until
recently), which is most of the gap between 108 and 151 opcodes.

**No test ROM can run today**, for the blunt reason that there is no way to get a ROM's program
code into memory.

## 2. The blocking prerequisites

In dependency order. None of these is optional — each test ROM below needs all four.

### P1 — Load PRG-ROM from `.nes` files

`cartridge/loader.rs` reads the iNES header and then does
`file.read_exact(&mut prg_rom)` purely to skip past it to the CHR data. The program is discarded.

- [ ] Return PRG-ROM alongside CHR-ROM from the loader
- [ ] Map PRG-ROM into `$8000–$FFFF`, mirroring a 16 KB image at both `$8000` and `$C000`
- [ ] Load the reset vector from `$FFFC` and start execution there, rather than at a fixed address
- [ ] Add `--rom file.nes` to the debugger and to the probe tools

### P2 — Complete the official instruction set

- [ ] `PHA` `PHP` `PLA` `PLP` — stack push/pull, including the B flag's behaviour in the pushed status byte
- [ ] `ROL` `ROR` — accumulator and memory forms
- [ ] `CPY`, `CLV`, `TSX`
- [ ] `RTI`
- [ ] Fill in missing addressing modes on existing instructions until all 151 official opcodes decode
- [ ] Decide the policy on the ~105 unofficial opcodes: `nestest` exercises them, and some
      commercial games rely on them. At minimum they must not be silently mis-decoded.

### P3 — Interrupts

The CPU has an `InterruptDisable` flag and nothing that reads it. The APU already maintains a
frame IRQ flag correctly with no line to assert it, and the PPU's vblank NMI is likewise inert.

- [ ] NMI: 7-cycle sequence, vector at `$FFFA`, triggered by PPU vblank when `$2000` bit 7 is set
- [ ] IRQ: vector at `$FFFE`, gated on the `InterruptDisable` flag, asserted by the APU frame counter
- [ ] `BRK` pushing the correct status byte, and `RTI` restoring it
- [ ] Wire `Apu::irq_pending()` — already implemented and tested — to the new CPU IRQ line

### P4 — A headless test-ROM runner

Blargg's ROMs are designed for exactly this: they write a status byte to `$6000` (`$80` = running,
`$00` = pass, other = fail) and a NUL-terminated message at `$6004`. So a runner needs no screen
and no eyes.

- [ ] `tools/rom_test`: load a `.nes`, run to completion or timeout, read `$6000`, print `$6004`
- [ ] Machine-readable output so it can gate CI
- [ ] `nestest` needs a different mode: run from `$C000` with a fixed initial state and diff the
      CPU trace against the published `nestest.log`, line by line, stopping at the first divergence

`apu_probe` is the model here — headless, measures rather than displays, reports the specific
failure. It found three real bugs in a day.

## 3. The test ROMs, in the order they should pass

Roughly easiest-first, and each one's prerequisites are already met by the time it appears.

### Tier 1 — CPU correctness

| ROM | What it proves | Needs |
| --- | --- | --- |
| `nestest.nes` (automated mode) | Every official opcode, and then the unofficial ones, against a cycle-exact golden log | P1, P2, P4 |
| `blargg` `instr_test-v5` | Official instruction behaviour, one sub-ROM per group | P1, P2, P3, P4 |
| `blargg` `instr_timing` | Cycle counts, including page-crossing penalties | as above |
| `blargg` `cpu_interrupts_v2` | NMI/IRQ timing, nested interrupts, `RTI` | P3 |

`nestest` is the right first target: it is self-contained, needs no PPU, and its golden log turns
"something is wrong somewhere" into "line 4711, register X differs".

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

1. **P1** — PRG loading. Nothing else is testable without it, and it is small.
2. **P2** — the ten missing instructions. Mechanical, and each is easy to unit-test first.
3. **P4** — the runner, with `nestest` in log-diff mode. This is the payoff step: the first time
   the emulator is checked against something it cannot argue with.
4. Fix whatever `nestest` finds. Expect this to take longer than writing the runner.
5. **P3** — interrupts, then `cpu_interrupts_v2`.
6. Tier 2 and 3 ROMs, which will also validate the PPU and the APU rebuild independently.

Steps 1–3 are worth doing together: they are the smallest change that turns "we think it works"
into "here is the line where it stops working".
