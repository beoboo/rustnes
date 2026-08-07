# RustNES Implementation Checklist 📋

> **The book is the deliverable; the emulator is the vehicle.** That was always the intent and this
> file had stopped reflecting it. See **[BOOK_PLAN.md](BOOK_PLAN.md)**, which supersedes every
> book-related box below and everything currently under `book/src` — the text is being rewritten
> from scratch.
>
> The strategy that failed, recorded so it is not chosen again: the book was "deferred, to be
> written later by reverse-engineering the finished emulator". Measured 2026-08-06, that produced
> **4,412 words across 47 files — 39 of them a single heading — and nothing at all in 335 commits**
> since `32dc3d0` (2025-03-08). Meanwhile TODO.md itself grew to 24,485 words of exactly the
> material the book needs. The book now gets its own code, written in book order, and this
> emulator becomes the reference implementation it is checked against.
>
> **Next step is the war-story harvest** — BOOK_PLAN.md step 1 — because that material decays and
> nothing else does.

> **Where the live work is.** This file mixes two things, and reading it as one list is misleading.
> Roughly 240 of its ~270 open boxes are in *Additional Important Areas*, *Milestone 10* and
> *Milestone 11* — packaging, WebAssembly, netplay, audio visualisation and so on. Those are a
> wishlist, not work in progress, and several of them are listed twice.
>
> What is actually being worked on is **Milestone 8 (Conformance & Test ROMs)** and the section
> *The two rewrites everything else is waiting on*, together about thirty boxes.
>
> **And treat every number here as a claim until it is re-run.** Four entries were found wrong in
> two days: CI recorded as never set up when it had been running for days, `sprite_hit_tests` at
> 7/11 when it was 11/11, `ppu_read_buffer` at 0/1 when it passes, and `cpu_timing_test6` recorded
> as unmeasurable when it passes. Each was repeated in conversation as fact before anyone re-ran it.
> A suite left alone for a while is a claim, not a measurement.

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
- [x] Configure CI/CD pipeline [T9] — `.github/workflows/ci.yml`
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
- [x] Improve memory access error handling to fail visibly on invalid accesses. Superseded: an
      unmapped access is ordinary on hardware, so it is answered with open bus and counted rather
      than refused. See "Implement open bus behavior" under Special Hardware Edge Cases.
- [x] Refactor AsmDebugger to use the NesSystem class for timing control
- [x] Implement NOP instruction to support timing-related tests
- [x] Test correct timing ratios between components

### [Memory] Essential Memory Components [T2]
- [x] Limit RAM to only handle the main memory region ($0000-$1FFF), as two kilobytes mirrored
      four times across it rather than as eight flat ones — the console decodes only eleven address
      lines for its work RAM
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
- [x] Give the DMC bus access so `load_next_byte` reads real memory, including CPU stall cycles.
      Three faults, each hiding the next: it played a placeholder byte and wrapped its address out
      of cartridge space; its memory reader was only reachable from inside the output unit, so a
      channel that had just started never asked for its first byte and no sample ever began; and it
      was clocked at the APU's half rate although its table is in CPU cycles, so everything played
      an octave low. The fetch now stalls the CPU four cycles, as hardware does.
- [x] $4017 write timing: three cycles if the write lands on an APU cycle, four if between two
- [x] Power-on and reset state of the frame counter: the mode survives a reset, and the machine
      runs the cycles the CPU spends starting up before its first instruction
