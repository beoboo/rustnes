//! Audio output implementations for the RustNES emulator.
//!
//! This crate provides two audio output implementations:
//! - SimpleAudioOutput: A simple audio output that just collects samples without playing them
//! - CpalAudioOutput: An audio output that uses cpal to play samples on the system's audio device

mod cpal_audio; 
mod channel_buffer;
mod multiplexer;
mod oscillator;
mod ring_buffer;
mod simple_audio;

// Re-export common types
pub use anyhow::Result;
pub use cpal_audio::{CpalAudioBuilder, CpalAudioPlayer, CpalAudioQueue};
pub use channel_buffer::{ChannelBuilder, ChannelConsumer, ChannelProducer};
pub use multiplexer::Multiplexer;
pub use oscillator::{Oscillator, Waveform};
use rn_core::audio::{Sample, SampleConsumer, SampleProducer};
pub use ring_buffer::{RingBufferBuilder, RingBufferConsumer, RingBufferProducer};
pub use simple_audio::SimpleAudioOutput;
