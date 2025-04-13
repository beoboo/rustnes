use std::sync::{
    atomic::{AtomicBool, AtomicU32},
    Arc,
};

use ringbuf::{
    storage::Heap,
    traits::{Consumer, Producer, Split},
    CachingCons, CachingProd, HeapRb, SharedRb,
};
use rn_core::audio::{Sample, SampleConsumer, SampleProducer};

pub struct RingBufferBuilder<T: Sample> {
    marker: std::marker::PhantomData<T>,
}

impl<T: Sample> RingBufferBuilder<T> {
    pub fn build(buffer_size: usize) -> (RingBufferProducer<T>, RingBufferConsumer<T>) {
        let volume = Arc::new(AtomicU32::new(f32::to_bits(1.0)));
        let muted = Arc::new(AtomicBool::new(false));

        let ring = HeapRb::<T>::new(buffer_size);
        let (producer, consumer) = ring.split();

        (
            RingBufferProducer::new(volume.clone(), muted.clone(), producer),
            RingBufferConsumer::new(volume.clone(), muted.clone(), consumer),
        )
    }
}

pub struct RingBufferProducer<T: Sample> {
    volume: Arc<AtomicU32>,
    muted: Arc<AtomicBool>,
    producer: CachingProd<Arc<SharedRb<Heap<T>>>>,
}

unsafe impl<T: Sample> Send for RingBufferProducer<T> {}
unsafe impl<T: Sample> Sync for RingBufferProducer<T> {}

impl<T: Sample> RingBufferProducer<T> {
    pub fn new(volume: Arc<AtomicU32>, muted: Arc<AtomicBool>, producer: CachingProd<Arc<SharedRb<Heap<T>>>>) -> Self {
        Self {
            volume,
            muted,
            producer,
        }
    }
}

impl<T: Sample> SampleProducer<T> for RingBufferProducer<T> {
    fn set_volume(&mut self, volume: f32) {
        self.volume
            .store(f32::to_bits(volume), std::sync::atomic::Ordering::Relaxed);
    }

    fn set_muted(&mut self, muted: bool) {
        self.muted.store(muted, std::sync::atomic::Ordering::Relaxed);
    }

    fn produce(&mut self, sample: T) {
        let _ = self.producer.try_push(sample);
    }
}
pub struct RingBufferConsumer<T: Sample> {
    volume: Arc<AtomicU32>,
    muted: Arc<AtomicBool>,
    consumer: CachingCons<Arc<SharedRb<Heap<T>>>>,
}

unsafe impl<T: Sample> Send for RingBufferConsumer<T> {}
unsafe impl<T: Sample> Sync for RingBufferConsumer<T> {}

impl<T: Sample> RingBufferConsumer<T> {
    pub fn new(volume: Arc<AtomicU32>, muted: Arc<AtomicBool>, consumer: CachingCons<Arc<SharedRb<Heap<T>>>>) -> Self {
        Self { volume, muted, consumer }
    }
}

impl<T: Sample> SampleConsumer<T> for RingBufferConsumer<T> {
    fn volume(&self) -> f32 {
        f32::from_bits(self.volume.load(std::sync::atomic::Ordering::Relaxed))
    }

    fn muted(&self) -> bool {
        self.muted.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn consume(&mut self) -> Option<T> {
        self.consumer.try_pop()
    }
}