- [x] Which length counters a reset leaves enabled: all of them. A reset clears `$4015` and nothing
      else, so `$4000-$4013` and everything derived from them survive
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
All 151 official opcodes decode, and of the 256 only the twelve JAM opcodes remain undecoded —
which is correct, since they halt. `report_undecoded_opcodes` prints the list.
- [x] PHA, PHP, PLA, PLP (including the B flag's behaviour in the pushed status byte)
- [x] ROL, ROR (accumulator and memory forms)
- [x] CPY, CLV, TSX
- [x] RTI
- [x] Fill in missing addressing modes until all 151 official opcodes decode
- [x] Unofficial opcodes: implemented, including the immediate-mode set nestest and 03-immediate need
- [x] The six unstable stores (SHA, SHX, SHY, TAS, LAS). Two things make them strange and both are
      modelled: the value stored is the register ANDed with the high byte of the *base* address
      plus one, and when the index crosses a page the high byte of the target is itself ANDed with
      the register, so the store lands somewhere other than where the operand said. Only the twelve
      JAM opcodes are now undecoded, which is correct — they halt.
      Not modelled: on hardware the AND with the high byte is skipped if a DMA interrupts the
      instruction just before its dummy read, which needs a DMA that can land mid-cycle.

### [CPU] Interrupts [T8]
- [x] NMI: vector at $FFFA, triggered by PPU vblank when $2000 bit 7 is set. /NMI is a level the
      PPU holds for as long as the flag and the enable bit are both set, and the CPU takes one
      interrupt per rising edge of it
- [x] IRQ: vector at $FFFE, gated on InterruptDisable, shared by the APU frame counter and the mapper
- [x] BRK pushing the correct status byte, RTI restoring it
- [x] Interrupt lines shared, so a device can assert one while the CPU is mid-instruction
- [x] Sample the lines at the cycle hardware samples them, so CLI and SEI take effect one
      instruction late (`cpu_interrupts_v2`). A one-cycle-delayed shadow of both lines, updated by
      the same clock that advances the PPU and checked after the instruction, as Mesen does — no
      computed polling cycle, so nothing has to know how long an instruction is.

### [Testing] Headless test-ROM runner [T8]
Blargg's ROMs write a status byte to $6000 and a message at $6004, so no screen is needed.
- [x] `tools/rom_test`: load a .nes, run to completion or timeout, report $6000 and $6004
- [x] nestest log-diff mode: run from $C000 and diff the CPU trace against nestest.log
- [x] `suite` mode over a directory, and `frame` mode to capture what the PPU drew
- [x] Press reset when a ROM asks for it, which several apu_reset ROMs wait on forever otherwise
- [x] Skip cleanly with a clear message when no ROMs are present
- [x] Read the result off the screen for the ROMs that predate the $6000 protocol. Blargg's earlier
      suites report on screen only, and every one of them was being counted as a failure for want
      of a way to read the answer. They write ASCII straight into the nametable — the tile index is
      the character code — so the fix was to look, not to guess. Seventeen ROMs were unmeasured;
      fourteen now report, and the three that do not are marked as such rather than as failures.
      A verdict read this way is labelled `[screen]` in the output, because it is this runner's
      reading of what a ROM drew rather than the ROM's own word for it.
- [x] Frame baselines that live in the repository — `rom_test baselines`. The rule is that anything
      touching rendering is gated on a pixel diff against a saved frame, never on a summary
      statistic, and the rule was being followed. The *storage* was not: the saved frames sat in
      `/tmp`, where nothing checks them and nothing keeps them. One was found to disagree with an
      unmodified build, so it had stopped being a gate at some earlier commit and had gone on
      reading as evidence ever since. A gate that can go stale in silence is worse than no gate.

      The frames cannot be committed — they are pictures of commercial games — but hashes can, and
      each baseline stores one per scanline as well as one for the whole frame. That is what keeps
      it a diff rather than a statistic: a mismatch says *which rows*, and "192-194 differ" is most
      of a diagnosis. To see the pixels, re-render with `rom_test frame --out`.

      Proved against the bug the practice exists for, by rotating the frame buffer sixteen pixels
      sideways — the shift a coverage percentage once hid. All three baselines report it, naming the
      rows, and the command exits non-zero so it can gate rather than merely inform. A ROM that is
      not in the checkout is a skip and exits zero: nobody else has these files.

- [ ] Machine-readable output suitable for CI

### [Testing] A second opinion — `tools/nesref`
A tetanes checkout beside this one, driven headlessly through `tetanes-core`, reading the same
`$6000` protocol our own runner reads. It answers the question that had been costing sittings:
is a failing ROM a bug of ours, or something obscure that nothing gets right?

Run the first time, it settled five of them at once. **tetanes passes every one of these, so they
are our bugs and its source is the reference:**

```
3-nmi_and_irq            PASS
4-irq_and_dma            PASS
5-branch_delays_irq      PASS
cpu_interrupts           PASS
test_cpu_exec_space_apu  PASS
```

One concrete difference already spotted while comparing, not yet chased: tetanes gives the mapper
`$4100..=$FFFF`, where this emulator gives it `$8000..=$FFFF` and serves `$6000-$7FFF` from a plain
RAM component. So `$4100-$5FFF` is open bus here and cartridge space there.

**Differential tracing** — `rom_test trace` against `nesref --trace` — is the technique that came
out of this, and it is far sharper than reading either source. Both emit a line per instruction in
nestest's shape; the first differing line is the bug, and everything above it is agreement rather
than an assumption. What it said the first time, on `5-branch_delays_irq`:

- **The CPU is exact.** Cycle counts agree with tetanes for 8190 consecutive instructions, to a
  constant offset of seven — their reset sequence, which they count and we do not. No drift at all.
  That retires a great deal of suspicion in one measurement.
- **The PPU was a scanline out.** Its position was a constant *336* dots behind at every
  instruction, which is one scanline less five. The cause was `scanline: -1` at power-on, described
  in the code as the pre-render line when the pre-render line here is 261 — so it was a line neither
  drawn nor pre-render, run once and never again. Fixed; no suite moves, and the trace is the
  demonstration.
- **Five dots remain, and they are not a bug.** Five is not a multiple of three, so it is a
  CPU/PPU *phase* difference of two dots, and the power-on phase is genuinely one of three
  possibilities on hardware. Ours is the one that passes `05-nmi_timing`, `4-scanline_timing` and
  `4017_timing`, all of which measure to the dot. Theirs is a different legal choice.

So `5-branch_delays_irq` is not an alignment fault. Diffing further, and what it has ruled out:

- **The remaining divergence in the trace is benign.** With the scanline fixed, the first differing
  instruction moves from 8198 to 42443, and it is a `BIT $2002 / BPL` loop waiting on vblank. Our
  read lands just after the flag is set and tetanes' just before, so tetanes waits an extra frame.
  That is the two-dot phase difference doing exactly what a poll sitting on the boundary must do,
  and both behaviours are legal.
- **The phase is not the fault either.** Shifting our power-on phase by one dot and by two changes
  nothing: `5-branch_delays_irq` still fails identically. The test syncs to vblank before measuring,
  so it is insensitive to where the machine started, which is what one would hope.
- **Our branch rule already matches tetanes exactly** — same `run_irq && !prev_run_irq` test, same
  placement before the dummy read. And the sub-test that actually fails is the first one,
  `test_jmp`, which contains no branches at all. The name is misleading about where the fault is.

**The refinement, and what it found.** Two changes to the technique, both in the repository now:

- `tools/tracediff.py` diffs two traces *with resynchronisation*. On a mismatch it scans ahead in
  both for a window that matches again, so a poll loop spinning a different number of times becomes
  one labelled block instead of the end of the comparison.
- `RN_RESET_DOTS=19` puts this machine exactly where tetanes starts. Without it the two part company
  at the first `$2002` poll on the vblank boundary and everything after compares different
  sub-tests. **It moved the first real divergence from instruction 42,443 to 175,788.**

At 175,788 the fault is visible, and it is not about `JMP` at all:

```
$E59F  BIT $4015      first read: bit 6 set, V set        (both agree)
$E5A2  BIT $4015      second read: ours bit 6 clear, tetanes bit 6 SET
$E5A5  BVC $E590      so ours branches and tetanes does not
```

Two `BIT $4015` reads four cycles apart, straddling the frame counter's three-cycle IRQ window. The
first read acknowledges the flag; whether the second sees it set again depends on whether the window
is still open when that read samples.

**The frame counter itself is not the difference.** tetanes' step table is
`[7457, 14913, 22371, 29828, 29829, 29830]` with the IRQ asserted from step 3 on — the same three
cycles as ours, and its half-frame clock is at 29829 as ours is.

**Measured, by logging the frame-counter cycle at every `$4015` read in both.** The ROM walks a pair
of reads four cycles apart across the window's edge, one cycle further each iteration:

```
ours     (29818,29822) (29819,29823) ... (29822,29826) (29823,29827)   and stops
tetanes  (29821,29825) (29822,29826) ... (29823,29827) (29824,29828)   one further
```

Every pair the two share agrees exactly — same cycles, same flag. The difference is that tetanes
reaches `(29824, 29828)`, where the second read lands *inside* the window and finds the flag set
again, and we never get there: our scan ends one iteration short, with the second read at 29827.

So at the same instruction, **our frame counter is one CPU cycle behind tetanes'**. That single
cycle decides bit 6 of the second read, which decides `V`, which decides the branch at `$E5A5`, and
the ROM goes a different way from there.

One cycle, in a counter whose table and window are already identical — and it was neither of the
candidates guessed at here. **The APU was being clocked after the bus access instead of before it.**

Both references do it the other way and we did not: Mesen clocks the APU from `StartCpuCycle`,
ahead of the memory access, and tetanes' `read_status` catches the APU up to the current master
clock before reading. Ticking it afterwards left the frame counter exactly one cycle behind at
every read — which is the whole of what this entry has been chasing. **`5-branch_delays_irq`
passes.**

Moving it broke `7-dmc_basics`, and the reason is worth keeping: the DMC only asked for its byte
while being *clocked*, so with the tick now ahead of the write, a `$4015` that starts a sample no
longer had a tick behind it in the same instruction and the fetch slipped by one. The ROM says what
should happen instead — "there should be a one-byte buffer that's filled immediately if empty" — so
enabling the channel now fills it there and then. Both pass.

Its read and write paths otherwise agree with ours closely — the PPU's write-only registers return
the PPU's own latch, everything unmapped returns the CPU's, and a write drives the bus either way.
And its MMC3 counter is the same logic as ours, clocked from real PPU bus reads.

### [Testing] Suites never measured until now
Measured for the first time, and three of them were already passing:
- [x] vbl_nmi_timing — 7/7
- [x] instr_test-v3 — 17/17
- [x] nes_instr_test — 11/11
- [x] sprite_hit_tests — 11/11, from 0/11 once CHR RAM worked. The 7/11 this line used to
      claim was stale; re-measured 2026-08-06.
- [x] sprite_overflow_tests — 5/5, from 3/5. `3.Timing` and `4.Obscure` were the same omission,
      which the code named in a comment and declined to implement: "only the flag is modelled here,
      not the diagonal scan that follows it".

      Once eight sprites are found, the overflow hardware goes on reading object memory — but it
      reads `m` bytes into each following sprite instead of its first, and advances `m` alongside
      `n` rather than resetting it. So the ninth sprite's *X* is compared against the scanline, the
      tenth's attributes, the eleventh's tile index, and so on around. That is the whole reason the
      overflow flag cannot be used to count sprites, and `4.Obscure` is seven tests of it, one per
      byte position.

      `3.Timing` fell out of the same change without being aimed at. Its failing test was #12, "set
      too late when the 9th sprite is way after the 8th", and the missing cycles were the three
      hardware spends reading the rest of an overflowing sprite it has nowhere to put — a detail
      that only shows up in a test measuring the flag to within a CPU cycle or two.

      Pinned by `the_overflow_scan_misreads_a_later_byte_as_a_y_coordinate`, which sets up sprites
      whose *Y* is off the line and whose byte at the scanned offset is on it, one offset at a time.
      A PPU reading real Y coordinates leaves the flag clear for every one of them. The ROMs report
      only pass or fail; the test says which byte was misread.
- [x] ppu_open_bus — 1/1. The PPU's I/O latch is a *decay* register: dynamic storage with nothing
      holding it up, so a bit not refreshed leaks to zero in about 600ms. And each register
      refreshes a different part of it — `$2002` supplies its top three bits and leaves the other
      five to rot, a palette read through `$2007` supplies six, a write supplies all eight, and a
      read of a write-only register supplies none, so holding a value by reading it repeatedly is
      exactly what hardware will not let you do.
- [x] ppu_read_buffer — passes. This line claimed 0/1 with 76 of 79 sub-tests passing and named
      57, 60 and 73 as the failures; re-measured 2026-08-06 and the ROM reports PASS outright.
      Whatever fixed it was not aimed at it — a reminder that a suite unmeasured for a while is a
      claim, not a fact.
- [x] blargg_nes_cpu_test5 — 2/2, including `cpu.nes`, which covers the unofficial instructions
      too. It was passing already; it reports by listing what it ran and ending with "All tests
      complete", which the screen reader could not interpret. The failing form — "Errors: n" and
      "Failed" — was produced deliberately, by breaking `ASL`'s carry flag, before that rule was
      written: "complete" is not "passed", and guessing would have turned a broken emulator green.
- [x] ny2011 — drew a blank screen and now draws its picture. **CHR ROM was writable**, which is
      the kind of fault that hides for months: it needs a cartridge whose tiles are in ROM *and* a
      program that writes to pattern space, and most programs have no reason to.

      This one has: it clears the whole of VRAM at startup, as plenty of demos do. On hardware
      those writes fall on ROM and do nothing. Here they landed, so the game zeroed its own
      character data and every pattern fetch afterwards came back empty — while its nametables and
      attributes stayed correct, which is exactly why the screen reader could still print the text
      it had written and the picture was still blank. The probe that settled it read
      `last_nt=2B last_pat_lo=00`: a valid tile index whose pattern bytes were zero, against a
      `2B` in the file holding `AB F2 85 8E ...`.

      `Nrom::write_chr` had carried the comment "Only cartridges with CHR RAM accept this; those
      with CHR ROM ignore it" while the code wrote regardless. Five of the six mappers did the
      same. What made it invisible is that the *loader* filled an absent CHR ROM with eight
      kilobytes of zeros, so by the time any mapper saw it there was nothing left to tell RAM from
      ROM — and CHR RAM only worked *because* the write went through unconditionally. The loader
      now leaves that vector empty, which is the honest representation, and each mapper records
      whether its character memory is RAM and honours it.

      Pinned by `chr_rom_ignores_writes_and_chr_ram_accepts_them`, which checks both directions
      across all six mappers and fails on the first without the guard. `sprite_hit_tests` is the
      suite that would break if the RAM half regressed — it is CHR RAM and reported "sprite hit
      isn't working at all" the last time this went wrong — and it stands at 11/11.

      What is left is animation phase: the falling snow sits differently after 600 frames, so the
      frames are 11.3% apart structurally while both hold the same 7 colours and the picture is
      the reference's. Two emulators only agree on such a thing if everything upstream is
      cycle-identical, and nothing here needs them to.
- [ ] blargg_litewall/litewall5 — **one pixel**, at x=255 on scanline 192, the last dot of the
      line: ours takes a palette colour where the reference takes black. Everything else in the
      frame agrees, both draw the same 9 colours, and the structural mismatch is 1 of 61440.

      It appeared when pixels started being emitted with rendering off (see `full_palette` below),
      and that is worth being precise about rather than calling it a regression: this ROM is
      another user of the backdrop hack — it turns rendering off mid-line at dot ~234 and writes
      `$2006 = $3F01`, a palette address, in hblank, which is how a "light wall" of colour bars is
      drawn at all. Emitting nothing there used to match by accident. Emitting the palette entry is
      right, and what it newly exposes is a **one-dot disagreement about when a `$2001` write takes
      effect**, landing on the one dot of the line where it can still be seen.

      **The delayed-`$2001` explanation was tried and is refuted.** The pixel path did read `mask`
      directly where it should read the one-dot-delayed `rendering_enabled` — tetanes reads its
      delayed copy in `render_pixel` — so pointing `emit_pixel` and the rendering-off emission gate
      at the delayed flag looked like the answer. It is **inert**: the pixel stays, and so does
      every other number, `full_palette` at 9,079 differing pixels and `litewall2` at 29,184 and
      the three baselines unmoved. Reverted rather than kept, since nothing demonstrates it.

      So the cause is still unknown, and it is *not* the mask arriving a dot early. What is known:
      it is the last visible dot of the line (x=255 is cycle 256, the dot the vertical position is
      incremented on), and the demo turns rendering off at dot ~234 and writes `$2006 = $3F01` in
      hblank. The next attempt should probe `v` and the mask at that exact dot in both emulators
      rather than reason about which flag is read, which is what this one did.
- [~] full_palette — 0/3 to a picture that matches the reference's layout, by fixing two PPU gaps
      the demo-ROM triage of 2026-08-06 turned up. Not yet an exact match, and the reason is
      understood; see the end.

      The triage itself is the point: these ROMs report by picture alone, so `rom_test` called all
      three failures and nobody had looked. Ours drew a **black screen** where tetanes drew all 64
      colours. Two things were missing, and the first is the one worth remembering:

      **Pixels must be emitted even when rendering is off** — that is *how* the screen goes blank,
      and the blanking colour is not always the backdrop. Ours emitted none at all: `emit_pixel`
      is called from `advance_background_fetch`, and `tick` only calls that `if rendering_now`.
      tetanes says it in a comment — "Pixels should be put even if rendering is disabled, as this
      is what blanks out the screen. Rendering disabled just means we don't evaluate/read
      bg/sprite info."

      **And with rendering off, a `v` left inside `$3F00-$3FFF` is shown instead of the backdrop.**
      The PPU has no fetched pixel, so what reaches the screen is whatever palette entry the
      address register points at. That is the whole of how this ROM paints: it never turns
      rendering on once — 65 writes of `$2001 = $00` in a frame and not one that enables it — and
      steps `v` through palette memory.

      **Colour emphasis was declared and never applied.** `MASK_EMPHASIZE_RED`/`GREEN`/`BLUE` had
      sat in this file as constants nothing read. The ROM writes `$2001 = $20` between bands to
      show more than 64 colours at once, so without emphasis ours drew 52 distinct colours against
      the reference's 426. Applying the documented attenuation — a channel is darkened when a bit
      it is not named by is set, 0.746 as 191/256 — brings it to 223.

      **The palette table was the rest of it, and it is now derived rather than approximated.**
      Done 2026-08-06: `ppu::palette` decodes the composite signal — a square wave whose phase is
      the hue, two voltage levels for brightness, de-emphasis attenuating the samples in phase with
      one primary — into all 512 entries, following the generator on the NESdev wiki. The old table
      was 64 hand-written triples with a uniform 0.746 multiplier over them, and the multiplier is
      the part that was wrong: de-emphasis does not dim uniformly, so entries hardware keeps apart
      collapsed together. That is unfixable by any amount of care in the emulator, because it is
      the *table* discarding information.

      It moved the numbers a long way. `full_palette` drew 223 distinct colours against the
      reference's 426 and now draws 420; its differing pixels fell from 53,432 to 9,079.
      `litewall3` now reports **SAME LAYOUT** outright, and `litewall5` is down to a single pixel.

      **What is left is not the palette, and it is now localised.** `full_palette` disagrees on
      0.54% of horizontal edge positions and 0.69% of vertical, and those are not spread over the
      picture: they sit in **six columns**, x=1,2,3 at the left edge and 252-254 at the right.
      Pixel by pixel, the reference holds one *more* black pixel at the start of each line than we
      do — so our palette-hack colour changes a dot earlier than its. `litewall5`'s single wrong
      pixel is the same effect at the other end of the line, at x=255.

      **A one-dot lag was tried on 2026-08-07 and is not the answer**, though it is close enough
      to be worth the numbers. Holding `v` a dot before showing it — kept exactly as
      `rendering_enabled` is — makes `litewall5` **IDENTICAL** and leaves `litewall3` edge-perfect,
      and takes `full_palette` from 0.54% to **7.87%** horizontal edge disagreement. It helps two
      ROMs and badly hurts the third, so it is not a blanket rule and was reverted.

      What that narrows it to: the lag is real for some source of a `v` change and not for
      another. `full_palette` steps `v` with `$2007` writes while the litewall demos set it with
      `$2006`, which is the obvious next thing to separate — and the next attempt should measure
      those two paths rather than delay everything at once, which is what this one did.

      Two things had to be re-recorded rather than argued with, and both were checked first: the
      three frame baselines are hashes of our own rendering, so every colour changing moved all
      three — they were confirmed **structurally identical** to the pre-change rendering before
      being updated, which is the whole reason the structural compare exists. And
      `test_complete_sprite_pipeline` asserted three literal `255`s for a white sprite; entry `$30`
      decodes to (254, 255, 255), so the test was really asserting which table was compiled in. It
      now asserts against `palette::rgb`.
- [x] read_joy3 — `thorough_test` passes, from hanging outright, and the bug it found was an
      unsigned underflow rather than anything about timing. Found 2026-08-06 by the first full
      sweep of every suite, which is also how it came to light that this file's "every failure is
      one of three documented non-bugs" was untrue: nothing had ever triaged this suite.

      The ROM hung ours in `sync_dmc`'s `BIT $4015 / BNE` — waiting for a sample to finish that
      never did. The DMC ledger showed why in one line: our fetches walked `DB52, DB53, DB54...`
      up through memory for ever, where tetanes refetched `C000` every time. A one-byte looping
      sample restarts in place; ours was playing 65535 bytes.

      **`$4015 = 0` zeroes `bytes_remaining`, but a DMA already asked for is still in flight**, and
      the byte lands afterwards. `supply_byte` counted it, taking a `u16` counter from zero to
      65535 — a sample that never ends, with `$4015` bit 4 stuck set, so any program waiting on
      that bit waits for good. `sync_dmc` is such a program, which is why this ROM and nothing
      else caught it. A byte with no sample to belong to now changes nothing: not the buffer, not
      the address, not the counter, and it neither loops nor raises the end-of-sample interrupt,
      because the sample was aborted rather than finished. tetanes guards `load_buffer` the same
      way.

      The other three are not failures of ours. `test_buttons` wants a human — it prints "Press
      indicated buttons". `count_errors` and `count_errors_fast` *count* rather than judge, and
      some errors are the point: ours reports 52/1000 conflicts and 12/1000 errors, against
      tetanes' 15/1000 and the `status.txt` reference emulators' 72/1000 and 7/1000, which both
      mark FAIL. Comparing those numbers needs a tolerance nobody has established; they should
      not be counted as failures until one is.
- [~] dmc_tests — 0/4, and now known to be audio-only rather than assumed. All four render a
      picture **structurally identical** to tetanes': one flat backdrop and nothing else. So they
      report by sound alone and cannot be judged here until there is something to compare audio
      against. Not a failure, and it should stop being counted as four. Corroborated 2026-08-06:
      the nes-test-roms repository's own `status.txt` marks all four `???? Not sure yet` — even
      its maintainers have no verdict channel for them.

      An earlier version of this entry said the two emulators' backdrops differed — ours grey
      117,117,117 against tetanes' 83,83,83 — and put that down to power-on palette RAM. **That was
      wrong twice over.** tetanes defaults to an NTSC composite filter, so the reference frames were
      filtered; and with the filter off the remaining difference is that the two ship different NES
      palette tables, ours rendering black as 0,0,0 and tetanes as 3,3,3. Nothing to do with palette
      RAM. Comparing frames between emulators has to ignore which RGB each gives a palette entry,
      which is what `rom_test compare` now does.
- [x] nestest.nes — 8991/8991
- [x] instr_test-v5 — 18/18. The last four were not failures at all: the system peeked at the next
      opcode after every step and, on seeing `$00`, declared the program Finished and switched the
      machine off. That is right for the debugger, where a hand-assembled snippet really does end
      with `BRK`, and fatal for a cartridge, where `BRK` is an instruction with a handler behind it.
      Now a property of how the code was loaded: `load_program` halts, `load_rom` does not.
- [x] instr_misc — 5/5, from 3/5. The unofficial NOPs were doing nothing at all: they take an
      operand and they *read* it, being a load whose result goes nowhere rather than a do-nothing
      that happens to be longer. Invisible against RAM, which is why it went unnoticed; not
      invisible against a register, where `NOP $4015,X` acknowledges the APU's frame IRQ exactly as
      `LDA $4015,X` would.
- [~] mmc3_test — 5/6 and mmc3_test_2 — 5/6, both failing only their MMC6 test. **Deliberately not
      done.** MMC6 is a different chip: same scanline counter, but 1 KB of on-cartridge work RAM
      with per-half write protection through `$8000`/`$A001` instead of MMC3's 8 KB. Two games use
      it — Startropics and its sequel — and nothing else here needs it. `mmc3_irq_tests/5.MMC3_rev_A`
      is the same shape of gap: the counter as the *earlier* revision of MMC3 implemented it, which
      differs in when a reload of zero fires. Both are worth doing if either game is ever run, and
      neither is evidence of anything wrong with the MMC3 that is here. Recorded so they stop
      reading as five open failures.
- [x] apu_test — 9/9, from 4/9, combined ROM included. Two faults in the frame counter. Its IRQ
      flag went up for one cycle at the sequence wrap where hardware holds it across the last
      *three* cycles, 29828 to 29830, so a program reading `$4015` inside that window clears it and
      finds it set again immediately. And a `$4017` write waited a fixed three cycles where the
      delay is three or four depending on whether the write landed on an APU cycle — the parity of
      the CPU cycle, and the jitter `4-jitter` is named for.
- [x] apu_reset — 6/6, from 3/6. Three causes, and the common thread is that reset is not
      power-on:
      - `$4000-$4013` survive a reset. Only `$4015` is cleared. Resetting every channel outright
        took the triangle's linear counter control with it, which is what `len_ctrs_enabled`
        checks by setting that flag before the reset and looking for the triangle still counting
        after it. The two paths are now separate: `power_on` clears everything, `reset` clears
        `$4015`.
      - The frame counter's mode survives a reset too (`4017_written`).
      - The machine runs for the cycles the CPU spends starting up, before its first instruction.
        `4017_timing` measures exactly that and wants 9 to 12; without it we reported 3.
- [x] ppu_vbl_nmi — 11/11, from 5/11, combined ROM included (the figure here read 2/11 for a while
      after it was no longer true; re-measured against the commit before the A12 work, which did
      not move it)

      **`05-nmi_timing`, `06-suppression`, `07-nmi_on_timing` and `08-nmi_off_timing` now pass.**
      The last two came from /NMI becoming a level the PPU holds rather than a one-shot latch the
      system consumes, which is what lets a program toggling `$2000` bit 7 during vblank take one
      interrupt per rising edge. The CPU needed nothing new for it — the edge detector was already
      there. The cause was one PPU dot, and it was
      neither the PPU's nor the flag's: our clock ran all three of a CPU cycle's dots and then
      performed the bus access, so the interrupt lines were read at the instant of the access. A
      6502 cycle runs on past its access, so the poll belongs one dot later, at the cycle's end.
      Two dots before the access and one after, with the lines read at the end of the second.
      Full account in [CYCLE_ACCURACY.md](CYCLE_ACCURACY.md), including the measurement that found
      the dot, and how the same dot turned out to be behind `ADDRESS_BUS_LEAD_DOTS` — a constant
      that existed only to cancel it, now deleted.

      What follows is the analysis that led there, kept because the reasoning is still the way to
      read the table. `05-nmi_timing` measures the fault precisely, and is the best handle on
      CPU/PPU alignment in the suite: it prints which instruction the NMI landed after, running one
      PPU clock later on each line. Expected against ours *as it then stood*:

      ```
      expected   00 4  01 4  02 4  03 3  04 3  05 3  06 3  07 3  08 3  09 2
      ours       00 4  01 4  02 4  03 4  04 3  05 3  06 3  07 3  08 3  09 3
      ```

      Every transition is one line late, so **the NMI reaches the CPU one PPU dot after it should**.
      Note what is *not* wrong: `02-vbl_set_time` and `03-vbl_clear_time` pass, so the flag itself
      is set and cleared on the right dot. It is the interrupt's delivery that lags, not the PPU.

      Tried and reverted, because it changed nothing at all — not the table, not even the
      instruction count: collecting the NMI after each of a CPU cycle's three dots rather than
      after all three. The suspicion was that an interrupt raised on the first dot waited for the
      third, but the CPU already samples the line after the clock has run within the same bus
      access, so the extra granularity buys nothing. The dot is lost somewhere else.
- [x] instr_timing — 3/3. Two causes. `2-branch_timing` was the BRK halt. `1-instr_timing` reported
      "40 was 8, should be 6": every pull did its own dummy stack read, where hardware winds the
      pointer forward once per pull *sequence* and the pulls that follow are a cycle each. That
      made `RTI` eight cycles. `RTS` gained the read it was missing at the address it pulled, which
      is its sixth cycle.
- [x] oam_read — 1/1, and oam_stress — 1/1. Recorded here as "none yet" long after they passed;
      re-measured against the commit before the sprite work, which did not move them.
- [x] branch_timing_tests — 3/3. Recorded as "none yet" for as long as the runner could not read
      an on-screen result; they had been passing.
- [x] mmc3_irq_tests — 5/6, likewise unmeasured until now. The one failure is `5.MMC3_rev_A`, the
      other revision's counter, which is the same gap as `mmc3_test/6-MMC6`.
- [x] blargg_ppu_tests — 4/5, from 2/5 once its own error codes could be read. Both faults were
      named by the ROM and fixed the same sitting:
      - `sprite_ram` $07 — the sprite DMA forced $2003 to each byte's index, so every copy started
        at sprite zero. Hardware never touches $2003: it writes $2004 256 times and each write
        advances OAMADDR itself, so the copy begins where the program pointed it, wraps, and leaves
        $2003 as it found it. That is how a game rotates which sprites win priority.
      - `vram_access` $06 — a palette read answered from palette RAM and left the read buffer
        alone. The read still happens on the bus, so the nametable byte under the palette's mirror
        ($3F00-$3FFF down to $2F00-$2FFF) belongs in the buffer for the next read to collect.
      - `power_up_palette` $02 — "palette differs from table", and **not a fault**: the suite's own
        readme says those values "are probably unique to my NES". It is a recording of one console's
        power-on palette RAM, which is indeterminate hardware. Nothing to chase, and it will fail
        for ever. Left in rather than skipped because, unlike the PAL suite, it is one ROM and its
        reason is written on it.
- [x] cpu_timing_test6 — passes. Recorded here as "draws nothing into the first nametable within
      600 frames" and therefore unmeasurable; re-measured 2026-08-06 and it reports PASS.
- [x] pal_apu_tests — skipped by the runner, not failed. It is `blargg_apu_2005.07.30` with PAL
      timings, so on an NTSC emulator every one of its ten ROMs fails by design; they had been
      putting ten permanent red marks in every sweep, which is the kind of noise that teaches people
      to skim a failure list rather than read it. Recognised by path rather than by header, and that
      is a heuristic where one would rather have a fact: the PAL ROMs' iNES headers are byte for
      byte their NTSC siblings', flags 9 and 10 both zero. Only the directory says which is which.

- [~] nmi_sync — measurable now, and very close. Both ROMs are visual demos with no verdict to
      read, so `nesref --frame` captures tetanes' picture and `rom_test compare` diffs it ignoring
      palette. `demo_ntsc` at 240 frames differs by **2 pixels of 61440**, one on row 0 and one on
      row 121. `demo_pal` is PAL and not applicable.

      An earlier run of this said 11 pixels; that was against a frame tetanes had put its NTSC
      composite filter through. What the 2 mean needs knowing what the demo draws, which nobody has
      looked up — recorded as a number, not a verdict.
- [x] cpu_interrupts_v2 — every ROM in the suite passes, singles and combined, and nothing hangs.

      `3-nmi_and_irq` passes: an interrupt sequence has to leave the shadows clear behind it, or an NMI arriving during its own seven
      cycles — too late to hijack the vector — is serviced the instant the sequence ends and the
      handler never reaches its first instruction. `BRK` already did this; the hardware sequence
      needed it too.

      **`4-irq_and_dma` passes.** It was down to a single row of its 528 — `+526`, where it wanted
      `8` and we said `9` — and the cause was neither of the two things this entry spent three
      attempts on.

      **There were two dividers where hardware has one.** A sprite DMA takes 513 cycles, or 514
      when the `$4014` write lands on the wrong half of the CPU's get/put divider. We read that
      parity off a cell of our own, toggled from the clock closure — which runs once per *bus
      access*. Every cycle without an access behind it was therefore missed: the leftover cycles at
      the end of an instruction, and all five hundred odd of every transfer. One DMA is an odd
      number of cycles, so each transfer inverted the cell against the real divider, and the *next*
      transfer's length came out wrong half the time. The APU's frame counter has the same divider
      and keeps it honestly — `apu_test/4-jitter` pins its phase, by measuring the `$4017` write
      delay on alternating cycles — so the cell now mirrors that one instead of counting for
      itself, and `Apu::apu_cycle` is taken from the frame counter rather than being a third copy.
      A soft reset used to clear our copy while deliberately leaving the frame counter's
      free-running, which put them permanently out of step.

      **The structural change this entry called for turned out to be inert.** The trace against
      tetanes showed the transfer running inside the instruction that triggers it, where ours ran
      as separate steps afterwards, and that looked like the answer:

      ```
      ours     $E24F  STA $4014     ... then 513 stalled steps at $E252
      tetanes  $E24F  STA $4014     ... then $E252, then the IRQ handler at $E226
      ```

      It was rebuilt that way and measured. With the divider fixed, the ROM passes either way: the
      transfer's cycles land in the same place on the timeline whichever step owns them, and
      neither structure polls the interrupt shadow across them, so the poll that decides the next
      instruction sees the same thing. The restructure was reverted. What the trace was really
      showing was our *trace tool* printing one line per step.

      **Polling the interrupt lines through the DMA's cycles is wrong, and that is now measured
      rather than guessed.** It was tried and reverted three times on the grounds that it moved
      nothing; the third attempt, on top of the restructure, showed what it actually does. The
      decision about the halted instruction was taken before the halt began — at the end of the
      `$4014` write's second-to-last cycle — so advancing the shadow across the stall takes the
      interrupt an instruction early, and turns a four-cycle window in the table into a
      517-cycle one.

      **The gate for all of this is a unit test, not the ROM.** `4-irq_and_dma` takes twenty
      minutes and reports one word. `dma_interrupt_timing` in `nes_system.rs` reproduces the ROM's
      landing sequence byte for byte, sweeps `/IRQ` across every arrival cycle, and asserts the
      table from the ROM's own source comment — which is a recording from real hardware. It runs in
      a tenth of a second and names the cycle. `/IRQ` is raised from outside any device for it,
      because the APU and the mapper are the only things that really hold the line and either would
      fold its own timing into the measurement.

      `3-nmi_and_irq` is closer but not
      `2-nmi_and_brk` passes, which is the test CYCLE_ACCURACY.md records as having hung on two
      previous attempts at interrupt timing; it was the BRK halt above, not the interrupt work.
      **The combined `cpu_interrupts.nes` now says "All 5 tests passed."** `4-nmi_and_dma` has no
      ROM in this checkout — only its source — so the five singles that exist are the whole suite.
- [x] cpu_reset — 2/2. Reset is not power-on: it sets the I flag, subtracts three from the stack
      pointer and does nothing else. A, X, Y and the other flags survive it. The three are the
      interrupt sequence going through the motions of its pushes with the writes suppressed.
- [x] dmc_dma_during_read4 — 4/5 by the readme's own accepted outputs; only `double_2007_read`
      is left, and it is a PPU question rather than a DMC one. See the two closes at the end of
      this entry. The DMC's DMA runs
      *inside* the instruction rather than after it, which is what lets it land on a read: the
      processor is halted with that address still on the bus, so the read happens a second time and
      its side effects with it. Invisible for RAM; `$4016`'s shift register advances again and
      `$2007`'s address does.

      `dma_4016_read` walks the DMA across a `LDA $4016` one clock per run and prints which run the
      halt landed on. It went from `08 08 08 08 08` — no double read anywhere — to
      `08 08 08 07 08`, against hardware's `08 08 07 08 08`. **The double read is right and its
      alignment is one CPU clock late.**

      Two attempts at that clock, both inert, both worth not repeating:

      - Raising the DMC's byte request at the *end* of its tick rather than the start. The cycle
        that empties the sample buffer into the shifter ought to be the cycle the reader asks in,
        and asking first is asking a cycle late — but it moved no column of the table.
      - Deferring the halt by one read cycle, on the grounds that `/RDY` is sampled during a cycle
        and the processor finishes the one it is on. Also moved nothing.

      - Making the stall 3 cycles or 4 by the get/put parity, as the sprite DMA's length is. This
        one is not merely inert but *refuted*: with a varying stall the ROM never leaves
        `sync_dmc`'s fine-sync loop at all, spinning in its delay for good. That loop is written
        around "4 DMC wait-states" as a constant, so the stall is a constant four and the parity
        idea can be crossed off rather than retried.

      - Fetching the byte at the *end* of the halt rather than at its start, which is where it
        happens: the four cycles the processor waits are four cycles in which the DMC still has a
        byte outstanding, and `$4015`'s bit 4 reports exactly that. This one was a real ordering
        error and is fixed — the fetch was arriving four cycles early, taking `bytes_remaining`,
        bit 4 and the end-of-sample interrupt with it — but it moves no column either, so it is
        pinned by `cpu::dma_halt` rather than by these ROMs.

      - Clocking the DMC's timer from the APU's divider rather than at CPU rate, so its fetches are
        locked to one half of it as hardware's are. Every entry in the rate table is even, so the
        period halves exactly and the pitch does not move — only the phase. Tried *both* halves;
        the table is unchanged by either, which rules out the phase rather than merely one choice
        of it.

      Five places the missing clock is not: the request's position within the DMC's tick, the
      halt's position relative to the read, the stall's length, the fetch's position within the
      stall, and the timer's phase against the APU divider.

      **What the fault actually is, localised 2026-08-06 by differential trace against tetanes.**
      The ROM times a DMA onto a `LDA $4016`, which is four read cycles — opcode, operand low,
      operand high, then the data read that clocks the controller's shift register. Each of its five
      runs moves the DMA one clock later, so each run doubles a different one of those four reads,
      and only the run that doubles the *data* read changes the count the ROM prints. Instrumenting
      which address gets doubled gives this:

      ```
      iter 1  $E20D   opcode fetch
      iter 2  $E20E   operand low
      iter 3  $E20F   operand high
      iter 4  $4016   the data read      <-- ours prints 07 here
      iter 5  $E210   next opcode
      ```

      Hardware prints 07 on iteration **3**. So the halt lands exactly one read cycle later than it
      should, and every earlier description of this as "one clock" was right without being usable —
      the useful form is that the doubled read is one position too far along the instruction.

      Two more things this ruled out. Deferring the halt by a read moves nothing: the doubled
      address is identical either way, which is worth knowing before someone tries it a third time.
      And the two emulators agree on the *cycle count* of the stalled instruction — both take 8
      cycles for that `LDA`, so the stall itself is placed and sized correctly; only which read it
      attaches to is wrong.

      Also measured and unexplained: tetanes takes a constant 20,543 cycles between runs of the
      test, where this emulator takes 15,367, 18,853 and 22,235 — roughly one extra pass of
      `sync_dmc`'s 3,425-cycle fine-sync loop each time. Locking the DMC's timer to the APU divider
      does not change those numbers at all, which is what rules the divider out.

      **Where this leaves it, and why it is being stopped rather than continued.** Seven changes
      have been tried and reverted. The doubled read has to move one position *earlier*, so the
      DMC's request has to be raised one CPU cycle sooner — and there is no cycle to find inside the
      DMC. Its period is 428 and the request is already raised the instant the buffer empties; the
      APU is clocked before the bus access it shares a cycle with, so a request raised on cycle T is
      already visible to the read on cycle T, which is the earliest it can be seen at all.

      **Eighth attempt, and it explains why two earlier ones were inert.** tetanes checks its
      `DMA_HALT` flag *before* spending the read's cycle — `read()` calls `handle_dma(addr)` first,
      and that re-drives the same address for the halt — where this emulator asked after the cycle.
      Matching its shape changes nothing, and cannot: nothing happens between `end_cycle` of one
      read and `start_cycle` of the next, so "ask before read N" and "ask after read N-1" are the
      same instant. Both pick the first read after the request is visible. That is a structural
      fact about this clock, not a measurement, and it retires the whole family of "move the check"
      ideas — three of the eight attempts were versions of it.

      One thing worth stealing from tetanes regardless, when this is next opened: its `handle_dma`
      carries `let skip_dummy_reads = addr == 0x4016 || addr == 0x4017`. It knows the controller
      ports must not be clocked by the DMA's *dummy* reads, only by the halt's repeat. Nothing here
      models dummy reads at all, so it does not bite yet.

      That points the remaining cycle at the APU's alignment against the CPU, not at anything in the
      DMC or the DMA. And that alignment is what `apu_test` 9/9, `apu_reset` 6/6 and
      `blargg_apu_2005.07.30` 11/11 currently pin — `4-jitter` measures it directly, by writing
      `$4017` on alternating cycles. So one digit of one ROM is in tension with three suites that
      pass, and moving it blind would trade twenty-six passing tests for one. Worth returning to
      with a cycle-level APU trace from tetanes to compare against; not worth another guess.

      The mechanism itself is pinned by `cpu::dma_halt` rather than by these ROMs, which report five
      numbers and can say the halt landed on the wrong run but not whether it doubles the read at
      all.

      **Closed, 2026-08-06, by the cycle-level trace the last sitting asked for — and the missing
      clock was in none of the eight places already ruled out. It was the stall's own length.**
      Both emulators now carry an `RN_DMC_TRACE` ledger (ours committed; tetanes' a local
      uncommitted patch in that checkout) that stamps every `$4015` write, fetch request, halt and
      fetch with a CPU-cycle count. Diffed at `dma_4016_read`'s test section, the two agreed to
      the cycle on everything the previous sittings had suspected — the refill request fires 3406
      cycles after the restart write in both — and disagreed on exactly one number: the restart's
      stall. Ours spent 4 cycles; tetanes spent 3. One cycle, and the doubled read lands one read
      later, which is the whole 08-08-08-07-08.

      What hardware does, read off tetanes' ledger and its source together, is a *pair* of parity
      rules that only work jointly:

      - A `$4015`-started fetch is requested 2 cycles after a write on one parity and 3 after the
        other, so the request always surfaces on one parity. (Which delay goes with which parity
        label is a calibration, and it was recalibrated together with the timer's power-on phase
        when `sprdma_and_dmc_dma` was closed later the same day — see that entry.)
      - The fetch itself must land on a fixed parity, so the halt costs 3 cycles from one parity
        and 4 from the other. With the request parity pinned by the first rule, a *starting*
        sample stalls a deterministic 3 while a mid-sample refill stalls 4.
      - And the refill's own parity is only stable because the DMC timer free-runs from power-on
        (`$4015` gates the memory reader, not the timer) with the refill requested in the same
        cycle the shifter reloads — its period is even, so every reload shares one parity for the
        whole run. tetanes carries a `timer.cycle += 1` at reset with a FIXME saying DMA tests
        fail without it: that is this phase being pinned.

      This vindicates the old refutation rather than contradicting it: a 3-or-4 stall *without*
      the other two rules lets the refill parity float with when the game last enabled the
      channel, and `sync_dmc`'s loops — calibrated around "4 DMC wait-states" — hang the moment a
      refill comes up 3. That was re-measured on the way here: each partial mechanism hung the
      ROM in a different sync loop, and only the full pair passed it. `08 08 07 08 08 Passed`,
      hardware's column.

      The suites this was feared to trade against did not move: apu_test 9/9, apu_reset 6/6,
      blargg_apu 11/11, cpu_interrupts_v2 6/6, nestest 8991, baselines MATCH. The mechanism is
      pinned in CI by `apu::dmc_channel::tests::dma_placement` — three tests, each proven to fail
      with its rule reverted — since the ROMs cannot be committed.

      Of the two still failing, both now fail *identically to tetanes*: `double_2007_read` prints
      byte-for-byte the same output including its CRC (`22 33 44 55 66 / 33 44 55 66 77 /
      D84F6815`), and `dma_2007_read` gets one of five reset-alignment reruns wrong just as
      tetanes does (a different rerun — ours `22 33`, tetanes `44 55`). Their own sources say the
      results "depend on CPU-PPU synchronization at reset", which is a different axis from the
      DMC and one the reference has not solved either. A future sitting needs a reference that
      passes them before there is anything to compare against.

      **`dma_2007_read` is correct as of 2026-08-06, and "blocked on a reference" above was
      written off eight lines of its source.** Read in full, both ROMs list *several* accepted
      outputs, so the question was never "does it print what tetanes prints" but "is what it
      prints on the list". Ours was not:

      ```
      accepted:  11 22 / 11 22 / 33 44 or 44 55 / 11 22 / 11 22   crc 159A7A8F or 5E3DF9C4
      ours:      11 22 / 11 22 / 22 33          / 11 22 / 11 22   crc 7036EAAC
      now:       11 22 / 11 22 / 44 55          / 11 22 / 11 22   crc 5E3DF9C4
      ```

      **The halted processor keeps driving the address**, and every cycle of the halt is another
      read with its side effects — the thing this file recorded as "tetanes has
      `skip_dummy_reads`; we don't model dummy reads at all" and left there. For `$2007` each one
      rotates the read buffer, so a halted `LDX $2007` should come away two to three bytes
      further along than an unhalted one. Ours performed exactly one extra read.

      The count came from a `$2007` ledger (`RN_2007_TRACE`, in both emulators) rather than from
      arithmetic, and the arithmetic would have got it wrong: tetanes' burst is four reads at
      T, T+1, T+2, T+4 — the gap being its DMC fetch — and driving every halt cycle but the fetch
      gives *five*, which prints `55 66` and is off the table. Both emulators spend the same five
      cycles on the halted read; they divide them differently. tetanes spends one on the real
      read, where this emulator performs the resumed read without a cycle of its own because the
      halt has already been charged for it. So two of the halt's cycles are spoken for, and
      `stalled - 2` of them drive the address. Both our stall lengths then land on the list: a
      4-cycle halt prints `44 55`, a 3-cycle halt `33 44`.

      Pinned by `cpu::dma_halt`, which gained the read count for both stall lengths and the
      `$4016`/`$4017` exemption — the last being what keeps `dma_4016_read`'s two reads at two.
      The old test asserted exactly two reads at any address and failed on this change, which is
      the test doing its job; it now encodes the rule instead.

      **Neither of these two ROMs can ever print "Passed"**, which is worth knowing before
      anyone chases their `NOPROTO` again: unlike `dma_4016_read` they end in `jsr print_crc` /
      `rts` with no `check_crc` and no `tests_passed`. They are informational — the verdict is
      the printed table against the readme's list, and the runner has no channel for that.

      **`double_2007_read` is decoded but not fixed, and it is not a DMC test at all** — it
      includes `shell.inc` rather than `common.inc`, so no DMA runs in it. It reads
      `lda $20F7,x` with `x=$10`, which page-crosses: the 6502 reads the unfixed address `$2007`
      and then the real `$2107`, and both mirror to `$2007`. Two reads on consecutive cycles.

      ```
      accepted line 2:  22 44 55 66 77 | 22 33 44 55 66 | 02 44 55 66 77 | 32 44 55 66 77
      ours:             33 44 55 66 77
      ```

      Ours advances the buffer fully on both reads, so `LDA` keeps `$33`. Hardware keeps `$22`:
      the second read sees the buffer *before* the first read's VRAM fetch landed, while the
      address still increments twice. The `02`/`32` variants are that fetch caught mid-flight —
      `$32` is the high nibble of `$33` with the low nibble of `$22` — and `22 33 44 55 66` is
      the second read being dropped outright.

      **Measured 2026-08-07, and the "different dots" guess above is wrong.** The `$2007` ledger
      records the scanline and dot of every read, and the two ROMs turn out to present *identical*
      conditions:

      ```
      double_2007_read  scanline 242, dots 113 and 116   3 dots apart, rendering off
      dma_2007_read     scanline 229, dots 278, 281, 284 3 dots apart, rendering off
      ```

      Consecutive CPU cycles, three dots apart, rendering off, in both. Yet hardware advances the
      buffer fully in `dma_2007_read` — its accepted `33 44` needs three full advances, and a
      stale second read would print `11 22`, which is what its *unhalted* rows print — and does
      not in `double_2007_read`. So the discriminator is neither the spacing, nor rendering, nor
      "a repeat is stale". Both hypotheses this entry has carried are refuted by the other ROM.

      **The power-on alignment was tried too, and cannot matter yet.** Sweeping the extra
      startup dot over 0, 1 and 2 changes neither ROM's output, and the reason is structural
      rather than a measurement: `read_data` is atomic here — it returns the buffer and refills it
      in the same instant — so there is no interval for a phase to fall inside. No alignment can
      produce staleness until the fetch has a duration. That also means the "depends on CPU-PPU
      synchronization at reset" in both headers cannot be reproduced at all in the current model,
      which is the more useful way to state what is missing.

      So this is genuinely blocked, and now with evidence rather than an assumption: the rule that
      separates these two ROMs is something neither this emulator nor the reference models, and
      tetanes prints our exact wrong line, CRC included. It needs an emulator that passes it, or
      hardware. Not another hypothesis.

- [x] sprdma_and_dmc_dma — 2/2, from 0/2, closed 2026-08-06 in the sitting after the ledger work
      above, with the same instruments. Three mechanisms, each measured before written:

      **The collision itself was not modelled.** A DMC fetch that comes due while the sprite DMA
      holds the bus is served from *inside* it — the transfer's cycles stand in for the halt and
      dummy reads, the fetch takes one read slot, and one alignment cycle restores the cadence:
      two cycles, stretching the transfer, instead of a 3-4 cycle stall queued behind it. Without
      this the ROM read `528` for every T+ row where hardware varies 525-528.

      **The DMC timer's power-on phase is a hardware constant, and ours was the wrong one.** With
      the steal in place, every measurement came out exactly one T+ row shifted from hardware's
      table. The ledger decomposed it: both emulators pick the long (514) sprite DMA on the same
      *divider* parity, but the same program point lands on opposite absolute parity — because
      the program is synced to the DMC grid and the grid-vs-divider phase differed by one. That
      phase is `timer_value: 1` at power-on (tetanes' `timer.cycle += 1` FIXME is the same
      constant), and moving it flips which parity refills land on, so the start-delay pair and
      the stall mapping had to be relabelled *with* it — the wrongly-paired flips were each tried
      and each hung a different sync loop or brought back `08 08 08 07 08`. The consistent triple
      passes `dma_4016_read`, both sprdma ROMs and `cpu_interrupts_v2` at once.

      Also measured on the way: flipping the sprite DMA's 513/514 parity *alone* trades
      `sprdma_and_dmc_dma` for `4-irq_and_dma` — the two ROMs read the same physical parity
      through different anchors, `4-irq_and_dma` from a fixed boot-relative write and sprdma
      through the DMC grid. The grid phase is what reconciles them; the length rule was never
      wrong.

      **A fetch due in the transfer's final pair costs `stall - 2`.** It cannot take a slot
      inside the transfer, but the transfer's cycles have already paid its halt and dummy read,
      so the stall that serves it right after is two short of a cold one. `_512` lands the fetch
      exactly there and reads 524 where a cold stall gave 526; blocking the steal without the
      discount read 526, with the steal 525, hardware 524.

      Not pinned by a unit test: all three mechanisms are system-level interleavings visible only
      through these ROMs, which cannot be committed. The `dma_placement` tests pin the parity
      machinery they calibrate against, and `RN_REQUIRE_ROMS` makes the ROM gate enforceable
      where the ROMs exist.

- [x] MMC1_A12/mmc1_a12 — **the emulation is right.** Its picture is structurally identical to
      tetanes', so nothing here is wrong with it; what fails is our ability to *read* its screen. It
      draws with its own character set rather than ASCII, so the nametable decodes to noise —
      "42$ 4;# +;1 3." and the like — and the screen reader used to search that noise for a `$`,
      find one, and report `FAILED #2`. Reading a result code now requires it alone on its line.

      Worth keeping as its own lesson: `screen.rs` already said "a wrong verdict read off the screen
      is worse than no verdict at all", and had the rule aimed only at false *passes*. A false
      failure is the same fault facing the other way — it sends someone after a bug that is not
      there, and looks exactly like a real failure in a summary line. This one was chased today.
- [x] blargg_apu_2005.07.30 — 11/11, from 9/11. `10.len_halt_timing` and `11.len_reload_timing`
      are one sentence each from the suite's own readme, and both turn on a coincidence a single
      CPU cycle wide:

      - "Changes to length counter halt occur after clocking length, not before." A halt written on
        the clock cycle is too late to stop that clock; the counter is decremented one last time and
        the halt applies after.
      - "Write to length counter reload should be ignored when made during length counter clocking
        and the length counter is not zero." With something left to clock the reload is dropped.
        With the counter already at zero it is accepted — *and the clock that was about to happen
        decided against the zero it found, so it must not take one off the reloaded value*. That
        last clause is the whole difference between failing test 4 and passing.

      **The direction is the part worth remembering.** The APU is advanced before the bus access it
      shares a cycle with, so a write hardware sees as landing *on* the length clock arrives, from
      this emulator's side, on the cycle *before* it. The first attempt asked the question backwards
      — a flag set by the clock just past — and moved the ROM from failing test 4 to failing test 3,
      trading one wrong answer for another. It has to be asked ahead: `clocks_length_next`.

      Found by sweeping the cycles either side of the clock in a harness, not by reading the ROM's
      verdict: each run takes ten minutes and reports `$03`. Four unit tests hold it, three of which
      fail without the change.

- [x] cpu_exec_space — 2/2. Both ROMs execute code *from* I/O space and follow where the open bus
      leads, so every wrong bit becomes a wrong opcode. Three separate faults, all of them found by
      differential trace against tetanes rather than by reading our source:

      - **`$4014` is write-only, and we answered reads of it** with the last page written to it.
        The trick the test turns on is that `JMP $4014` leaves `$40` — its own target's high byte —
        as the last value on the bus, so the opcode fetched at `$4014` is `$40`, which is `RTI`,
        which returns to where the jump came from. We handed back the DMA page and ran off into
        whatever that decoded to.
      - **A controller port does not drive its top three lines.** `$4016` and `$4017` put the shift
        register's output on the bottom bits and leave bits 5, 6 and 7 floating, so they hold what
        the bus last carried — the same `$40`, by the same route. Games mask the button bit out and
        never notice. This wanted a seam rather than a special case, so `Addressable` grew
        `open_bus_mask`: which bits a component does *not* drive, with the bus merging the rest.
      - **Our own runner was breaking the ROM.** `rom_test` peeks `$6000` after every instruction to
        see whether a ROM has finished, and did it through a real read — which left the status byte
        on the open bus each time. Invisible to every other ROM in the tree; fatal to this one. It
        reported `Failed #2` against an emulator that was getting the answer right, and it is why
        the trace and the ROM disagreed for a long time: `rom_test trace` does not poll, so under
        tracing we ran in step with tetanes for 2.5 million instructions while `rom_test run`
        insisted we had failed at 286,000. `Addressable` now has a real `peek_byte`, which the bus
        implements without touching the open bus at all.

      Each was checked by reverting it alone: any one of the three puts the ROM back to `FAIL(02)`.

      Two things in the plumbing are worth keeping in mind, because both failed silently:

      - `Bus` needed its own `handles_write`. Inheriting it from `handles_address` meant that as
        soon as a component claimed an address in one direction only, the bus reported that address
        unhandled and dropped the write — which silently disabled every sprite DMA.
      - `CpuWrapper` and the DMA wrapper have to *forward* `peek_byte` and `handles_write` rather
        than inherit them. The default `peek_byte` is a real read, which is the one thing a peek
        must not be, and the first attempt at the fix changed nothing at all because of it.
- [x] cpu_dummy_writes — 2/2. A read-modify-write makes *two* writes: the processor has nowhere to
      hold the result while it computes, so it spends that cycle putting back what it just read.
      `INC` and `DEC` had this right through `modify_memory`; every shift and rotate with a memory
      operand went through a path that wrote once, which is the `0E 2E 4E 6E 1E 3E 5E 7E` the ROM
      printed.
- [x] cpu_dummy_reads — 1/1, once CNROM existed to load it.


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
- [x] Save and restore the APU. A snapshot carried the CPU, RAM, cartridge RAM, PPU and mapper and
      nothing of the sound hardware, so a restored machine was silent until the game happened to
      rewrite every register. Found while trying to reproduce a timing bug from a snapshot, where
      an idle DMC is precisely the difference that matters. Snapshots written before this still
      load: the field is optional and defaulted, rather than refusing every save anyone already
      had. Not saved, deliberately: the output device's sample rate and resampling accumulator,
      which belong to the sound card a snapshot is restored *onto*, and the output filter's
      memory, which settles inaudibly in milliseconds.
- [ ] MMC2 (9), Color Dreams (11) — each small, none needed by anything tried so far. CNROM (3) is
      done and was listed here as outstanding long after it landed.
- [x] MMC3's A12 timing: the counter is clocked from the real PPU address bus, not from a count of
      scanlines. `3-A12_clocking` and `4-scanline_timing` both pass, the latter to PPU-clock
      accuracy. See the fetch pipeline below for what made it possible.

## MILESTONE 10: Full Desktop System [T10]
- [x] Load and play commercial ROMs
- [x] Save states, with tests that assert a restored machine continues identically
- [x] Fullscreen display, overscan cropping, frame dump with the PPU state that produced it
- [ ] Controller 2 surfaced in the key mapping (wired in the core already)
- [ ] Audio: verify against real game music now that the pipeline works
- [ ] Test with a wider variety of games

### [UI] Display scaling and filters [T10]
The picture is 256x240 and every screen it is shown on is larger, so something has to decide what
happens between the pixels. Today nothing does: `PixelDisplay` hard-codes
`egui::TextureOptions::NEAREST` and multiplies by a `zoom` float, so the window can land on a
non-integer scale and the sampler then duplicates some source rows and not others — even
nearest-neighbour is only uniform when the factor is a whole number. Fullscreen and overscan
cropping already exist and this is the piece between them.

- [ ] A `ScalingMode` the display widget takes, replacing the hard-coded `NEAREST`, with the choice
      surfaced in the UI and saved with the other preferences
- [ ] Integer scaling: pick the largest whole multiple that fits and letterbox the remainder, so no
      row or column is ever wider than its neighbours. This is the fix for the uneven-pixel
      artefact above and should be the default
- [ ] Correct aspect ratio as an option: the NES pixel is not square (8:7 on NTSC), so 256x240
      belongs on screen at roughly 292x240. Independent of the scaling mode, and a case where an
      integer *vertical* scale with a stretched horizontal one is the usual compromise
- [ ] Bilinear, for anyone who prefers it — one line once the mode exists, and worth having as the
      baseline the sharper filters are compared against
- [ ] Scale2x/AdvMAME2x, then Eagle or hq2x/hq3x: pixel-art scalers that read a 3x3 neighbourhood
      and interpolate along detected edges rather than uniformly. Pure functions from one frame
      buffer to another, so they test directly — a handful of known input patterns and their exact
      expected output, no emulator needed
- [ ] A CRT-style pass (scanline darkening at least, phosphor/aperture-grille shape if it earns its
      keep), which is a different thing from interpolation and should be composable with it
- [ ] Decide where the work happens. All of the above is per-frame over 61440 pixels; on the CPU
      that is fine at 2x and questionable at hq3x, so measure before committing. A shader is the
      other answer and is also what [T10] Web Integration will want, since WebGL is already on that
      list — a filter written as a fragment shader would be shared rather than written twice
- [ ] Benchmark the chosen path and confirm it does not cost frames on a 60Hz budget

### [CPU] Advanced Instructions [T9]
- [x] Every one of these — flags, branches, transfers, stack, increments, shifts, compares,
      logical operations and their tests — has been done for a long time, and this list sat open
      the whole while. Re-measured 2026-08-07: **nestest 8991/8991**, `instr_test-v5` 18/18,
      `instr_misc` 5/5, `instr_timing` 3/3, `nes_instr_test` 11/11, `branch_timing_tests` 3/3.
      Those suites do not pass with `CLD` missing.

      Left ticked rather than deleted as a specimen. Nine unticked boxes claiming the CPU could not
      branch or push, in a file whose own header warns that its numbers are claims until re-run —
      and every one of them was read past, repeatedly, by people who then quoted this file. A
      checklist that is not re-measured does not merely go stale; it actively misinforms, because
      its format promises a status it is no longer reporting.

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
- [x] Implement sprite zero hit detection — `sprite_hit_tests` 7/11, from 0/11. It was implemented
      all along; the pattern tables were blank. See the CHR RAM entry below.
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
Counted from the checkboxes themselves rather than kept by hand, which is why several figures moved
when they were last recounted: the totals had drifted as items were added, and three tracks read 0%
long after their work had landed. An item belongs to the milestone it sits under, unless its own
line carries a track tag.

- Track 1 (Memory Visualization): 100% complete (70/70 tasks)
- Track 2 (PPU Pixel Display): 100% complete (40/40 tasks)
- Track 3 (Basic Sprite Rendering): 100% complete (73/73 tasks)
- Track 4 (Animated Sprites): 100% complete (47/47 tasks)
- Track 5 (Input Controllers): 92% complete (35/38 tasks) - Controller input is fully implemented with keyboard mapping support; what remains is the D-pad movement demo ROM
- Track 6 (Basic Sound Output): 100% complete (48/48 tasks) - APU registers, pulse channel, volume/mute controls, and audio output through CPAL. The output pipeline was rebuilt (resampling, non-linear mixing, APU clock divider, audio-clock pacing) — see [AUDIO_PLAN.md](AUDIO_PLAN.md)
- Track 7 (Complete Audio System): 76% complete (44/58 tasks) - All audio channels implemented (pulse, triangle, noise, DMC) with hardware non-linear mixing, output filters, envelope generators, sweep units and length counters, and the frame IRQ, 5-step mode and DMC bus reads are all done. What remains is $4017 write and reset timing, and the per-channel test ROMs
- Track 8 (Conformance & Test ROMs): 72% complete (23/32 tasks) - PRG-ROM loading, the headless runner and the official instruction set are done; the suites that still fail are mostly waiting on the two rewrites at the end of this file. See [CONFORMANCE_PLAN.md](CONFORMANCE_PLAN.md)
- Track 9 (Mappers & Cartridges): 83% complete (10/12 tasks) - NROM, MMC1, UxROM, MMC3 (with A12 timing) and AxROM, saved and restored with snapshots. CNROM, MMC2 and Color Dreams are unstarted and unneeded so far
- Track 10 (Full Desktop System): 4% complete (3/74 tasks) - Commercial ROMs, save states and fullscreen work; the bulk of the count is the CPU/PPU/debugger sections filed here, much of which is implemented but tracked in earlier milestones' boxes
- Track 11 (Web Integration): 0% complete (0/41 tasks)
- Additional Areas: 8% complete (15/177 tasks) - Including cycle-accurate timing, background rendering, testing, edge cases, distribution, extended features, legal considerations, documentation, performance optimization, demo ROMs, and the two rewrites

**Total Progress: 408/710 tasks complete (57.5%)** 🚀

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
- [x] Model every cycle as the bus access it is on hardware, so accesses and cycles agree. Done:
      `rom_test cycles` reports "every opcode accounts for all of its cycles" across all 8991 of
      nestest's instructions, from 6463 short when this was written. Pinned by
      `every_instruction_accesses_the_bus_once_per_cycle`.
- [x] The page-cross penalty, which was never added to any instruction's cycle count.
      `crosses_page_boundary` and `get_additional_cycles` both existed, were both correct, and were
      called by nothing but their own tests — `execute` only ever added cycles for branches. So
      `LDA $02FF,X` with X carrying reported four cycles where hardware takes five. It went
      unnoticed because the *bus* was right all along (the addressing mode performs the fix-up
      access, so the PPU is advanced for it and nothing rendered looked wrong) and because nestest
      compares registers and never looks at a cycle count.
      Fixed by taking the cycle from what the addressing actually did rather than from a table: the
      fix-up access *is* the extra cycle, so the count cannot drift from the bus. A store performs
      that access whether or not the index carries and its base length already says so, which is
      why the rule cannot simply be "add one when the pages differ".
- [x] Interrupts sampled before the last cycle — `1-cli_latency` passes, the first in that suite
- [x] The computed `poll_at` replaced by a one-cycle-delayed shadow of both lines, and the poll
      moved from the instant of the bus access to the end of the cycle, one dot later
- [x] NMI hijacking BRK, and no polling inside an interrupt sequence — including that the sequence
      leaves no shadow behind it, so the handler's first instruction always runs
- [x] Branch polling: a taken branch ignores an IRQ that only became eligible during its own last
      cycle, so the instruction at the target runs first. The last of the three documented
      exceptions, and the only part of the sampling rule that does not fall out of the shadow.
      Proved by a unit test rather than by `5-branch_delays_irq`, which does not move on it: that
      ROM fails in its first sub-test, which measures `JMP` and never reaches the branch cases.
- [x] The rest of `cpu_interrupts_v2` — done. This line said the remaining tests were "blocked on
      the PPU, not on the CPU", waiting on cycle-exact CPU/PPU alignment. That alignment landed, and
      the suite is 6/6 as of 2026-08-06: all five singles and the combined ROM. The last of them
      turned out not to be a CPU/PPU alignment question at all but two dividers where hardware has
      one — see `4-irq_and_dma` above.
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
- [x] Background and sprite fetches issued per dot, in hardware's order

      Attempted background fetches alone, driving the mapper from a filtered A12 instead of from a
      scanline count. It regressed: Super Mario Bros 3 lost a third of its picture and mmc3_test
      fell from 3 to 2. The reason is that MMC3's A12 rise comes from the *sprite* pattern fetches
      at dots 257-320, which use $1000 while the background uses $0000 — with only background
      fetches the line never rises in the pattern the mapper is counting.

      So the fetches cannot be done in halves: background and sprite fetches have to arrive
      together before A12 means anything, and the switch away from scanline counting has to happen
      in the same change. Preserved in `scratchpad/ppu-fetches/`.

      Done, with both together, and mmc3_test went 3/6 to 5/6. SMB3's frame is byte-identical
      throughout, which is the gate that matters: the address bus changed and the picture did not.

      Four things the earlier note had wrong or did not know:

      1. **The nametable fetches do not drive A12 high.** $2000 and $23C0 both have bit 12 clear,
         so they hold the line *low* — which is what creates the four-dot gaps between one
         sprite's patterns and the next, and those gaps are what the filter exists to ignore.
         There was never a filter-resetting problem to solve.
      2. **The address bus leads the read it serves by one dot.** The sprite pattern fetch is at
         dots 261-262 and its address is asserted at 260, which is the figure the documentation
         quotes for when the counter clocks. A lead of zero or of two both fail
         `4-scanline_timing`; one passes it outright. This was measured, not reasoned — the sweep
         was over lead and filter threshold together.
      3. **The filter is ten dots and the boundary is sharp at the low end.** Nine fails, because
         the gap between one line's last prefetch and the next line's first background fetch comes
         to exactly nine dots and hardware does not count it. Ten to sixty-six all pass; ten is the
         physical figure (~3 CPU cycles) and one dot clear of the only nearby edge.
      4. **Rendering being on does not mean the counter runs.** With both pattern tables at $0000
         nothing is ever fetched above $0FFF and the line never rises at all. A unit test asserted
         the opposite — it was written against the scanline model, where the arrangement of the
         pattern tables cannot matter. Corrected, and the new fact pinned by its own test.

      A fifth thing fell out of the same mechanism rather than out of the fetches: `$2007`'s
      address increment drives the bus too. A program pointing at $0FFF and reading raises bit 12
      by incrementing to $1000, having read from an address that never had it set. That alone was
      the whole of `3-A12_clocking`'s remaining failure.

- [x] The fetch machinery itself: nametable, attribute and both bitplanes into latches, loaded into
      shift registers, with fine X selecting a bit. Tested directly; does not drive pixels.
- [x] Switch pixel output to the shift registers

      Attempted and reverted. 23% of Super Mario Bros 3's picture changed — not a horizontal shift
      of it, and not fixed by correcting the reload dots to 9, 17 ... 257, 329 and 337, which
      changed nothing measurable.

      What it turns on is that a 16-bit shifter takes pixels from bit 15 while reloads enter at the
      low byte, so a tile becomes visible *eight dots after* it is loaded. Making that chain line up
      across the line boundary is the whole difficulty: the prefetch groups run at 321-336, shifting
      stops at 337, and the line is 341 dots, so the two prefetched tiles have to arrive at the top
      of the register exactly as dots 1 and 9 of the next line come round. Getting that right needs
      the register contents traced dot by dot against a known-good sequence rather than reasoned
      about — a test that walks one tile through the pipeline and asserts which dot it appears on.

      Write that test first, then switch. Preserved in `scratchpad/ppu-pixel-switch/`.

      Done, and it disproved the diagnosis above: the alignment is already right. A tile appears on
      dots 1-8 and the next on 9-16, exactly as it should.

      Comparing the two renderers directly on a synthetic scene found the real cause in one line.
      They disagree by exactly one tile: the pipeline draws the tile at the address in `v` starting
      at x=0, the per-line renderer draws it from x=8. Every tile displaced by eight pixels, which
      is why a quarter of the frame changed and why it did not look shifted — each tile moved into
      its neighbour's place.

      Resolved: the pipeline was right. The per-line renderer was reading `v` two tiles past the
      start of its line, because the prefetch groups advance it. It is now handed the address
      captured at dot 257, and the two agree pixel for pixel — the comparison test is no longer
      ignored.

      The switch itself was then attempted again and reverted a second time. It gets to 2356 pixels
      of 61440 rather than 14268, and the remainder is *not* background: the comparison test proves
      the two background paths agree on a synthetic scene. It is sprite compositing. Drawing the
      background dot by dot means sprites can only be composited once the line exists, at dot 257,
      which also moves when the sprite-zero hit is reported — from the start of a line to two
      thirds of the way along it. Super Mario Bros 3's split responds to that timing, so the
      picture moves.

      Selecting the sprites at the line's start and drawing them at 257 changes nothing, which
      rules out object memory being sampled later and confirms it is the hit timing.

      So the switch is blocked on sprite evaluation moving to its real dots — the hit has to be
      reported as the beam reaches the overlapping pixel, not at either end of the line. That is
      the next item below rather than a separate problem, and the two have to land together.
      Preserved in `scratchpad/ppu-switch-attempt2/`.

      Landed on the third attempt: both layers are drawn a pixel at a time and the hit is reported
      as the beam reaches the overlapping pixel. This box stayed unticked afterwards, which is why
      the note above still reads as though it were pending.

      **Outstanding: a flickering line at the status-bar split in Super Mario Bros 3.** Reported
      from the running emulator during a level, and since reproduced exactly, from a save state
      loaded headlessly.

      **Measured, 2026-08-05, and it is far smaller than this entry used to claim.** The sentence
      here read "rows 193 and 194 alternate between mostly black and mostly backdrop", which was
      written before most of this file's fixes and is no longer true. Across six consecutive frames
      from the save state:

      - Row 193 holds 208-216 black pixels and 40-48 of sky. The boundary between them moves by
        exactly **8 pixels — one tile** — on some frame transitions and not others.
      - Row 194 changes by 6 pixels on some frames, and those pixels are sprite colours. That is a
        sprite animating, not the artifact.

      So it is an eight-pixel boundary wobble on a single row, intermittently. It moves in eight-
      pixel steps because the background is fetched in eight-dot groups: a frame is 29780.67 CPU
      cycles, the CPU/PPU phase drifts about two dots a frame, and the drift only shows when it
      crosses a fetch group. On hardware the burst lands in hblank where the same drift moves it
      harmlessly; here it lands 70 dots earlier, inside the visible line, so the drift moves
      something visible.

      **Not established: whether row 193's static content is right either.** With the burst 70 dots
      later the `$2001 = $00` would land around dot 305 rather than 235, so line 193 would render as
      playfield for its whole visible width rather than being mostly black. That is a prediction
      from the same arithmetic that gives the 23 cycles, not a measurement, and it needs the same
      reference trace to settle.

      Bisected by loading that state under each commit: it arrives with this one and is unchanged
      by everything after it. The A12 work and the per-dot sprite evaluation are both ruled out —
      1400 frames of a driven run are byte-identical across each of them.

      What is actually happening, traced rather than guessed:

      - The MMC3 IRQ is raised at scanline 191 dot 260, and stably so — the same dot every frame.
      - The handler's first `$2006` write lands at scanline **193, dot ~190**. That is 1.7
        scanlines after the interrupt, about 197 CPU cycles, and it is inside the *visible* part
        of the line rather than in hblank.
      - `v` changes there and then, so the rest of line 193 is fetched from the new address. The
        write dot drifts a few dots per frame as the CPU and PPU realign, which is the flicker:
        the boundary moves, so the row alternates.
      - The final pair of the handler's writes, at dots 259 and 271, does land in hblank and is
        fine. It is the earlier pairs that corrupt the line.

      This is not a rendering fault, which is why nothing in the renderer fixes it. A mid-line
      `$2006` write *does* corrupt the rest of the line on hardware; the picture is right on
      hardware because the write does not land there. The per-line renderer hid it by drawing every
      line from the address captured at dot 257, so mid-line writes could not affect the line they
      fell in — correct-looking for the wrong reason.

      So the question is why the handler reaches its first write ~197 cycles after the interrupt,
      which is either the interrupt arriving too early or the CPU being too slow to the write. That
      is the cycle-exact CPU/PPU alignment already named above as what blocks `cpu_interrupts_v2` —
      the two rewrites meeting again. Reproduce with a save state on that screen and diff the split
      rows frame to frame; a scratch harness for it is straightforward and was thrown away.

      **Switched back on**, on hardware's evidence rather than on the reasoning above. With the
      per-dot path enabled `sprite_hit_tests` goes 7/11 to **11/11**, including all four timing
      ROMs, which measure to the dot when the hit is reported; no other suite moves either way; and
      on a fresh boot of Super Mario Bros 3 or Donkey Kong the two paths differ on scanline 0 alone,
      where the per-dot path is again right — sprite evaluation does not run on the pre-render line,
      so no sprite can appear on the first line and the per-line path drew them there.

      The judgement that follows: the per-line renderer was *concealing* the split fault, not
      avoiding it, and trading a hidden fault in every game's sprite-zero timing for a visible one
      in a single game's split is the wrong way round. The conditions below were written when the
      comparison was between the two paths rather than against hardware.

      - [x] The two paths agree pixel for pixel on a static scene. They now do. They did not: the
            shift registers were reloaded *after* the dot's pixel was taken, so the first pixel of
            every tile came from the tile before it. Four pipeline tests had encoded the fault,
            because they sampled the shift registers between ticks rather than the pixels actually
            drawn — a different instant, and the reason the note above concluded the alignment was
            already right when it was not.
      - [ ] The interrupt lands on the right cycle, so the split's write burst starts at dot 257
            rather than dot ~190.

            **What is actually happening — and the entry above this one had it wrong.** The
            `$2006` writes landing in the middle of line 193 do *not* corrupt it, because nothing is
            being drawn there: the game turns rendering **off** for that span. Measured 2026-08-06
            with a new `$2001` diagnostic, over three consecutive frames from the save state:

            ```
            frame 3   scanline 193 dot 235 -> $00 rendering OFF   scanline 194 dot 212 -> $18 ON
            frame 4   scanline 193 dot 233 -> $00 rendering OFF   scanline 194 dot 210 -> $18 ON
            frame 5   scanline 193 dot 241 -> $00 rendering OFF   scanline 194 dot 218 -> $18 ON
            ```

            Forced blanking, which is how a game buys itself video-memory time outside vblank. The
            black span on row 193 is deliberate, and the eight-pixel wobble is *that edge* moving —
            235, 233, 241 — not a corrupted fetch. Everything this entry previously said about a
            mid-line `$2006` write redrawing the rest of the line was describing a mechanism that
            is not in play here.

            Where the wobble comes from is now a much smaller question. The MMC3 IRQ is PPU-locked
            and fires at 191:261 every single frame — verified by logging every A12 rise, all of
            them at dot 261 from `$1FF7`, a sprite fetch. So the handler starts at a fixed PPU
            position. What is *not* fixed is the CPU's latency in taking it: the game is spinning in
            a three-cycle loop at `$96F4`, so the interrupt is taken up to three cycles — nine dots
            — after it fires, and which of those three the CPU is on drifts as the CPU/PPU phase
            does. Nine dots is the observed spread.

            That leaves a real question rather than a guess: **does hardware wobble here too?** The
            same three-cycle loop and the same phase drift exist on a real console, so the edge must
            move there as well unless the blank span is positioned so that nine dots of movement
            fall somewhere invisible. Answering it needs a reference at this scene, which needs an
            input route into a level — `--into-level` reached only the world map when this was
            written; the route was completed and the question answered in the close below.

            **The handler's timing is fixed code, so the fault is not the CPU's.** Traced from the
            save state, 2026-08-06. The in-level handler at `$F77B` ends with:

            ```
            $F7CD  STA $E001      ; acknowledge the MMC3 IRQ
            $F7D0  LDX #$0C       ; a hard-coded delay of twelve
            $F7D2  NOP
            $F7D3  DEX
            $F7D4  BNE $F7D2
            ```

            A fixed delay, not a poll — the developers tuned that twelve so the `$2006` burst lands
            in hblank on real hardware. Nothing about it can vary between emulators. The handler
            enters at scanline 191 dot 288 and reaches its first completed `$2006` write 593 dots
            later, at 193:199; for that write to land at 193:257 the handler would have to enter 58
            dots later, at about 192:5, which needs the IRQ at about 191:325 rather than 191:261.

            So the whole 58 dots are in **when the MMC3 IRQ fires**, and none of it is in the CPU or
            the handler. That retires "the CPU is too slow to the write", which this entry has
            carried as one of two possibilities since it was opened.

            What that does *not* explain: the A12 rise really does look right. `address_on_bus`
            derives the address from the actual fetches — `CTRL_BACKGROUND_PATTERN` for background
            and the real `sprite_patterns` for sprites — so a game with the two tables swapped gets
            its rise at dot 321 rather than 261 without anything being special-cased. Dot 261 is
            also what Mesen computes. The next thing to measure is the counter itself: which
            scanline the latch is aimed at, and whether the reload and the decrement agree with
            hardware about the line the IRQ belongs to.

            **The map screen is not the fault and can stop being looked at.** Its split handler is a
            different routine at `$A83C`, writes `$2005` rather than `$2006`, and runs entirely in
            hblank at dots 277-338. Only the in-level path at `$F77B` goes wrong.

            **The means to see this is now committed rather than rebuilt each time.** Every
            completed `$2006` write is recorded with the dot it landed on, and `rom_test frame`
            prints them, saying for each whether it fell in the visible picture, in hblank, or in
            vblank where nothing is drawn. This entry has been opened three times and each time
            began by writing a scratch harness and throwing it away; the dot is the whole
            measurement, and it should not have to be re-derived. Current output from the save
            state, 2026-08-06:

            ```
            scanline 193  dot 199  -> $0000   VISIBLE — redraws the rest of the line from here
            scanline 193  dot 223  -> $0000   VISIBLE — redraws the rest of the line from here
            scanline 193  dot 271  -> $0B00   hblank
            ```

            Everything else in the frame is a vblank write and harmless. Two writes land inside
            line 193 and one makes it to hblank; the flicker is those first two moving as the
            CPU/PPU phase drifts.

            **Re-measured after the CPU/PPU alignment work of 2026-08-04/05, which moved the
            interrupt poll by a dot, deleted `ADDRESS_BUS_LEAD_DOTS`, closed the cycle/access gap to
            zero and added the eight settle cycles, and again after the interrupt, DMA and sprite
            evaluation work of 2026-08-06. None of it fixed this.** Rows 193 and 194 still wobble by
            eight pixels between frames.

            Two things changed, and only one of them is good.

            **The picture is nearly right now.** Switching the per-dot path on changes 71 pixels of
            61440, against 2356 when it was last reverted, and every one of them is in rows 193 and
            194 — the split itself. The sprite compositing and hit timing that blocked the second
            attempt are no longer the problem; this one fault is all that is left.

            **The burst still starts early, and marginally more so.** Measured from the save state:

            ```
            MMC3 IRQ raised   scanline 191 dot 261   (260 before; ADDRESS_BUS_LEAD_DOTS moved it)
            $2006 x4          scanline 193 dots 187, 199, 211, 223   — want the first at 257
            $2001 = $00       scanline 193 dot 235
            $2006 = $0B,$00   scanline 193 dots 259, 271
            $2001 = $18       scanline 194 dot 212
            ```

            IRQ to first write is 608 dots, or 202.7 CPU cycles. It should be 678 dots, 226 cycles.
            So the burst is **23 CPU cycles early**, against 21 before.

            **And that gap cannot be interrupt delivery latency**, which is the thing this entry has
            always blamed and the reason the two rewrites were called coupled. The most a 6502 can
            take between a line being asserted and a handler's first instruction is the rest of the
            current instruction (at most seven cycles), one for the shadow, and seven for the
            sequence: about fifteen. With the handler hand-counted at 190 cycles to its first write,
            190 + 15 = 205 is the *slowest* we could possibly be, and we measure 203. There is no
            room in the CPU for the missing 23 cycles.

            So one of these is wrong, and the next sitting should find out which before touching any
            code:
            - the 190-cycle hand-count of the handler,
            - the assumption that the first write belongs at dot 257,
            - or the dot the MMC3 counter reaches zero on, which is a property of what the game
              programmed into `$C000`/`$C001` and not of the A12 rise this project has already
              checked twice.

            **Done, and it is not the mapper either.** Traced every clock the MMC3 counter takes.
            The game writes `$C000 = 192` and `$C001` during vblank; the counter reloads to 192 on
            the pre-render line, is clocked exactly once a line at dot 261, and reaches zero on
            line 191 — precisely what the game asked for.

            The 243-clocks-a-frame figure recorded above is real but **harmless**, and here is why
            it was never the bug: all three extra clocks land in vblank *before* the `$C001`
            reload, so the reload overwrites whatever they did. That is why suppressing them
            changed nothing, and it can be struck off.

            So the CPU is ruled out, the handler is ruled out, and the mapper is ruled out. What
            remains is a 32-cycle disagreement between the handler's measured duration — 194 cycles
            from entry to its first write, matching the hand-count, and trustworthy now that every
            opcode accounts for all of its cycles — and the 226 cycles that a first write at dot 257
            would need.

            One of those numbers is wrong, and **dot 257 is the one that has never been measured**:
            it was derived here by reasoning that the burst spans 84 dots and hblank is 84 dots.
            Settling it needs a reference — the same ROM at the same point in Mesen, logging when
            its MMC3 IRQ fires and where the handler's first `$2006` write lands. That cannot be
            done from this checkout: Mesen2 is present but wants a .NET toolchain that is not
            installed, and its save states are its own format, so reaching the same scene means
            playing the game.

            **Traced the handler instruction by instruction, which settles most of it.** From
            `$F77B`, SMB3's IRQ handler saves registers, dispatches on a state byte at `$0101`,
            acknowledges the interrupt with `STA $E001`, and then runs a *fixed* delay:

            ```
            F7D0: A2 0C     LDX #$0C
            F7D2: EA        NOP
            F7D3: CA        DEX
            F7D4: D0 FC     BNE $F7D2     ; 12 iterations, 7 cycles each
            ```

            Three things follow, and together they move the search away from the CPU:

            1. **The handler polls nothing.** It is straight-line code and a counted delay loop, so
               sprite-zero hit, `$2002` and every other observable are not involved in its timing.
               Whatever is wrong, the handler cannot be being misled by the PPU.
            2. **Its duration is therefore pure cycle counting, and ours is now exact** — every
               opcode accounts for all of its cycles and `instr_timing` passes 3/3. Entry to first
               write measures 194 cycles against the 190 hand-counted here, which is agreement.
            3. **The interrupt is entered 2 cycles after the line is asserted**, which is a
               legitimate best case and cannot be 23 cycles slower on hardware: the most a 6502 can
               take is the rest of the current instruction, one cycle for the shadow and seven for
               the sequence.

            So the missing 23 cycles are not in the CPU at all, and this entry's long-standing claim
            that the two rewrites are coupled is wrong. **The next sitting should look at when the
            MMC3 counter reaches zero, not at interrupt delivery.** Specifically: what the game
            writes to `$C000`/`$C001`, how many times the counter is clocked per frame — this file
            already records it being clocked 243 times rather than 241 because palette accesses
            through `$2007` put `$3Fxx` on the bus — and which scanline it therefore fires on.

            **And the flicker is still present on the per-dot path**, measured across five
            consecutive frames from the save state: rows 193 and 194 change between frames while
            the per-line path holds them steady. It is the same drift as before — the frame is not
            a whole number of CPU cycles, so the burst's dot moves a little each frame, and at dot
            187 that lands inside the visible line where it shows. At dot 257 it would have the
            whole of hblank to drift in. So the switch stays off, and the 71-pixel figure above is
            not "nearly right" so much as "wrong in a much smaller place".

            Sharpened, since the numbers are exact and worth not re-deriving. The handler's six
            `STX $2006` writes land on dots 194, 206, 218, 230, 266 and 278 — the first four four
            cycles apart, then twelve cycles of other work, then the last pair. First to last is
            **84 dots, and hblank is 84 dots**, so the burst is built to drop into it exactly and
            ours begins **21 CPU cycles early**. Everything downstream of that is right: the
            handler is 190 cycles from entry to the first write, hand-counted from the 6502's own
            timings and matched by the emulator to the dot.

            **What the handler is actually doing**, which is worth having before touching any of
            this, because it is not only a scroll change. Traced from every PPU register write
            around the split:

            ```
            $2006 x4    line 193, dots 194-230   rendering still on — these corrupt v
            $2001 = $00 line 193, dot 242        rendering off
            $2006 = $0B,$00  line 193, dots 266,278   the real address, set while blanked
            $2001 = $18 line 194, dot 219        rendering on again
            ```

            So the black band between the playfield and the status bar is *deliberate blanking*,
            and the whole burst is meant to sit in hblank where corrupting `v` costs nothing. Being
            21 cycles early puts the first four writes inside the visible line, so the rest of line
            193 is fetched from `v = $0000` — the top of the nametable, which is sky. That is the
            line that was reported.

            It also explains why the per-line renderer is immune, and that its immunity is not a
            virtue: it samples `$2001` once per scanline, so it cannot express mid-line blanking at
            all. Neither the corruption nor the deliberate blank reaches the screen. The per-dot
            path renders both, faithfully, which is why it looks worse while the timing is wrong
            and will look right when it is not.

            **The mapper is not where the missing cycles are**, settled against the documentation
            rather than by more measurement of our own. The NESdev wiki puts the clock at PPU cycle
            260 for the standard arrangement, which is what we do and what `4-scanline_timing`
            checks. A scope measurement on an MMC3B puts the delay from A12 rising to /IRQ falling
            at about 69 ns — a third of a pixel, negligible. There is no cycle-scale delay hiding
            in the cartridge.

            The same thread contains an observation worth keeping: on hardware, Super Mario Bros 3
            "acknowledges the IRQ just over one scanline later". Ours reaches the acknowledging
            `STA $E001` about 0.8 scanlines after the interrupt, which is close enough that the
            interrupt delivery may well be right and the error be somewhere after it.

            **The DMC was suspected and is not the answer**, which is worth stating plainly
            because it was chased on a mistake. The reasoning was that its DMA stalls the CPU four
            cycles a sample byte, so a handler counting cycles takes longer while a sample plays.
            The arithmetic was an order of magnitude out: the rate table is CPU cycles *per bit*,
            not per byte. Fifty-four a bit at the fastest rate is 432 a byte, so the stall is four
            cycles in 432 — under one percent of the CPU's time, about two cycles across a
            190-cycle handler. It cannot account for twenty-one.

            The hunt was still worth it. It found the DMC playing a placeholder byte rather than
            reading memory, its memory reader unreachable so no sample ever started, and the whole
            channel clocked at half speed; and it found that a save state carried no APU state at
            all. All four are fixed. But the split's missing cycles are still missing.

            **What to do next**, having ruled out the CPU's rate, the mapper's clock dot, the
            mapper's clock count and the DMC: compare against another implementation rather than
            reason further. That is this project's own recorded lesson — every correct diagnosis
            here has come from running two implementations side by side, and every one argued from
            the mechanism alone has been wrong. This bug has now consumed several sittings of the
            latter. A reference emulator, the same ROM, the same point in the game, and a trace of
            which CPU cycle the IRQ is taken on.

            Ruled out so far, each measured rather than argued:

            - The CPU running fast or slow. A frame costs 29776 cycles against hardware's 29781
              once the sprite DMA is counted, and the handler path is exact.
            - The A12 rise being on the wrong dot. It is at dot 260 with `$2000 = $A8`, which is
              what `mmc3_test/4-scanline_timing` checks and passes.
            - The mapper being clocked too often. It *is* — 243 times a frame rather than 241,
              because a palette access through `$2007` puts `$3Fxx` on the bus and bit 12 of
              `$3F00` is set, so every palette update clocks the counter. Suppressing those clocks
              changes neither `mmc3_test` nor the dot the burst starts on, so it is left alone and
              recorded here as a question: whether a palette access should reach the cartridge at
              all is worth settling, but it is not this bug.

            **Closed, 2026-08-06, by the comparison the previous sitting asked for — and the
            answer was in neither the CPU, the handler, the mapper, nor the IRQ. It was in the
            clock.** `--into-level` now actually enters level 1-1 in both emulators (the four
            Starts only reached the world map; Right, Up, A were missing — each step read off a
            frame dump), and at the identical scene, same frame, same Goomba:

            - Ours wobbles: row 194 alternates between two images 16 pixels apart. tetanes is
              pixel-identical for ten straight frames.
            - But tetanes' `$2001 = $00` write *jitters exactly as ours does* — dots 263, 266,
              271 of line 193 on successive frames, the same 0-3 cycle spread from the spin loop
              at `$96F4`. The jitter is authentic. It is invisible there because all of those
              dots are in hblank.
            - The instruction streams from `$F77B` to the write are identical — same PCs, same
              206 CPU cycles, handler entered at the same dot (~290 of line 191) in both. Yet
              tetanes' PPU advances **659 dots** in those 206 cycles against our 618. That is
              3.2 dots per CPU cycle: **PAL**. `super-mario-3-eu.nes` is a PAL cart; tetanes'
              game database says so and clocks it accordingly (the iNES header claims NTSC —
              byte 9 is 0 — and tetanes rightly ignores it). We clock everything NTSC.

            So the `LDX #$0C` delay was tuned by the developers for 3.2 dots a cycle, and at
            that rate the whole burst sits in hblank with room for the jitter. Run at 3.0 it
            lands 41 dots early — mid-scanline, where the jitter paints the wobble. The "23
            cycles early", "21 cycles early" and "58 dots are in when the MMC3 IRQ fires" claims
            above were all this same number seen from different places, and "the first write
            belongs at dot 257" was NTSC arithmetic applied to a PAL calibration. Nothing was
            ever missing from the IRQ.

            **On the NTSC US ROM the split is already right.** Measured at the same scene:
            `$2001` off at 193:7-17, on at 193:331-194:0, and the split rows' interior is
            pixel-identical across ten consecutive frames. The only motion is 6 pixels (ours) /
            8-14 pixels (tetanes) at columns 10-23 of row 194 — the well-known SMB3 garbage
            shimmer above the status bar, which the reference shows too and hardware is famous
            for. Guarded by `crates/rn_core/tests/status_bar_split.rs`, which drives the US ROM
            into the level and asserts the band still — pointed at the EU ROM it fails at row
            194, which is the proof it can see the wobble.

            **PAL is in as of 2026-08-07, and the EU cartridge no longer wobbles.** Run with
            `--pal`, rows 186-200 are identical across six consecutive frames where NTSC timing
            moved them sixteen pixels. The mechanism is right there in the write dots:
            `$2001 = $00` lands at dot 284 rather than 235 — hblank rather than the visible line —
            so the CPU's interrupt jitter is still there, still 0-3 cycles, and paints nothing.
            The diagnosis is confirmed by construction rather than by argument.

            Implemented: `region::Region`, and a `DotClock` carrying 3.2 as sixteen dots per five
            cycles rather than rounding — the fifth cycle really does get a fourth dot and a game
            counting cycles can see it. The PPU's frame length (312 lines) and the odd-frame dot
            skip, which is NTSC-only. The region read from the iNES header, with an override for
            the cartridges that lie about it: this one claims NTSC in byte 9.

            **NTSC is unchanged by construction, not by luck.** The per-access clock became "all
            but one of the cycle's dots before the access, one after", which for three dots is
            exactly the 2-and-1 it always was. 528 tests, nestest 8991, baselines MATCH and every
            timing suite unmoved.

            **The PAL APU followed on 2026-08-07, so the region is now whole.** Three tables move
            with it, and none is a rescaling of the others — on hardware each is its own list, and
            they are carried that way here: the frame sequencer's steps (8313/16627/24939/33253,
            wrapping at 33254 against NTSC's 29830), the DMC's sixteen rates (398 down to 50
            against 428 down to 54) and the noise channel's sixteen periods. Taken from the
            reference emulator rather than from memory.

            Pinned by three tests that drive the APU through `write_byte` and `tick` rather than
            reading the constants back — a table that exists but is never *selected* is the
            failure this guards against, and reading the constant would not catch it. NTSC is
            unmoved: apu_test 9/9, apu_reset 6/6, blargg_apu 11/11, apu_mixer 4/4, sprdma 2/2,
            dmc_dma_during_read4 unchanged, 531 tests, baselines MATCH.

      Also waiting on the same switch: a transparent pixel should show the backdrop as it stands at
      that dot rather than the colour the frame was cleared to. `emit_pixel` does it; the per-line
      renderer deliberately does not, because it makes every transparent pixel follow a mid-frame
      $3F00 change and the row where that change lands jitters for exactly the same reason the
      split does. A steady approximation beats a flickering correctness until the timing is right.
- [x] Sprite evaluation per dot, so $2004 reads during rendering return what it holds

      Secondary OAM now exists and is worked on its real schedule: wiped over dots 1-64, evaluated
      into over 65-256, read back into the eight output units over 257-320. The single pass at dot
      257 is gone.

      It changed nothing measurable — SMB3 is byte-identical, every suite stands where it did — and
      that is the expected result: the *set* of sprites was already right, so what moved is only
      when and how it is arrived at. What is new is what can now be observed while it happens:

      - $2004 reads $FF for the whole of dots 1-64, because that is all secondary OAM holds. It
        does not reach object memory at all while the beam is drawing.
      - Evaluation starts at OAMADDR rather than at sprite zero, so a game that leaves the address
        elsewhere gets a different sprite acting as sprite zero.
      - OAMADDR is held at zero across dots 257-320.
      - No sprite can appear on scanline 0, because evaluation runs a line ahead and the
        pre-render line does not evaluate.

      Two implementations of the selection now exist — this one and the whole-line pass the
      debugging renderer still uses. They are compared directly against each other across thirty
      lines, both sprite heights and both flips, rather than left to be reasoned about. Tile
      addressing, flipping and the pixel decode are shared outright, so only the selection is
      written twice.

      Not modelled: the diagonal scan hardware performs after the eighth sprite, which is what
      makes the overflow flag unreliable. Only the flag itself is set, on the ninth sprite.
- [x] Vblank flag set and cleared at the exact dot, /NMI held as a level while both the flag and
      the enable bit are set, and the odd-frame dot skip decided on dot 339 from a $2001 that takes
      effect one cycle after it is written — `ppu_vbl_nmi` passes 11/11
- [ ] Carry the delayed `$2001` further, as Mesen does: a second copy another cycle behind again,
      which the scroll and fetch work reads. Changes what is drawn rather than only when a frame
      ends, so the gate is a pixel diff, not this suite.
- Acceptance: `ppu_vbl_nmi`, `blargg_ppu_tests`, and — all now passing — ~~`oam_read`~~,
  ~~`oam_stress`~~, ~~`mmc3_test/3-A12_clocking`~~, ~~`mmc3_test/4-scanline_timing`~~

### Housekeeping
- [x] CI: the workspace has a clean clippy gate and a full test suite, and nothing ran them.
      `.github/workflows/ci.yml` now does, on every push: build, clippy and the tests, all
      `--locked`, and the tests a second time because the failures that come from tests running in
      parallel are the ones a single green run is worst at catching.

      **It runs.** This paragraph used to say "it has never actually run, because the repository has
      no git remote", and that was read off this file and repeated as fact for a whole working
      session before anybody ran `git remote -v`. The remote is github.com/beoboo/rustnes, CI has
      been green there since 2026-08-04, and the workflow is verified rather than merely written.

      It deliberately does not run `cargo fmt --check`: the tree is not formatted to its own `.rustfmt.toml`, and that
      file asks for options only nightly rustfmt understands, so the check would fail on every
      commit for reasons unrelated to the commit. Worth fixing and worth adding then; a check
      nobody can make pass is a check everyone learns to ignore.
- [x] `frame_alternation.rs` and `nametable_map.rs` depend on local ROM paths and skipped silently.
      A skipped test and a passing one are indistinguishable in `cargo test`, and `println!` in a
      passing test is captured and shown to nobody — so both had been measuring nothing, invisibly,
      on any machine without the ROMs. Setting `RN_REQUIRE_ROMS` now turns a missing ROM into a
      failure, so a machine that has them can say so once and stop wondering. The default is still
      to skip, because nobody else can have these files.

      Same fault as the frame baselines that lived in `/tmp`: a check that quietly stops checking
      goes on reading as evidence.
