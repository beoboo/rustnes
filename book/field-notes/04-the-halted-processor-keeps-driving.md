# The halted processor keeps driving

*A ROM declared blocked on evidence nobody had read.*

## The symptom

`dmc_dma_during_read4/dma_2007_read.nes` prints five rows of two bytes and a checksum. We printed:

```
11 22
11 22
22 33      ← the third row is the one that matters
11 22
11 22
7036EAAC
```

The reference emulator printed `44 55` on that row. So did neither of us match, or did one?

## What we thought

This ROM had been written off twice, in the same words both times: *"fails identically to the
reference, so it needs hardware to resolve"*. That is a real category — some ROMs test behaviour no
emulator models — and it is a comfortable place to put something.

It was also wrong, and the reason is embarrassing enough to be worth stating plainly: the
conclusion had been drawn from the **first eight lines** of the ROM's source, which is where the
description ends and before the expected output begins.

Read to line 13, and it says:

```
; Output:
;11 22
;11 22
;33 44 or 44 55
;11 22
;11 22
;159A7A8F or 5E3DF9C4
```

There are **two accepted answers**. The question was never "do we match the reference" — the
reference is not the specification. The question is whether we are on the list. `44 55` is; `22 33`
is not.

## The measurement

Row 3 is the one where a DMC DMA lands on the `LDX $2007`, halting the CPU mid-read. Reading
`$2007` returns a buffered byte and advances the PPU's address, so *counting* the reads is the whole
measurement — each one rotates the buffer to the next byte.

Working backwards from the expected outputs: `33 44` means three reads of `$2007` for that one
instruction, `44 55` means four. We were doing two.

So the halt should cause more reads, and a `$2007` ledger — printing every read with its cycle —
showed the reference's burst directly:

```
R2007 cyc=369794   R2007 cyc=369795   R2007 cyc=369796   [gap]   R2007 cyc=369798
```

Four reads, on consecutive cycles, with one cycle skipped in the middle. That gap is the DMC's own
fetch, which reads the sample address rather than `$2007`.

This is the mechanism: **a halted 6502 does not let go of the bus.** It holds the address it was
reading for every cycle of the halt, and every one of those cycles is a real read with real side
effects. Invisible against RAM. Very visible against `$2007`, whose address register advances each
time, and against `$4016`, whose shift register does.

The project's notes had recorded, weeks earlier, that the reference emulator carried a flag called
`skip_dummy_reads` and that "we don't model dummy reads at all". It had been filed as an
observation, not a defect.

## The fix

The arithmetic is where this gets interesting, because the obvious version is wrong.

Driving the address on every cycle of the halt except the DMC's fetch gives *five* reads, which
prints `55 66` — off the list in the other direction. Both emulators spend the same five cycles on
the halted read; they divide them differently. The reference spends one of them on the real read.
We perform the resumed read without a cycle of its own, because the halt has already been charged
for it.

So two of the halt's cycles are spoken for, and `stalled - 2` of them drive the address. Both of
our stall lengths then land on the accepted list: a four-cycle halt prints `44 55`, a three-cycle
halt `33 44`.

`$4016` and `$4017` stay exempt, which is not a special case so much as what makes the *other* test
in the suite readable — the controller port is clocked by the halt's repeat alone, so
`dma_4016_read` sees exactly one doubled read where `$2007` sees three. The reference carries the
same exemption.

## What it cost

Two sittings of being filed under "blocked", and then about an hour.

**Read the whole specification before concluding it is unsatisfiable.** The ROM told us what it
wanted in a comment at the top of its own source. Nobody scrolled.

There is a subtler trap here too. Matching the reference emulator *felt* like evidence, and it is
the technique this whole project is built on — but a reference is only an oracle where it is
correct. Two emulators agreeing tells you they share an implementation choice; it does not tell you
the choice is right. When a ROM ships its own expected output, that output outranks any emulator,
including the good one.
