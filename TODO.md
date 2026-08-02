# RustNES Implementation Checklist 📋

This document provides a detailed task breakdown for developing the RustNES emulator alongside writing the book, organized by learning tracks.

## Track System Legend
- [T1] Track 1: Memory Visualization - Display memory contents as pixels in an egui widget
- [T2] Track 2: PPU Pixel Display - Using the PPU to show pixels 
- [T3] Track 3: Basic Sprite Rendering - Basic sprite rendering
- [T4] Track 4: Animated Sprites - Add animation capabilities with multi-tile sprites
- [T5] Track 5: Input Controllers - Implement controller input for user interaction
- [T6] Track 6: Basic Sound Output - Implementing fundamental APU functionality for simple tones
- [T7] Track 7: Complete Audio System - Adding full music and sound effects capabilities
- [T8] Track 8: Conformance & Test ROMs - Validating against the community's NES test suites
- [T8] Track 8: Mappers & Cartridges - Support for different ROM mappers
- [T9] Track 9: Full Desktop System - Complete playable NES emulator application
- [T10] Track 10: Web Integration - WebAssembly support for browser play

## MILESTONE 1: Memory Visualization [T1]
- [x] Create a memory visualization component for egui
- [x] Map memory range 0x0200-0x05FF to pixels in the visualization
- [x] Implement color mapping for memory values
- [x] Create a test program that sets specific memory values
- [x] Display the memory contents as pixels
- [x] Implement real-time updates as memory changes
- [x] Document the implementation

### [Meta] Project Setup [T1]
- [x] Initialize Rust project with Cargo
- [x] Set up module structure
- [x] Create README.md
- [x] Set up GitHub repository
- [x] Create initial documentation structure
- [x] Set up book framework with mdBook
- [ ] Configure CI/CD pipeline [T9]
- [ ] Add benchmark infrastructure [T9]
- [ ] Set up WebAssembly build infrastructure [T10]

### [Meta] NES Architecture Overview [T1]
- [x] Document NES hardware components
- [x] Create diagrams of system architecture
- [x] Document memory map
- [x] Document system bus
- [x] Research hardware specifications
- [x] Collect reference materials
- [x] Document hardware limitations and quirks

### [CPU] CPU Basics [T1]
- [x] Implement basic CPU struct with registers
- [x] Implement CPU flags enum and methods
- [x] Create Addressable trait
- [x] Implement basic RAM struct
- [x] Write tests for CPU flag operations
- [x] Write tests for memory operations
- [x] Document power-up state
- [x] Implement connection between CPU and memory

### [CPU] Essential Addressing Modes [T1]
- [x] Define addressing mode enum
- [x] Implement immediate addressing
- [x] Implement absolute addressing
- [x] Implement zero page addressing
- [x] Write tests for essential addressing modes

### [CPU] Essential Instructions [T1]
- [x] Create instruction decoder
- [x] Implement load instructions (LDA, LDX, LDY)
- [x] Implement store instructions (STA, STX, STY)
- [x] Implement basic jumps (JMP)
- [x] Implement subroutine calls (JSR, RTS)
- [x] Implement BRK instruction for interrupt handling
- [x] Write tests for essential instructions

### [Assembler] Basic Assembler Framework [T1]
- [x] Create instruction parser module with error handling
- [x] Implement parsing for basic addressing modes (immediate, zero page, absolute)
- [x] Implement parsing for essential load instructions (LDA, LDX, LDY)
- [x] Implement parsing for essential store instructions (STA, STX, STY)
- [x] Implement parsing for basic jumps and subroutines (JMP, JSR, RTS) (JMP implemented)
- [x] Implement parsing for BRK instruction
- [x] Add parser tests for essential instructions

### [Disassembler] Basic Disassembler [T1]
- [x] Create basic disassembler module
- [x] Implement disassembling of essential instructions
- [x] Support disassembling of all implemented addressing modes
- [x] Add disassembler tests

### [Memory] Component-Based Bus Architecture [T1]
- [x] Refactor `Ram` to its own module for better code organization
- [x] Implement `Bus` struct as mediator for memory-mapped devices
- [x] Rename `Memory` trait to `Addressable` for better semantics
- [x] Refactor `Ram` to work with the new bus architecture
- [x] Create memory address range utilities for device registration
- [x] Test bus routing and device registration

### [Debugger] AsmDebugger Tool [T1]
- [x] Set up basic egui application structure
- [x] Implement memory visualization (0x0200-0x05FF as pixels)
- [x] Add assembly code editor with basic controls
- [x] Connect assembly editor to parser and display assembled code
- [x] Implement basic CPU state display
- [x] Implement program loading and execution
- [x] Move widgets to rn_ui for reuse
- [x] Add disassembly view for code
- [x] Implement step-by-step execution
- [x] Add BRK instruction support for program termination
- [x] Implement register modification capabilities
- [x] Connect existing memory editor widget to debugger

