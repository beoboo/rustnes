//! Audio output implementations for the RustNES emulator.
//!
//! This crate provides two audio output implementations:
//! - SimpleAudioOutput: A simple audio output that just collects samples without playing them
//! - CpalAudioOutput: An audio output that uses cpal to play samples on the system's audio device

mod cpal_audio;
mod simple_audio;

// Re-export common types
pub use anyhow::Result;
pub use cpal_audio::CpalAudioOutput;
pub use simple_audio::SimpleAudioOutput;
