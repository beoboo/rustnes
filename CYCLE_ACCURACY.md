# Building a per-cycle CPU

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

Done when the report says every opcode accounts for all of its cycles. That check then becomes a
test rather than a tool, and any future instruction that forgets a cycle fails it.

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

`2-nmi_and_brk` and the combined `cpu_interrupts` hang. Reading their source settles it: both
include `sync_vbl.s` and spin in a vblank synchronisation loop — the address they hang at is that
loop. `1-cli_latency`, which passes, does not include it.

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

**An unresolved dot, recorded so it is not rediscovered.** `mmc3_test/4-scanline_timing` measures
when the mapper's IRQ arrives *in the program's own time*, so it constrains the A12 rise and the
poll together. Moving the poll a dot later therefore broke it, and `ADDRESS_BUS_LEAD_DOTS` had to
come down from one to zero to compensate — at which point the A12 rise is reported on dot 261 rather
than the documented 260. Both cannot be satisfied at once:

| `ADDRESS_BUS_LEAD_DOTS` | A12 rise | `4-scanline_timing` |
| --- | --- | --- |
| 1 | dot 260, as documented | fails |
| 0 | dot 261 | passes |

So something between the A12 rise and the CPU noticing /IRQ is still a dot out. It is not the
cartridge: a scope measurement on an MMC3B puts that delay at about 69 ns, a third of a pixel. The
unit test that pins the rise now asserts 261 and says why. This is the next thing to settle, and by
this project's own record it wants a reference implementation and a trace, not another sweep.

Still failing and not moved by any of this: `07-nmi_on_timing` and `08-nmi_off_timing`. Both turn
the NMI enable on or off around the vblank flag, so they are about the PPU's `$2000` handling rather
than about when the CPU polls — our NMI is a one-shot latch raised by the PPU, where hardware holds
a level for as long as the flag and the enable bit are both set. That is the next piece of
`ppu_vbl_nmi`, and it is PPU work, not CPU work.

### 3. The special cases

Each is a documented exception rather than a consequence of the model, so each needs its own code
and its own test:

- branches: poll before the operand fetch, not before the third cycle of a taken branch
- the interrupt sequence: no polling within it
- `BRK` hijacking: NMI asserted during the first four cycles changes the vector

Expected to fix `2-nmi_and_brk`, `3-nmi_and_irq` and `5-branch_delays_irq`.

## How it will be judged

`cpu_interrupts_v2` (0/6), `instr_timing` (1/3), `branch_timing_tests` (0/3), `cpu_dummy_writes`
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