## MILESTONE 2: System Integration [T2]
- [x] Implement `NesSystem` struct to coordinate components
- [x] Connect CPU, Memory Bus, and PPU through clean interfaces
- [x] Design proper ownership model with minimal unsafe code
- [x] Extend AsmDebugger to support both memory and PPU display

### [System] Component Timing System [T2]
- [x] Implement system-level timing controller for component synchronization
- [x] Ensure PPU ticks at 3x the CPU rate
- [x] Improve memory access error handling to fail visibly on invalid accesses
- [x] Refactor AsmDebugger to use the NesSystem class for timing control
- [x] Implement NOP instruction to support timing-related tests
- [x] Test correct timing ratios between components

### [Memory] Essential Memory Components [T2]
- [x] Limit RAM to only handle the main memory region ($0000-$1FFF)
- [x] Make RAM configurable with custom address ranges
- [x] Implement PPU component with registers at $2000-$2007
- [x] Test basic memory component interactions

### [PPU] Essential PPU I/O [T2]
- [x] Define `PpuRegister` enum for type-safe register access
- [x] Implement PPU register reading with proper side effects
- [x] Implement PPU register writing with proper side effects
- [x] Test PPU register access through the bus
- [x] Implement `PpuRegisters` adapter for memory-mapped access
- [x] Create a common `PixelDataProvider` trait for memory and PPU data sources
- [x] Extend the `MemoryVisualizer` to work with the `PixelDataProvider` trait

### [PPU] PPU Architecture [T2]
- [x] Design PPU component with clear internal interfaces
- [x] Implement internal register access methods
- [x] Create PPU state struct with proper encapsulation
- [x] Implement interior mutability pattern for bus access
- [x] Test PPU internal functionality in isolation

### [PPU] PPU Basics [T2]
⚠️ NOTE: Temporary hardcoded pattern data implemented for pixel rendering without ROM
- [x] Implement PPU registers
- [x] Implement PPU memory map
- [x] Implement basic VRAM
- [x] Implement color palette
- [x] Implement basic frame buffer rendering logic
- [x] Implement PPU display widget for the frame buffer

### System integration [T2]
- [x] Create a test ROM that sets a single pixel using the PPU
- [x] Integrate CPU, Memory, and PPU components
- [x] Implement basic main loop with proper timing
- [x] Add display mode switching in the debugger UI

### [Assembler] Label Support [T2]
- [x] Add label declaration support in the parser
- [x] Implement label resolution for jump instructions
- [x] Add tests for label parsing and resolution
- [x] Update the AsmDebugger to properly handle infinite loops

## MILESTONE 3: Basic Sprite Rendering [T3]
- [x] Create a test ROM that displays a single sprite
- [x] Implement basic sprite rendering functionality
- [x] Implement BIT instruction for PPU status checking
- [x] Implement BPL branch instruction for PPU loop control 
- [x] Write tests for these PPU-specific instructions
- [x] Create examples that use BIT/BPL for PPU synchronization
- [x] Test sprite rendering with a single sprite
- [x] Support basic NES ROM assembly with essential directives
- [x] Document the achievement

### [CPU] Extended Addressing Modes [T3] 
- [x] Implement zero page,X addressing
- [x] Implement zero page,Y addressing
- [x] Implement absolute,X addressing
- [x] Implement absolute,Y addressing
- [x] Write tests for extended addressing modes

### [CPU] Essential PPU Instructions [T3]
- [x] Implement BIT instruction for PPU status checking
- [x] Implement BPL branch instruction for PPU loop control 
- [x] Write tests for these PPU-specific instructions
- [x] Create examples that use BIT/BPL for PPU synchronization

### [PPU] Pattern Tables [T3]
- [x] Implement pattern table memory access for ROM data
- [x] Implement pattern table bit plane handling (2 planes → pixel data)
- [x] Implement a pattern table widget for visualizing tile data
- [x] Implement basic palette mapping for sprite pixels
- [x] Test pattern rendering

### [PPU] Basic Sprites [T3]
- [x] Implement OAM (Object Attribute Memory) at $0200-$02FF
- [x] Track and process writes to OAM memory
- [x] Implement basic sprite evaluation (position, tile number, attributes)
- [x] Implement single sprite rendering pipeline
- [x] Test basic sprite rendering with simple pattern

### [Cartridge] Basic ROM Loading [T3]
- [x] Implement simplified iNES file format parser
- [x] Extract CHR ROM data from test ROM
- [x] Make CHR ROM data accessible to pattern tables
- [x] Test ROM loading with sprite pattern

