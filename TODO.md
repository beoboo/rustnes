# RustNES Implementation Checklist 📋

This document provides a detailed task breakdown for developing the RustNES emulator alongside writing the book, organized by learning tracks.

## Track System Legend
- [T1] Track 1: Pixel Display - Minimal components to show a pixel
- [T2] Track 2: Pattern & Sprite Rendering - Animation capabilities 
- [T3] Track 3: Interactive Graphics - User input and full graphics
- [T4] Track 4: Complete NES - Full system emulation

## Project Setup [T1]
- [x] Initialize Rust project with Cargo
- [x] Set up module structure
- [x] Create README.md
- [x] Set up GitHub repository
- [x] Create initial documentation structure
- [x] Set up book framework with mdBook
- [ ] Configure CI/CD pipeline [T4]
- [ ] Add benchmark infrastructure [T4]

## NES Architecture Overview [T1]
- [ ] Document NES hardware components
- [ ] Create diagrams of system architecture
- [ ] Document memory map
- [ ] Document system bus
- [ ] Research hardware specifications
- [ ] Collect reference materials
- [ ] Document hardware limitations and quirks

## CPU Implementation

### CPU Basics [T1]
- [x] Implement basic CPU struct with registers
- [x] Implement CPU flags enum and methods
- [x] Create Memory trait
- [x] Implement basic RAM struct
- [x] Write tests for CPU flag operations
- [x] Write tests for memory operations
- [x] Document power-up state
- [x] Implement connection between CPU and memory

### Essential Addressing Modes [T1]
- [x] Define addressing mode enum
- [x] Implement immediate addressing
- [x] Implement absolute addressing
- [x] Implement zero page addressing
- [x] Write tests for essential addressing modes

### Essential Instructions [T1]
- [x] Create instruction decoder
- [x] Implement load instructions (LDA, LDX, LDY)
- [x] Implement store instructions (STA, STX, STY)
- [x] Implement basic jumps (JMP)
- [x] Implement subroutine calls (JSR, RTS)
- [ ] Write tests for essential instructions

### Extended Addressing Modes [T2]
- [x] Implement zero page,X addressing
- [x] Implement zero page,Y addressing
- [x] Implement absolute,X addressing
- [x] Implement absolute,Y addressing
- [x] Write tests for extended addressing modes

### Control Flow Instructions [T2]
- [ ] Implement status flag changes (CLC, CLD, CLI, CLV, SEC, SED, SEI)
- [ ] Implement branches (BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS)
- [ ] Implement register transfers (TAX, TAY, TXA, TYA)
- [ ] Implement stack operations (TSX, TXS, PHA, PHP, PLA, PLP)
- [ ] Write tests for control flow instructions

### Data Manipulation Instructions [T2]
- [ ] Implement logical operations (AND, EOR, ORA)
- [ ] Implement arithmetic operations (ADC, SBC)
- [ ] Implement increment/decrement (INC, INX, INY, DEC, DEX, DEY)
- [ ] Write tests for data manipulation instructions

### Advanced Instructions [T3]
- [ ] Implement shifts/rotates (ASL, LSR, ROL, ROR)
- [ ] Implement compare operations (CMP, CPX, CPY)
- [ ] Implement bit test (BIT)
- [ ] Write tests for advanced instructions

### Advanced Addressing Modes [T3]
- [x] Implement indirect addressing
- [x] Implement indexed indirect (indirect,X) addressing
- [x] Implement indirect indexed (indirect,Y) addressing
- [x] Write tests for advanced addressing modes

### Special Instructions [T4]
- [ ] Implement NOP and unofficial NOPs
- [ ] Implement other unofficial instructions
- [ ] Document cycle costs for all instructions
- [ ] Write tests for special instructions

## Instruction Parser System

