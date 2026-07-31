use rn_core::audio::{Sample, SampleProducer};

/// Fans one sample stream out to several destinations — speakers, a visualiser, a recorder.
///
/// This is itself a [`SampleProducer`], so it drops into the audio path wherever a single producer
/// is expected and needs no thread, no intermediate queue and no periodic `tick` to pump it. The
/// earlier design consumed from a channel and had to be driven externally, which meant that
/// forgetting to drive it produced silence with nothing to indicate why.
///
/// Destinations must not block: each is called inline on the sample-producing thread, so a slow
/// consumer would stall emulation. The queue-backed producers in this crate all drop rather than
/// block, which is exactly the required behaviour.
pub struct Multiplexer<S: Sample> {
    producers: Vec<Box<dyn SampleProducer<S>>>,
}

impl<S: Sample> Multiplexer<S> {
    pub fn new() -> Self {
        Self { producers: Vec::new() }
    }

    pub fn add_producer(&mut self, producer: Box<dyn SampleProducer<S>>) {
        self.producers.push(producer);
    }

    pub fn with_producer(mut self, producer: Box<dyn SampleProducer<S>>) -> Self {
        self.add_producer(producer);
        self
    }

    pub fn len(&self) -> usize {
        self.producers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.producers.is_empty()
    }
}

impl<S: Sample> Default for Multiplexer<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Sample> SampleProducer<S> for Multiplexer<S> {
    fn set_volume(&mut self, volume: f32) {
        for producer in self.producers.iter_mut() {
            producer.set_volume(volume);
        }
    }

    fn set_muted(&mut self, muted: bool) {
        for producer in self.producers.iter_mut() {
            producer.set_muted(muted);
        }
    }

    fn produce(&mut self, sample: S) {
        for producer in self.producers.iter_mut() {
            producer.produce(sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{channel, Receiver, Sender};

    use super::*;

    struct Recorder(Sender<f32>);

    impl SampleProducer<f32> for Recorder {
        fn set_volume(&mut self, _volume: f32) {}
        fn set_muted(&mut self, _muted: bool) {}
        fn produce(&mut self, sample: f32) {
            let _ = self.0.send(sample);
        }
    }

    fn recorder() -> (Box<dyn SampleProducer<f32>>, Receiver<f32>) {
        let (sender, receiver) = channel();
        (Box::new(Recorder(sender)), receiver)
    }

    #[test]
    fn every_destination_receives_every_sample() {
        let (a, a_rx) = recorder();
        let (b, b_rx) = recorder();

        let mut mux = Multiplexer::new().with_producer(a).with_producer(b);
        for sample in [0.25, -0.5, 1.0] {
            mux.produce(sample);
        }

        assert_eq!(a_rx.try_iter().collect::<Vec<_>>(), vec![0.25, -0.5, 1.0]);
        assert_eq!(b_rx.try_iter().collect::<Vec<_>>(), vec![0.25, -0.5, 1.0]);
    }

    #[test]
    fn producing_with_no_destinations_is_harmless() {
        let mut mux: Multiplexer<f32> = Multiplexer::new();
        mux.produce(1.0);
        assert!(mux.is_empty());
    }
}
