# Building a per-cycle CPU

> **Status: landed.** The design below was implemented; `cpu_interrupts_v2` passes 6/6 and
> `branch_timing` 3/3 — see [TODO.md](TODO.md) for the measured suite table. Kept as the design
> record, because three intuition-first attempts failed and this document is why the fourth didn't.

Three attempts at cycle-accurate interrupt timing have been reverted. This document is the result
of stopping to check how it is actually done, rather than attempting a fourth from intuition.

## What the hardware does

Two facts from the NESdev documentation decide the whole design.

**Interrupts are sampled before an instruction's last cycle, not at its end.** The wiki is precise:
it is "the status of the interrupt lines at the end of the second-to-last cycle that matters". The
CPU reads the lines during one cycle and acts on them during the next.

That single rule explains the behaviour every test is checking:

- `CLI`, `SEI` and `PLP` change the I flag *after* the poll, so the flag they set is not the flag
  the poll saw. One further instruction always runs before the interrupt arrives. This is what
  `cpu_interrupts_v2/1-cli_latency` measures.
- Branches poll before their operand fetch but *not* before the third cycle of a taken branch, with
  an extra poll before the page-crossing fixup. This is `5-branch_delays_irq`.
- The interrupt sequence itself never polls, so at least one instruction of a handler always runs
  before another interrupt is taken.
- NMI can hijack a `BRK` in progress: during the first four cycles, after the program counter has
  been pushed but before the status byte, whichever line is asserted decides the vector. This is
  `2-nmi_and_brk` — the test that hung on two of the three attempts.

**The 6502 has no bus-idle state.** Every cycle drives the address bus and performs a read or a
write, including cycles that do nothing useful with the result:

- implied and accumulator instructions read the byte after the opcode and discard it
- a push reads that byte too, before writing to the stack
- a pull reads it, then reads the stack while incrementing the pointer
- read-modify-write writes the *unmodified* value back before writing the modified one
- indexed addressing reads the unfixed address when the index carries — already implemented
- a taken branch reads at the unfixed program counter before correcting it

These accesses are invisible against RAM, which is why they can be missing for a long time without
anything appearing wrong. They stop being invisible against a register with side effects: a
discarded read of `$2007` still advances the PPU address.

## Why the previous attempts failed

Attempts 2 and 3 both tried to name the sampling cycle from *outside* the instruction — from a
latch taken before it, and from its cycle count watched for by the bus clock. Neither can work
while the emulator performs fewer bus accesses than the instruction takes cycles, because then
there is no correspondence between "cycle 4 of this instruction" and anything observable.

That gap is measurable. `rom_test cycles nestest.nes` reports it per opcode:

```
37 of 225 distinct opcodes account for every cycle
6463 of 8991 executed instructions are short of at least one access
```

The gaps fall into addressing-mode families rather than being scattered — `$EA` (NOP, implied) is
short one, `$EE` (INC absolute, read-modify-write) short three, `$FE` (INC absolute,X) short four.
So the work is bounded by addressing mode, not by opcode.

## The plan

**Once every cycle is modelled as the bus access it is on hardware, accesses and cycles are the
same number, and a cycle can be named from outside the instruction.** Everything else follows from
that, which is why it comes first.

### 1. Close the gap

Add the missing accesses, one addressing-mode family at a time, checking `rom_test cycles` after
each. Every instruction in a family shares the fix, so the count should fall in large steps.

Order, easiest and highest-count first:

- implied and accumulator: the discarded read of the following byte
- push and pull: the same read, plus the stack read a pull performs
- read-modify-write: the write of the unmodified value, in all its addressing modes
- taken branches: the read at the unfixed program counter, and the fixup read on a page cross
- `JSR`, `RTS`, `RTI`, `BRK`: their explicit sequences

**Done.** `rom_test cycles` reports "every opcode accounts for all of its cycles" across all 8991
of nestest's instructions, from 6463 short when this was written. The check is now a test as well as
a tool — `every_instruction_accesses_the_bus_once_per_cycle` runs it on synthesised programs, since
nestest cannot be committed, and any future instruction that forgets a cycle fails it.

The last of it came from unexpected places rather than from the addressing-mode families listed
above: the unofficial `NOP`s were not reading their operands at all, every pull was doing its own
dummy stack read, `RTS` was missing the read on its sixth cycle, and the interrupt sequence was
missing the two discarded reads it begins with.

### 2. Move the sample to the right cycle

Step 1 is done: the gap is 78 of 8991, from 6463.

