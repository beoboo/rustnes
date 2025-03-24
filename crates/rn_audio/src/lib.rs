use std::{
    cell::RefCell, collections::VecDeque, rc::Rc, sync::{Arc, Mutex}
};

use rn_core::audio::AudioOutput;

/// Simple implementation of AudioOutput that collects samples but doesn't output them
#[derive(Debug)]
pub struct SimpleAudioOutput {
    sample_buffer: Arc<Mutex<VecDeque<f32>>>,
    sample_rate: f32,
}

impl SimpleAudioOutput {
    pub fn new() -> Self {
        Self {
            sample_buffer: Arc::new(Mutex::new(VecDeque::new())),
            sample_rate: 44100.0, // Default sample rate
        }
    }
}

impl AudioOutput for SimpleAudioOutput {
    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
    }
    
    fn queue_sample(&mut self, sample: f32) {
        if let Ok(mut buffer) = self.sample_buffer.lock() {
            buffer.push_back(sample);
            
            // Keep buffer at a reasonable size
            if buffer.len() > 8192 {
                buffer.pop_front();
            }
        }
    }
    
    fn clear(&mut self) {
        if let Ok(mut buffer) = self.sample_buffer.lock() {
            buffer.clear();
        }
    }
    
    fn is_ready(&self) -> bool {
        true // Always ready to receive samples
    }
} 

#[derive(Clone, Debug)]
pub struct SimpleAudioOutputWrapper {
  inner: Rc<RefCell<SimpleAudioOutput>>,
}

impl SimpleAudioOutputWrapper {
  pub fn new() -> Self {
    Self { inner: Rc::new(RefCell::new(SimpleAudioOutput::new())) }
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
