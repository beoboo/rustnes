use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use rn_core::audio::AudioOutput;

/// Simple implementation of AudioOutput that collects samples but doesn't output them
#[derive(Debug)]
pub struct SimpleAudioOutput {
    sample_buffer: VecDeque<f32>,
    sample_rate: f32,
}

impl SimpleAudioOutput {
    pub fn new() -> Self {
        Self {
            sample_buffer: VecDeque::with_capacity(8192),
            sample_rate: 44100.0, // Default sample rate
        }
    }
}

impl AudioOutput for SimpleAudioOutput {
    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
    }

    fn queue_sample(&mut self, sample: f32) {
        self.sample_buffer.push_back(sample);

        // Keep buffer at a reasonable size
        if self.sample_buffer.len() > 8192 {
            self.sample_buffer.pop_front();
        }
    }

    fn clear(&mut self) {
        self.sample_buffer.clear();
    }

    fn is_ready(&self) -> bool {
        true // Always ready to receive samples
    }
}

/// Cloneable wrapper for SimpleAudioOutput
#[derive(Clone, Debug)]
pub struct SimpleAudioOutputWrapper {
    inner: Rc<RefCell<SimpleAudioOutput>>,
}

impl SimpleAudioOutputWrapper {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(SimpleAudioOutput::new())),
        }
    }
}

impl AudioOutput for SimpleAudioOutputWrapper {
    fn set_sample_rate(&mut self, rate: f32) {
        self.inner.borrow_mut().set_sample_rate(rate);
    }

    fn queue_sample(&mut self, sample: f32) {
        self.inner.borrow_mut().queue_sample(sample);
    }

    fn clear(&mut self) {
        self.inner.borrow_mut().clear();
    }

    fn is_ready(&self) -> bool {
        self.inner.borrow().is_ready()
    }
}
