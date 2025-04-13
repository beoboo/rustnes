use std::fmt::Debug;


pub trait Sample: Clone + Copy + 'static {}

impl Sample for f32 {}

pub trait SampleProducer<T: Sample>: Send + Sync + 'static {
    fn produce(&mut self, sample: T);
}

pub trait SampleConsumer<T: Sample>: Send + Sync + 'static {
    fn consume(&mut self) -> Option<T>;
}

/// Trait for audio output devices that can receive samples from the APU
pub trait AudioOutput: Debug {
    /// Set the volume for audio output
    fn set_volume(&mut self, volume: f32);

    /// Set the muted state for audio output
    fn set_muted(&mut self, muted: bool);

    /// Queue a sample for playback
    ///
    /// The sample should be normalized to the range [-1.0, 1.0]
    fn queue_sample(&mut self, sample: f32);

    /// Clear any queued samples and stop playback
    fn clear(&mut self);

    /// Check if the audio device is ready for output
    fn is_ready(&self) -> bool;
}

/// Null audio output implementation that discards all samples
#[derive(Debug)]
pub struct NullAudioOutput;

impl AudioOutput for NullAudioOutput {
    fn set_volume(&mut self, _volume: f32) {
        // Do nothing
    }

    fn set_muted(&mut self, _muted: bool) {
        // Do nothing
    }

    fn queue_sample(&mut self, _sample: f32) {
        // Discard sample
    }

    fn clear(&mut self) {
        // Nothing to clear
    }

    fn is_ready(&self) -> bool {
        true // Always ready, since we're not doing anything
    }
}
