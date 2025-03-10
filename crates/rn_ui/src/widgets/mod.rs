// Re-export widget modules
mod cpu_widget;
mod memory_widget;
mod memory_viz;
mod asm_widget;

// Re-export the widgets for easier access
pub use cpu_widget::CpuWidget;
pub use memory_widget::MemoryWidget;
pub use memory_viz::MemoryVisualizer;
pub use asm_widget::AsmWidget;
