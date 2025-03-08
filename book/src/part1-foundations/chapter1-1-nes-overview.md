# NES Architecture Overview

<div class="track-tags">
<span class="track1-tag">T1</span>
<span class="track2-tag">T2</span>
<span class="track3-tag">T3</span>
<span class="track4-tag">T4</span>
</div>

This chapter provides an overview of the Nintendo Entertainment System (NES) hardware architecture. Understanding how the NES works at a hardware level is essential for building an emulator.

## Core Components

The NES consists of several key hardware components that work together:

<div class="nes-diagram">
```
+----------------------------------+
|             NES Hardware         |
+----------------------------------+
|                                  |
|  +--------+        +--------+    |
|  |        |        |        |    |
|  |  CPU   | <----> |  PPU   |    |
|  |        |        |        |    |
|  +--------+        +--------+    |
|      ^                |          |
|      |                |          |
|      v                v          |
|  +--------+      +----------+    |
|  |        |      |          |    |
|  | Memory | <--> | Video    |    |
|  |        |      | Output   |    |
|  +--------+      +----------+    |
|      ^                           |
|      |                           |
|      v                           |
|  +--------+      +----------+    |
|  |        |      |          |    |
|  |  APU   | ---> | Audio    |    |
|  |        |      | Output   |    |
|  +--------+      +----------+    |
|      ^                           |
|      |                           |
|      v                           |
|  +--------+                      |
|  |        |                      |
|  | Input  |                      |
|  |        |                      |
|  +--------+                      |
|                                  |
+----------------------------------+
```
</div>

Let's explore each component in detail:

### Central Processing Unit (CPU) [T1]

<div class="track1">

The NES uses a modified version of the 8-bit MOS Technology 6502 processor running at 1.79 MHz (NTSC) or 1.66 MHz (PAL). This CPU, often referred to as the Ricoh 2A03 (NTSC) or 2A07 (PAL), is the brain of the NES.

Key CPU characteristics:
- 8-bit data bus
- 16-bit address bus (can address up to 64KB of memory)
- 56 different instructions with various addressing modes
- 3 registers: A (accumulator), X, and Y (index registers)
- No built-in multiplication or division instructions
- Limited stack operations

For Track 1, we'll focus on implementing just enough CPU functionality to get a pixel on screen.

</div>

### Picture Processing Unit (PPU) [T1]

<div class="track1">

The PPU (Ricoh 2C02) is responsible for generating the video signal that displays graphics on the screen. It operates independently from the CPU but can be controlled through memory-mapped registers.

Basic PPU features:
- Generates a 256×240 pixel display
- Supports up to 64 sprites
- Can display up to 25 colors simultaneously (from a palette of 54)
- Manages background and sprite rendering

For Track 1, we'll implement just the basic registers needed to display a single pixel.

</div>

### Memory Architecture [T1]

<div class="track1">

The NES uses a complex memory mapping system:

- CPU Memory Map (64KB):
  - $0000-$07FF: 2KB internal RAM
  - $0800-$1FFF: Mirrors of internal RAM
  - $2000-$2007: PPU registers
  - $2008-$3FFF: Mirrors of PPU registers
  - $4000-$401F: APU and I/O registers
  - $4020-$FFFF: Cartridge space (PRG ROM, PRG RAM, and mapper registers)

- PPU Memory Map (64KB):
  - $0000-$1FFF: Pattern tables (CHR ROM or RAM from cartridge)
  - $2000-$2FFF: Name tables (VRAM)
  - $3000-$3EFF: Mirrors of name tables
  - $3F00-$3F1F: Palette RAM
  - $3F20-$3FFF: Mirrors of palette RAM

For Track 1, we'll focus primarily on the CPU memory map and the necessary PPU memory locations.

</div>

### Audio Processing Unit (APU) [T4]

<div class="track4">

The APU is integrated into the CPU chip and generates the audio for the NES. It features:

- 5 sound channels:
  - 2 pulse wave channels
  - 1 triangle wave channel
  - 1 noise channel
  - 1 delta modulation channel (for sample playback)

We'll implement the APU in Track 4 to complete our NES emulator.

</div>

### Input Devices [T3]

<div class="track3">

The NES supports various input devices, with the standard controller being the most common:

- Standard controller: D-pad, A, B, Select, and Start buttons
- Other peripherals: Zapper light gun, Power Pad, etc.

Controller input will be implemented in Track 3.

</div>

### Cartridges and Mappers [T1-T4]

<div class="track1">

NES cartridges contain the game ROM and sometimes additional hardware called "mappers" that extend the system's capabilities:

- PRG ROM: Program code (4KB to 512KB)
- CHR ROM/RAM: Graphics data (0KB to 256KB)
- Mappers: Circuits that allow bank switching and other features

For Track 1, we'll implement only the simplest mapper (NROM, Mapper 0).

</div>

<div class="track2">

In Track 2, we'll explore how the mapper interacts with the pattern tables to display sprites and backgrounds.

</div>

<div class="track3">

Track 3 will implement additional mappers to support more complex games with scrolling and other features.

</div>

<div class="track4">

In Track 4, we'll implement a wide range of mappers to support most commercial NES games.

</div>

## NES History

<div class="sidebar">

### The Birth of the NES

The Nintendo Entertainment System (called the Famicom in Japan) was released in Japan in 1983 and North America in 1985. It helped revitalize the video game industry after the video game crash of 1983.

The hardware was designed to be cost-effective while still providing impressive capabilities for its time. The decision to use a modified 6502 processor was partly because it was inexpensive and already had good developer support.

</div>

## What You'll Need to Know

To build an NES emulator, you'll need to understand:

1. How the 6502 CPU works and its instruction set
2. The memory mapping system
3. How the PPU renders graphics
4. How cartridges and mappers extend the system
5. How the APU generates sound (for Track 4)

Don't worry if this seems overwhelming—we'll cover each component in detail as we build our emulator step-by-step.

## Looking Ahead

In the next chapter, we'll set up our Rust project and establish the testing framework we'll use throughout development.

## Track Navigation

- [Next for Track 1: Project Setup](chapter1-2-project-setup.md)
- [Next for Track 2: Project Setup](chapter1-2-project-setup.md)
- [Next for Track 3: Project Setup](chapter1-2-project-setup.md)
- [Next for Track 4: Project Setup](chapter1-2-project-setup.md)
