# The tiles the game erased itself

*A comment that described the behaviour, and code that did the opposite.*

## The symptom

`ny2011`, a small demo, drew a blank screen. Not a wrong picture — one flat colour, the whole
frame, from the first frame to the last.

Everything about it looked healthy. The CPU ran, five million instructions in six hundred frames.
Rendering was enabled for all 240 scanlines. The emulator's screen reader, which decodes the
nametable into text, could read the message the demo had written there. The backdrop was the right
colour.

The game had drawn its screen and the screen was empty.

## What we thought

A blank picture with a live CPU usually means one of three things, and this had none of them:

**Rendering is off.** It was not — `$2001` was `$1E` from frame 20 onwards.

**The nametable is empty.** It was not — the screen reader printed the demo's text out of it.

**The mapper is wrong.** Plausible, since a mapper fault often shows as missing graphics. But the
header says mapper 0, NROM, the simplest board that exists: no banking at all, 32 KB of program and
8 KB of tiles, both fixed in place. There is nothing in NROM to get wrong.

The remaining suspicion was the palette — that everything was being drawn in one colour rather than
not drawn. That was checked and dismissed: the flat colour was the backdrop entry, and the tiles
simply were not on top of it.

## The measurement

Rather than reason further, a probe was added to the background fetch to print what it was actually
getting, once a frame:

```
frame: nonzero_bg_pixels=0 nonzero_shift_dots=0 last_nt=2B last_pat_lo=00
```

Two numbers side by side, and they disagree. `last_nt=2B` — the nametable fetch returned tile
`$2B`, a real tile index, so nametable memory is fine. `last_pat_lo=00` — the pattern fetch for
that tile returned zero.

The attribute fetch was also working (it reported palette 2). So the PPU was reading nametable
memory correctly and pattern memory as all zeros.

Then the last check, straight against the file on disk:

```
tile 2B: AB F2 85 8E 45 AC 38 30 58 A1 86 7D 3C D4 F8 F0
```

The tile is *in the cartridge*. We were fetching zeros from a ROM that contains data.

## The fix

The demo clears the whole of VRAM at startup, as plenty of demos do — a loop that writes zeros from
`$0000` to `$3FFF`. On hardware, the first `$2000` bytes of that range are the pattern tables, which
on this cartridge are **ROM**. The writes land on a chip that cannot be written and nothing happens.

Here, they landed. The game zeroed its own tiles on the first frame and drew a blank screen ever
after.

`Nrom::write_chr` had carried this comment for months:

```rust
fn write_chr(&mut self, address: u16, value: u8) {
    // Only cartridges with CHR RAM accept this; those with CHR ROM ignore it.
    let index = address as usize & 0x1FFF;
    if index < self.chr.len() {
        self.chr[index] = value;   // ← writes regardless
    }
}
```

The comment states the rule exactly. The code does not implement it. Five of the six mappers had
the same pair.

What hid it is more interesting than the bug. The mapper *could not* have implemented the rule,
because the information had already been thrown away: the ROM loader, on finding a cartridge with
no CHR ROM, helpfully allocated 8 KB of zeros for it to use as CHR RAM. By the time any mapper saw
the data, a RAM board and a ROM board were indistinguishable — both were an 8 KB vector. And CHR
RAM worked *only because* the write went through unconditionally.

Two wrongs holding each other upright. The loader now leaves that vector empty, which is the honest
representation of "this board has no CHR ROM", and each mapper records whether its character memory
is RAM and honours it.

## What it cost

About an hour, once someone looked. It had been latent for months, and would have stayed latent
indefinitely, because triggering it needs two things at once: a cartridge whose tiles are in ROM,
and a program that writes to pattern space. Most programs have no reason to do the second.

Two things to take from it.

**A comment that describes behaviour is not a test.** This one was accurate, specific, and had been
read approvingly by everyone who passed it, including in review. It described what the function was
*for*. Nothing anywhere checked that it did it.

**Helpful normalisation destroys evidence.** The loader filling in 8 KB of zeros was a small
kindness that removed the only signal distinguishing two kinds of hardware. The fix was to make the
loader *less* helpful and return an empty vector, because "empty" is the truth and the truth is what
the mappers needed. Be suspicious of code that fills in a default early: whatever it is papering
over, something downstream may have needed to know.
