# Comparing two emulators through a filter

*The reference was lying, and we had asked it to.*

## The symptom

An early comparison against the reference emulator reported something startling: on a ROM whose
screen is a flat grey backdrop and nothing else, **every pixel differed**. Ours came out
`117,117,117`; the reference `83,83,83`.

A uniform difference across an entire flat screen is a satisfying kind of bug. It looks like one
wrong number in one place.

## What we thought

The obvious reading, and the one that got written up: the two emulators disagree about the
**power-on contents of palette RAM**. Real hardware powers up with unspecified values there, ROMs
exist that document what a real console tends to hold, and a different starting palette would tint
a backdrop exactly like this.

It was plausible, it explained the observation completely, and it was published in the project's
notes as a finding.

It was wrong twice over.

## The measurement

The first error surfaced when someone tried to *use* the finding and looked at a second ROM. The
difference was not a constant offset — it varied by colour, and it varied in a way that looked less
like a different palette and more like a different *rendering* of the same one.

The reference emulator defaults to an NTSC composite video filter. It simulates what a television
would have made of the signal: neighbouring pixels blend, and every colour shifts. We had been
capturing filtered frames and comparing them, pixel for pixel, against our unfiltered ones.

Setting the reference to its `Pixellate` mode — no filter, one output pixel per PPU pixel — removed
most of the difference at a stroke.

What remained was smaller and more mundane: the two emulators ship **different palette tables**.
Ours rendered NES black as `0,0,0`, the reference as `3,3,3`. Nothing to do with palette RAM. The
same palette *entry*, two opinions about its RGB.

## The fix

Two, and the second matters more than the first.

The immediate fix was one line in the reference driver, forcing the unfiltered output — with a
comment recording why, because a future reader would otherwise "helpfully" restore the default.

The real fix was to stop comparing RGB at all. Frames are now compared **palette-independently**:
two pictures match if the same regions took the same palette entries, whatever RGB either emulator
assigns them. That comparison survives both problems and every future disagreement of the same
shape.

It needed one more revision later, for the same underlying reason. Requiring a strict
one-to-one mapping between the two colour sets turns out to be unsatisfiable between emulators whose
palettes are derived differently — one will always quantise a pair of entries onto the same RGB
where the other keeps them apart, and every pixel of both entries then reads as "differing". So the
comparison now asks a weaker question when the strict one fails: an **edge map**, comparing where
the colour *changes* rather than what it is. On the light-wall demos it reads 0.00% against a
reference whose exact RGB differs in almost every pixel.

## What it cost

A published wrong conclusion, and a stretch of time in which the reference emulator — the project's
one oracle — was quietly untrustworthy.

That is the thing worth carrying out of this note. The technique this whole book recommends is to
compare against an implementation that already works, and this is the failure mode of that
technique: **your oracle has settings.** It has a default output pipeline designed for human eyes,
not for diffing. It has a palette table that is an opinion. It may have a game database that
overrides the header you are reading.

None of that makes the oracle less useful. It means the comparison itself is a thing to be
validated before its results are believed, ideally on a case where you already know the answer. The
cheapest version: point both emulators at the same static, boring screen and confirm they agree
*before* you go looking at the screen you actually care about.

And when they disagree everywhere at once, suspect the apparatus before the emulation. A bug in
your machine is usually specific. A difference in every pixel is usually a lens.
