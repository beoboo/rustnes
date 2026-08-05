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

One cycle, in a counter whose table and window are already identical. The candidates worth checking
first are the `$4017` write delay — three cycles or four depending on the parity of the cycle the
write lands on, which this session added and `4-jitter` validates — and the eight settle cycles the
machine runs before its first instruction, which tick the APU as well as the PPU.

Its read and write paths otherwise agree with ours closely — the PPU's write-only registers return
the PPU's own latch, everything unmapped returns the CPU's, and a write drives the bus either way.
And its MMC3 counter is the same logic as ours, clocked from real PPU bus reads.

### [Testing] Suites never measured until now
Measured for the first time, and three of them were already passing:
- [x] vbl_nmi_timing — 7/7
- [x] instr_test-v3 — 17/17
- [x] nes_instr_test — 11/11
- [ ] sprite_hit_tests — 7/11, from 0/11 once CHR RAM worked
- [ ] sprite_overflow_tests — 3/5 (`3.Timing`, `4.Obscure`)
- [x] ppu_open_bus — 1/1. The PPU's I/O latch is a *decay* register: dynamic storage with nothing
      holding it up, so a bit not refreshed leaks to zero in about 600ms. And each register
      refreshes a different part of it — `$2002` supplies its top three bits and leaves the other
      five to rot, a palette read through `$2007` supplies six, a write supplies all eight, and a
      read of a write-only register supplies none, so holding a value by reading it repeatedly is
      exactly what hardware will not let you do.
- [ ] ppu_read_buffer — 0/1, but 76 of its 79 sub-tests pass. Failing 57, 60 and 73; 57 is
      "setting PPU address to 3FFF and reading $2007 thrice should give the contents of $0000".
- [x] blargg_nes_cpu_test5 — 2/2, including `cpu.nes`, which covers the unofficial instructions
      too. It was passing already; it reports by listing what it ran and ending with "All tests
      complete", which the screen reader could not interpret. The failing form — "Errors: n" and
      "Failed" — was produced deliberately, by breaking `ASL`'s carry flag, before that rule was
      written: "complete" is not "passed", and guessing would have turned a broken emulator green.
- [ ] dmc_tests — 0/4, still unmeasured rather than failing. All four draw nothing at all to the
      nametable and end on a self-jump at `$E14E`, so they report by some means this runner cannot
      read — audio, most likely, since the suite is about DMC buffering and latency.

### [Testing] Pass the suites [T8]
Standing as of the last run. Most of what remains is blocked on the two rewrites at the end of this
file rather than on anything specific to the suite.
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
- [ ] mmc3_test — 5/6 (only `6-MMC6`, which is the other chip's counter behaviour)
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
      - `power_up_palette` $02 — "palette differs from table" and still failing, which the suite's
        readme says is expected: those values "are probably unique to my NES". Not a fault to chase.
- [ ] nmi_sync, cpu_timing_test6 — still unmeasured. The `nmi_sync` ROMs are visual demos with no
      verdict to read; `cpu_timing_test6` draws nothing into the first nametable within 600 frames.
- [ ] cpu_interrupts_v2 — 2/6, and no longer hanging anywhere. `3-nmi_and_irq` is closer but not
      passing: an NMI arriving while an IRQ is being vectored now takes the vector, as it should,
      but hardware's window for that is wider than ours — the table alternates where it should hold
      a run of the same value. `2-nmi_and_brk` passes, which is the
      test CYCLE_ACCURACY.md records as having hung on two previous attempts at interrupt timing;
      it was the BRK halt above, not the interrupt work. The combined `cpu_interrupts.nes` now
      fails rather than hangs.
- [x] cpu_reset — 2/2. Reset is not power-on: it sets the I flag, subtracts three from the stack
      pointer and does nothing else. A, X, Y and the other flags survive it. The three are the
      interrupt sequence going through the motions of its pushes with the writes suppressed.
- [ ] cpu_exec_space — 1/2. `ppuio` passes: the PPU has an I/O latch of its own, so a read of one
      of its five write-only registers returns whatever its lines last carried, and `$2002`
      supplies only its top three bits with the latch supplying the other five. The APU's
      write-only registers now fall through to the CPU's open bus for the same reason, which moved
      `test_cpu_exec_space_apu` from landing at $0234 to $1634 but not to the end. It executes code
      *from* $4000 and follows where the open bus leads; what our bus returns there is evidently
      still not what hardware returns.
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
- [ ] CNROM (3), MMC2 (9), Color Dreams (11) — each small, none needed by anything tried so far
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
- [x] NMI hijacking BRK, and no polling inside an interrupt sequence
- [x] Branch polling: a taken branch ignores an IRQ that only became eligible during its own last
      cycle, so the instruction at the target runs first. The last of the three documented
      exceptions, and the only part of the sampling rule that does not fall out of the shadow.
      Proved by a unit test rather than by `5-branch_delays_irq`, which does not move on it: that
      ROM fails in its first sub-test, which measures `JMP` and never reaches the branch cases.
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

            **Re-measured after the CPU/PPU alignment work of 2026-08-04/05, which moved the
            interrupt poll by a dot, deleted `ADDRESS_BUS_LEAD_DOTS`, closed the cycle/access gap to
            zero and added the eight settle cycles. It did not fix this, and the current numbers
            are below.** `rom_test frame --state ... --per-dot` reproduces it in seconds.

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

      Two things to know about it. It has never actually run, because the repository has no git
      remote — the commands are verified locally, the workflow is not. And it deliberately does
      not run `cargo fmt --check`: the tree is not formatted to its own `.rustfmt.toml`, and that
      file asks for options only nightly rustfmt understands, so the check would fail on every
      commit for reasons unrelated to the commit. Worth fixing and worth adding then; a check
      nobody can make pass is a check everyone learns to ignore.
- [ ] `crates/rn_core/tests/frame_alternation.rs` depends on a local ROM path and skips silently
