# RustNES Implementation Checklist 📋

This document provides a detailed task breakdown for developing the RustNES emulator alongside writing the book, organized by learning tracks.

## Track System Legend
- [T1] Track 1: Memory Visualization - Display memory contents as pixels in an egui widget
- [T2] Track 2: PPU Pixel Display - Using the PPU to show pixels 
- [T3] Track 3: Pattern & Sprite Rendering - Animation capabilities 
- [T4] Track 4: Interactive Graphics - User input and full graphics
- [T5] Track 5: Complete NES - Full system emulation

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
- [ ] Configure CI/CD pipeline [T5]
- [ ] Add benchmark infrastructure [T5]

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

### [Parser] Basic Parser Framework [T1]
- [x] Create instruction parser module with error handling
- [x] Implement parsing for basic addressing modes (immediate, zero page, absolute)
- [x] Implement parsing for essential load instructions (LDA, LDX, LDY)
- [x] Implement parsing for essential store instructions (STA, STX, STY)
- [x] Implement parsing for basic jumps and subroutines (JMP, JSR, RTS) (JMP implemented)
- [x] Implement parsing for BRK instruction
- [x] Add parser tests for essential instructions
- [x] Create basic disassembler for essential instructions

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

### [Parser] Label Support [T2]
- [ ] Add label declaration support in the parser
- [ ] Implement label resolution for jump instructions
- [ ] Add tests for label parsing and resolution
- [ ] Update the AsmDebugger to properly handle infinite loops

## MILESTONE 3: Pattern & Sprite Animation [T3]
- [ ] Create a test ROM that animates sprites
- [ ] Demonstrate pattern table functionality
- [ ] Show multiple sprites with different attributes
- [ ] Document the achievement

### [CPU] Extended Addressing Modes [T3]
- [x] Implement zero page,X addressing
- [x] Implement zero page,Y addressing
- [x] Implement absolute,X addressing
- [x] Implement absolute,Y addressing
- [x] Write tests for extended addressing modes

### [CPU] Control Flow Instructions [T3]
- [ ] Implement status flag changes (CLC, CLD, CLI, CLV, SEC, SED, SEI)
- [ ] Implement branches (BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS)
- [ ] Implement register transfers (TAX, TAY, TXA, TYA)
- [ ] Implement stack operations (TSX, TXS, PHA, PHP, PLA, PLP)
- [ ] Write tests for control flow instructions

### [CPU] Data Manipulation Instructions [T3]
- [ ] Implement logical operations (AND, EOR, ORA)
- [ ] Implement arithmetic operations (ADC, SBC)
- [ ] Implement increment/decrement (INC, INX, INY, DEC, DEX, DEY)
- [ ] Write tests for data manipulation instructions

### [Parser] Extended Parser Capabilities [T3]
- [ ] Implement parsing for extended addressing modes (X/Y indexed)
- [ ] Implement parsing for control flow instructions (branches, flag operations)
- [ ] Implement parsing for register transfers (TAX, TAY, etc.)
- [ ] Implement parsing for stack operations (PHA, PHP, etc.)
- [ ] Implement parsing for logical/arithmetic operations (AND, EOR, ORA, ADC, SBC)
- [ ] Implement parsing for increment/decrement (INC, INX, etc.)
- [ ] Add parser tests for extended instruction set
- [ ] Enhance disassembler for extended instruction set
- [ ] Support local labels

### [Memory] Memory Enhancements [T3]
- [ ] Implement RAM mirroring ($0800-$1FFF mirrors $0000-$07FF)
- [ ] Implement ROM component for cartridge memory ($8000-$FFFF)

### [PPU] Ppu Enhancements [T3]
- [ ] Implement PPU register mirroring ($2008-$3FFF mirrors $2000-$2007)

### [Memory] Extended Memory Features [T3]
- [ ] Implement RAM mirroring
- [ ] Implement expanded PPU register access ($2008-$200F)
- [ ] Test extended memory features

