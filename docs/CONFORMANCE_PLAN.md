# Conformance 🎯

How this emulator is validated against the test ROMs the NES community uses. The live results
table lives in [TODO.md](TODO.md); the bugs the campaign found are told in
[research-log.md](research-log.md).

> Every suite that has a verdict to give passes — nestest (8991/8991 instructions, official and
> unofficial), blargg's instruction/timing/interrupt/reset suites, the PPU vblank/NMI/sprite
> suites, and the APU suites, on NTSC and PAL. What remains open is listed, with hypotheses, in
> [TODO.md](TODO.md).

## Why end-to-end ROMs, not just unit tests

Every audio defect this project had was invisible to its unit tests: each channel behaved
correctly in isolation while the assembled pipeline produced silence, wrong pitch, or a DC level.
Only an end-to-end measurement caught it. The same argument applies to the CPU and PPU, and the
emulator community has already written those end-to-end tests. They are the difference between
"my tests pass" and "it runs Donkey Kong".

## How validation runs

[`tools/rom_test`](../tools/rom_test) is the headless harness:

- **`nestest`** — instruction-by-instruction diff against the golden log, registers and cycles.
- **blargg-protocol ROMs** — run to completion; the ROM writes a status byte to `$6000` and a
  message at `$6004`, so no screen is needed.
- **screen-reading** — for older ROMs that predate the `$6000` protocol, `rom_test screen`
  decodes the text they draw.
- **frame captures** — `rom_test frame` renders any ROM at any frame (with `--press` for
  button taps) and `rom_test baselines` pins committed frame hashes; `compare` diffs a capture
  against a reference emulator's, ignoring palette interpretation differences.
- **`rom_test suite`** — runs every `.nes` under a directory and summarises.

## The ROMs themselves

None can be committed to this repository — `nestest.nes` and blargg's suites are freely
*distributed* but not licensed for redistribution here, and commercial games obviously not. The
runner therefore takes a path or directory, skips cleanly with a clear message when ROMs are
absent, and CI stays green without them. The suites come from the community's
[nes-test-roms](https://github.com/christopherpow/nes-test-roms) collection; nestest and its
golden log from the NESdev wiki's nestest page.
