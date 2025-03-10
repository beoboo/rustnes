#![allow(unused_imports)]

// Re-export widget modules
mod asm_widget;
mod cpu_widget;
mod disasm_widget;
mod memory_viz;
mod memory_widget;

// Re-export the widgets for easier access
pub use asm_widget::AsmWidget;
pub use cpu_widget::CpuWidget;
pub use disasm_widget::DisasmWidget;
pub use memory_viz::MemoryVisualizer;
pub use memory_widget::MemoryWidget;
