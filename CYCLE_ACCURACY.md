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

With the counts equal, sampling before the last cycle is a position that exists. The latch already
written in attempt 3 works unchanged once `poll_at` refers to a real cycle — it is preserved in
`scratchpad/cycle-sampling/`.

Expected to fix `1-cli_latency` and `instr_timing`, whose cycle counts stop being a table and start
emerging from the work done.

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
