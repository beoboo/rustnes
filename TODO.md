# TODO

This file is the *live* list and nothing else. Two things used to live here and have moved, because
mixing them made both useless:

- **[docs/research-log.md](docs/research-log.md)** — how each accuracy bug was found and fixed.
  Finished work, kept because the diagnosis is the valuable part and the code only shows the answer.
- **[IDEAS.md](IDEAS.md)** — someday-maybe. Roughly 240 unticked boxes of netplay, WebAssembly,
  shaders and audio visualisation, none of it planned, all of it making the project read as 57%
  finished when every conformance suite that has a verdict already passes.

**Numbers here are claims until re-run.** That warning is kept because it keeps being needed: nine
boxes claiming the CPU could not branch or push survived months of being read past, in a file whose
own header said exactly this. Everything below was measured on 2026-08-07.

## Where the emulator actually stands

Every suite that has a verdict to give, and every remaining failure understood:

```
nestest 8991/8991     instr_test-v5 18/18   instr_misc 5/5        instr_timing 3/3
nes_instr_test 11/11  branch_timing 3/3     cpu_interrupts_v2 6/6 cpu_reset 2/2
cpu_dummy_reads 1/1   cpu_dummy_writes 2/2  cpu_exec_space 2/2    cpu_timing_test6 1/1
ppu_vbl_nmi 11/11     vbl_nmi_timing 7/7    sprite_hit 11/11      sprite_overflow 5/5
ppu_open_bus 1/1      ppu_read_buffer 1/1   oam_read 1/1          oam_stress 1/1
apu_test 9/9          apu_reset 6/6         blargg_apu 11/11      apu_mixer 4/4
sprdma_and_dmc_dma 2/2                      blargg_nes_cpu_test5 2/2
```

531 unit tests, clippy clean, three frame baselines matching, CI green. NTSC and PAL.

## Open — real work, most tractable first

- [ ] **`full_palette` and `litewall5`, one hypothesis away.** The residual is six columns — x=1,2,3
      and 252-254, the line edges — where our palette-hack colour changes a dot earlier than the
      reference's. A blanket one-dot lag was tried and refuted: it makes `litewall5` identical and
      takes `full_palette` from 0.54% to 7.87% edge disagreement. So the lag is real for one source
      of a `v` change and not another. `full_palette` steps `v` with `$2007` writes where the
      litewall demos set it with `$2006` — **separate and measure those two paths** rather than
      delaying everything at once, which is what the refuted attempt did.

- [ ] **The ~25 visual demo ROMs, half triaged.** Confirmed matching: `240pee`, `dpcmletterbox`,
      `nes15`, `nrom368/test1`, `stars_se`, `tvpassfail`, two of five `litewall`. Still unjudged and
      mostly *animated*, so a frame-exact diff at frame 600 cannot judge them: `scanline`,
      `scrolltest`, `spritecans`, `stomper`, `tutor`, `stress`, `window5`. These need a
      phase-tolerant comparison before they mean anything at all.

- [ ] **`double_2007_read` — blocked on a reference, with evidence rather than assumption.** It and
      `dma_2007_read` present identical conditions (consecutive CPU cycles, three dots apart,
      rendering off) and need opposite outcomes, so every hypothesis is refuted by the other ROM.
      The power-on alignment cannot matter until the `$2007` fetch has a *duration*: `read_data` is
      atomic here, so there is no interval for a phase to fall inside. Needs an emulator that
      passes it, or hardware. Not another guess.

- [ ] **Audio has never been judged against real game music.** The pipeline works and `apu_mixer`
      passes 4/4, but nobody has listened to a game and compared it against anything. There is no
      reference-audio harness, which is the actual gap.

- [ ] **Controller 2 is wired in the core and not surfaced in the key mapping.**

- [ ] Machine-readable output from `rom_test`, suitable for gating CI on a suite rather than on the
      unit tests alone.

- [ ] Drop the `objc2` `relax-sign-encoding` workaround in the root Cargo.toml once eframe/winit no
      longer need it.

## Deliberately not done

Recorded so they stop being rediscovered as bugs:

- **MMC5, VRC2/4 (mapper 22), NROM-368, BNROM.** Unimplemented, so `exram`, `mmc5test`,
  `m22chrbankingtest`, `nrom368/fail368` and `240pee-bnrom` cannot run. Nothing needs them;
  NROM, UxROM, CNROM, MMC1, MMC3 and AxROM are all implemented.
- **The paddle controller**, so `PaddleTest3` and `vaus-test` cannot run.
- **MMC6** (`mmc3_test`/`mmc3_test_2` 5/6) and **MMC3 revision A** (`mmc3_irq_tests` 5/6) are
  different chips, not faults in the MMC3 that is here.
- **`power_up_palette`** is machine-specific by its own readme.
- **`dmc_tests` 0/4** report by sound alone, and render a picture structurally identical to the
  reference's. The nes-test-roms repository's own `status.txt` marks all four `???? Not sure yet`.
- **`read_joy3/test_buttons`** wants a human to press buttons. **`count_errors`** and
  `count_errors_fast` *count* rather than judge, and the reference emulators log conflicts too.
