# The split that was not a bug

*Super Mario Bros 3, three investigations, and a cartridge from the wrong continent.*

## The symptom

Super Mario Bros 3 draws a black band between the playfield and its status bar. Two rows of it —
193 and 194 — moved sideways by eight pixels between one frame and the next, then moved back. Not
corruption: the picture either side was perfect, and the band was the right size. It wobbled.

You could watch it for a long time. Several people did.

## What we thought

The wobble is eight pixels, which is one tile, which means something is being fetched one tile
early or late. Work outwards from there and you find plenty of suspects, all of them reasonable:

**The interrupt is late.** The split is driven by the MMC3's scanline counter, and the game spins
in a three-cycle loop at `$96F4` waiting for it. A three-cycle loop means the CPU can notice the
interrupt anywhere in a three-cycle window — nine PPU dots — and nine dots is close enough to eight
pixels to feel like an answer. This was the leading theory for two sittings.

**The mapper is counting wrong.** MMC3's counter is clocked by bit 12 of the PPU address rising,
which happens once a line during rendering. We were clocking it 243 times a frame rather than 241,
because a palette access through `$2007` puts `$3Fxx` on the bus and bit 12 of `$3F00` is set.
A real discrepancy, found and measured.

**The handler is too slow.** Timed from the interrupt to the first `$2006` write: 203 CPU cycles,
where landing in hblank would need 226. Twenty-three cycles missing, and no obvious place for them.

Each was investigated. The mapper was cleared — all three extra clocks land in vblank before the
game reloads the counter, so they change nothing. The interrupt was cleared — the handler ends in
`LDX #$0C`, a hard-coded delay loop, so its duration cannot vary between emulators, and every A12
rise was verified at dot 261 on every frame. The CPU was cleared — a 6502 cannot take more than
about fifteen cycles to enter an interrupt, and we measured two.

Everything was cleared, and the wobble was still there. That is usually the moment to notice that
the question is wrong.

## The measurement

The rule in this project is to compare two implementations at the same scene. That had not been
possible here, because the scene is inside a level and the only route to it was a save state that
the reference emulator cannot read. So the first real step was unglamorous: teach both emulators to
*play* their way in, with the same button presses, and dump a frame from each.

That took a while, and produced its own story ([note 7](07-the-route-that-never-arrived.md)). With
it working, the comparison was immediate:

- Ours wobbled: row 194 alternating between two images sixteen pixels apart.
- The reference was pixel-identical for ten consecutive frames.
- **But the reference's `$2001` write jittered exactly as ours did** — dots 263, 266, 271 on
  successive frames. The same 0-3 cycle spread, from the same spin loop.

So the jitter was real, authentic, present in both, and *invisible* in the reference. It was
invisible because all of those dots are in hblank, where nothing is being drawn.

Then the number that settled it. Both emulators traced the same instructions from `$F77B`, took the
same 206 CPU cycles, and entered the handler at the same dot. Yet the reference's PPU advanced
**659 dots** in those cycles where ours advanced **618**.

659 ÷ 206 = 3.2. Ours was 3.

`super-mario-3-eu.nes` is a PAL cartridge. Its iNES header claims NTSC — byte 9 bit 0 is clear —
and the reference ignores the header and consults a database of cartridge hashes instead.

## The fix

There is no bug in the interrupt, the mapper, the CPU or the handler. The game's `LDX #$0C` delay
was tuned by its developers for a machine that runs 3.2 PPU dots to a CPU cycle. Run it at 3.0 and
the same 206 cycles span 41 fewer dots, which puts the whole write burst out of hblank and into the
visible line — where the CPU's entirely genuine interrupt jitter becomes something you can see.

The fix was to implement PAL: a dot clock carrying 3.2 as sixteen dots per five cycles rather than
rounding it, a 312-line frame, the odd-frame dot skip made NTSC-only, and the region read from the
header with an override for the cartridges that lie about it.

Run as the machine it is, the same scene is identical across six consecutive frames, and
`$2001 = $00` lands at dot 284 instead of 235 — hblank, where the jitter paints nothing.

## What it cost

Three separate investigations, spread over weeks, each ending in a confident and wrong write-up.
Along the way it produced several *correct* findings that fixed nothing: the mapper really is
clocked 243 times a frame, the handler really is 194 cycles to its first write, the interrupt
really is taken two cycles after assertion.

That is the part worth taking away. Every one of those measurements was accurate. They were
accurate answers to questions that did not matter, and there was no way to tell from inside the
emulator which kind of question was being asked. The only thing that distinguished them was
comparing against a machine that got the scene right — and the reason it took three attempts is
that the comparison was hard to set up, so it kept being deferred in favour of reasoning, which
was easy.

The correct order is the uncomfortable one: build the comparison first, even when it is a day's
work and the answer feels one thought away.
