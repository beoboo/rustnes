use std::fmt::Debug;


pub trait Sample: Clone + Copy + 'static {}

impl Sample for f32 {}

pub trait SampleProducer<T: Sample>: Send + Sync + 'static {
    /// Set the volume for audio output
    fn set_volume(&mut self, volume: f32);

    /// Set the muted state for audio output
    fn set_muted(&mut self, muted: bool);

    /// Produce a sample
    fn produce(&mut self, sample: T);
}

pub trait SampleConsumer<T: Sample>: Send + Sync + 'static {
    /// Get the volume for audio output
    fn volume(&self) -> f32;

    /// Get the muted state for audio output
    fn muted(&self) -> bool;

    /// Consume a sample
    fn consume(&mut self) -> Option<T>;
}

/// Null audio output implementation that discards all samples
#[derive(Debug)]
pub struct NullAudioOutput;

impl SampleProducer<f32> for NullAudioOutput {
    fn set_volume(&mut self, _volume: f32) {
        // Do nothing
    }

    fn set_muted(&mut self, _muted: bool) {
        // Do nothing
    }

    fn produce(&mut self, _sample: f32) {
        // Discard sample
    }
}
