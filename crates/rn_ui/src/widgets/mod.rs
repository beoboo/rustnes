#![allow(unused_imports)]

// Export all widget modules from here
mod asm_widget;
mod audio_capture_output;
mod audio_widget;
mod controller_widget;
mod cpu_widget;
mod disasm_widget;
mod dma_widget;
mod hex_edit_text;
mod key_conversion;
mod keyboard_mappings_widget;
mod memory_viz;
mod memory_widget;
mod pattern_table_widget;
mod pixel_display;
mod pixel_provider;
mod ppu_widget;
mod waveform_widget;

// Re-export types
// For backward compatibility with existing code
pub use asm_widget::AsmWidget;
pub use audio_capture_output::AudioCaptureOutput;
pub use audio_widget::AudioWidget;
pub use controller_widget::ControllerWidget;
pub use cpu_widget::CpuWidget;
pub use disasm_widget::DisasmWidget;
pub use dma_widget::DmaControllerWidget;
pub use hex_edit_text::{HexEditText, ValueType};
// Input conversion from egui keys to rn_input keys
pub use key_conversion::convert_egui_key;
pub use keyboard_mappings_widget::KeyboardMappingsWidget;
pub use memory_viz::MemoryVisualizer;
pub use memory_widget::MemoryWidget;
pub use pattern_table_widget::PatternTableWidget;
pub use pixel_display::PixelDisplay;
pub use pixel_provider::{MemoryPixelAdapter, PixelDataProvider, PpuPixelAdapter};
pub use ppu_widget::PpuWidget;
pub use waveform_widget::WaveformWidget;
