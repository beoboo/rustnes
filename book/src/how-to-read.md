# How to Read This Book

This book on building a NES emulator in Rust uses a multi-track approach that allows you to learn at your own pace and focus on the aspects of emulation that interest you most.

## The Multi-Track System

We've organized content into four distinct tracks, each with its own milestone:

<div class="track1">

### Track 1: Pixel Display [T1]
**Goal**: Get a single pixel working on screen.  
**Focus**: Essential CPU instructions, basic memory mapping, minimal PPU registers.  
**For**: Readers who want to get something visible quickly, or who are new to emulation.  
**Milestone**: Successfully display a colored pixel using a test ROM.

</div>

<div class="track2">

### Track 2: Pattern & Sprite Rendering [T2]
**Goal**: Display patterns and sprites.  
**Focus**: Extended CPU features, tile and pattern rendering, sprite capabilities.  
**For**: Readers who want to see actual NES-style graphics.  
**Milestone**: Display and animate sprites on screen.

</div>

<div class="track3">

### Track 3: Interactive Graphics [T3]
**Goal**: Create an interactive demo with controller input.  
**Focus**: Complete CPU instruction set, background rendering with scrolling, controller input.  
**For**: Readers who want to create something interactive.  
**Milestone**: Build an interactive demo with controller input.

</div>

<div class="track4">

### Track 4: Complete NES [T4]
**Goal**: Build a fully-functional NES emulator.  
**Focus**: Audio processing, all memory mappers, cycle-accurate timing, debugging features.  
**For**: Readers who want the complete experience.  
**Milestone**: Play commercial NES games with full compatibility.

</div>

## How the Book is Organized

The book is divided into seven parts, which follow the natural development process of the emulator:

1. **Foundations**: NES architecture overview and project setup
2. **Core Architecture**: CPU, memory, and basic PPU functionality (Track 1 milestone)
3. **Basic Rendering**: Pattern tables, backgrounds, and sprites (Track 2 milestone)
4. **Enhanced Graphics**: Full backgrounds, scrolling, and PPU features
5. **Input & Interaction**: Controller input and UI (Track 3 milestone)
6. **Audio & Advanced Features**: Sound, mappers, and cycle accuracy
7. **Debugging & Polish**: Tools, optimization, and compatibility (Track 4 milestone)

Each part contains chapters that build upon one another, and each chapter clearly marks which track(s) the content belongs to.

## Reading Strategies

You can read this book in several ways:

### Sequential Reading
Start at the beginning and read through to the end. This gives you the complete, step-by-step experience of building a NES emulator from scratch.

### Track-Based Reading
Follow only the chapters and sections relevant to your chosen track:
- For Track 1, follow the [Track 1 Guide](tracks/track1-guide.md)
- For Track 2, follow the [Track 2 Guide](tracks/track2-guide.md)
- For Track 3, follow the [Track 3 Guide](tracks/track3-guide.md)
- For Track 4, follow the [Track 4 Guide](tracks/track4-guide.md)

### Hybrid Approach
Start with Track 1 to get the basics working, then decide whether to continue with sequential reading or jump to a specific track.

## Track Indicators

Throughout the book, you'll see track indicators that help you navigate:

- <span class="track1-tag">T1</span> Content essential for Track 1
- <span class="track2-tag">T2</span> Content for Track 2
- <span class="track3-tag">T3</span> Content for Track 3
- <span class="track4-tag">T4</span> Content for Track 4

Sections will also be color-coded to help you quickly identify which track they belong to.

## Milestones and Checkpoints

Each track culminates in a milestone chapter that helps you verify your implementation works correctly. These chapters include:

- Self-assessment questions
- Component checklists
- Troubleshooting guidance
- Next steps

## Fast Forward Sections

If you're following a specific track, you'll find "Fast Forward" sections at the end of milestone chapters that help you jump ahead to the next relevant content for your track.

## Let's Get Started!

Now that you understand how to navigate the book, let's begin building our NES emulator! If you're ready to start coding, head to [NES Architecture Overview](part1-foundations/chapter1-1-nes-overview.md).
