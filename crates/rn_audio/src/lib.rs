//! Audio output implementations for the RustNES emulator.
//!
//! This crate provides two audio output implementations:
//! - SimpleAudioOutput: A simple audio output that just collects samples without playing them
//! - CpalAudioOutput: An audio output that uses cpal to play samples on the system's audio device

mod cpal_audio;
mod multiplexer;
mod oscillator;
mod simple_audio;

use std::sync::Arc;

// Re-export common types
pub use anyhow::Result;
pub use cpal_audio::{CpalAudioBuilder, CpalAudioPlayer, CpalAudioQueue};
use crossbeam_channel::{Receiver, Sender};
pub use multiplexer::Multiplexer;
pub use oscillator::{Oscillator, Waveform};
use ringbuf::{
    storage::Heap,
    traits::{Consumer, Producer, Split},
    CachingCons, CachingProd, HeapRb, SharedRb,
};
use rn_core::audio::{Sample, SampleConsumer, SampleProducer};
pub use simple_audio::SimpleAudioOutput;

pub struct RingBufferBuilder<T: Sample> {
    marker: std::marker::PhantomData<T>,
}

impl<T: Sample> RingBufferBuilder<T> {
    pub fn build(buffer_size: usize) -> (RingBufferProducer<T>, RingBufferConsumer<T>) {
        let ring = HeapRb::<T>::new(buffer_size);
        let (producer, consumer) = ring.split();
        (RingBufferProducer(producer), RingBufferConsumer(consumer))
    }
}

pub struct RingBufferProducer<T: Sample>(CachingProd<Arc<SharedRb<Heap<T>>>>);
unsafe impl<T: Sample> Send for RingBufferProducer<T> {}
unsafe impl<T: Sample> Sync for RingBufferProducer<T> {}

impl<T: Sample> SampleProducer<T> for RingBufferProducer<T> {
    fn produce(&mut self, sample: T) {
        let _ = self.0.try_push(sample);
    }
}
pub struct RingBufferConsumer<T: Sample>(CachingCons<Arc<SharedRb<Heap<T>>>>);
unsafe impl<T: Sample> Send for RingBufferConsumer<T> {}
unsafe impl<T: Sample> Sync for RingBufferConsumer<T> {}

impl<T: Sample> SampleConsumer<T> for RingBufferConsumer<T> {
    fn consume(&mut self) -> Option<T> {
        self.0.try_pop()
    }
}

pub struct ChannelBuilder<T: Sample> {
    marker: std::marker::PhantomData<T>,
}

impl<T: Sample> ChannelBuilder<T> {
    pub fn build(buffer_size: usize) -> (ChannelProducer<T>, ChannelConsumer<T>) {
        let (sender, receiver) = crossbeam_channel::bounded(buffer_size);
        (ChannelProducer(sender), ChannelConsumer(receiver))
    }
}

pub struct ChannelProducer<T: Sample>(Sender<T>);
unsafe impl<T: Sample> Send for ChannelProducer<T> {}
unsafe impl<T: Sample> Sync for ChannelProducer<T> {}

impl<T: Sample> SampleProducer<T> for ChannelProducer<T> {
    fn produce(&mut self, sample: T) {
        let _ = self.0.send(sample);
    }
}

pub struct ChannelConsumer<T: Sample>(Receiver<T>);
unsafe impl<T: Sample> Send for ChannelConsumer<T> {}
unsafe impl<T: Sample> Sync for ChannelConsumer<T> {}

impl<T: Sample> SampleConsumer<T> for ChannelConsumer<T> {
    fn consume(&mut self) -> Option<T> {
        self.0.try_recv().ok()
    }
}