Sampling was then tried, and **it works** — `cpu_interrupts_v2/1-cli_latency` passes, the first test
in that suite ever to. That is the proof the mechanism is right, because that test measures the
sampling rule directly. The attempt is preserved in `scratchpad/step2-sampling/`.

It was reverted anyway, because on its own it is not finishable:

- `2-nmi_and_brk` and the combined `cpu_interrupts` go from failing to **hanging**. Both need NMI
  hijacking of BRK, which is step 3. A hang is worse than a failure: in a game it is a freeze.
- Six interrupt unit tests break, and they are not wrong so much as written against the old model.
  They assert a line raised between instructions is serviced by the very next `step`. Under the
  sampling rule an instruction must first run to take the sample, so each needs an instruction
  placed in between — and since their RAM is zeroed, the instruction that would otherwise run is
  BRK. Their program-counter assertions shift accordingly.

One detail worth keeping, because it cost a wrong measurement: the opcode fetch is cycle one and it
happens *before* the opcode is known, so `poll_at` cannot be set until after it. A two-cycle
instruction samples at the end of cycle one, which is therefore already past by the time the target
is set, and has to be taken immediately rather than waited for. Without that, the sample never
fires on short instructions and the whole thing behaves like the crude latch it replaced.

**Steps 2 and 3 landed together**, and sampling works: `1-cli_latency` passes, the first test in
that suite ever to. It measures the sampling rule directly, so it is the one that matters.

Three architectural faults were found on the way, each of which had produced a misleading symptom:

1. **The NMI edge latch had two owners.** `sample_interrupts` consumed it and so did the BRK
   hijack, so whichever looked first destroyed it for the other. NMI is edge-triggered: the edge
   sets a latch that persists until the interrupt is *serviced*, and reading it is not servicing
   it. Sampling now reads without taking.
2. **The hijack tested a level, not an edge.** Once the latch correctly persisted, every BRK saw a
   pending NMI and was redirected — so BRK never reached the IRQ vector at all. Only an NMI that
   *arrives during* the sequence takes it over; one already waiting would have been serviced
   instead of running BRK.
3. **The opcode fetch is cycle one and happens before the opcode is known**, so the sample target
   cannot be set until after it. A two-cycle instruction samples at the end of cycle one, already
   past by then, and must be sampled immediately rather than waited for.

### What still hangs, and why it is not the CPU

**Resolved, and the diagnosis below was wrong.** `2-nmi_and_brk` passes. It was never the vblank
synchronisation loop: the system peeked at the next opcode after every step and, on seeing `$00`,
declared the program Finished and switched the machine off. The program counter then sat on the
`BRK` for the rest of the run, which is what made it look like a spin in whatever loop happened to
contain it. A convenience for the debugger — where a hand-assembled snippet really does end with
`BRK` — and fatal for a cartridge, where `BRK` is an instruction with a handler behind it.

It was found by asking the runner where a hung ROM was spinning and disassembling what it found
there, which took a few minutes and answered "`BRK`, and only `BRK`" for every one of them. The
guess below stood for several sittings and shaped the plan: it is the reason this document says the
remaining interrupt tests are blocked on the PPU. They are not.

The original note follows, as a record of what a plausible unverified diagnosis costs.

> Reading their source settles it: both include `sync_vbl.s` and spin in a vblank synchronisation
> loop — the address they hang at is that loop. `1-cli_latency`, which passes, does not include it.

That loop needs cycle-exact alignment between the CPU and the PPU, which the PPU cannot yet
provide. **The remaining interrupt tests are therefore blocked on the PPU work, not on more CPU
work.** The two rewrites are coupled, which was not obvious before and changes their order: there
is little point attempting `3-nmi_and_irq` or the combined ROM until the PPU runs per dot.

### How Mesen does it, and why ours is harder than it needs to be

Read from the Mesen2 source after a session of reasoning failed to close a one-dot discrepancy.
Worth writing down because it is *structurally* different from what is here, not a detail.

**Mesen keeps no notion of "the cycle that polls".** Every CPU cycle ends like this:

```cpp
void NesCpu::EndCpuCycle(bool forRead) {
    _masterClock += forRead ? (_endClockCount + 1) : (_endClockCount - 1);
    _console->GetPpu()->Run(_masterClock - _ppuOffset);

    _prevNeedNmi = _needNmi;                                   // one-cycle-delayed shadow
    if(!_prevNmiFlag && _state.NmiFlag) { _needNmi = true; }   // edge detect, during phi-2
    _prevNmiFlag = _state.NmiFlag;

    _prevRunIrq = _runIrq;                                     // and the same for IRQ
    _runIrq = ((_state.IrqFlag & _irqMask) > 0 && !CheckFlag(PSFlags::Interrupt));
}
```

