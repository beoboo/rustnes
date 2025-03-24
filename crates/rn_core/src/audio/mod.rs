use std::fmt::Debug;
/// Trait for audio output devices that can receive samples from the APU
pub trait AudioOutput: Debug {
    /// Set the sample rate for audio output
    fn set_sample_rate(&mut self, sample_rate: f32);
    
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
    fn set_sample_rate(&mut self, _sample_rate: f32) {
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

// First, let's see what's in the audio module to understand the AudioOutput trait 