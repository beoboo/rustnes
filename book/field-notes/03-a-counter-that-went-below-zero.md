# A counter that went below zero

*The shortest bug in this book, and the one that hung an emulator forever.*

## The symptom

`read_joy3/thorough_test.nes` never finished. The reference emulator prints `thorough_test /
Passed`; ours spun at `$E059`, on these two instructions:

```
$E059  BIT $4015
$E05C  BNE $E059
```

That is a program waiting for a DMC sample to finish. Bit 4 of `$4015` is set while the channel
still has bytes to fetch, and the loop exits when it clears. It never cleared.

## What we thought

Honestly, not much — this one did not survive long enough to accumulate theories. The interesting
part is not the hypothesis, it is how quickly the right instrument produced the answer.

The obvious guess was the loop flag. The test writes `$4010 = $4F`, which sets bit 6, and a looping
sample restarts forever by design. But the routine it hangs in is `sync_dmc`, which begins by
writing `$4010 = $80` — clearing the loop bit — precisely so the sample it starts will end.

## The measurement

The emulator had, by this point, a DMC ledger: an environment variable that makes it print every
`$4015` write, fetch request, halt and fetch with a cycle count. It had been built a few days
earlier for an unrelated problem. Turning it on and reading the last few lines:

```
DMC FETCH addr=DB52 cyc=105023588
DMC FETCH addr=DB53 cyc=105024020
DMC FETCH addr=DB54 cyc=105024452
DMC FETCH addr=DB55 cyc=105024884
```

The sample address is walking upward, one byte every 428 cycles, forever.

The reference's ledger at the same point:

```
DMC FETCH addr=C000
DMC FETCH addr=C000
DMC FETCH addr=C000
```

The same address every time.

That is the whole diagnosis, visible in one glance. `sync_dmc` sets a sample **one byte long**. A
one-byte looping sample restarts in place, so the address never moves. Ours was walking, which means
it was not restarting at all — it was still playing a sample it thought was enormous.

How enormous is easy to guess once you have seen the shape of it.

## The fix

`$4015 = 0` sets `bytes_remaining` to zero to stop the channel. But a DMA the channel asked for
*before* that write is still in flight, and its byte arrives afterwards. `supply_byte` counted it:

```rust
self.bytes_remaining -= 1;
```

`bytes_remaining` is a `u16`. It was zero. In a release build, `0 - 1` is 65535.

A channel with 65535 bytes to play never stops, walks its address up through memory for the rest of
the run, and keeps `$4015` bit 4 set the entire time — so any program waiting for the sample to end
waits forever. `sync_dmc` is such a program, which is why this one ROM caught it and nothing else in
nineteen suites did.

A byte with no sample to belong to now changes nothing: not the buffer, not the address, not the
counter. It also raises no end-of-sample interrupt and does not loop, because the sample was
*aborted*, not finished — a distinction the code had never needed to make before. The reference
guards the whole of its equivalent function the same way.

## What it cost

Twenty minutes, because the instrument already existed.

That is the point of including such a small bug. The fix is one `if`; the reason it was found at
all is that someone had previously built a way to watch the DMC's decisions, for a different
problem, and left it in the codebase behind an environment variable rather than deleting it.

The lesson is about the ledger, not the underflow:

**Instruments outlive the bug they were built for.** This one was written to compare two emulators'
DMA timing cycle by cycle. It then found an unsigned underflow, a sprite-DMA collision, and a
misplaced stall, none of which it was designed for. Debugging output that gets deleted the moment
it works has to be rebuilt from memory the next time, and rebuilding it is the expensive part — in
this project, once costing most of a sitting.

Keep the probe. Gate it behind a flag, latch the flag once so it costs nothing on the hot path, and
leave it in.
