pub trait Sample: Clone + Copy + Send + 'static {}

impl Sample for f32 {}

/// The producing end of an audio path.
///
/// Only `Send` is required, not `Sync`: a producer is always owned by exactly one place and used
/// through `&mut self`, so it never needs to be shared by reference across threads. Demanding
/// `Sync` here would force lock-free queue types — whose handles are deliberately not `Sync` — to
/// paper over the mismatch with `unsafe impl`, which is exactly the kind of assertion that hides a
/// real bug later.
pub trait SampleProducer<T: Sample>: Send + 'static {
    /// Set the volume for audio output
    fn set_volume(&mut self, volume: f32);

    /// Set the muted state for audio output
    fn set_muted(&mut self, muted: bool);

    /// Produce a sample
    fn produce(&mut self, sample: T);
}

/// The consuming end of an audio path. `Send` for the same reason as [`SampleProducer`]: it is
/// moved onto the realtime callback thread and used exclusively from there.
pub trait SampleConsumer<T: Sample>: Send + 'static {
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
