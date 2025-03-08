# Track 1 Guide: Pixel Display

<div class="track1">

Welcome to Track 1 of our NES emulator building journey! This track focuses on implementing just enough of the NES architecture to display a single colored pixel on screen.

## Track 1 Overview

**Goal**: Display a single colored pixel using a test ROM  
**Estimated time**: 5-10 hours  
**Complexity**: ⭐ (Beginner friendly)

## What You'll Build

By following Track 1, you'll implement:
- A basic 6502 CPU with essential instructions
- Simple memory mapping
- Fundamental PPU registers
- NROM (Mapper 0) support
- A minimal rendering pipeline

## Track 1 Roadmap

Follow these chapters in sequence to complete Track 1:

### Part 1: Foundations (For Everyone)
1. [NES Architecture Overview](../part1-foundations/chapter1-1-nes-overview.md)  
   *Learn the basics of the NES hardware architecture*

2. [Project Setup](../part1-foundations/chapter1-2-project-setup.md)  
   *Set up your Rust development environment*

3. [Testing Framework](../part1-foundations/chapter1-3-test-framework.md)  
   *Learn about the test-driven approach we'll be using*

### Part 2: Core Architecture
4. [CPU Basics](../part2-core-architecture/chapter2-1-cpu-basics.md)  
   *Implement the core 6502 CPU structure*

5. [Memory Management](../part2-core-architecture/chapter2-2-memory-management.md)  
   *Create the memory management system*

6. [Address Decoding](../part2-core-architecture/chapter2-3-address-decoding.md)  
   *Implement the address decoding logic*

7. [NROM Mapper](../part2-core-architecture/chapter2-4-nrom-mapper.md)  
   *Build the simplest NES cartridge mapper*

8. [PPU Registers](../part2-core-architecture/chapter2-5-ppu-registers.md)  
   *Implement the basic PPU registers needed for rendering*

9. [Milestone: Display a Pixel](../part2-core-architecture/chapter2-6-milestone-pixel.md)  
   *Complete the Track 1 milestone by displaying a colored pixel*

## Next Steps After Track 1

After completing Track 1, you have several options:

1. **Continue to Track 2**: Move on to [Track 2](track2-guide.md) to implement pattern tables and sprite rendering.

2. **Explore Components in Depth**: Revisit specific components to better understand how they work:
   - Learn more about the [6502 CPU architecture](https://www.nesdev.org/wiki/CPU)
   - Explore the [PPU in more detail](https://www.nesdev.org/wiki/PPU)

3. **Experiment with Your Implementation**:
   - Try modifying the test ROM to display pixels in different positions
   - Implement a few more CPU instructions
   - Add support for changing the pixel color dynamically

## Track 1 Checklist

Use this checklist to verify you've completed all the essential components for Track 1:

- [ ] Basic CPU structure implemented
- [ ] Essential CPU instructions working
- [ ] Memory read/write functionality
- [ ] PPU register access implemented
- [ ] NROM mapper functionality
- [ ] Test ROM loads successfully
- [ ] Single colored pixel displays on screen

## Resources for Track 1

- [6502 Instruction Reference](https://www.masswerk.at/6502/6502_instruction_set.html)
- [NES Dev Wiki: CPU](https://www.nesdev.org/wiki/CPU)
- [NES Dev Wiki: PPU](https://www.nesdev.org/wiki/PPU)
- [NES Dev Wiki: Mapper 0](https://www.nesdev.org/wiki/NROM)

## Common Questions for Track 1

**Q: How many CPU instructions do I need to implement for Track 1?**  
A: You only need to implement the minimal set required by the test ROM, typically around 15-20 instructions.

**Q: Do I need to implement accurate CPU timing for Track 1?**  
A: No, cycle-accurate timing is not required for Track 1. Simple instruction execution is sufficient.

**Q: How much of the PPU do I need to implement?**  
A: For Track 1, you only need to implement register access and the minimal functionality needed to set a pixel color.

</div>
