# The book

A free online book teaching how to build a cycle-accurate NES emulator in Rust, in the spirit of
*Crafting Interpreters*. The emulator is the vehicle; the book is the deliverable.

Everything currently under `book/src` is to be rewritten from scratch. This document is the plan
for that, and it is deliberately opinionated: the previous approach — a 47-file scaffold filled in
"later, by reverse-engineering the finished emulator" — produced 4,412 words in 335 commits, 39 of
its 47 chapter files still being a single heading. The scaffold is not the problem. The strategy is.

## Why the old strategy failed, and what replaces it

**Reverse-engineering a book out of a finished emulator does not work**, for a reason worth stating
plainly because it will be tempting again: this emulator's code is an *end state*. It carries a
per-dot renderer, a parity-calibrated DMC stall, an eight-cycle settle, `chr_is_ram` threaded
through six mappers. Every one of those is correct and none of them is teachable as a first step.
A reader cannot be handed the last version of a file; they have to be walked from a version that is
wrong-but-simple to one that is right, and that path does not exist in this repository's history —
the history is a research log, not a curriculum.

So the book gets **its own code**, written in book order, and this emulator becomes the *reference
implementation*: the place accuracy was won, the source of every war story, and the thing the
book's code is checked against. Two lineages, one repository.

## The thesis

Every NES emulator tutorial on the internet ends at "it runs Donkey Kong". This one starts there
and spends the rest of the book on the question nobody else answers:

> **How do you know your emulator is right?**

That is this project's actual expertise and its only real differentiator. It has the tooling
(`rom_test`, `nesref`), the discipline (compare two implementations on a synthetic scene; gate
rendering on a pixel diff, never on summary stats), and — most valuable of all — two dozen
*resolved* accuracy bugs whose diagnosis is written down. No other book on this subject has that.

## Structural decisions

These are the decisions that must be made before a word is written, because everything else
depends on them.

### 1. Two implementations, as *Crafting Interpreters* has jlox and clox

| | Book I | Book II |
|---|---|---|
| Name | `simple` | `exact` |
| Granularity | a frame at a time | a dot at a time |
| Goal | plays Donkey Kong and Super Mario Bros | passes blargg's conformance suites |
| Reader gets | a working emulator, fast | an *accurate* one, and the means to prove it |

This is not an invented structure — it is this project's real history. The per-line renderer
genuinely did run games, and it genuinely did *conceal* the Super Mario Bros 3 status-bar fault
because it samples `$2001` once a scanline and so cannot express mid-line blanking at all. Book II
opens by breaking Book I's emulator on purpose, with the games and ROMs that expose it. That
transition is the best chapter in the book and it is already paid for.

### 2. The book's code is extracted from code that compiles

Every snippet in the book comes out of a real file in `book/code/`, by marker, never by hand. A
chapter that shows code which does not build is the single failure mode most likely to kill this
book's credibility, and it is entirely preventable by tooling. CI fails on a stale snippet exactly
as it fails on a broken test.

### 3. Chapter states are checkoutable

`git checkout book-ch07` gives the reader the emulator as of the end of chapter 7, compiling and
running. Readers join in the middle, get stuck, and want to diff against a known-good state.

### 4. Field Notes

Short interstitial pieces between chapters, each telling one real debugging story from this
project: the wrong hypotheses, the measurement that settled it, and what it cost. They are where
TODO.md's 24,485 words go, and they are the book's voice. Drafts of several already exist as
TODO.md entries — the Super Mario Bros 3 split turning out to be a PAL cartridge, the DMC DMA's
missing clock being the stall's own length, CHR ROM being writable for months because the loader
filled an absent CHR ROM with zeros and destroyed the only signal that told RAM from ROM.

## The arc

Chapter titles are provisional; the shape is not.

### Front matter
- Introduction — what you will build, what you need, why the NES
- A map of the territory — the 2A03 and 2C02, and why a machine this small is this hard

### Book I — A NES that runs
1. The 6502: registers, memory, fetch–decode–execute
2. Addressing modes
3. The instruction set
4. The bus: RAM mirroring and address decoding
5. Cartridges: the iNES header, and NROM
6. **Knowing you are right, part one: nestest** — a golden log, diffed line by line
7. The PPU, a frame at a time: pattern tables, nametables, palettes
8. Sprites and OAM
9. NMI and the frame loop
10. Controllers
11. **Milestone: Donkey Kong**

