# RustNES Emulator Project Plan 📝

> **Where things stand.** Tracks 1–7 are done: CPU (46 of 56 official instructions), memory bus,
> PPU rendering, sprites, controllers, and a full APU that produces correct audio (see
> [AUDIO_PLAN.md](AUDIO_PLAN.md)). The debugger runs and ten of the `asm/` demos play sound.
>
> **The next milestone is conformance, not features.** The emulator has never been checked against
> anything it cannot argue with. It cannot yet load a `.nes` file at all — the loader parses the
> header and discards the program — so no standard test ROM has ever run. That, the ten missing
> instructions, and interrupts are the gate; see **[CONFORMANCE_PLAN.md](CONFORMANCE_PLAN.md)**.

## Project Overview 🎮

RustNES is a Nintendo Entertainment System (NES) emulator written in Rust, following a test-driven development approach. The project aims to:

- Create a full-featured, performant NES emulator
- Support all NES games, including those using special/hidden features
- Run on both desktop and web platforms (via WebAssembly)
- Include comprehensive debugging capabilities
- Show results incrementally throughout development

## Development Philosophy 🧠

- **Test-First Approach**: All features will be implemented with tests first
- **Incremental Development**: Following approaches similar to "Crafting Interpreters" and "The Ray Tracer Challenge"
- **Readability Over Performance**: While performance is important, code readability takes precedence
- **Clean Code Principles**: Concise methods, idiomatic Rust, easily testable code, solid error management

## Multi-Track Learning Approach 🛤️

This project supports multiple learning tracks, allowing readers to progress at different paces based on their interests:

### Track 1: Pixel Display [T1]
Focus on implementing just enough to display a single pixel:
- Basic CPU with essential instructions
- Simple memory mapping
- Fundamental PPU registers
- NROM (Mapper 0)
- Basic rendering pipeline

**Milestone:** Display a colored pixel using a test ROM

### Track 2: Pattern & Sprite Rendering [T2]
Build on Track 1 by adding:
- Extended CPU instructions
- Tile and pattern rendering
- Sprite capabilities
- Animation support
- More complete PPU implementation

**Milestone:** Display and animate sprites on screen

### Track 3: Interactive Graphics [T3]
Expand functionality with:
- Complete CPU instruction set
- Full background rendering with scrolling
- Controller input
- Game-like demo capabilities
- Additional mappers

**Milestone:** Create an interactive demo with controller input

### Track 4: Complete NES [T4]
Finalize the emulator with:
- Audio Processing Unit
- All memory mappers
- Cycle-accurate timing
- Full game compatibility
- Debugging tools
- Performance optimizations

**Milestone:** Play commercial NES games with full compatibility

## Implementation Phases 📊

### Phase 1: Core Architecture [T1]
- 6502 CPU implementation (essential instructions)
- Memory management (basic mapping)
- Address decoding (fundamental)
- NROM mapper support
- PPU registers and minimal rendering

### Phase 2: Basic Rendering [T1-T2]
- PPU implementation (tile and sprite rendering)
- Pattern tables
- Background/playfield basics
- Palette handling
- Simple animation capabilities

### Phase 3: Enhanced Graphics [T2-T3]
- Full background rendering
- Sprite collision
- Scrolling implementation
- Name table handling
- Advanced PPU features

### Phase 4: Input & Interaction [T3]
- Controller emulation
- Input mapping
- UI for configuration
- Interactive demo capabilities

### Phase 5: Audio & Advanced Features [T4]
- APU implementation
- Sound channels
- Mixer and output
- Additional mappers
- Cycle-accurate timing

### Phase 6: Debugging & Polish [T4]
- Debugging tools
- Performance optimizations
- Full game compatibility
- Enhanced features

### Phase 7: Documentation & Release [T4]
- Complete documentation
- Book finalization
- Community feedback

## Component Architecture 🧩

The project is a Cargo workspace. `rn_core` depends on no host graphics or audio library, which
is what keeps it testable headlessly and portable to WebAssembly later; the outer crates implement
narrow traits (`Addressable` for the bus, `SampleProducer`/`SampleConsumer` for audio).