### Basic Parser Framework [T1]
- [x] Create instruction parser module with error handling
- [x] Implement parsing for basic addressing modes (immediate, zero page, absolute)
- [x] Implement parsing for essential load instructions (LDA, LDX, LDY)
- [x] Implement parsing for essential store instructions (STA, STX, STY)
- [x] Implement parsing for basic jumps and subroutines (JMP, JSR, RTS) (JMP implemented)
- [x] Add parser tests for essential instructions
- [ ] Create basic disassembler for essential instructions

### Extended Parser Capabilities [T2]
- [ ] Implement parsing for extended addressing modes (X/Y indexed)
- [ ] Implement parsing for control flow instructions (branches, flag operations)
- [ ] Implement parsing for register transfers (TAX, TAY, etc.)
- [ ] Implement parsing for stack operations (PHA, PHP, etc.)
- [ ] Implement parsing for logical/arithmetic operations (AND, EOR, ORA, ADC, SBC)
- [ ] Implement parsing for increment/decrement (INC, INX, etc.)
- [ ] Add parser tests for extended instruction set
- [ ] Enhance disassembler for extended instruction set

### Advanced Parser Features [T3]
- [ ] Implement parsing for advanced addressing modes (indirect modes)
- [ ] Implement parsing for advanced instructions (shifts, rotates, compares)
- [ ] Add parsing for relative addressing (for branches)
- [ ] Support parsing multi-line assembly programs
- [ ] Implement label support in parser
- [ ] Add parser tests for advanced instructions
- [ ] Create full disassembler with machine code to assembly conversion

### Complete Parser System [T4]
- [ ] Implement parsing for all official and unofficial instructions
- [ ] Add advanced error messages with suggestions
- [ ] Implement comprehensive validation and diagnostics
- [ ] Create bi-directional assembler/disassembler
- [ ] Support custom syntax variations and assembly dialects
- [ ] Integrate parser with debugging tools
- [ ] Create interactive assembly editor for debugging
- [ ] Add comprehensive documentation with examples

## Memory System

### Basic Memory Map [T1]
- [ ] Implement basic RAM ($0000-$07FF)
- [ ] Implement PPU register mapping ($2000-$2007)
- [ ] Implement ROM space mapping ($8000-$FFFF)
- [ ] Test basic memory operations

### Essential PPU I/O [T1]
- [ ] Implement PPU register reading
- [ ] Implement PPU register writing
- [ ] Test PPU register access

### Extended Memory Features [T2]
- [ ] Implement RAM mirroring
- [ ] Implement expanded PPU register access ($2008-$200F)
- [ ] Test extended memory features

### Controller & DMA [T3]
- [ ] Implement controller register mapping
- [ ] Implement controller reading
- [ ] Implement DMA controller
- [ ] Implement DMA transfers
- [ ] Test controller input
- [ ] Test DMA functionality

### Advanced Memory Features [T4]
- [ ] Implement APU register mapping
- [ ] Implement APU register handling
- [ ] Implement memory read/write timing
- [ ] Test advanced memory features

## Cartridge System

### Basic ROM Loading [T1]
- [ ] Implement iNES file format parser
- [ ] Implement basic cartridge interface
- [ ] Implement PRG ROM access
- [ ] Implement CHR ROM access
- [ ] Test ROM loading
- [ ] Test PRG ROM access

### NROM Mapper [T1]
- [ ] Implement Mapper trait
- [ ] Implement NROM (Mapper 0)
- [ ] Implement simple test ROMs
- [ ] Test with NROM games

### Simple Mappers [T3]
- [ ] Implement UxROM (Mapper 2)
- [ ] Implement CNROM (Mapper 3)
- [ ] Test with simple mapper games

### Advanced Mappers [T4]
- [ ] Implement MMC1 (Mapper 1)
- [ ] Implement MMC3 (Mapper 4)
- [ ] Implement battery-backed RAM
- [ ] Implement mapper detection
- [ ] Test mapper switching behavior
- [ ] Test games that use specific mappers

## PPU Implementation

