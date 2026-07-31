
use crossbeam_channel::{Receiver, Sender, TrySendError};
use rn_core::audio::{Sample, SampleConsumer, SampleProducer};

use crate::controls::AudioControls;

/// A bounded-channel audio path.
///
/// Used for taps that must never block or stall the speaker path — the waveform visualiser, for
/// example, which reads whenever the UI happens to repaint.
pub struct ChannelBuilder<T: Sample> {
    marker: std::marker::PhantomData<T>,
}

impl<T: Sample> ChannelBuilder<T> {
    pub fn build(buffer_size: usize) -> (ChannelProducer<T>, ChannelConsumer<T>) {
        Self::build_with(buffer_size, AudioControls::new())
    }

    pub fn build_with(buffer_size: usize, controls: AudioControls) -> (ChannelProducer<T>, ChannelConsumer<T>) {
        let (sender, receiver) = crossbeam_channel::bounded(buffer_size);
        (
            ChannelProducer::new(controls.clone(), sender),
            ChannelConsumer::new(controls, receiver),
        )
    }
}

pub struct ChannelProducer<T: Sample> {
    controls: AudioControls,
    sender: Sender<T>,
}

impl<T: Sample> ChannelProducer<T> {
    pub fn new(controls: AudioControls, sender: Sender<T>) -> Self {
        Self { controls, sender }
    }
}

impl<T: Sample> SampleProducer<T> for ChannelProducer<T> {
    fn set_volume(&mut self, volume: f32) {
        self.controls.set_volume(volume);
    }

    fn set_muted(&mut self, muted: bool) {
        self.controls.set_muted(muted);
    }

    fn produce(&mut self, sample: T) {
        // `try_send`, never `send`: a slow or absent reader must not stall sample generation.
        match self.sender.try_send(sample) {
            Ok(()) => {},
            Err(TrySendError::Full(_)) => self.controls.record_dropped(),
            Err(TrySendError::Disconnected(_)) => {},
        }
    }
}

pub struct ChannelConsumer<T: Sample> {
    controls: AudioControls,
    receiver: Receiver<T>,
}

impl<T: Sample> ChannelConsumer<T> {
    pub fn new(controls: AudioControls, receiver: Receiver<T>) -> Self {
        Self { controls, receiver }
    }
}

impl<T: Sample> SampleConsumer<T> for ChannelConsumer<T> {
    fn volume(&self) -> f32 {
        self.controls.volume()
    }

    fn muted(&self) -> bool {
        self.controls.muted()
    }

    fn consume(&mut self) -> Option<T> {
        self.receiver.try_recv().ok()
    }
}
