
use std::sync::{atomic::{AtomicBool, AtomicU32}, Arc};
use crossbeam_channel::{Receiver, Sender};
use rn_core::audio::{Sample, SampleConsumer, SampleProducer};

pub struct ChannelBuilder<T: Sample> {
  marker: std::marker::PhantomData<T>,
}

impl<T: Sample> ChannelBuilder<T> {
  pub fn build(buffer_size: usize) -> (ChannelProducer<T>, ChannelConsumer<T>) {
      let volume = Arc::new(AtomicU32::new(f32::to_bits(1.0)));
      let muted = Arc::new(AtomicBool::new(false));

      let (sender, receiver) = crossbeam_channel::bounded(buffer_size);
      (ChannelProducer::new(volume.clone(), muted.clone(), sender), ChannelConsumer{volume, muted, receiver})
  }
}

pub struct ChannelProducer<T: Sample>{
  volume: Arc<AtomicU32>,
  muted: Arc<AtomicBool>,

  sender: Sender<T>,
}

impl<T: Sample> ChannelProducer<T> {
  pub fn new(volume: Arc<AtomicU32>, muted: Arc<AtomicBool>, sender: Sender<T>) -> Self {
      Self { volume, muted, sender }
  }
}

unsafe impl<T: Sample> Send for ChannelProducer<T> {}
unsafe impl<T: Sample> Sync for ChannelProducer<T> {}

impl<T: Sample> SampleProducer<T> for ChannelProducer<T> {
  fn set_volume(&mut self, volume: f32) {
      self.volume.store(f32::to_bits(volume), std::sync::atomic::Ordering::Relaxed);
  }

  fn set_muted(&mut self, muted: bool) {
      self.muted.store(muted, std::sync::atomic::Ordering::Relaxed);
  }

  fn produce(&mut self, sample: T) {
      let _ = self.sender.send(sample);
  }
}

pub struct ChannelConsumer<T: Sample>{ 
  volume: Arc<AtomicU32>,
  muted: Arc<AtomicBool>,
  receiver: Receiver<T>,
}

unsafe impl<T: Sample> Send for ChannelConsumer<T> {}
unsafe impl<T: Sample> Sync for ChannelConsumer<T> {}

impl<T: Sample> SampleConsumer<T> for ChannelConsumer<T> {
  fn volume(&self) -> f32 {
      f32::from_bits(self.volume.load(std::sync::atomic::Ordering::Relaxed))
  }

  fn muted(&self) -> bool {
      self.muted.load(std::sync::atomic::Ordering::Relaxed)
  }

  fn consume(&mut self) -> Option<T> {
      self.receiver.try_recv().ok()
  }
}
