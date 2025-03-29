#![allow(dead_code)]
use std::{
    collections::VecDeque,
    fmt,
    sync::{Arc, Mutex},
};

use rn_core::audio::AudioOutput;

/// Audio output implementation that captures samples for visualization
#[derive(Debug, Clone)]
pub struct AudioCaptureOutput {
    // The actual audio output that will play the sound
    inner: Arc<Mutex<Box<dyn AudioOutput>>>,

    // Captured samples for visualization
    samples: Arc<Mutex<VecDeque<f32>>>,

    // Audio state
    volume: f32,
    muted: bool,
}

impl AudioCaptureOutput {
    /// Create a new audio capture wrapper around an existing audio output
    pub fn new(inner: Box<dyn AudioOutput>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
            samples: Arc::new(Mutex::new(VecDeque::with_capacity(1024))),
            volume: 1.0,
            muted: false,
        }
    }

    /// Get a clone of the sample buffer for visualization
    pub fn get_samples(&self) -> Arc<Mutex<VecDeque<f32>>> {
        self.samples.clone()
    }
}

impl AudioOutput for AudioCaptureOutput {
    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        if let Ok(mut inner) = self.inner.lock() {
            inner.set_volume(volume);
        }
    }

    fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        if let Ok(mut inner) = self.inner.lock() {
            inner.set_muted(muted);
        }
    }

    fn queue_sample(&mut self, sample: f32) {
        // Store the sample for visualization
        if let Ok(mut samples) = self.samples.lock() {
            samples.push_back(sample);

            // Keep the buffer at a reasonable size
            if samples.len() > 1024 {
                samples.pop_front();
            }
        }

        // Forward the sample to the inner audio output
        if let Ok(mut inner) = self.inner.lock() {
            inner.queue_sample(sample);
        }
    }

    fn clear(&mut self) {
        // Clear the visualization buffer
        if let Ok(mut samples) = self.samples.lock() {
            samples.clear();
        }

        // Clear the inner audio output
        if let Ok(mut inner) = self.inner.lock() {
            inner.clear();
        }
    }

    fn is_ready(&self) -> bool {
        if let Ok(inner) = self.inner.lock() {
            inner.is_ready()
        } else {
            false
        }
    }
}
