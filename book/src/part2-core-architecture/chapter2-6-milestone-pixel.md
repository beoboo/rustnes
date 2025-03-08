# Milestone: Display a Pixel

<div class="track-tags">
<span class="track1-tag">T1</span>
</div>

<div class="milestone">

## Congratulations! 🎉

You've reached the first milestone in our NES emulator development journey! By now, you should have a basic emulator that can:

1. Load a simple test ROM
2. Execute basic CPU instructions
3. Handle memory reads and writes
4. Access PPU registers
5. Display a single colored pixel on screen

This may seem like a small achievement compared to playing full NES games, but it's a crucial foundation for everything that follows. Getting that first pixel to display means your core architecture is working properly.

</div>

## Track 1 Test ROM

Let's use our test ROM to verify everything is working correctly. The test ROM is designed specifically to display a single red pixel at coordinates (128, 120) - right in the middle of the screen.

Here's what the test ROM does:
1. Initializes the stack pointer
2. Sets the PPU address to the palette location
3. Writes a red color value to the palette
4. Sets the PPU address to a specific location in the name table
5. Writes a value that will display our pixel
6. Enables rendering

```rust
// Test ROM - Display a Single Red Pixel
const TEST_ROM: [u8; 32] = [
    // Header (simplified)
    0x4E, 0x45, 0x53, 0x1A, // NES<EOF> header
    0x01, 0x00, 0x00, 0x00, // 1 PRG ROM bank, 0 CHR ROM banks
    0x00, 0x00, 0x00, 0x00, // Mapper 0, vertical mirroring
    0x00, 0x00, 0x00, 0x00, // Unused
    
    // PRG ROM data
    0xA2, 0xFF,             // LDX #$FF
    0x9A,                   // TXS
    0xA9, 0x3F,             // LDA #$3F
    0x8D, 0x06, 0x20,       // STA $2006
    0xA9, 0x00,             // LDA #$00
    0x8D, 0x06, 0x20,       // STA $2006
    0xA9, 0x16,             // LDA #$16 (red color)
    0x8D, 0x07, 0x20,       // STA $2007
    0xA9, 0x1E,             // LDA #$1E
    0x8D, 0x00, 0x20,       // STA $2000 (enable rendering)
    0x4C, 0x20, 0x80,       // JMP $8020 (infinite loop)
];
```

## Verifying Your Implementation

When you run your emulator with this test ROM, you should see a single red pixel in the center of the screen like this:

<div class="nes-diagram">
```
+------------------------------------------+
|                                          |
|                                          |
|                                          |
|                                          |
|                                          |
|                                          |
|                                          |
|                                          |
|                                          |
|                                          |
|                                          |
|                  [X]                     |  <- Red pixel at (128, 120)
|                                          |
|                                          |
|                                          |
|                                          |
|                                          |
|                                          |
|                                          |
|                                          |
|                                          |
+------------------------------------------+
```
</div>

## Track 1 Completion Checklist

Use this checklist to verify you've completed all the essential components for Track 1:

<ul class="checklist">
  <li>CPU can execute the basic instructions: LDA, LDX, STA, TXS, JMP</li>
  <li>Memory mapping routes addresses to the correct components</li>
  <li>PPU registers are accessible via memory-mapped I/O ($2000-$2007)</li>
  <li>Simple NROM (Mapper 0) functionality works</li>
  <li>PPU can write to the palette memory</li>
  <li>Basic rendering pipeline can display at least one pixel</li>
  <li>Test ROM loads and runs correctly</li>
</ul>

## Troubleshooting Common Issues

If your pixel isn't displaying correctly, check these common issues:

<div class="track1">

### CPU Problems
- Ensure your CPU correctly implements all the instructions used in the test ROM
- Verify the CPU can read from and write to memory properly
- Check that the program counter advances correctly

### Memory Mapping Issues
- Confirm PPU register writes are being properly handled
- Ensure memory mirrors are implemented correctly
- Verify that the mapper is correctly exposing PRG ROM

### PPU Problems
- Check that palette memory is being written to correctly
- Ensure the PPU is actually rendering (even minimally)
- Verify PPU address register ($2006) is working properly

</div>

## What You've Learned

By reaching this milestone, you've learned:

1. The basics of 6502 CPU architecture and instruction execution
2. How the NES memory mapping system works
3. How the CPU communicates with the PPU
4. Fundamentals of NES cartridge structure
5. The beginning of the rendering pipeline

## Next Steps

<div class="fast-forward">

### Fast Forward Navigation

Now that you've completed Track 1, you have several options:

- **Continue with Track 1**: Go deeper into CPU implementation to prepare for Track 2
  - Next chapter: [PPU Implementation](../part3-basic-rendering/chapter3-1-ppu-implementation.md)

- **Jump to Track 2**: Move directly to implementing pattern and sprite rendering
  - Fast forward to: [Pattern Tables](../part3-basic-rendering/chapter3-2-pattern-tables.md)

- **Experiment with Your Current Implementation**:
  - Try modifying the test ROM to display pixels in different positions
  - Implement a few more CPU instructions
  - Add support for changing the pixel color dynamically

</div>

## Emulation Insight

<div class="sidebar">

### The Importance of Small Steps

While displaying a single pixel might seem trivial compared to emulating full games, this approach of building incrementally is exactly how many successful emulators started. 

By focusing on getting one pixel working first, you ensure your core architecture is solid before adding more complex features. This makes debugging easier and gives you the confidence that each component is working correctly before moving on.

Many professional emulator developers still use this approach when tackling new systems!

</div>

## Track Navigation

- [Next for Track 1: PPU Implementation](../part3-basic-rendering/chapter3-1-ppu-implementation.md)
