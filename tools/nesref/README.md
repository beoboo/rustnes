# nesref — a second opinion

Runs a test ROM in [tetanes](https://github.com/lukexor/tetanes) and prints what the ROM says,
so a failure here can be checked against an emulator that is known to pass a great deal.

It answers one question, and it is the question this project keeps needing:

> Is this ROM failing because we have a bug, or because it is testing something obscure that
> nothing gets right?

A ROM tetanes passes is a bug of ours **with a readable reference beside it**. One it also fails
is worth deprioritising. Guessing which is which had previously cost several sittings.

## Building it

Deliberately **not** a workspace member. It depends on a tetanes checkout by relative path, which
most people will not have, and `cargo build` at the repository root must not break for them.

```sh
git clone https://github.com/lukexor/tetanes ../../tetanes   # a sibling of this repository
cargo run --manifest-path tools/nesref/Cargo.toml -- <rom.nes> [frames]
```

tetanes pins a nightly toolchain in its own `rust-toolchain.toml`; cargo will fetch it.

## What it reads

Blargg's later ROMs write a status byte to `$6000`, a signature at `$6001`, and a message at
`$6004`. This reads them straight off tetanes' bus after running the ROM for a while, and prints
the same verdict our own runner would. ROMs that predate that protocol report `NO PROTOCOL`; for
those, `rom_test screen` reads the screen instead and there is nothing here to compare against yet.

## The DMC cycle ledger

`tetanes-dmc-probe.patch` adds four `eprintln!`s to tetanes' `cpu.rs` that stamp every `$4015`
write, DMC fetch request, halt, fetch and OAM-DMA start/end with a CPU-cycle count, in the same
format `RN_DMC_TRACE=1` makes this emulator print. Diffing the two ledgers at the same ROM is how
`dma_4016_read` and both `sprdma_and_dmc_dma` ROMs were fixed: each time, the two agreed on
everything that had been suspected for weeks and differed on exactly one number.

```sh
cd ../../tetanes && git apply ../rustnes4/tools/nesref/tetanes-dmc-probe.patch
RN_DMC_TRACE=1 cargo run --manifest-path tools/nesref/Cargo.toml -- <rom.nes> 600 2> tet.log
RN_DMC_TRACE=1 ./target/release/rom_test run <rom.nes> 2> ours.log
```

Kept as a patch rather than left uncommitted in the tetanes checkout, where a `git pull` would
silently discard it — and rebuilding it from memory cost most of a sitting the first time.

## What it found the first time it was run

Five ROMs this emulator fails and tetanes passes — so five bugs of ours, not five obscurities:

```
3-nmi_and_irq            PASS
4-irq_and_dma            PASS
5-branch_delays_irq      PASS
cpu_interrupts           PASS
test_cpu_exec_space_apu  PASS
```
