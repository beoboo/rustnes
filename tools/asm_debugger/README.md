# AsmDebugger

An assembly debugger tool for RustNES emulator with a graphical interface based on egui.

## Features (Planned)

- Memory visualization for range 0x0200-0x05FF as pixels
- CPU state display and manipulation
- Code disassembly view
- Step-by-step execution control
- Breakpoint support
- Memory editor
- PPU memory viewing and editing
- Visual memory map
- Performance profiling

## Usage

```bash
# Build and run the debugger
cargo run --package asm_debugger
```

## Architecture

The debugger connects to the RustNES core components and provides visual representation and control through an egui-based interface. It can be used both as a standalone tool and integrated with the main emulator UI.

## Development Status

This tool is currently in early development. See the project-wide TODO.md for current development progress and roadmap. 