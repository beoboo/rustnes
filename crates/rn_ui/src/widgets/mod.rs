#![allow(unused_imports)]

// Re-export widget modules
mod asm_widget;
mod cpu_widget;
mod disasm_widget;
mod hex_edit_text;
mod memory_viz;
mod memory_widget;
mod pattern_table_widget;
mod pixel_display;
mod pixel_provider;
mod ppu_widget;

// Re-export the widgets for easier access
pub use asm_widget::AsmWidget;
pub use cpu_widget::CpuWidget;
pub use disasm_widget::DisasmWidget;
pub use hex_edit_text::{HexEditText, ValueType};
pub use memory_viz::MemoryVisualizer;
pub use memory_widget::MemoryWidget;
pub use pattern_table_widget::PatternTableWidget;
pub use pixel_display::PixelDisplay;
pub use pixel_provider::{MemoryPixelAdapter, PixelDataProvider, PpuPixelAdapter};
pub use ppu_widget::PpuWidget;