and every instruction ends like this:

```cpp
(this->*_opTable[opCode])();
if(_prevRunIrq || _prevNeedNmi) { IRQ(); }
```

The rule "it is the status of the interrupt lines at the end of the second-to-last cycle that
matters" is not computed from anywhere. It *falls out*: the shadow is one cycle behind, so at the
end of the instruction it holds what the lines said one cycle earlier. No cycle counting, no
`poll_at`, nothing that has to know how long an instruction is.

That is the same rule this document already describes, arrived at without the thing that made
attempts 2 and 3 fail — needing to name a cycle from outside the instruction. **A one-cycle-delayed
shadow of the interrupt lines, updated by the same clock that advances the PPU, is worth trying in
place of the computed `poll_at`.** It cannot be wrong about instruction length because it never
asks.

Two more differences, both measurable:

- **The PPU is advanced around each access, not before it.** NTSC is twelve master clocks a CPU
  cycle and four a dot. Mesen runs the PPU five clocks in, performs the access, then runs the
  remaining seven — and for a *write* it is seven then five. Here the whole three dots run and then
  the access happens, so every read sees a PPU nearly a full cycle further along than it should,
  and reads and writes see it at the same point when hardware does not.
- **Interrupts are sampled after the access, not before it.** The access is part of the cycle and
  can be what changes the lines: writing `$2000` to enable the vblank NMI is exactly that case, and
  it is what `ppu_vbl_nmi/07-nmi_on_timing` measures.

Tried, and inert — recorded so they are not tried again. Moving our sampling to after the access,
and collecting the NMI after each of a cycle's three dots rather than after all three, each changed
nothing at all: not the failing table, not even the instruction count. Both are correct in
principle and neither is where the dot is going, because the sampling that decides the outcome is
the one driven from `poll_at` at the end of `step`, not the one in `tick_bus`. The shadow-register
rewrite above replaces that machinery outright, which is why it is the thing to try rather than
another adjustment around it.

### Done, and what it took

`poll_at` is gone. The CPU keeps `need_nmi`/`prev_need_nmi` and `run_irq`/`prev_run_irq`, updated
every cycle by the same clock that advances the PPU, and `poll_interrupts` acts on the shadows.
Nothing computes a polling cycle or asks how long an instruction is.

**The shadow alone changed nothing.** Not the `05-nmi_timing` table, not the instruction count, not
one test — it was byte-for-byte the behaviour `poll_at` had produced. That is the right result and
worth stating plainly: for a two-cycle instruction the computed poll had always landed on the same
cycle the shadow does, so the rewrite bought structure, not accuracy. Everything below is what
actually moved the dot, and none of it was reachable while `poll_at` decided the outcome.

**Where the dot was, measured rather than argued.** Instrumenting the clock to record which of a
CPU cycle's three dots raised the NMI, across the ten lines of `05-nmi_timing`, gave the position
directly: dot 0, 2, 1, 0, 2, 1 … one earlier per line, as a test that runs one PPU clock later each
line should. Lining that up against the table showed the failing lines were exactly the ones where
the NMI arrived on the *first* dot of a cycle, and that those needed to be attributed to the cycle
before. Nothing else fitted: a shift of two dots would have moved lines that were already right.

**What that means, and it is not about the poll's granularity.** Our clock ran all three of a
cycle's dots and then performed the bus access, so the lines were read at the instant of the access.
A 6502 cycle does not end there — the access happens partway through and the cycle runs on past it.
So the poll belongs one dot later, at the cycle's end. The clock now takes a `ClockPhase`: two dots
before the access, one after, and the interrupt lines read at the end of the second. Mesen does the
same thing by construction, running the PPU five master clocks in, performing the access, then the
remaining seven, and sampling after.

That one dot is the whole of it: `05-nmi_timing` passes with the expected table, and `06-suppression`
followed without being aimed at. `ppu_vbl_nmi` went 5/11 to 7/11 and `apu_test` 4/9 to 5/9.

**The alignment dot.** Splitting the cycle also moved every bus access one dot earlier against the
PPU, which was not the intention — only the poll was meant to move. One extra PPU tick at power-on
puts the accesses back, and it is a real hardware degree of freedom rather than a fudge: the CPU/PPU
alignment is settled at power-on and is not always the same. It is verifiable rather than a matter
of taste — with it, `02-vbl_set_time` and `03-vbl_clear_time` run to exactly the instruction counts
they did before the split.

