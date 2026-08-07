# The route that never arrived

*A test fixture that measured the wrong scene, and said nothing about it.*

## The symptom

None. That is what makes this one worth writing down.

## The background

To compare two emulators at a scene inside a game, both have to *reach* that scene. Save states are
no help: each emulator's format is its own, and the reference cannot read ours. The only route that
works for both is to play the game — the same buttons, the same waits, in both.

So both emulators grew a function called `into_a_level`, carrying the same sequence, with a comment
explaining that this was the whole point:

```rust
// The identical sequence `rom_test frame --into-level` runs: Start, four times, with pauses,
// which carries Super Mario Bros 3 from its title screen into a level.
run(&mut deck, 240)?;
for _ in 0..4 {
    tap(&mut deck, JoypadBtn::Start, 68)?;
}
run(&mut deck, 120)?;
```

It was used to investigate the game's status-bar split for two sittings.

## The measurement

The first thing anyone did with it, on the third sitting, was dump the frame it arrived at and look
at the picture.

It was the world map. Four presses of Start take the game from its title screen to the map screen,
and stop there. There is no level.

The comparison had been running at the wrong scene from the beginning. Worse, it had been running at
a scene that *looks* plausible in a summary — a real screen, with sprites and a status bar and
mid-frame register writes — and whose split is driven by an entirely different routine at `$A83C`,
writing `$2005` instead of `$2006`, and running comfortably inside hblank where nothing goes wrong.

Getting the rest of the way took three more presses, each one confirmed against a frame dump rather
than recalled from the game: **Right** to walk off the START panel onto level 1, **Up** onto it, and
**A** to enter. An earlier attempt had tapped A alone and gone nowhere, because Mario was standing
on the panel marked START, where A does nothing at all.

## The fix

Three button presses, in both emulators, and a comment that now says where each step came from:

```rust
// ...where Mario stands on the START panel, on which A does nothing — measured, not assumed:
// this sequence used to tap A alone here and its frame dump was still the world map. Right
// and then Up walk him onto the level 1 panel — each step read off a frame dump, not recalled
// from the game...
```

With the route working, the comparison it enabled produced the answer to the split in a single
afternoon, after three sittings of failing to find it. That story is
[note 1](01-the-split-that-was-not-a-bug.md).

## What it cost

Two sittings of investigation aimed at the wrong screen, and — harder to price — the confidence
those sittings produced. Measurements taken at the map screen were real measurements, carefully
made, and several were written up as facts about the split. They were facts about something else.

**A fixture that reaches the wrong state is worse than one that crashes.** A crash is a bug report.
This produced frames, diagnostics, register logs and dot positions, all internally consistent, all
about a screen nobody wanted. Nothing in the output said "world map"; nothing could have.

The general form: whenever a test harness has to *navigate* to its subject — replaying inputs,
seeking a file, stepping to a breakpoint, logging into an environment — it needs an assertion that
it arrived. Not a comment claiming it arrives. An assertion, or at minimum a human looking at the
artefact once.

The version now in the emulator's test suite asserts on the picture it ends at, so a route that
stops early fails rather than quietly measuring the lobby.
