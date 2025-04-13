use anyhow::Result;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait}, SizedSample, Stream,
};
use cpal::{FromSample, Sample};
use rn_core::audio::{AudioOutput, SampleProducer, SampleConsumer};
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::{fmt, sync::Arc};

use crate::ring_buffer::{RingBufferBuilder, RingBufferProducer};

pub struct CpalAudioBuilder;

impl CpalAudioBuilder {
    pub fn build_default() -> Result<(CpalAudioQueue, CpalAudioPlayer)> {
        // Get default host
        let host = cpal::default_host();

        // Get default output device
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No output device available"))?;

        println!("Using audio device: {}", device.name()?);

        // Get supported config
        let config = device.default_output_config()?;
        println!("Default output config: {:?}", config);

        Self::build(device, config)
    }

    pub fn build(
        device: cpal::Device,
        config: cpal::SupportedStreamConfig,
    ) -> Result<(CpalAudioQueue, CpalAudioPlayer)> {
        let latency_ms = 250.0; // Reduced from 1000ms to 250ms for better responsiveness
        let sample_rate = config.sample_rate().0 as f32;
        let num_channels = config.channels() as usize;

        let latency_frames = (latency_ms / 1_000.0) * sample_rate;
        let latency_samples = latency_frames as usize * num_channels;

        // Create a larger buffer to prevent underruns
        let (producer, consumer) = RingBufferBuilder::build(latency_samples * 4);
        let volume = Arc::new(AtomicU32::new(f32::to_bits(1.0)));
        let muted = Arc::new(AtomicBool::new(false));
        let clear = Arc::new(AtomicBool::new(false));

        let audio_queue = CpalAudioQueue::new(Box::new(producer), volume.clone(), muted.clone(), clear.clone());

        let mut audio_output = CpalAudioPlayer::new(device, volume, muted, clear)?;
        audio_output.initialize(consumer, config)?;

        Ok((audio_queue, audio_output))
    }
}

pub struct CpalAudioQueue {
    producer: Box<dyn SampleProducer<f32>>,
    volume: Arc<AtomicU32>,
    muted: Arc<AtomicBool>,
    clear: Arc<AtomicBool>,
}

impl fmt::Debug for CpalAudioQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CpalAudioQueue {{ volume: {:?}, muted: {:?} }}",
            self.volume, self.muted
        )
    }
}

impl CpalAudioQueue {
    fn new(producer: Box<dyn SampleProducer<f32>>, volume: Arc<AtomicU32>, muted: Arc<AtomicBool>, clear: Arc<AtomicBool>) -> Self {
        Self {
            producer,
            volume,
            muted,
            clear,
        }
    }
}

impl SampleProducer<f32> for CpalAudioQueue {
    fn set_volume(&mut self, volume: f32) {
        self.volume.store(f32::to_bits(volume), std::sync::atomic::Ordering::Relaxed);
    }

    fn set_muted(&mut self, muted: bool) {
        self.muted.store(muted, std::sync::atomic::Ordering::Relaxed);
    }

    fn produce(&mut self, sample: f32) {
        let _ = self.producer.produce(sample);
    }
}

impl AudioOutput for CpalAudioQueue {
    fn set_volume(&mut self, volume: f32) {
        self.volume
            .store(f32::to_bits(volume), std::sync::atomic::Ordering::Relaxed);
    }

    fn set_muted(&mut self, muted: bool) {
        self.muted.store(muted, std::sync::atomic::Ordering::Relaxed);
    }

    fn queue_sample(&mut self, sample: f32) {
        let _ = self.producer.produce(sample);
    }

    fn clear(&mut self) {
        self.clear.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    fn is_ready(&self) -> bool {
        true
    }
}

pub struct CpalAudioPlayer {
    device: cpal::Device,
    volume: Arc<AtomicU32>,
    muted: Arc<AtomicBool>,
    clear: Arc<AtomicBool>,
    stream: Option<Stream>,
    sample_rate: f32,
}

impl CpalAudioPlayer {
    fn new(
        device: cpal::Device,
        volume: Arc<AtomicU32>,
        muted: Arc<AtomicBool>,
        clear: Arc<AtomicBool>,
    ) -> Result<Self> {
        Ok(Self {
            device,
            volume,
            muted,
            clear,
            stream: None,
            sample_rate: 0.0,
        })
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    fn initialize<C: SampleConsumer<f32>>(&mut self, consumer: C, config: cpal::SupportedStreamConfig) -> Result<()> {
        self.sample_rate = config.sample_rate().0 as f32;

        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => self.make_stream::<i8, C>(consumer, &config.into()),
            cpal::SampleFormat::I16 => self.make_stream::<i16, C>(consumer, &config.into()),
            // cpal::SampleFormat::I24 => make_stream::<I24>(&device, &config.into()),
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
            println!("Playing stream");
            stream.play()?;
        }
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        if let Some(stream) = &self.stream {
            println!("Pausing stream");
            stream.pause()?;
        }
        Ok(())
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume.store(f32::to_bits(volume), std::sync::atomic::Ordering::Relaxed);
    }

    fn make_stream<S, C>(&mut self, mut consumer: C, config: &cpal::StreamConfig) -> Result<cpal::Stream>
    where
        S: SizedSample + FromSample<f32>,
        C: SampleConsumer<f32>,
    {
        let num_channels = config.channels as usize;
        let err_fn = |err| eprintln!("Error building output sound stream: {}", err);
        let volume = self.volume.clone();
        let muted = self.muted.clone();
        let clear = self.clear.clone();

        let stream = self.device.build_output_stream(
            &config,
            move |output: &mut [S], _: &cpal::OutputCallbackInfo| {
                Self::process_frame(
                    output,
                    &mut consumer,
                    num_channels,
                    volume.clone(),
                    muted.clone(),
                    clear.clone(),
                )
            },
            err_fn,
            None,
        )?;

        Ok(stream)
    }

    fn process_frame<S, C>(
        output: &mut [S],
        consumer: &mut C,
        num_channels: usize,
        volume: Arc<AtomicU32>,
        muted: Arc<AtomicBool>,
        clear: Arc<AtomicBool>,
    ) where
        S: Sample + FromSample<f32>,
        C: SampleConsumer<f32>,
    {
        if clear.load(std::sync::atomic::Ordering::Relaxed) {
            // consumer.clear();
            clear.store(false, std::sync::atomic::Ordering::Relaxed);
            return;
        }

        for frame in output.chunks_mut(num_channels) {
            // Get sample from buffer or use silence (0.0) if buffer is empty
            let sample = consumer.consume().unwrap_or(0.0);
            let muted = muted.load(std::sync::atomic::Ordering::Relaxed);
            let volume = f32::from_bits(volume.load(std::sync::atomic::Ordering::Relaxed));

            // Copy the same value to all channels
            for s in frame.iter_mut() {
                *s = S::from_sample(if muted { 0.0 } else { sample * volume });
            }
        }
    }
}