**A second dot, which turned out to be the same one.** `mmc3_test/4-scanline_timing` measures when
the mapper's IRQ arrives *in the program's own time*, so it constrains the A12 rise and the poll
together. Moving the poll a dot later therefore broke it, and `ADDRESS_BUS_LEAD_DOTS` had to come
down from one to zero to compensate — at which point the A12 rise was reported on dot 261 rather
than the documented 260, and the two ROMs appeared to want different things.

They do not. Reading Mesen settled it in one line. Its sprite fetch runs at
`(_cycle - 257) % 8 == 4`, which is **cycle 261** — the comment beside it says "Cycle 260, 268, etc."
and is as loose as the wiki's figure. Both are describing the same fetch. Dots 257-320 fetch eight
sprites in groups of eight: a garbage nametable read, a garbage attribute read, then the two pattern
bitplanes. Only the patterns reach $1000, and their group begins four dots into the group, at 261.
Mesen's dot numbering is ours — it increments the cycle and then processes it, and sets the vblank
flag at cycle 1 of the NMI scanline — so the two are directly comparable, and its background
schedule matches ours dot for dot as well.

So `ADDRESS_BUS_LEAD_DOTS` was never a property of the hardware. It was a knob holding a real error
still: with the interrupt poll a dot early, giving the address bus a one-dot lead cancelled it for
anything measured through the mapper, and the pair agreed often enough to look right. Fixing the
poll exposed it. **The constant is deleted** rather than left at zero, so there is no fudge factor
sitting there to be bent the next time a dot goes missing; the address goes on the bus for the dot
the read begins on, which in a per-dot model is the same event. The unit test now asserts 261, says
why, and records that it once read 260 and what that cost.

There is no delay to model between the counter reaching zero and the CPU seeing /IRQ. Mesen's
`TriggerIrq` sets the flag the CPU's own end-of-cycle poll reads, exactly as ours does, and a scope
measurement on an MMC3B puts the cartridge's own delay at about 69 ns — a third of a pixel.

**`07-nmi_on_timing` and `08-nmi_off_timing` followed, once /NMI became a level.** They were not
moved by any of the above, and correctly so: they turn the NMI enable on and off around the vblank
flag, which is about what the PPU drives rather than about when the CPU looks.

Our /NMI was a one-shot latch — the PPU set it at vblank and the system consumed it. That can
express a vblank *arriving* but not the line being *released*, so a program toggling `$2000` bit 7
during vblank got one interrupt where hardware gives it one per rising edge. It is now a level, as
Mesen has it: the PPU holds the line down for as long as the vblank flag and the enable bit are both
set, and drives it at each of the four places that can change either — vblank set, pre-render clear,
`$2002` read, `$2000` write.

Nothing had to be added to the CPU to take advantage of it. The edge detector the shadow rewrite
already put in `end_cpu_cycle` is exactly what turns a level into an interrupt, and it was sitting
there detecting edges on a latch that only ever had one. The only CPU change was that servicing an
NMI no longer clears the line — the CPU does not drive it and cannot take it away.

Two quirks stopped being special cases and became consequences, which is the sign the model is
right rather than merely tuned:

- A `$2002` read on the dot vblank begins, or the dot after, suppresses the interrupt. There is no
  longer a rule for this. The read clears the flag, the line goes up with it, and the line was down
  for less than a whole CPU cycle — so the CPU's once-per-cycle poll never saw it. A read well into
  vblank releases the line just the same and does *not* suppress anything, because by then the edge
  has long since been counted.
- Releasing the line cannot take back an interrupt already detected, and holding it down cannot
  produce a second. Both fall out of edge detection over a persistent signal, and both have a test.

**`10-even_odd_timing`, and the suite is 11/11.** The odd-frame dot skip was wrong in two ways at
once, and — this is the point — each was cancelling the other, so fixing either alone changed
nothing whatsoever. Both were measured that way rather than assumed:

- **The decision was taken on dot 340, not 339.** Declining to process dot 340 once it arrives is
  not the same as jumping from 339 to 340. The frame comes out the same length either way; the
  question is asked a dot later.
- **It read `$2001` directly rather than a delayed copy.** A write to `$2001` does not reach the
  rendering hardware in the cycle that performs it — the wiki and Mesen agree on one cycle:
  "setting it at cycle 5 will render cycle 6 like cycle 5 and then take the new settings for cycle
  7". So the write took effect a dot early.