### [Assembler] Assembler Directives Support [T3]
- [x] Implement `.segment` directive for defining basic sections (HEADER, STARTUP, VECTORS, CHARS)
- [x] Implement `.byte` directive for defining byte arrays
- [x] Implement `.word` directive for defining word values
- [x] Implement `.res` directive for reserving space
- [x] Add support for multiple segments with different load addresses
- [x] Implement basic NES ROM structure generation with header and layout
- [x] Add tests for essential directives
- [x] Update AsmDebugger to support simple NES ROM assembly

### [System] Component Testing & Debugging [T3]
- [x] Add unit test for NesSystem component connections
  - [x] Verify PPU has valid cartridge reference 
  - [x] Verify DMA controller has valid CPU and PPU references
  - [x] Verify Bus contains all expected components
- [x] Add trace test for sprite rendering pipeline
  - [x] CPU writes to OAM memory
  - [x] DMA transfers data to PPU OAM
  - [x] PPU renders sprite from pattern table
- [x] Create focused PPU sprite rendering tests
  - [x] Direct OAM manipulation test (bypassing DMA)
  - [x] Pattern table loading and access test
  - [x] Palette mapping test for sprite pixels
- [x] Add test for PPU sprite attribute handling
  - [x] Test sprite flipping (horizontal/vertical)
  - [x] Test palette selection
  - [x] Test sprite priority
- [x] Create cartridge connection validation test
  - [x] Test loading minimal ROM with sprite pattern
  - [x] Verify PPU can access pattern data
  - [x] Confirm CHR ROM properly mapped to PPU space
- [x] Implement OAM content visualization for debugging
- [x] Create simplified "hello world" sprite test
  - [x] Single 8x8 block sprite
  - [x] Fixed screen position
  - [x] Direct OAM manipulation
- [x] Add logging points in NesSystem for component interaction tracing
- [x] Fix sprite rendering issues
  - [x] Fix palette memory mirroring handling in write_palette
  - [x] Fix sprite rendering to properly write pixels to frame buffer
  - [x] Fix background rendering with proper palette handling
  - [x] Fix sprite attribute handling (horizontal/vertical flipping)
  - [x] Implement proper sprite priority handling
  - [x] Fix sprite palette selection

## MILESTONE 4: Animated Sprites [T4]
- [x] Create and run the simple multi-tile bouncing ball animation example
- [x] Implement basic sprite movement in a test ROM
- [x] Demonstrate multi-tile sprite rendering (2x2 tiles as a single object)

### [CPU] Essential Animation Instructions [T4]
- [x] Implement status flag changes (CLC, SEC) for arithmetic
- [x] Implement additional branches needed (BEQ, BNE) for flow control
- [x] Implement basic arithmetic (ADC, SBC) for position updates
- [x] Implement comparison (CMP) for bounds checking
- [x] Implement register transfer instructions (TXS) for stack initialization
- [x] Write tests for these animation instructions

### [CPU] Advanced Addressing Modes [T4]
- [x] Implement indirect addressing
- [x] Implement indexed indirect (indirect,X) addressing
- [x] Implement indirect indexed (indirect,Y) addressing
- [x] Write tests for advanced addressing modes

### [Assembler] Essential Animation Instruction Support [T4]
- [x] Implement parsing for status flag instructions (CLC, SEC)
- [x] Implement parsing for branch instructions (BEQ, BNE)
- [x] Implement parsing for arithmetic instructions (ADC, SBC)
- [x] Implement parsing for comparison instruction (CMP)
- [x] Add tests for these instructions

### [Disassembler] Extended Support [T4]
- [x] Add support for disassembling status flag instructions (CLC, SEC)
- [x] Add support for disassembling branch instructions (BEQ, BNE)
- [x] Add support for disassembling arithmetic instructions (ADC, SBC)
- [x] Add support for disassembling comparison instruction (CMP)
- [x] Enhance disassembler to support advanced addressing modes

