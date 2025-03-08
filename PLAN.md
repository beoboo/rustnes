# RustNES Emulator Project Plan 📝

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

```
rustnes/
├── src/
│   ├── cpu/         # 6502 CPU emulation
│   ├── ppu/         # Picture Processing Unit
│   ├── apu/         # Audio Processing Unit [T4]
│   ├── mappers/     # Memory mapper implementations
│   ├── input/       # Controller input handling [T3]
│   ├── memory/      # Memory management
│   ├── debugger/    # Debugging tools [T4]
│   ├── ui/          # User interface (desktop) [T3-T4]
│   ├── web/         # WebAssembly-specific code [T4]
│   └── main.rs      # Application entry point
├── tests/           # Integration tests
├── benches/         # Performance benchmarks [T4]
└── examples/        # Example usage
```

## Testing Strategy 🧪

1. **Unit Tests**
   - Test each component in isolation
   - Verify against known behavior
   - Test edge cases and special conditions

2. **Integration Tests**
   - Test component interactions
   - Verify system-wide behaviors
   - ROM test suites (nestest, etc.)

3. **Track Milestone Tests**
   - Specific test ROMs for each track milestone
   - Verification of track completion requirements
   - Self-assessment checkpoints

4. **Acceptance Tests**
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

The project will use minimal dependencies, with a focus on:

- Graphics libraries (TBD: SDL2, pixels, wgpu)
- Audio libraries (TBD: cpal, rodio)
- WebAssembly support (wasm-bindgen, web-sys)
- Testing frameworks (standard Rust testing + criterion for benchmarks)

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