### Book II — A NES that is right
12. Why a frame at a time lies — the games that break, and how you find out
13. Clocks: the master clock, three dots to a cycle, the odd-frame skip
14. The PPU as a state machine: the dot schedule, fetches, shift registers
15. Scrolling: `v`, `t`, `x`, `w`
16. Sprite zero hit and overflow, per dot
17. Mid-frame writes: forced blanking, split screens, and Super Mario Bros 3's status bar
18. Interrupts properly: NMI timing, IRQ latency, hijacking
19. Mappers with state: UxROM, MMC1, and MMC3's scanline counter
20. The APU: the frame counter, pulse, triangle, noise
21. The DMC, and DMA that steals cycles from the CPU
22. Open bus, and the things that show up in exactly one game

### Book III — Knowing you are right
23. Conformance ROMs: what they measure and how to read one that only draws
24. Differential testing: driving a second emulator and diffing traces
25. Frame baselines, palette-independent diffing, and gating on pixels
26. Debugging what you cannot see: probes, ledgers, and bisecting a hypothesis

### Appendices
- 6502 reference; PPU register reference; mapper reference; where to get ROMs; further reading

## Production

**mdBook**, kept. It is already in the repository, it is boring, and the effort belongs in the
prose and the code rather than in a static-site generator. Revisit only if a concrete need appears.

To build:

- `book/code/simple/` and `book/code/exact/` — the two lineages, each its own crate, each in the
  workspace so `cargo build --workspace` and CI cover them.
- A snippet preprocessor: markers in the source (`// snip: ch07-bus-read`), an mdBook preprocessor
  that inlines them, and a CI check that fails when a marker referenced by the book is missing.
- Chapter tags, `book-ch01`…, cut when a chapter is finished.
- A CI job that builds the book and fails on a broken internal link.
- Hosting: GitHub Pages from the existing CI.

Housekeeping the plan depends on: the stray empty `book.toml` at the repository root should go
(the real one is `book/book.toml`), and README's "the book is deferred" paragraph is now wrong.

## Sequencing

Not chapter one first. In this order:

**Step 1 — Harvest the war stories. Started 2026-08-07; seven of about twenty written.** Before
anything else, because this material is *decaying*: TODO.md is 2,389 lines in which six claims were
found wrong in two days. Each resolved accuracy bug becomes a `book/field-notes/` file — symptom,
wrong hypotheses, the measurement that settled it, the fix, what it cost.

Written so far: the Super Mario Bros 3 split turning out to be a PAL cartridge; CHR ROM being
writable; the DMC counter that went below zero; the halted processor that keeps driving its
address; the screen that was never drawn; comparing two emulators through a video filter; and the
input route that never reached the level it claimed to. Every code quotation in them has been
checked against the commit it came from rather than reconstructed.

Still to write: the DMC DMA's missing clock, the sprite DMA collision and the grid phase, the
baseline that lived in `/tmp`, the false failure read off a screen, the unofficial NOPs that were
doing nothing, and the `BRK` that switched the machine off.

**Step 2 — Prove the pipeline on one vertical slice.** Pick a single code-heavy chapter — chapter 4,
the bus, is a good size — and take it all the way: code in `book/code/simple/`, snippets extracted,
prose written, chapter tag cut, built and published. The point is to find the voice and shake out
the tooling on something small, before committing to a shape twenty-six times over.

**Step 3 — Book I's code, in book order.** Write `book/code/simple/` chapter by chapter, each state
compiling. Do not write prose yet. This is where it becomes clear whether the chapter boundaries in
this plan survive contact.

**Step 4 — Book I's prose**, against code that already exists and runs.

**Step 5 — Book II**, which is where the emulator in `crates/` finally pays off: `exact` is allowed
to converge on it, and the Field Notes attach to the chapters they belong to.

## What this means for the emulator

The emulator is no longer the point, and work on it should be justified by what it teaches. That
does not mean stopping — Book II needs a correct reference to converge on, and the remaining
accuracy work is genuinely book material. It does mean the bar changes: a fix worth a Field Note is
worth doing; polish that no chapter will ever mention is not.

Two open items are now clearly *worth* doing because chapters need them:
- **The palette table.** A measured 512-entry table replacing the 64-entry approximation — chapter
  25 is about palette-independent diffing and cannot honestly recommend a comparison the book's own
  emulator fails.
- **PAL.** Chapter 13 is about clocks; a book that says "three dots to a cycle" without ever
  showing the machine where it is 3.2 has skipped the reason the number matters.

## Risks

- **Scope.** Twenty-six chapters is a multi-year book written alone. The mitigation is Book I being
  independently shippable: it ends at Donkey Kong and is a complete thing on its own.
- **The reference implementation pulling rank.** The temptation will be to paste from `crates/`
  because it is right. It is right *and* twenty refactors past teachable.
- **Accuracy work as procrastination.** It is more fun than writing, it always has a next bug, and
  it produces the feeling of progress without producing chapters. Step 1 exists partly to make the
  writing start before the next interesting bug appears.