One dot late and one dot early. `10-even_odd_timing` is built to catch exactly this: it enables the
background at a chosen dot and counts the PPU clocks in the resulting frame, and it runs the same
sequence at two sync offsets one clock apart. We passed the first and failed the second — the
classic signature of a boundary in the right place for the wrong reasons.

Recorded because the measurement is the lesson: the delayed flag on its own left the test
byte-identical, same instruction count and all, and so did moving the decision to dot 339 on its own.
Either one looks inert and would have been reverted by this project's own rule about not keeping
changes that cannot be demonstrated. They are only visible together, and the way to find that out
was to try both singly and check.

**`cpu_interrupts_v2` is complete, and the last row of it was not an interrupt bug at all.**
`4-irq_and_dma` walks an IRQ across a sprite DMA one cycle at a time — 528 rows — and every row
matched except `+526`. The cause was that the machine had **two dividers where hardware has one**.
A transfer is 513 cycles, or 514 when the `$4014` write lands on the wrong half of the CPU's
get/put divider, and we read that parity off a cell of our own toggled from the clock closure —
which runs once per *bus access*. Cycles with no access behind them were missed: the leftover ones
at the end of an instruction, and all five hundred odd of every transfer. A transfer is an odd
number of cycles, so each one inverted the cell against the real divider and the *next* transfer's
length came out wrong half the time. The APU's frame counter runs the same divider and its phase is
pinned by evidence — `apu_test/4-jitter` measures the `$4017` write delay on alternating cycles —
so the parity now comes from there, and `Apu::apu_cycle` is taken from the frame counter rather
than kept as a third copy that a soft reset could knock out of step.

Two things were tried on the way and both are recorded as not-the-answer, because each looked
compelling:

- **Running the transfer inside the instruction that triggers it** — which is what tetanes does, and
  what a differential trace appeared to show. It was built and measured: with the divider right, the
  ROM passes either way. The transfer's cycles land in the same place on the timeline whichever step
  owns them. Reverted. What the trace was really showing was our own trace tool printing a line per
  step.
- **Polling the interrupt shadow through the transfer's cycles.** Tried and reverted three times.
  The third attempt finally measured what it does rather than only noting that it moved no row: it
  takes the interrupt an instruction early, turning a four-cycle window in the table into a
  517-cycle one. The decision about the halted instruction was taken before the halt began.

The gate for this is `dma_interrupt_timing` in `nes_system.rs`, not the ROM. It reproduces the
ROM's landing sequence byte for byte, raises `/IRQ` from outside any device — the APU and the
mapper are the only real sources and either would fold its own timing into the measurement — sweeps
every arrival cycle, and asserts the table printed in the ROM source's header comment, which is a
recording from a real NES. A tenth of a second against twenty minutes, and it names the cycle
instead of saying "Failed".

The delayed flag is currently read by the skip and nothing else. Mesen carries the idea further —
a second copy another cycle behind again, which its scroll and fetch work uses — and that is worth
doing, but it changes what is drawn rather than only when a frame ends, so it wants the pixel diff
as its gate rather than this suite.

### 3. The special cases

Each is a documented exception rather than a consequence of the model, so each needs its own code
and its own test:

- branches: poll before the operand fetch, not before the third cycle of a taken branch
- the interrupt sequence: no polling within it
- `BRK` hijacking: NMI asserted during the first four cycles changes the vector

Expected to fix `2-nmi_and_brk`, `3-nmi_and_irq` and `5-branch_delays_irq`.

## How it will be judged

`cpu_interrupts_v2` (was 0/6, now every ROM in it passes), `instr_timing` (1/3), `branch_timing_tests` (0/3), `cpu_dummy_writes`
(0/2), `cpu_exec_space` (0/2). nestest must stay at 8991/8991 throughout — it is the check that the
added accesses have not changed what instructions compute, only when they touch the bus.

Step 1 is safe to do incrementally and is worth landing on its own even if the rest waits: the
dummy accesses are correct behaviour regardless, and `cpu_dummy_writes` tests them directly.

## Sources

- [CPU interrupts — NESdev Wiki](https://www.nesdev.org/wiki/CPU_interrupts)
- [6502 cycle-by-cycle bus operation tables](https://www.nesdev.org/6502_cpu.txt)
- [Visual6502 6502 Timing States — NESdev Wiki](https://www.nesdev.org/wiki/Visual6502wiki/6502_Timing_States)

A note on the last one: it says some cycles are "idle in terms of instruction work", which reads at
first as contradicting the claim that every cycle drives the bus. It does not — it is describing
internal work, not bus activity. The cycle tables settle it by listing a bus operation for every
cycle of every addressing mode.