```
crates/
├── rn_core/      cpu, ppu, apu, memory, cartridge, dma, input, system bus
├── rn_audio/     host audio: cpal output, ring buffer, multiplexer, oscillator
├── rn_input/     controller profiles and key mapping
└── rn_ui/        egui widgets, one per subsystem
tools/
├── apu_probe/    headless audio harness — run a program, measure what the APU produced
├── nes_asm/      command-line 6502 assembler
├── nes_debugger/ the main application: dockable debugger workspace
└── waveform_player/  oscillator playground, no emulation
asm/              6502 test programs
```

Still to be created: a mapper layer (currently the iNES mapper field is parsed and ignored), a
headless test-ROM runner (see [CONFORMANCE_PLAN.md](CONFORMANCE_PLAN.md)), and the WebAssembly
target.

## Testing Strategy 🧪

1. **Unit Tests**
   - Test each component in isolation
   - Verify against known behavior
   - Test edge cases and special conditions

2. **Integration Tests**
   - Test component interactions
   - Verify system-wide behaviours
   - Assert on measurable properties rather than exact bytes where output is a signal: the audio
     tests in `crates/rn_core/tests/audio_pipeline.rs` check sample rate, dominant frequency, DC
     offset and peak level, which is what caught defects the per-channel unit tests could not see

3. **Conformance Tests**
   - The community's NES test ROMs — `nestest`, blargg's CPU/PPU/APU suites — run headlessly
   - These are the independent check on everything above; see
     [CONFORMANCE_PLAN.md](CONFORMANCE_PLAN.md) for what has to exist first and in what order
   - ROMs cannot be committed here, so the suite skips cleanly when they are absent

4. **Track Milestone Tests**
   - Specific test ROMs for each track milestone
   - Verification of track completion requirements
   - Self-assessment checkpoints

5. **Acceptance Tests**
   - End-to-end emulator testing
   - Known game compatibility
   - Performance benchmarks

## Milestones & Deliverables 📅

1. **M1: Display a Pixel [T1]** - Single pixel rendering with minimal components
2. **M2: Pattern & Sprite Display [T2]** - Static and animated graphics
3. **M3: Interactive Demo [T3]** - User-controlled sprites and scrolling
4. **M4: Basic Game Support** - Simple games working (e.g., Donkey Kong)
5. **M5: Audio Support [T4]** - Sound channels implemented
6. **M6: Advanced Mapper Support [T4]** - Broader game compatibility
7. **M7: Desktop UI [T4]** - Full desktop application
8. **M8: Web Support [T4]** - Browser-based emulation
9. **M9: Debugging Tools [T4]** - Complete debugging capabilities
10. **M10: Full Release [T4]** - Polished, documented release

## Book Structure

The book will follow the multi-track approach with chapters organized to support both fast-track and comprehensive learning:

1. Each chapter will have sections clearly marked with track indicators [T1-T4]
2. Track milestone points will be highlighted with:
   - Self-assessment questions
   - Component checklists
   - Skip-ahead guidance for fast-track readers
3. Track convergence points will provide catch-up guidance for readers who took the fast track

## Dependencies 📦

Minimal dependencies, chosen and now settled:

- **Graphics/UI**: `eframe` + `egui` + `egui_dock` (the debugger's dockable workspace)
- **Audio**: `cpal` for output, `ringbuf` for the lock-free queue, `crossbeam-channel` for taps
- **CLI**: `clap`
- **Testing**: standard Rust testing; `criterion` for benchmarks when they arrive
- **WebAssembly**: `wasm-bindgen`/`web-sys` — not yet started

Warnings are denied workspace-wide (`[workspace.lints]`), so a warning fails the build. The dev
profile is optimized (`opt-level = 2`, dependencies at 3): an unoptimized build runs the emulator
at only ~1.4x real time, which is not enough to keep the audio buffer fed once the UI is also
rendering.

## Future Enhancements 🔮

1. Enhanced video output (CRT simulation, scanlines)
2. Savestate support
3. Peripheral emulation (Zapper, Power Pad, etc.)
4. Multiplayer over network
5. ROM hacking and modification tools

## Success Criteria ✅

1. **Technical Success:**
   - All commercial NES games run correctly
   - Performance exceeds real-time requirements (60fps) on modest hardware
   - Passes standard test ROMs
   - Usable debugging interface
   - Code is well-documented and maintainable
   - Web version runs smoothly in modern browsers

2. **Educational Success:**
   - Readers can follow either learning track successfully
   - Each track milestone provides satisfying results
   - Code remains professional and well-architected despite track divisions
   - Readers gain deep understanding of NES architecture
   - Book provides clear, engaging explanations of emulation concepts 