### [PPU] Pattern Tables [T3]
- [ ] Fix pattern table memory access for actual ROMs
- [ ] Update rendering logic to properly read pattern data from cartridge
- [ ] Implement pattern table access
- [ ] Implement tile fetching
- [ ] Add proper pattern table bit plane handling
- [ ] Implement complete palette mapping system
- [ ] Implement palette selection
- [ ] Test pattern rendering

### [PPU] Sprites [T3]
- [ ] Implement OAM (Object Attribute Memory)
- [ ] Implement sprite evaluation
- [ ] Implement sprite rendering
- [ ] Implement sprite priority
- [ ] Implement sprite attributes (flip, palette)
- [ ] Test sprite rendering

### [PPU] PPU Timing [T3]
- [ ] Implement scanline/cycle tracking (261 scanlines, 341 cycles per scanline)
- [ ] Implement VBLANK flag setting and NMI generation
- [ ] Test frame timing with appropriate PPU cycles
- [ ] Implement frame synchronization
- [ ] Test animation capabilities

### [Cartridge] Basic ROM Loading [T3]
- [ ] Implement iNES file format parser
- [ ] Implement basic cartridge interface
- [ ] Implement PRG ROM access
- [ ] Implement CHR ROM access
- [ ] Test ROM loading
- [ ] Test PRG ROM access

### [Cartridge] NROM Mapper [T3]
- [ ] Implement Mapper trait
- [ ] Implement NROM (Mapper 0)
- [ ] Implement simple test ROMs
- [ ] Test with NROM games

### [Debugger] AsmDebugger Improvements [T3]
- [ ] Add breakpoint support
- [ ] Implement visual memory map (showing NES memory regions graphically)
- [ ] Add support for viewing/editing PPU memory

## MILESTONE 4: Interactive Demo [T4]
- [ ] Create a demo with user-controlled sprite
- [ ] Implement scrolling background
- [ ] Demonstrate controller input
- [ ] Document the achievement

### [CPU] Advanced Instructions [T4]
- [ ] Implement shifts/rotates (ASL, LSR, ROL, ROR)
- [ ] Implement compare operations (CMP, CPX, CPY)
- [ ] Implement bit test (BIT)
- [ ] Write tests for advanced instructions

### [CPU] Advanced Addressing Modes [T4]
- [x] Implement indirect addressing
- [x] Implement indexed indirect (indirect,X) addressing
- [x] Implement indirect indexed (indirect,Y) addressing
- [x] Write tests for advanced addressing modes

### [Parser] Advanced Parser Features [T4]
- [ ] Implement parsing for advanced addressing modes (indirect modes)
- [ ] Implement parsing for advanced instructions (shifts, rotates, compares)
- [ ] Add parsing for relative addressing (for branches)
- [ ] Support parsing multi-line assembly programs
- [ ] Implement label support in parser
- [ ] Add parser tests for advanced instructions
- [ ] Create full disassembler with machine code to assembly conversion

### [Memory] Controller & DMA [T4]
- [ ] Implement controller register mapping
- [ ] Implement controller reading
- [ ] Implement DMA controller
- [ ] Implement DMA transfers
- [ ] Test controller input
- [ ] Test DMA functionality

### [Cartridge] Simple Mappers [T4]
- [ ] Implement UxROM (Mapper 2)
- [ ] Implement CNROM (Mapper 3)
- [ ] Test with simple mapper games

### [PPU] Background Rendering [T4]
- [ ] Implement name table handling
- [ ] Implement tile rendering
- [ ] Implement background priority
- [ ] Test background rendering

### [PPU] Scrolling [T4]
- [ ] Implement scroll registers
- [ ] Implement fine scrolling
- [ ] Implement name table switching
- [ ] Test scrolling functionality

### [PPU] Advanced PPU Features [T4]
- [ ] Implement sprite zero hit detection
- [ ] Implement sprite overflow
- [ ] Implement sprite-background interaction
- [ ] Test advanced PPU features