### [Assembler] Additional NES Assembler Features [T4]
- [x] Implement ZEROPAGE segment support for variable declarations
- [x] Implement stack manipulation instructions (LDX #$FF, TXS)
- [x] Support variable declarations with labels (ball_x: .res 1)
- [x] Implement support for multi-tile sprite patterns
- [x] Add support for complex expressions in sprite positioning (ADC #$08)
- [x] Support recognition of all NES-specific memory segments ("HEADER", "ZEROPAGE", "STARTUP", "VECTORS", "CHARS")
- [x] Test assembler with the animation example program

### [PPU] Advanced Sprite Features [T4]
- [x] Fix OAM DMA transfer implementation for proper sprite rendering
- [x] Implement multiple sprite rendering
- [x] Implement sprite priority
- [x] Implement sprite attributes (flip, palette selection)
- [x] Test multi-tile sprite rendering
- [x] Fix sprite rendering issues
  - [x] Fix palette memory mirroring handling in write_palette
  - [x] Fix sprite rendering to properly write pixels to frame buffer
  - [x] Fix background rendering with proper palette handling
  - [x] Fix sprite attribute handling (horizontal/vertical flipping)
  - [x] Implement proper sprite priority handling
  - [x] Fix sprite palette selection

### [PPU] Advanced PPU Features [T4]
- [x] Implement PPU register mirroring ($2008-$3FFF mirrors $2000-$2007)
- [x] Implement frame timing controls with accurate NES cycles per frame
- [x] Implement speed control with proper FPS limiting
- [x] Support authentic NES timing (29,780 cycles/frame)
- [x] Test advanced PPU features

## MILESTONE 5: Input Controllers [T5]
- [x] Create a demo with controller-responsive sprite
- [x] Implement controller input handling
- [x] Test different controller inputs (D-pad, A, B, Start, Select)
- [x] Document the achievement

### [Input] Input System [T5]
- [x] Implement controller registers ($4016-$4017)
- [x] Implement standard controller (D-pad, A, B, Start, Select)
- [x] Implement controller strobe
- [x] Implement controller polling
- [x] Test controller input
- [x] Implement key mapping
- [x] Implement input configuration

### [CPU] Input Handling Instructions [T5]
- [x] Implement logical operations (AND) for button masks
- [x] Implement bit shifting operations (ASL, LSR) for controller button reading
- [x] Implement logical OR operation (ORA) for combining button states
- [x] Implement register transfers (TAY, TYA) for controller state manipulation
- [x] Implement X register operations (INX, DEX, CPX) for controller polling loops
- [x] Write tests for input handling instructions

### [Memory] Controller Mapping [T5]
- [x] Implement controller register mapping in memory ($4016-$4017)
- [x] Implement controller reading through the bus
- [x] Test controller memory mapping

### [System] Controller Integration [T5]
- [x] Connect controller input to the main system

### [UI] Controller Visualization [T5]
- [x] Create a controller state visualization widget for debugging
- [x] Display button states for both controllers
- [x] Implement real-time updates of controller state
- [x] Add controller state manipulation through the UI
- [x] Connect controller widget to controller handler
- [x] Test controller visualization with input changes
- [x] Add visual feedback when buttons are pressed
- [x] Document the controller visualization widget

### [Testing] Controller Test ROM [T5]
- [x] Create a test ROM for visualizing controller input on screen
- [x] Run the controller test ROM on the emulator
- [x] Verify all controller buttons work correctly
- [x] Use the test ROM to diagnose any input issues

### [Demo] Controller Demo Application [T5]
- [x] Create a ROM that displays a sprite on screen
- [x] Implement controller input handling for sprite movement
- [ ] Add support for moving the sprite using D-pad buttons
- [ ] Add visual feedback when the sprite moves
- [ ] Test the ROM with all directional controls

## MILESTONE 6: Basic Sound Output [T6]
- [x] Create a test ASM file that plays simple tones using APU registers
- [x] Implement fundamental audio framework
- [x] Successfully output simple sounds

### [APU] Basic Audio Framework [T6]
- [x] Define a simple tone generator test ASM that uses APU registers
- [x] Design minimal APU component structure to support the test ASM
- [x] Implement core APU registers ($4000-$4015, $4017)
- [x] Add APU component to the Bus architecture
- [x] Implement basic register reading/writing
- [x] Set up audio output device connection
- [x] Implement minimal audio buffer with proper timing
- [x] Create basic audio callback for device output

### [APU] Pulse Channel Implementation [T6]
- [x] Implement pulse channel 1 with basic frequency control
- [x] Support frequency control via period timer
- [x] Implement basic volume control
- [x] Add simple duty cycle control
- [x] Test single tone output (samples generated but not heard)
- [x] Enable/disable channel functionality

### [UI] Basic Sound Controls [T6]
- [x] Create minimal audio control widget
- [x] Add master volume control
- [x] Implement mute/unmute functionality
- [x] Add simple channel enable/disable controls
- [x] Show basic audio status in UI

### [Audio] System Integration [T6]
- [x] Create a CpalAudioOutput implementation in the rn_audio crate
- [x] Set up audio device enumeration and initialization using cpal
- [x] Implement audio stream handling with proper error management
- [x] Create sample conversion pipeline for hardware compatibility
- [x] Add audio buffer management to prevent underruns
- [x] Implement proper audio device lifecycle management
- [x] Test system integration with the basic tone example
- [x] Add support for volume control at the audio output level
- [x] Create smooth audio stream startup/shutdown

### [Testing] Sound Test ROM [T6]
- [x] Implement and test the simple tone generator ASM
- [x] Add ascending/descending tone patterns
- [x] Test volume modulation
- [x] Verify audio timing with CPU execution

### [Testing] APU Unit Tests [T6]
- [x] Create basic unit tests for PulseChannel functionality
  - [x] Test duty cycle selection and output patterns
  - [x] Verify volume control functionality
  - [x] Confirm enable/disable channel behavior
- [x] Test APU register interactions
  - [x] Verify $4015 status register controls
  - [x] Test register reset behavior on power cycle
- [x] Test sample generation accuracy
  - [x] Verify correct frequency output for known values
  - [x] Test sample amplitude with different volume settings
- [x] Verify integration with audio system
  - [x] Confirm samples reach audio output properly
  - [x] Verify volume and mute controls affect output

## MILESTONE 7: Complete Audio System [T7]
- [x] Create demo ROMs that showcase music and sound effects
- [ ] Implement all audio channels with full features
- [ ] Support complete NES audio functionality
- [ ] Document the complete APU implementation

### [APU] Complete Channel Implementation [T7]
- [x] Finish pulse channel 1 with all features ($4000-$4003)
  - [x] Implement envelope generator for volume control
  - [x] Add sweep units for frequency modulation
  - [x] Implement length counter
- [x] Implement pulse channel 2 with all features ($4004-$4007)
  - [x] Implement envelope generator for volume control
  - [x] Add sweep units for frequency modulation
  - [x] Implement length counter
- [x] Implement triangle channel ($4008-$400B)
  - [x] Implement linear counter
  - [x] Implement length counter
  - [x] Implement triangle wave generation
  - [x] Add proper register handling
  - [x] Test all triangle channel functionality
- [x] Implement noise channel ($400C-$400F)
  - [x] Implement noise generator with mode 0 and mode 1
  - [x] Implement envelope generator
  - [x] Implement length counter
  - [x] Add proper register handling
  - [x] Test all noise channel functionality
- [x] Implement DMC channel ($4010-$4013)
  - [x] Implement delta modulation sample playback
  - [x] Implement IRQ generation
  - [x] Implement loop flag
  - [x] Implement frequency control
  - [x] Add proper register handling
  - [x] Test all DMC channel functionality
- [x] Test all channel functionality
- [x] Add proper channel mixing

### [APU] Outstanding Accuracy Work [T7]
- [x] Implement the frame counter's 5-step mode ($4017 bit 7)
- [x] Implement the frame IRQ: $4015 bit 6, the $4017 inhibit flag, and the CPU IRQ line
- [x] Fix the PPU vblank flag so the `asm/` audio demos reach their APU init
- [x] Report the DMC's remaining bytes in $4015 bit 4, and clear them when the channel is disabled
- [ ] Give the DMC bus access so `load_next_byte` reads real memory, including CPU stall cycles
- [ ] $4017 write timing: the delay before an effective write takes effect (`apu_reset/4017_timing`)
- [ ] Power-on and reset state of the frame counter (`apu_reset/4017_written`, `works_immediately`)
- [ ] Which length counters a reset leaves enabled (`apu_reset/len_ctrs_enabled`)
- [ ] Drop the `objc2` `relax-sign-encoding` workaround in the root Cargo.toml once eframe/winit
      move to objc2 0.6+ (see AUDIO_PLAN.md section 4)

### [APU] Advanced Audio Features [T7]
- [x] Implement length counters for sound duration
- [x] Complete envelope generators for volume control
  - [x] Create dedicated Envelope struct for reuse across channels
  - [x] Implement all envelope functionality (decay, looping, etc.)
- [x] Implement sweep units for frequency modulation
  - [x] Create dedicated Sweep struct for pulse channels
  - [x] Support different negation behavior for Pulse 1 vs Pulse 2
  - [x] Implement proper muting conditions
- [x] Implement frame counter for timing

### [Testing] Advanced Audio Test ROMs [T7]
- [ ] Create test ROMs for each channel type
- [ ] Develop envelope and sweep effect tests
- [ ] Create audio pattern test suite
- [ ] Implement audio timing test ROM
- [ ] Test full game music examples
- [ ] Verify correct audio output using reference audio files
- [ ] Develop audio accuracy test suite

## MILESTONE 8: Conformance & Test ROMs [T8]

The community's test ROMs are the independent check on everything built so far. Commercial and test
ROMs cannot be committed here, so `tools/rom_test` skips cleanly without them and the unit tests
synthesise their own iNES images. Full rationale and ROM list in
[CONFORMANCE_PLAN.md](CONFORMANCE_PLAN.md).

### [Cartridge] PRG-ROM loading [T8]
- [x] Return PRG-ROM from the loader instead of skipping past it to the CHR data
- [x] Map PRG-ROM into $8000-$FFFF, mirroring a 16KB image at both $8000 and $C000
- [x] Start execution from the reset vector at $FFFC rather than a fixed load address
- [x] Load a ROM by path in nes_debugger and the probe tools, detected by content not extension

### [CPU] Complete the official instruction set [T8]
All 151 official opcodes decode. Of the 256, eighteen remain undecoded: twelve are JAM, which
correctly halt, and six are the unstable stores. `report_undecoded_opcodes` prints the list.
- [x] PHA, PHP, PLA, PLP (including the B flag's behaviour in the pushed status byte)
- [x] ROL, ROR (accumulator and memory forms)
- [x] CPY, CLV, TSX
- [x] RTI
- [x] Fill in missing addressing modes until all 151 official opcodes decode
- [x] Unofficial opcodes: implemented, including the immediate-mode set nestest and 03-immediate need
- [ ] The six unstable stores (SHA, SHX, SHY, TAS, LAS), whose behaviour varies with the chip

### [CPU] Interrupts [T8]
- [x] NMI: vector at $FFFA, triggered by PPU vblank when $2000 bit 7 is set
- [x] IRQ: vector at $FFFE, gated on InterruptDisable, shared by the APU frame counter and the mapper
- [x] BRK pushing the correct status byte, RTI restoring it
- [x] Interrupt lines shared, so a device can assert one while the CPU is mid-instruction
- [ ] Sample the lines at the cycle hardware samples them, so CLI and SEI take effect one
      instruction late (`cpu_interrupts_v2`). Needs the per-cycle CPU below.

### [Testing] Headless test-ROM runner [T8]
Blargg's ROMs write a status byte to $6000 and a message at $6004, so no screen is needed.
- [x] `tools/rom_test`: load a .nes, run to completion or timeout, report $6000 and $6004
- [x] nestest log-diff mode: run from $C000 and diff the CPU trace against nestest.log
- [x] `suite` mode over a directory, and `frame` mode to capture what the PPU drew
- [x] Press reset when a ROM asks for it, which several apu_reset ROMs wait on forever otherwise
- [x] Skip cleanly with a clear message when no ROMs are present
- [ ] Machine-readable output suitable for CI

### [Testing] Pass the suites [T8]
Standing as of the last run. Most of what remains is blocked on the two rewrites at the end of this
file rather than on anything specific to the suite.
- [x] nestest.nes — 8991/8991
- [ ] instr_test-v5 — 13/18
- [ ] instr_misc — 3/5
- [ ] mmc3_test — 3/6
- [ ] apu_test — 3/9
- [ ] apu_reset — 2/6
- [ ] ppu_vbl_nmi — 2/11
- [ ] instr_timing — 1/3
- [ ] cpu_interrupts_v2, branch_timing_tests, blargg_ppu_tests, cpu_dummy_writes, cpu_exec_space,
      oam_read, oam_stress — none yet


## MILESTONE 9: Mappers & Cartridges [T9]
Five mappers are implemented: NROM (0), MMC1 (1), UxROM (2), MMC3 (4) and AxROM (7). A ROM asking
for anything else is refused by name at load time rather than run with silently wrong banking.

- [x] Mapper trait, with detection from the iNES header
- [x] NROM (0), including a 16KB image mirrored at both $8000 and $C000
- [x] UxROM (2)
- [x] MMC1 (1), including the serial shift register and switchable mirroring
- [x] MMC3 (4), including CHR banking and the scanline IRQ
- [x] AxROM (7)
- [x] Refuse an unimplemented mapper by name instead of running it wrongly
- [x] Save and restore mapper state, so a snapshot resumes with the right banks
- [ ] CNROM (3), MMC2 (9), Color Dreams (11) — each small, none needed by anything tried so far
- [ ] MMC3's remaining A12 timing, which needs the per-dot PPU below

## MILESTONE 10: Full Desktop System [T10]
- [x] Load and play commercial ROMs
- [x] Save states, with tests that assert a restored machine continues identically
- [x] Fullscreen display, overscan cropping, frame dump with the PPU state that produced it
- [ ] Controller 2 surfaced in the key mapping (wired in the core already)
- [ ] Audio: verify against real game music now that the pipeline works
- [ ] Test with a wider variety of games

### [CPU] Advanced Instructions [T9]
- [ ] Implement remaining status flag changes (CLD, CLI, CLV, SED, SEI)
- [ ] Implement remaining branches (BCC, BCS, BMI, BVC, BVS)
- [ ] Implement register transfers (TAX, TAY, TXA, TYA)
- [ ] Implement stack operations (TSX, TXS, PHA, PHP, PLA, PLP)
- [ ] Implement increment/decrement (INC, INX, INY, DEC, DEX, DEY)
- [ ] Implement shifts/rotates (ASL, LSR, ROL, ROR)
- [ ] Implement compare operations (CPX, CPY)
- [ ] Implement logical operations (EOR, ORA)
- [ ] Write tests for all instructions

### [PPU] Background Rendering [T9]
- [ ] Implement name table handling
- [ ] Implement tile rendering
- [ ] Implement background priority
- [ ] Test background rendering

### [PPU] Scrolling [T9]
- [ ] Implement scroll registers
- [ ] Implement fine scrolling
- [ ] Implement name table switching
- [ ] Test scrolling functionality

### [PPU] Advanced Display Features [T9]
- [ ] Implement sprite zero hit detection
- [ ] Implement sprite overflow
- [ ] Implement sprite-background interaction
- [ ] Test advanced PPU features

### [PPU] PPU Timing [T9]
- [ ] Implement scanline/cycle tracking (261 scanlines, 341 cycles per scanline)
- [ ] Implement VBLANK flag setting and NMI generation
- [ ] Test frame timing with appropriate PPU cycles
- [ ] Implement frame synchronization
- [ ] Test animation capabilities

### [System] System Integration [T9]
- [ ] Implement full synchronization between components
- [ ] Optimize performance bottlenecks
- [ ] Test system with full games
- [ ] Implement game reset
- [ ] Implement game pause
- [ ] Implement save state mechanism
- [ ] Implement rewind functionality
- [ ] Implement speed control
- [ ] Add proper APU interrupts

### [UI] Game Application [T9]
- [ ] Create a standalone game application
- [ ] Implement ROM browser and loader
- [ ] Create configuration UI for controls and settings
- [ ] Add game state management
- [ ] Support screenshots and recording
- [ ] Implement fullscreen and window size options
- [ ] Create controller configuration UI
- [ ] Add visual feedback for game state

### [Parser] Complete Parser System [T9]
- [ ] Implement parsing for all official and unofficial instructions
- [ ] Add advanced error messages with suggestions
- [ ] Implement comprehensive validation and diagnostics
- [ ] Create bi-directional assembler/disassembler
- [ ] Support custom syntax variations and assembly dialects
- [ ] Add comprehensive documentation with examples

### [Debugger] Enhanced Debugging Tools [T9]
- [ ] Implement memory viewer/editor with advanced features
- [ ] Create CPU state inspector with history
- [ ] Implement PPU viewer with nametable and pattern display
- [ ] Add breakpoint system with conditional breaks
- [ ] Implement execution tracing
- [ ] Create disassembly view with code navigation
- [ ] Add performance profiling features
- [ ] Implement watchpoints for memory locations
- [ ] Support runtime code modification
- [ ] Add support for debugging symbols

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

## Progress Tracking
- Track 1 (Memory Visualization): 100% complete (50/50 tasks)
- Track 2 (PPU Pixel Display): 100% complete (40/40 tasks)
- Track 3 (Basic Sprite Rendering): 100% complete (54/54 tasks)
- Track 4 (Animated Sprites): 100% complete (47/47 tasks)
- Track 5 (Input Controllers): 100% complete (33/33 tasks) - Controller input is fully implemented with keyboard mapping support
- Track 6 (Basic Sound Output): 100% complete (47/47 tasks) - APU registers, pulse channel, volume/mute controls, and audio output through CPAL. The output pipeline was rebuilt (resampling, non-linear mixing, APU clock divider, audio-clock pacing) — see [AUDIO_PLAN.md](AUDIO_PLAN.md)
- Track 7 (Complete Audio System): 100% complete (31/31 tasks) - All audio channels implemented (pulse, triangle, noise, DMC) with hardware non-linear mixing, output filters, envelope generators, sweep units and length counters. Still outstanding: frame IRQ, 5-step frame counter mode, and DMC bus reads (tracked below)
- Track 8 (Conformance & Test ROMs): 0% complete (0/25 tasks) - Blocked on PRG-ROM loading; see CONFORMANCE_PLAN.md
- Track 9 (Mappers & Cartridges): 0% complete (0/25 tasks) 
- Track 10 (Full Desktop System): 0% complete (0/91 tasks) - Additional branch instructions (BCC, BCS, BMI) will be implemented here
- Track 11 (Web Integration): 0% complete (0/40 tasks)
- Additional Areas: 0% complete (0/113 tasks) - Including cycle-accurate timing, background rendering, testing, edge cases, distribution, extended features, legal considerations, documentation, performance optimization, and demo ROMs

**Total Progress: 299/535 tasks complete (55.9%)** 🚀

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
- [ ] Set up continuous integration pipeline
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
- [ ] Implement open bus behavior
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
## The two rewrites everything else is waiting on

Most of what still fails in the community suites fails for one of two reasons, not for reasons of
its own. Both are substantial and each deserves its own sitting; the groundwork for the first is
already in place.

### [CPU] Per-cycle state machine
Design and ordering in [CYCLE_ACCURACY.md](CYCLE_ACCURACY.md), written after checking the hardware
documentation rather than attempting a fourth time from intuition.

Today an instruction is one indivisible step: it runs to completion and the rest of the system is
advanced afterwards. The bus clock now advances the system on each memory access, which places most
cycles correctly, but the instruction still cannot be interrupted or observed partway through.

What that costs: interrupt sampling happens at the wrong moment, so CLI and SEI do not take effect
one instruction late as they must; and cycle counts come from a table rather than emerging from the
work, so nothing measuring them can pass.

- [x] Shared interrupt lines, so a device can assert one while the CPU is borrowed
- [x] Mapper in a shared slot, so the clock can reach it
- [x] Bus clock advancing the system on each access
- [x] Measure the gap: `rom_test cycles nestest.nes` reports it per opcode
- [ ] Model every cycle as the bus access it is on hardware, so accesses and cycles agree
      (37 of 225 opcodes do already; 6463 of 8991 executed instructions are short of one)
- [x] Interrupts sampled before the last cycle — `1-cli_latency` passes, the first in that suite
- [x] NMI hijacking BRK, and no polling inside an interrupt sequence
- [ ] Branch polling: before the operand fetch, not before the third cycle of a taken branch
- [ ] The rest of `cpu_interrupts_v2` is blocked on the PPU, not on the CPU: those tests spin in a
      vblank synchronisation loop that needs cycle-exact CPU/PPU alignment
- Acceptance: `cpu_interrupts_v2`, `instr_timing`, `branch_timing_tests`, `cpu_dummy_writes`,
  `cpu_exec_space`

Three attempts have been made and reverted. They are recorded here so a fourth does not repeat
them.

1. A full cycle-accurate rewrite, abandoned: no measurable gain, and no test suite existed then to
   say whether it had helped.
2. A latch sampling "the flag as it stood before the instruction". Moved `cli_latency` from its
   first sub-test to its tenth, but hung `nmi_and_brk`, which had been failing rather than hanging.
   A hang is worse than a failure: in a game it is a freeze, and nestest cannot catch it because it
   never exercises NMI.
3. Sampling at a cycle chosen from the instruction's cycle count, watched for by the bus clock.
   Worse than (2) — `cli_latency` reached only its third sub-test — because **the clock counts bus
   accesses and the cycle count counts cycles, and they are not the same number**. Roughly two
   thirds of cycles are accesses; the internal ones are not modelled. So a cycle chosen from the
   instruction's length cannot be found by counting accesses, and the sample lands early, late, or
   never.

The lesson common to (2) and (3): the sampling point cannot be approximated from outside the
instruction. It has to be a position *within* an explicit sequence of cycles, which is what the
remaining work below actually is. Anything cheaper has now been tried three times.

### [PPU] Per-dot fetch pipeline
The PPU draws a whole scanline at once, sampling its registers as the line begins. Hardware fetches
tiles and sprites throughout the line, and several things depend on *where* in the line a fetch
falls — including which pattern table is being read, which is what drives MMC3's counter.

- [x] The scroll address advances on the dot schedule (coarse X per fetch group, down at 256,
      horizontal restore at 257, vertical restore across 280-304)
- [ ] Background and sprite fetches issued per dot, in hardware's order

      Attempted background fetches alone, driving the mapper from a filtered A12 instead of from a
      scanline count. It regressed: Super Mario Bros 3 lost a third of its picture and mmc3_test
      fell from 3 to 2. The reason is that MMC3's A12 rise comes from the *sprite* pattern fetches
      at dots 257-320, which use $1000 while the background uses $0000 — with only background
      fetches the line never rises in the pattern the mapper is counting. The nametable fetches at
      $2xxx also drive A12 high every eight dots, resetting the low-period filter, so genuine
      rises are swallowed too.

      So the fetches cannot be done in halves: background and sprite fetches have to arrive
      together before A12 means anything, and the switch away from scanline counting has to happen
      in the same change. Preserved in `scratchpad/ppu-fetches/`.
- [ ] Sprite evaluation per dot, so $2004 reads during rendering return what it holds
- [ ] Vblank flag set and cleared at the exact dot
- Acceptance: `ppu_vbl_nmi`, `blargg_ppu_tests`, `oam_read`, `oam_stress`,
  `mmc3_test/3-A12_clocking`, `mmc3_test/4-scanline_timing`

### Housekeeping
- [ ] CI: the workspace has a clean clippy gate and a full test suite, and nothing runs them
- [ ] `crates/rn_core/tests/frame_alternation.rs` depends on a local ROM path and skips silently
