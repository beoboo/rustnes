# Ideas, someday-maybe

Things that have been written down and are **not planned work**. They were mixed into TODO.md's
checkboxes, where roughly 240 unticked boxes made the project look 57% finished when the emulator
had in fact passed every conformance suite that has a verdict to give.

None of this is needed by the book, which is the project's deliverable. Kept because some of it is
genuinely good, and deleting ideas is a different decision from not doing them now.

## MILESTONE 11: Web Integration [T11]
- [ ] Create a web-based version of the NES emulator
- [ ] Implement WebAssembly (WASM) support
- [ ] Create browser-based UI
- [ ] Support game loading in browser
- [ ] Test cross-browser compatibility

### [Web] WebAssembly Setup [T10]
- [ ] Set up wasm-bindgen
- [ ] Create WebAssembly build target
- [ ] Configure Rust for WASM compilation
- [ ] Create basic web shell for testing
- [ ] Test core functionality in browser

### [Web] Graphics Adaptation [T10]
- [ ] Adapt rendering pipeline for web canvas
- [ ] Implement WebGL support (if needed)
- [ ] Optimize frame rendering for browser
- [ ] Support scaling and display options
- [ ] Test rendering performance

### [Web] Input Handling [T10]
- [ ] Implement browser keyboard input
- [ ] Add gamepad API support
- [ ] Create touch controls for mobile
- [ ] Implement input configuration in browser
- [ ] Test input responsiveness

### [Web] Audio Integration [T10]
- [ ] Adapt audio output for Web Audio API
- [ ] Implement audio buffering for web
- [ ] Add volume control
- [ ] Test audio synchronization
- [ ] Optimize audio performance

### [Web] Storage & State [T10]
- [ ] Implement save states using browser storage
- [ ] Support ROM loading via browser
- [ ] Add local storage for game progress
- [ ] Implement preferences saving
- [ ] Test data persistence

### [Web] UI & Experience [T10]
- [ ] Create responsive web UI
- [ ] Implement mobile-friendly controls
- [ ] Add game library browser
- [ ] Support fullscreen mode
- [ ] Create shareable game links (if applicable)
- [ ] Optimize for various device sizes

### [Web] Performance Optimization [T10]
- [ ] Profile and optimize WASM performance
- [ ] Implement targeted performance improvements
- [ ] Add performance settings for different devices
- [ ] Test on low-powered devices
- [ ] Document optimization strategies

## Additional Important Areas (To Be Defined Better Later)

### Audio Quality & Performance
- [ ] Test audio synchronization with game state
  - [ ] Verify sound effects play at correct game events
  - [ ] Ensure music transitions are synchronized with game state
  - [ ] Test timing accuracy of audio events
  - [ ] Verify proper audio timing during frame drops
- [ ] Support configurable audio quality settings
  - [ ] Add master volume control
  - [ ] Implement individual channel volume controls
  - [ ] Add basic audio filter options
  - [ ] Support sample rate configuration
  - [ ] Allow buffer size adjustment
- [ ] Refine audio output system
  - [ ] Optimize audio buffer management to prevent underruns
  - [ ] Implement proper sample interpolation and rate conversion
  - [ ] Add anti-aliasing and DC offset removal
  - [ ] Handle audio device changes and errors gracefully
- [ ] Implement proper sample rate conversion
- [ ] Optimize audio performance
- [ ] Enhance audio visualization widget for debugging
  - [ ] Add real-time waveform display with zoom and time-scaling
  - [ ] Show channel-specific parameters and states
  - [ ] Implement channel mute/solo controls
  - [ ] Add diagnostic information display
  - [ ] Support freezing display for inspection
  - [ ] Add buffer underrun/overflow indicators
- [ ] Add audio buffer visualization to display recent sample data
- [ ] Add audio spectrum analyzer for frequency visualization
- [ ] Implement basic oscilloscope view for waveform visualization
- [ ] Implement real-time audio visualization with proper scaling
- [ ] Create visual indicators for audio channel activity
- [ ] Add audio response visualization for different frequency ranges
- [ ] Implement audio export to WAV file for offline analysis
- [ ] Add support for visualizing individual channels separately

### Cycle-Accurate Timing
- [ ] Implement precise cycle counting between components
- [ ] Synchronize CPU/PPU/APU at exact cycle level
- [ ] Add timing validation with test ROMs
- [ ] Handle special timing edge cases for specific games
- [ ] Implement accurate PPU scanline timing (341 cycles per scanline)
- [ ] Implement accurate frame timing (261 scanlines per frame)
- [ ] Support cycle-accurate interrupt timing
- [ ] Implement cycle penalties for memory access across page boundaries
- [ ] Support cycle-accurate DMA timing
- [ ] Implement accurate timing for CPU addressing modes
- [ ] Add cycle-accurate test ROMs to verify emulation accuracy

### Background Rendering
- [ ] Implement complete nametable handling (4 nametables)
- [ ] Support mirroring modes (horizontal, vertical, single-screen, four-screen)
- [ ] Implement tile rendering with proper pixel priority
- [ ] Implement background scrolling with proper wrap-around
- [ ] Add fine X and Y scrolling
- [ ] Support split-screen scrolling (common for status bars in games)
- [ ] Implement mid-frame updates to scroll registers
- [ ] Support attribute table for background color selection
- [ ] Handle glitchy rendering behavior at scroll boundaries
- [ ] Support proper background palette selection

