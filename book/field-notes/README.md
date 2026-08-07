# Field Notes

Short pieces about bugs that were actually found in this emulator, one story each. They go between
the chapters of the book.

They exist because of something the rest of the book cannot easily teach. A chapter shows you
working code and explains why it is right, which is exactly the view you will not have when your
own emulator draws a black screen. These are the other view: what the symptom looked like, which
plausible explanations turned out to be wrong, and what measurement finally settled it.

## The format

Each note answers five questions, in this order, because that is the order the work happened in:

1. **The symptom** — what you could see, stated as narrowly as possible.
2. **What we thought** — the hypotheses, including the wrong ones, and why each was reasonable.
3. **The measurement** — the observation that decided it. This is the part worth reading.
4. **The fix** — usually small, often anticlimactic.
5. **What it cost** — sittings, wrong turns, and the lesson if there is one.

A note earns its place by having a measurement in step 3. A bug that was fixed by staring at the
code until the answer appeared is not a story, however satisfying it was at the time.

## The one lesson underneath all of them

Stated here rather than repeated twenty times:

> Every correct diagnosis in this project came from running two implementations side by side and
> comparing them on a specific, reproducible scene. Every diagnosis argued from the mechanism alone
> — "it must be the interrupt latency", "it has to be the mapper" — was wrong.

That is not a claim about intelligence. The mechanisms in this machine interlock so tightly that a
plausible story can be told about almost any of them, and plausibility is worthless here. The
emulator that already works is the only oracle you have, and the discipline is to consult it before
forming an opinion rather than after.

## The notes

| | Note | The lesson in a line |
|---|---|---|
| 1 | [The split that was not a bug](01-the-split-that-was-not-a-bug.md) | Three investigations, and the cartridge was from a different continent |
| 2 | [The tiles the game erased itself](02-the-tiles-the-game-erased-itself.md) | A comment that described behaviour the code did not have |
| 3 | [A counter that went below zero](03-a-counter-that-went-below-zero.md) | 65535 is what `0 - 1` looks like in a release build |
| 4 | [The halted processor keeps driving](04-the-halted-processor-keeps-driving.md) | Read the whole ROM's source before believing it is blocked |
| 5 | [The screen that was never drawn](05-the-screen-that-was-never-drawn.md) | Rendering off is not the same as drawing nothing |
| 6 | [Comparing two emulators through a filter](06-comparing-through-a-filter.md) | The reference was lying, and it was our fault |
| 7 | [The route that never arrived](07-the-route-that-never-arrived.md) | A test fixture that silently measured the wrong scene |

More to come: the DMC DMA's missing clock, the sprite DMA collision and the grid phase, the
baseline that lived in `/tmp`, and the false failure read off a screen.
