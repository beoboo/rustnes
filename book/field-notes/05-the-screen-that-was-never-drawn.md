# The screen that was never drawn

*Rendering off is not the same as drawing nothing.*

## The symptom

`full_palette.nes` shows all 64 NES colours at once. The reference draws a screen of colour bands.
We drew black. Entirely black, all 61,440 pixels, one distinct colour in the whole frame.

Three ROMs in that suite, all black. They report by picture alone, so the test runner had been
calling them failures for as long as they had been in the repository, and nobody had looked at one.

## What we thought

The diagnostic output made this look, briefly, like a ROM doing nothing:

```
scanlines drawn   24 / 240   <-- partial frame
blank frames      13
distinct colours  1
BLANK — the PPU drew a single flat colour, so nothing was rendered
```

Then the register log, which is where it stops looking like a broken ROM and starts looking
impossible:

```
$2001 writes during the visible picture (65 of 66 in the frame)
  scanline  23  dot 269  -> $00  rendering OFF
  scanline  24  dot 276  -> $00  rendering OFF
  scanline  27  dot 276  -> $00  rendering OFF
  ...
```

Sixty-five writes to `$2001` in one frame, every one of them `$00`, and **not a single write that
enables rendering**. The ROM never turns the PPU on. It is not failing to draw; it is deliberately
never drawing, and the reference produces a full-colour screen anyway.

## The measurement

A probe in the pixel emitter, printing its state at one dot, answered it in a line — by printing
nothing at all. `emit_pixel` was never called.

Following that upward:

```rust
if rendering_now && on_a_rendered_line {
    self.advance_background_fetch();   // ← which is where emit_pixel is called from
    ...
}
```

With rendering disabled, we emitted no pixels whatsoever, and the frame kept whatever the
frame-clear had put there.

The reference has a comment at the equivalent place, which is the whole lesson in two sentences:

> Pixels should be put even if rendering is disabled, as this is what blanks out the screen.
> Rendering disabled just means we don't evaluate/read bg/sprite info.

And the second half, which is what this ROM is actually exercising: with rendering off, if the PPU's
address register `v` happens to point inside palette memory at `$3F00-$3FFF`, **the colour on the
screen is the palette entry it points at**. The PPU has no fetched pixel to draw, so what reaches
the output is whatever the address register is aimed at.

That is how the demo paints. It never enables rendering, and steps `v` through palette memory with
`$2007` writes, one entry per dot.

## The fix

Emit pixels on visible dots whether or not rendering is enabled, and when rendering is off and `v`
is in palette space, show `palette[v]` rather than the backdrop.

That turned black into a recognisable picture, and revealed a second gap immediately behind it. The
ROM writes `$2001 = $20` between bands — the red de-emphasis bit — to show more than 64 colours.
We drew 52 distinct colours against the reference's 426, because the emphasis constants
`MASK_EMPHASIZE_RED`, `GREEN` and `BLUE` had been *declared in the file for months and never read
by anything*.

Applying the documented attenuation brought it to 223. Deriving the palette properly — all 512
entries decoded from the composite signal, rather than 64 triples with a multiplier over them —
brought it to 420, and took the differing pixels from 53,432 to 9,079.

## What it cost

The bug itself, an afternoon. Its discovery, months of not looking.

**A test that cannot report is not a passing test.** These three ROMs draw a picture and say
nothing a runner can read, so they had been sitting in the "0 passed, 3 failed" column, indistinguishable
from ROMs that fail for real, for the entire life of the project. The moment someone captured a
reference frame and put it next to ours, the answer was one screenshot away. The whole category —
around twenty-five ROMs that report visually — had never been triaged, and this was the first one
looked at.

**"Never read" is a thing to grep for.** Three constants, correctly named and correctly valued, sat
in the PPU with no reader. Nothing warns about that: they are `pub`, so the compiler is satisfied.
A defined-but-unused table is a strong hint that a feature was designed and then not finished, and
it is worth periodically asking which of your constants nothing consumes.
