//! Host audio output for the RustNES emulator.
//!
//! `rn_core` defines the whole contract as two traits — [`SampleProducer`] and [`SampleConsumer`] —
//! so the emulator core has no dependency on any host audio library. This crate implements them:
//!
//! - [`CpalAudioBuilder`] — the real output path: a lock-free ring buffer feeding a `cpal` stream
//! - [`ChannelBuilder`] — a bounded-channel path for taps that must never stall the speakers,
//!   such as the waveform visualiser
//! - [`Multiplexer`] — fans one stream out to several destinations
//! - [`Oscillator`] — a signal generator for testing the output path without the emulator
//! - [`SimpleAudioOutput`] — collects samples without playing them
//!
//! [`AudioControls`] carries the volume, mute state and telemetry shared along one path.

mod channel_buffer;
mod controls;
mod cpal_audio;
mod multiplexer;
mod oscillator;
mod ring_buffer;
mod simple_audio;

// Re-export common types
pub use anyhow::Result;
pub use channel_buffer::{ChannelBuilder, ChannelConsumer, ChannelProducer};
pub use controls::AudioControls;
pub use cpal_audio::{CpalAudioBuilder, CpalAudioConsumer, CpalAudioProducer};
pub use multiplexer::Multiplexer;
pub use oscillator::{Oscillator, Waveform};
pub use ring_buffer::{RingBufferBuilder, RingBufferConsumer, RingBufferProducer};
pub use simple_audio::SimpleAudioOutput;

// Re-exported so downstream crates can name the traits without also depending on rn_core.
pub use rn_core::audio::{Sample, SampleConsumer, SampleProducer};
