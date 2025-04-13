use std::collections::VecDeque;

use rn_core::audio::SampleProducer;

/// Simple implementation of AudioOutput that collects samples but doesn't output them
#[derive(Debug)]
pub struct SimpleAudioOutput {
    sample_buffer: VecDeque<f32>,
    sample_rate: f32,
    volume: f32,
    muted: bool,
}

impl SimpleAudioOutput {
    pub fn new() -> Self {
        Self {
            sample_buffer: VecDeque::with_capacity(8192),
            sample_rate: 44100.0, // Default sample rate
            volume: 1.0,
            muted: false,
        }
    }
}

impl SampleProducer<f32> for SimpleAudioOutput {
    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    fn produce(&mut self, sample: f32) {
        self.sample_buffer.push_back(sample);

        // Keep buffer at a reasonable size
        if self.sample_buffer.len() > 8192 {
            self.sample_buffer.pop_front();
        }
    }
}
