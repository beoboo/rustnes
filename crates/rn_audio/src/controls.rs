use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering},
    Arc,
};

/// Shared, lock-free control and telemetry state for one audio path.
///
/// A single `AudioControls` is created per stream and cloned to everyone who needs it: the
/// producer side (driven by the APU), the consumer side, and the realtime callback. Cloning shares
/// the underlying atomics, so a volume change made on any handle is visible to all of them.
///
/// Everything here is `Relaxed`: these are independent scalars with no ordering relationship to
/// each other or to the sample data, and the callback must never block.
#[derive(Clone, Debug)]
pub struct AudioControls {
    volume: Arc<AtomicU32>,
    muted: Arc<AtomicBool>,
    underruns: Arc<AtomicU64>,
    dropped: Arc<AtomicU64>,
    queued: Arc<AtomicUsize>,
    capacity: Arc<AtomicUsize>,
}

impl AudioControls {
    pub fn new() -> Self {
        Self {
            volume: Arc::new(AtomicU32::new(f32::to_bits(1.0))),
            muted: Arc::new(AtomicBool::new(false)),
            underruns: Arc::new(AtomicU64::new(0)),
            dropped: Arc::new(AtomicU64::new(0)),
            queued: Arc::new(AtomicUsize::new(0)),
            capacity: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn volume(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::Relaxed))
    }

    pub fn set_volume(&self, volume: f32) {
        self.volume.store(f32::to_bits(volume.clamp(0.0, 1.0)), Ordering::Relaxed);
    }

    pub fn muted(&self) -> bool {
        self.muted.load(Ordering::Relaxed)
    }

    pub fn set_muted(&self, muted: bool) {
        self.muted.store(muted, Ordering::Relaxed);
    }

    /// Number of times the output callback wanted a sample and the buffer was empty.
    ///
    /// A non-zero and growing count means the emulator is not producing fast enough.
    pub fn underruns(&self) -> u64 {
        self.underruns.load(Ordering::Relaxed)
    }

    pub fn record_underrun(&self) {
        self.underruns.fetch_add(1, Ordering::Relaxed);
    }

    /// Number of samples the producer had to throw away because the buffer was full.
    ///
    /// The mirror image of an underrun: the emulator is producing faster than the device drains.
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    pub fn record_dropped(&self) {
        self.dropped.fetch_add(1, Ordering::Relaxed);
    }

    pub fn reset_stats(&self) {
        self.underruns.store(0, Ordering::Relaxed);
        self.dropped.store(0, Ordering::Relaxed);
    }

    /// Samples currently queued for playback.
    ///
    /// Lives here rather than on the producer so the emulator can still read it after the producer
    /// has been handed off to the APU — that reading is what lets emulation pace itself against
    /// the audio clock instead of against the UI's repaint rate.
    pub fn queued(&self) -> usize {
        self.queued.load(Ordering::Relaxed)
    }

    pub fn set_queued(&self, queued: usize) {
        self.queued.store(queued, Ordering::Relaxed);
    }

    pub fn capacity(&self) -> usize {
        self.capacity.load(Ordering::Relaxed)
    }

    pub fn set_capacity(&self, capacity: usize) {
        self.capacity.store(capacity, Ordering::Relaxed);
    }

    /// Buffer occupancy: 0.0 is empty and about to underrun, 1.0 is full.
    pub fn fill_level(&self) -> f32 {
        let capacity = self.capacity();
        if capacity == 0 {
            return 0.0;
        }
        (self.queued() as f32 / capacity as f32).min(1.0)
    }
}

impl Default for AudioControls {
    fn default() -> Self {
        Self::new()
    }
}
