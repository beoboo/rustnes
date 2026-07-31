use std::sync::Arc;

use ringbuf::{
    storage::Heap,
    traits::{Consumer, Observer, Producer, Split},
    CachingCons, CachingProd, HeapRb, SharedRb,
};
use rn_core::audio::{Sample, SampleConsumer, SampleProducer};

use crate::controls::AudioControls;

pub struct RingBufferBuilder<T: Sample> {
    marker: std::marker::PhantomData<T>,
}

impl<T: Sample> RingBufferBuilder<T> {
    /// Build a single-producer / single-consumer ring buffer sharing `controls`.
    ///
    /// Both ends see the same volume, mute state and telemetry — there is deliberately no
    /// second set of atomics anywhere in the path.
    pub fn build(buffer_size: usize, controls: AudioControls) -> (RingBufferProducer<T>, RingBufferConsumer<T>) {
        let ring = HeapRb::<T>::new(buffer_size);
        let (producer, consumer) = ring.split();

        // Occupancy is published through the shared controls so the emulator can read it even
        // after the producer has been handed off, and without touching the consumer (which lives
        // on the realtime thread).
        controls.set_capacity(buffer_size);
        controls.set_queued(0);

        (
            RingBufferProducer::new(controls.clone(), producer),
            RingBufferConsumer::new(controls, consumer),
        )
    }
}

pub struct RingBufferProducer<T: Sample> {
    controls: AudioControls,
    producer: CachingProd<Arc<SharedRb<Heap<T>>>>,
}

impl<T: Sample> RingBufferProducer<T> {
    pub fn new(controls: AudioControls, producer: CachingProd<Arc<SharedRb<Heap<T>>>>) -> Self {
        Self { controls, producer }
    }

    /// How many samples are currently queued for playback.
    pub fn queued(&self) -> usize {
        self.controls.queued()
    }

    /// Buffer occupancy in the range 0.0 (empty, about to underrun) to 1.0 (full).
    pub fn fill_level(&self) -> f32 {
        self.controls.fill_level()
    }

    pub fn capacity(&self) -> usize {
        self.controls.capacity()
    }

    pub fn controls(&self) -> &AudioControls {
        &self.controls
    }
}

impl<T: Sample> SampleProducer<T> for RingBufferProducer<T> {
    fn set_volume(&mut self, volume: f32) {
        self.controls.set_volume(volume);
    }

    fn set_muted(&mut self, muted: bool) {
        self.controls.set_muted(muted);
    }

    fn produce(&mut self, sample: T) {
        if self.producer.try_push(sample).is_err() {
            // Buffer full: the emulator is running ahead of the sound card. Dropping is the right
            // recovery, but it must be visible rather than silent.
            self.controls.record_dropped();
        }
        self.controls.set_queued(self.producer.occupied_len());
    }
}

pub struct RingBufferConsumer<T: Sample> {
    controls: AudioControls,
    consumer: CachingCons<Arc<SharedRb<Heap<T>>>>,
}

impl<T: Sample> RingBufferConsumer<T> {
    pub fn new(controls: AudioControls, consumer: CachingCons<Arc<SharedRb<Heap<T>>>>) -> Self {
        Self { controls, consumer }
    }
}

impl<T: Sample> SampleConsumer<T> for RingBufferConsumer<T> {
    fn volume(&self) -> f32 {
        self.controls.volume()
    }

    fn muted(&self) -> bool {
        self.controls.muted()
    }

    fn consume(&mut self) -> Option<T> {
        let sample = self.consumer.try_pop();
        if sample.is_none() {
            self.controls.record_underrun();
        }
        self.controls.set_queued(self.consumer.occupied_len());
        sample
    }
}
