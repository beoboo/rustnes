// Re-export widget modules
mod cpu_widget;
mod memory_widget;

// Re-export the widgets for easier access
pub use cpu_widget::CpuWidget;
pub use memory_widget::MemoryWidget; 