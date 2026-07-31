use anyhow::Result;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SizedSample, Stream,
};
use log::{debug, info, warn};
use rn_core::audio::{SampleConsumer, SampleProducer};
use std::fmt;

use crate::{
    controls::AudioControls,
    ring_buffer::{RingBufferBuilder, RingBufferProducer},
};

/// Target buffering, in milliseconds.
///
/// This is the latency between the emulator producing a sample and hearing it. It also sets how
/// much slack the emulator has to fall behind before the device underruns, so it cannot be made
/// arbitrarily small; ~100 ms is a comfortable compromise for a debugger that also has to render a
/// UI on the same machine.
const TARGET_LATENCY_MS: f32 = 100.0;

pub struct CpalAudioBuilder;

impl CpalAudioBuilder {
    pub fn build_default() -> Result<(CpalAudioProducer, CpalAudioConsumer)> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No output device available"))?;

        info!("Using audio device: {}", device.name()?);

        let config = device.default_output_config()?;
        debug!("Default output config: {:?}", config);

        Self::build(device, config)
    }

    pub fn build(
        device: cpal::Device,
        config: cpal::SupportedStreamConfig,
    ) -> Result<(CpalAudioProducer, CpalAudioConsumer)> {
        let sample_rate = config.sample_rate().0 as f32;

        // The buffer holds mono samples: the callback duplicates each one across output channels,
        // so its size is independent of the channel count.
        let buffer_size = ((TARGET_LATENCY_MS / 1_000.0) * sample_rate) as usize;

        // One set of controls for the whole path — producer, consumer and callback all share it.
        let controls = AudioControls::new();
        let (producer, consumer) = RingBufferBuilder::build(buffer_size, controls.clone());

        let audio_queue = CpalAudioProducer::new(producer, controls.clone(), sample_rate);

        let mut audio_output = CpalAudioConsumer::new(device, controls)?;
        audio_output.initialize(consumer, config)?;

        Ok((audio_queue, audio_output))
    }
}

pub struct CpalAudioProducer {
    producer: RingBufferProducer<f32>,
    controls: AudioControls,
    sample_rate: f32,
}

impl fmt::Debug for CpalAudioProducer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CpalAudioProducer")
            .field("sample_rate", &self.sample_rate)
            .field("volume", &self.controls.volume())
            .field("muted", &self.controls.muted())
            .field("fill_level", &self.producer.fill_level())
            .finish()
    }
}

impl CpalAudioProducer {
    fn new(producer: RingBufferProducer<f32>, controls: AudioControls, sample_rate: f32) -> Self {
        Self {
            producer,
            controls,
            sample_rate,
        }
    }

    /// The device's actual sample rate. The APU needs this to resample correctly, rather than
    /// assuming a fixed 44.1 kHz.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Buffer occupancy, 0.0 (empty) to 1.0 (full). The emulator paces itself against this.
    pub fn fill_level(&self) -> f32 {
        self.producer.fill_level()
    }

    pub fn queued(&self) -> usize {
        self.producer.queued()
    }

    pub fn controls(&self) -> AudioControls {
        self.controls.clone()
    }
}

impl SampleProducer<f32> for CpalAudioProducer {
    fn set_volume(&mut self, volume: f32) {
        self.controls.set_volume(volume);
    }

    fn set_muted(&mut self, muted: bool) {
        self.controls.set_muted(muted);
    }

    fn produce(&mut self, sample: f32) {
        self.producer.produce(sample);
    }
}

pub struct CpalAudioConsumer {
    device: cpal::Device,
    controls: AudioControls,
    stream: Option<Stream>,
    sample_rate: f32,
}

impl CpalAudioConsumer {
    fn new(device: cpal::Device, controls: AudioControls) -> Result<Self> {
        Ok(Self {
            device,
            controls,
            stream: None,
            sample_rate: 0.0,
        })
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn controls(&self) -> AudioControls {
        self.controls.clone()
    }

    fn initialize<C: SampleConsumer<f32>>(&mut self, consumer: C, config: cpal::SupportedStreamConfig) -> Result<()> {
        self.sample_rate = config.sample_rate().0 as f32;

        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => self.make_stream::<i8, C>(consumer, &config.into()),
            cpal::SampleFormat::I16 => self.make_stream::<i16, C>(consumer, &config.into()),
            cpal::SampleFormat::I32 => self.make_stream::<i32, C>(consumer, &config.into()),
            cpal::SampleFormat::I64 => self.make_stream::<i64, C>(consumer, &config.into()),
            cpal::SampleFormat::U8 => self.make_stream::<u8, C>(consumer, &config.into()),
            cpal::SampleFormat::U16 => self.make_stream::<u16, C>(consumer, &config.into()),
            cpal::SampleFormat::U32 => self.make_stream::<u32, C>(consumer, &config.into()),
            cpal::SampleFormat::U64 => self.make_stream::<u64, C>(consumer, &config.into()),
            cpal::SampleFormat::F32 => self.make_stream::<f32, C>(consumer, &config.into()),
            cpal::SampleFormat::F64 => self.make_stream::<f64, C>(consumer, &config.into()),
            sample_format => Err(anyhow::Error::msg(format!(
                "Unsupported sample format '{sample_format}'"
            ))),
        }?;

        self.stream = Some(stream);

        Ok(())
    }

    pub fn play(&mut self) -> Result<()> {
        if let Some(stream) = &self.stream {
            debug!("Starting audio stream");
            stream.play()?;
        }
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        if let Some(stream) = &self.stream {
            debug!("Pausing audio stream");
            stream.pause()?;
        }
        Ok(())
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.controls.set_volume(volume);
    }

    fn make_stream<S, C>(&mut self, mut consumer: C, config: &cpal::StreamConfig) -> Result<cpal::Stream>
    where
        S: SizedSample + FromSample<f32>,
        C: SampleConsumer<f32>,
    {
        let num_channels = config.channels as usize;
        let err_fn = |err| warn!("Audio output stream error: {}", err);
        let controls = self.controls.clone();

        let stream = self.device.build_output_stream(
            config,
            move |output: &mut [S], _: &cpal::OutputCallbackInfo| {
                Self::process_frame(output, &mut consumer, num_channels, &controls)
            },
            err_fn,
            None,
        )?;

        Ok(stream)
    }

    /// The realtime audio callback.
    ///
    /// This runs on a thread with a hard deadline. It must not allocate, lock, log or perform any
    /// I/O — it only reads atomics and pops from the lock-free queue. `consume()` returning `None`
    /// is recorded as an underrun by the consumer and substituted with silence.
    fn process_frame<S, C>(output: &mut [S], consumer: &mut C, num_channels: usize, controls: &AudioControls)
    where
        S: Sample + FromSample<f32>,
        C: SampleConsumer<f32>,
    {
        // Read the controls once per callback rather than per sample: they change at UI rate, and
        // reloading them for every frame buys nothing but cache traffic.
        let gain = if controls.muted() { 0.0 } else { controls.volume() };

        for frame in output.chunks_mut(num_channels) {
            let value = S::from_sample(consumer.consume().unwrap_or(0.0) * gain);

            // Mono source: the same value goes to every output channel.
            for s in frame.iter_mut() {
                *s = value;
            }
        }
    }
}