### PPU Basics [T1]
- [ ] Implement PPU registers
- [ ] Implement PPU memory map
- [ ] Implement basic VRAM
- [ ] Implement color palette
- [ ] Implement basic frame buffer
- [ ] Test basic pixel rendering

## MILESTONE 1: Display a Pixel [T1]
- [ ] Create a test ROM that sets a single pixel
- [ ] Integrate CPU, Memory, and PPU components
- [ ] Implement basic main loop
- [ ] Display a colored pixel
- [ ] Document the achievement

### Pattern Tables [T2]
- [ ] Implement pattern table access
- [ ] Implement tile fetching
- [ ] Implement palette selection
- [ ] Test pattern rendering

### Sprites [T2]
- [ ] Implement OAM (Object Attribute Memory)
- [ ] Implement sprite evaluation
- [ ] Implement sprite rendering
- [ ] Implement sprite priority
- [ ] Implement sprite attributes (flip, palette)
- [ ] Test sprite rendering

### PPU Timing [T2]
- [ ] Implement PPU timing
- [ ] Implement frame synchronization
- [ ] Test animation capabilities

## MILESTONE 2: Pattern & Sprite Animation [T2]
- [ ] Create a test ROM that animates sprites
- [ ] Demonstrate pattern table functionality
- [ ] Show multiple sprites with different attributes
- [ ] Document the achievement

### Background Rendering [T3]
- [ ] Implement name table handling
- [ ] Implement tile rendering
- [ ] Implement background priority
- [ ] Test background rendering

### Scrolling [T3]
- [ ] Implement scroll registers
- [ ] Implement fine scrolling
- [ ] Implement name table switching
- [ ] Test scrolling functionality

### Advanced PPU Features [T3]
- [ ] Implement sprite zero hit detection
- [ ] Implement sprite overflow
- [ ] Implement sprite-background interaction
- [ ] Test advanced PPU features

## Input System [T3]
- [ ] Implement controller registers
- [ ] Implement standard controller (D-pad, A, B, Start, Select)
- [ ] Implement controller strobe
- [ ] Implement controller polling
- [ ] Test controller input
- [ ] Implement key mapping
- [ ] Implement input configuration

## MILESTONE 3: Interactive Demo [T3]
- [ ] Create a demo with user-controlled sprite
- [ ] Implement scrolling background
- [ ] Demonstrate controller input
- [ ] Document the achievement

## Audio Processing Unit [T4]
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

## System Integration [T4]
- [ ] Implement full synchronization between components
- [ ] Optimize performance bottlenecks
- [ ] Test system with full games
- [ ] Implement game reset
- [ ] Implement game pause
- [ ] Implement save state mechanism
- [ ] Implement rewind functionality
- [ ] Implement speed control
- [ ] Implement cheat codes

## Debugging Tools [T4]
- [ ] Implement memory viewer/editor
- [ ] Implement CPU state inspector
- [ ] Implement PPU viewer
- [ ] Implement pattern table viewer
- [ ] Implement name table viewer
- [ ] Implement logger
- [ ] Implement breakpoint system
- [ ] Implement execution tracing

## WebAssembly Support [T4]
- [ ] Set up wasm-bindgen
- [ ] Create WebAssembly build target
- [ ] Adapt graphics output for web
- [ ] Adapt audio output for web
- [ ] Implement browser input handling
- [ ] Optimize for web performance
- [ ] Create web interface
- [ ] Test in multiple browsers

## MILESTONE 4: Complete NES [T4]
- [ ] Run commercial games with full compatibility
- [ ] Verify audio functionality
- [ ] Demonstrate advanced features
- [ ] Document full compatibility

## Progress Tracking
- Track 1 (Pixel Display): 0% complete
- Track 2 (Pattern & Sprite Rendering): 0% complete
- Track 3 (Interactive Graphics): 0% complete
- Track 4 (Complete NES): 0% complete

**Total Progress: 22/182 tasks complete (12.1%)** 