### [Input] Input System [T4]
- [ ] Implement controller registers
- [ ] Implement standard controller (D-pad, A, B, Start, Select)
- [ ] Implement controller strobe
- [ ] Implement controller polling
- [ ] Test controller input
- [ ] Implement key mapping
- [ ] Implement input configuration

### [Debugger] AsmDebugger Improvements [T4]
- [ ] Connect to running emulator instance
- [ ] Create user-friendly UI with dockable panels

## MILESTONE 4: Complete NES [T5]
- [ ] Run commercial games with full compatibility
- [ ] Verify audio functionality
- [ ] Demonstrate advanced features
- [ ] Document full compatibility

### [CPU] Special Instructions [T5]
- [ ] Implement unofficial NOPs
- [ ] Implement other unofficial instructions
- [ ] Document cycle costs for all instructions
- [ ] Write tests for special instructions

### [Parser] Complete Parser System [T5]
- [ ] Implement parsing for all official and unofficial instructions
- [ ] Add advanced error messages with suggestions
- [ ] Implement comprehensive validation and diagnostics
- [ ] Create bi-directional assembler/disassembler
- [ ] Support custom syntax variations and assembly dialects
- [ ] Integrate parser with debugging tools
- [ ] Create interactive assembly editor for debugging
- [ ] Add comprehensive documentation with examples

### [Memory] Advanced Memory Features [T5]
- [ ] Implement APU register mapping
- [ ] Implement APU register handling
- [ ] Implement memory read/write timing
- [ ] Test advanced memory features

### [Cartridge] Advanced Mappers [T5]
- [ ] Implement MMC1 (Mapper 1)
- [ ] Implement MMC3 (Mapper 4)
- [ ] Implement battery-backed RAM
- [ ] Implement mapper detection
- [ ] Test mapper switching behavior
- [ ] Test games that use specific mappers

### [APU] Audio Processing Unit [T5]
- [ ] Implement APU framework
- [ ] Implement APU registers
- [ ] Implement APU timing
- [ ] Implement frame counter
- [ ] Implement APU interrupts
- [ ] Test APU register access
- [ ] Implement pulse channels (1 & 2)
- [ ] Implement triangle channel
- [ ] Implement noise channel
- [ ] Implement DMC channel
- [ ] Implement length counters
- [ ] Implement sweep units
- [ ] Implement envelope generators
- [ ] Test individual channel output
- [ ] Implement audio mixer
- [ ] Implement audio buffer
- [ ] Implement sample rate conversion
- [ ] Connect to audio output device
- [ ] Test audio output with games

### [System] System Integration [T5]
- [ ] Implement full synchronization between components
- [ ] Optimize performance bottlenecks
- [ ] Test system with full games
- [ ] Implement game reset
- [ ] Implement game pause
- [ ] Implement save state mechanism
- [ ] Implement rewind functionality
- [ ] Implement speed control
- [ ] Implement cheat codes

### [Debugger] Debugging Tools [T5]
- [ ] Implement memory viewer/editor
- [ ] Implement CPU state inspector
- [ ] Implement PPU viewer
- [ ] Implement pattern table viewer
- [ ] Implement name table viewer
- [ ] Implement logger
- [ ] Implement breakpoint system
- [ ] Implement execution tracing

### [Web] WebAssembly Support [T5]
- [ ] Set up wasm-bindgen
- [ ] Create WebAssembly build target
- [ ] Adapt graphics output for web
- [ ] Adapt audio output for web
- [ ] Implement browser input handling
- [ ] Optimize for web performance
- [ ] Create web interface
- [ ] Test in multiple browsers

### [Debugger] AsmDebugger Improvements [T5]
- [ ] Implement performance profiling features

## Progress Tracking
- Track 1 (Memory Visualization): 100% complete
- Track 2 (PPU Pixel Display): 100% complete
- Track 3 (Pattern & Sprite Rendering): 15% complete
- Track 4 (Interactive Graphics): 10% complete
- Track 5 (Complete NES): 0% complete

**Total Progress: 83/205 tasks complete (40.5%)** 🚀 