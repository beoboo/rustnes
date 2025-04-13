
use crate::{Sample, SampleConsumer, SampleProducer};

/// Audio sample multiplexer that distributes audio samples to multiple consumers
/// without requiring mutexes or additional threads.
///
/// This uses bounded crossbeam channels for lock-free communication.
pub struct Multiplexer<S: Sample, C: SampleConsumer<S>> {
    consumer: C,

    producers: Vec<Box<dyn SampleProducer<S>>>,
}

impl<S: Sample, C: SampleConsumer<S>> Multiplexer<S, C> {
    pub fn new(consumer: C) -> Self {
        Self {
            consumer,
            producers: Vec::new(),
        }
    }

    pub fn add_producer(&mut self, producer: Box<dyn SampleProducer<S>>) {
        self.producers.push(producer);
    }

    pub fn tick(&mut self) {
        if let Some(sample) = self.consumer.consume() {
            for producer in self.producers.iter_mut() {
                producer.produce(sample);
            }
        }
    }
}