### Testing Infrastructure
- [ ] Create automated test suite for all components
- [ ] Implement integration tests with real-world ROMs
- [ ] Add benchmark suite for performance comparisons
- [x] Set up continuous integration pipeline — `.github/workflows/ci.yml`, running against
      github.com/beoboo/rustnes since 2026-08-04: build, clippy, and the test suite twice over so
      an order-dependent failure has a chance to show.
- [ ] Add regression testing framework
- [ ] Implement test coverage reporting
- [ ] Create visual verification tests for rendering accuracy
- [ ] Add automated audio output verification tests
- [ ] Support for standard NES test ROMs
- [ ] Add performance testing for optimization verification

### Special Hardware Edge Cases
- [ ] Identify and implement known NES hardware quirks
- [ ] Support undocumented CPU instructions properly
- [ ] Handle games that rely on hardware glitches
- [ ] Test against diagnostic ROMs that verify edge cases
- [x] Implement open bus behavior. Reads of an address nothing drives — `$4018-$401F`,
      `$4020-$5FFF`, `$6000-$7FFF` without cartridge RAM — return the last value the data bus
      carried rather than failing. The bus used to refuse them outright, deliberately, so that a
      hole in the memory map would be loud; but programs read unmapped addresses *on purpose*
      (every indexed addressing mode does a dummy read at an unfixed address), so refusing them
      stopped correct programs dead. The count of fall-throughs is kept for visibility instead.
- [ ] Support PPU race conditions and edge cases
- [ ] Implement accurate sprite overflow handling
- [ ] Support illegal opcodes with correct timing
- [ ] Handle CPU/PPU interaction edge cases
- [ ] Support PAL vs NTSC timing differences
- [ ] Implement accurate behavior for uninitialized memory
- [ ] Support APU hardware quirks
  - [ ] Implement DMC channel CPU slowdown
  - [ ] Add length counter edge cases and reload behaviors
  - [ ] Support frame counter mode-specific timing differences
  - [ ] Implement channel interaction quirks
  - [ ] Add register write timing-dependent behaviors

### Distribution & Packaging
- [ ] Create distributable packages for different platforms
- [ ] Implement release management process
- [ ] Develop update/installation mechanisms
- [ ] Add platform-specific optimizations
- [ ] Create macOS application bundle
- [ ] Create Windows installer package
- [ ] Create Linux distribution packages
- [ ] Support automated build process for all platforms
- [ ] Implement code signing for distribution
- [ ] Create user-friendly installation guides

### Extended Features
- [ ] Add support for game-specific configurations
- [ ] Implement cheat system/Game Genie support
- [ ] Add ROM patching capabilities
- [ ] Support recording and playback of gameplay sessions
- [ ] Implement frame advance for precision gameplay
- [ ] Add screenshot and video recording
- [ ] Support netplay for multiplayer games
- [ ] Implement turbo button functionality
- [ ] Add high score saving
- [ ] Support for enhanced visualization modes
- [ ] Implement debugging overlays for development

### Legal Considerations
- [ ] Develop proper ROM copyright handling
- [ ] Ensure attribution for any third-party components
- [ ] Implement license compliance checks
- [ ] Create appropriate disclaimers and notices
- [ ] Develop privacy policy for any data collection
- [ ] Ensure FOSS license compliance
- [ ] Document any patent considerations
- [ ] Create clear contributor guidelines
- [ ] Ensure proper handling of game assets
- [ ] Implement proper attribution system

### Documentation
- [ ] Write comprehensive user documentation
- [ ] Create developer documentation for extending the emulator
- [ ] Generate API documentation for integrations
- [ ] Document compatibility status of commercial games
- [ ] Create tutorials for common use cases
- [ ] Develop README with quickstart guide
- [ ] Write architecture documentation
- [ ] Create contribution guidelines
- [ ] Document build process for all platforms
- [ ] Add examples for API usage
- [ ] Create diagrams for system architecture

### Performance Optimization
- [ ] Profile CPU-intensive operations
- [ ] Implement display optimization for modern GPUs
- [ ] Add configurable frameskip for slower systems
- [ ] Optimize memory access patterns
- [ ] Implement fast-forward functionality
- [ ] Reduce allocation during emulation
- [ ] Optimize bus communication
- [ ] Add rendering pipeline optimization
- [ ] Implement audio buffer optimization
- [ ] Add JIT compilation for CPU emulation
- [ ] Support threaded PPU/APU processing where beneficial

### Audio Visualization
- [ ] Create audio buffer visualization to display recent sample data
- [ ] Add audio spectrum analyzer for frequency visualization
- [ ] Implement basic oscilloscope view for waveform visualization
- [ ] Implement real-time audio visualization with proper scaling
- [ ] Create visual indicators for audio channel activity
- [ ] Add audio response visualization for different frequency ranges
- [ ] Implement audio export to WAV file for offline analysis
- [ ] Add support for visualizing individual channels separately

### Demo ROMs and Example Code
- [ ] Create beginner's guide to NES assembly programming
- [ ] Develop "Hello World" example with detailed comments
- [ ] Create sprite movement demo with controller input
- [ ] Develop audio demonstration ROM with all channels
- [ ] Create background scrolling demonstration ROM
- [ ] Implement split-screen demo for status bars
- [ ] Develop tile animation example ROM
- [ ] Create palette cycling example
- [ ] Implement collision detection example
- [ ] Create game-like demo combining multiple techniques
- [ ] Develop performance test ROM for benchmarking
- [ ] Add comprehensive comments to all example code
- [ ] Create tutorial videos explaining demo code
- [ ] Document assembly techniques for efficient NES programming 
