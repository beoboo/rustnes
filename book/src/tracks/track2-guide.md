# Track 2 Guide: Pattern & Sprite Rendering

<div class="track2">

Welcome to Track 2 of our NES emulator building journey! This track builds upon Track 1 and focuses on implementing pattern tables and sprites to display actual NES-style graphics.

## Track 2 Overview

**Goal**: Display and animate sprites using pattern tables  
**Estimated time**: 15-20 hours  
**Complexity**: ⭐⭐ (Intermediate)  
**Prerequisites**: Completion of Track 1 or equivalent knowledge

## What You'll Build

By following Track 2, you'll extend your emulator to implement:
- A more complete 6502 CPU with additional instructions
- Pattern table rendering from CHR ROM/RAM
- Sprite rendering capabilities
- Basic animation support
- A more comprehensive PPU implementation

## Track 2 Roadmap

Follow these chapters in sequence to complete Track 2:

### Part 1-2: Prerequisites (Complete Track 1 First)
1. Follow the [Track 1 Guide](track1-guide.md) to implement the fundamental components

### Part 3: Basic Rendering
2. [PPU Implementation](../part3-basic-rendering/chapter3-1-ppu-implementation.md)  
   *Implement a more complete Picture Processing Unit*

3. [Pattern Tables](../part3-basic-rendering/chapter3-2-pattern-tables.md)  
   *Build support for rendering pattern tables from CHR ROM*

4. [Background Basics](../part3-basic-rendering/chapter3-3-background-basics.md)  
   *Implement basic background rendering*

5. [Palette Handling](../part3-basic-rendering/chapter3-4-palette-handling.md)  
   *Add support for color palettes*

6. [Sprite Basics](../part3-basic-rendering/chapter3-5-sprite-basics.md)  
   *Implement sprite rendering and OAM (Object Attribute Memory)*

7. [Milestone: Sprites](../part3-basic-rendering/chapter3-6-milestone-sprites.md)  
   *Complete the Track 2 milestone by displaying and animating sprites*

## Next Steps After Track 2

After completing Track 2, you have several options:

1. **Continue to Track 3**: Move on to [Track 3](track3-guide.md) to implement scrolling, full backgrounds, and controller input.

2. **Explore Pattern and Sprite Rendering in Depth**:
   - Learn more about [NES graphics programming](https://www.nesdev.org/wiki/PPU_pattern_tables)
   - Study how commercial NES games used creative sprite techniques

3. **Experiment with Your Implementation**:
   - Create custom sprite patterns and animations
   - Implement sprite 0 hit detection
   - Add support for sprite priorities and overlapping

## Track 2 Checklist

Use this checklist to verify you've completed all the essential components for Track 2:

- [ ] Pattern table loading from CHR ROM
- [ ] Background tile rendering
- [ ] Sprite rendering from OAM
- [ ] Palette implementation (including sprite palettes)
- [ ] Basic sprite attributes (position, flipping, palette selection)
- [ ] Test ROM with sprite animation loads and runs correctly
- [ ] Rendering multiple sprites simultaneously

## Resources for Track 2

- [NES Dev Wiki: PPU Pattern Tables](https://www.nesdev.org/wiki/PPU_pattern_tables)
- [NES Dev Wiki: PPU OAM](https://www.nesdev.org/wiki/PPU_OAM)
- [NES Dev Wiki: PPU Palettes](https://www.nesdev.org/wiki/PPU_palettes)
- [NES Dev Wiki: PPU Rendering](https://www.nesdev.org/wiki/PPU_rendering)

## Common Questions for Track 2

**Q: How complex should my PPU implementation be for Track 2?**  
A: For Track 2, you need to implement pattern table rendering, background tiles, and sprites, but don't need to worry about scrolling, sprite overflow, or cycle-accurate timing yet.

**Q: Do I need to implement all sprite attributes?**  
A: For Track 2, implement the core attributes: position (x, y), tile index, attributes (palette, flip), and priority.

**Q: Should my emulator support CHR RAM as well as CHR ROM?**  
A: For Track 2, supporting CHR ROM is sufficient, but if you want to support mappers that use CHR RAM, you'll need to add that as well.

</div>
