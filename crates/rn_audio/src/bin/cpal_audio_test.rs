use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait}, Device, SizedSample, Stream, StreamConfig, SupportedStreamConfig
};
use cpal::{FromSample, Sample};
use ringbuf::{
    storage::Heap,
    traits::{Consumer, Producer, Split},
    CachingCons, CachingProd, HeapRb, SharedRb,
};
use rn_core::audio::AudioOutput;
use std::{fmt, sync::Arc};
use anyhow::Result;
use std::sync::atomic::{AtomicBool, AtomicU32};
type AudioProducer = CachingProd<Arc<SharedRb<Heap<f32>>>>;
type AudioConsumer = CachingCons<Arc<SharedRb<Heap<f32>>>>;

struct CpalAudioBuilder;

impl CpalAudioBuilder {
    fn build(device: cpal::Device, config: cpal::SupportedStreamConfig) -> Result<(CpalAudioQueue, CpalAudioOutput)> {
        let latency_ms = 250.0; // Reduced from 1000ms to 250ms for better responsiveness
        let sample_rate = config.sample_rate().0 as f32;
        let num_channels = config.channels() as usize;

        let latency_frames = (latency_ms / 1_000.0) * sample_rate;
        let latency_samples = latency_frames as usize * num_channels;

        // Create a larger buffer to prevent underruns
        let ring = HeapRb::<f32>::new(latency_samples * 4); // 4x buffer size
        let (producer, consumer) = ring.split();

        let volume = Arc::new(AtomicU32::new(f32::to_bits(1.0)));
        let muted = Arc::new(AtomicBool::new(false));
        let clear = Arc::new(AtomicBool::new(false));
        let audio_queue = CpalAudioQueue::new(producer, volume.clone(), muted.clone(), clear.clone());
        let mut audio_output = CpalAudioOutput::new(device, volume, muted, clear)?;
        audio_output.initialize(consumer, config)?;

        Ok((audio_queue, audio_output))
    }
}

struct CpalAudioQueue {
    producer: AudioProducer,
    volume: Arc<AtomicU32>,
    muted: Arc<AtomicBool>,
    clear: Arc<AtomicBool>,
}

impl fmt::Debug for CpalAudioQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CpalAudioQueue {{ volume: {:?}, muted: {:?} }}", self.volume, self.muted)
    }
}

impl CpalAudioQueue {
    fn new(producer: AudioProducer, volume: Arc<AtomicU32>, muted: Arc<AtomicBool>, clear: Arc<AtomicBool>) -> Self {
        Self { producer, volume, muted, clear }
    }
}

struct CpalAudioOutput {
    device: cpal::Device,
    volume: Arc<AtomicU32>,
    muted: Arc<AtomicBool>,
    clear: Arc<AtomicBool>,
    stream: Option<Stream>,
}

impl AudioOutput for CpalAudioQueue {
    fn set_volume(&mut self, volume: f32) {
        self.volume.store(f32::to_bits(volume), std::sync::atomic::Ordering::Relaxed);
    }

    fn set_muted(&mut self, muted: bool) {
        self.muted.store(muted, std::sync::atomic::Ordering::Relaxed);
    }

    fn queue_sample(&mut self, sample: f32) {
        let _ = self.producer.try_push(sample);
    }

    fn clear(&mut self) {
        self.clear.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    fn is_ready(&self) -> bool {
        true
    }
}

impl CpalAudioOutput {
    fn new(device: cpal::Device, volume: Arc<AtomicU32>, muted: Arc<AtomicBool>, clear: Arc<AtomicBool>) -> Result<Self> {
        Ok(Self { device, volume, muted, clear, stream: None })
    }

    fn initialize(&mut self, consumer: AudioConsumer, config: cpal::SupportedStreamConfig) -> Result<()> {
        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => self.make_stream::<i8>(consumer, &config.into()),
            cpal::SampleFormat::I16 => self.make_stream::<i16>(consumer, &config.into()),
            // cpal::SampleFormat::I24 => make_stream::<I24>(&device, &config.into()),
            cpal::SampleFormat::I32 => self.make_stream::<i32>(consumer, &config.into()),
            cpal::SampleFormat::I64 => self.make_stream::<i64>(consumer, &config.into()),
            cpal::SampleFormat::U8 => self.make_stream::<u8>(consumer, &config.into()),
            cpal::SampleFormat::U16 => self.make_stream::<u16>(consumer, &config.into()),
            cpal::SampleFormat::U32 => self.make_stream::<u32>(consumer, &config.into()),
            cpal::SampleFormat::U64 => self.make_stream::<u64>(consumer, &config.into()),
            cpal::SampleFormat::F32 => self.make_stream::<f32>(consumer, &config.into()),
            cpal::SampleFormat::F64 => self.make_stream::<f64>(consumer, &config.into()),
            sample_format => Err(anyhow::Error::msg(format!(
                "Unsupported sample format '{sample_format}'"
            ))),
        }?;

        self.stream = Some(stream);

        Ok(())
    }

    fn play(&mut self) -> Result<()> {
        if let Some(stream) = &self.stream {
            stream.play()?;
        }
        Ok(())
    }

fn make_stream<S>(
    &mut self,
    mut consumer: AudioConsumer,
    config: &cpal::StreamConfig,
) -> Result<cpal::Stream>
where
    S: SizedSample + FromSample<f32>
{
    let num_channels = config.channels as usize;
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);
    let volume = self.volume.clone();
    let muted = self.muted.clone();
    let clear = self.clear.clone();
    
    let stream = self.device.build_output_stream(
        &config,
        move |output: &mut [S], _: &cpal::OutputCallbackInfo| Self::process_frame(output, &mut consumer, num_channels, volume.clone(), muted.clone(), clear.clone()),
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn process_frame<S, C>(output: &mut [S], consumer: &mut C, num_channels: usize, volume: Arc<AtomicU32>, muted: Arc<AtomicBool>, clear: Arc<AtomicBool>)
where
    S: Sample + FromSample<f32>,
    C: Consumer<Item = f32> + Send + 'static,
{
    if clear.load(std::sync::atomic::Ordering::Relaxed) {
        consumer.clear();
        clear.store(false, std::sync::atomic::Ordering::Relaxed);
        return;
    }

    for frame in output.chunks_mut(num_channels) {
        // Get sample from buffer or use silence (0.0) if buffer is empty
        let sample_value = consumer.try_pop().unwrap_or(0.0);

        let muted = muted.load(std::sync::atomic::Ordering::Relaxed);
        let volume = f32::from_bits(volume.load(std::sync::atomic::Ordering::Relaxed));
        
        // Copy the same value to all channels
        for sample in frame.iter_mut() {
            *sample = S::from_sample(if muted { 0.0 } else { sample_value * volume });
        }
    }
}
}

fn main() -> Result<()> {
    let (_host, device, config) = host_device_setup()?;
    let sample_rate = config.sample_rate().0 as f32;

    let (mut audio_queue, mut audio_output) = CpalAudioBuilder::build(device, config)?;

    let mut oscillator = Oscillator {
        waveform: Waveform::Sine,
        sample_rate,
        current_sample_index: 0.0,
        frequency_hz: 440.0,
    };

    let time_at_start = std::time::Instant::now();
    println!("Time at start: {:?}", time_at_start);

    // Pre-fill buffer with initial samples before starting playback
    let pre_fill_samples = (sample_rate * 0.1) as usize; // 100ms worth of samples
    println!("Pre-filling buffer with {} samples", pre_fill_samples);
    for _ in 0..pre_fill_samples {
        let sample = oscillator.tick();
        audio_queue.queue_sample(sample);
    }
    println!("Buffer pre-filled");

    // Start audio playback immediately since we've pre-filled the buffer
    audio_output.play()?;
    println!("Playback started");

    // Clone the audio queue for the thread
    std::thread::spawn(move || {
        // Calculate time per sample in microseconds
        let sample_time_us = (1.0 / sample_rate * 1_000_000.0) as u64;
        let mut next_sample_time = std::time::Instant::now();

        loop {
            let time_since_start = std::time::Instant::now().duration_since(time_at_start).as_secs_f32();

            // Set waveform based on elapsed time
            if time_since_start < 1.0 {
                oscillator.set_waveform(Waveform::Sine);
            } else if time_since_start < 2.0 {
                oscillator.set_waveform(Waveform::Triangle);
            } else if time_since_start < 3.0 {
                oscillator.set_waveform(Waveform::Square);
            } else if time_since_start < 4.0 {
                oscillator.set_waveform(Waveform::Saw);
            } else {
                oscillator.set_waveform(Waveform::Sine);
            }

            // Generate the next sample
            let data = oscillator.tick();

            // Push the sample to the buffer
            audio_queue.queue_sample(data);

            // Calculate time until next sample
            next_sample_time += std::time::Duration::from_micros(sample_time_us);

            // Sleep until it's time for the next sample
            if next_sample_time > std::time::Instant::now() {
                std::thread::sleep(next_sample_time.duration_since(std::time::Instant::now()));
            } else {
                // We're falling behind, reset the next sample time
                next_sample_time = std::time::Instant::now();
            }
        }
    });

    // No need to sleep before playing since we're now pre-filling the buffer
    // and starting playback immediately

    std::thread::sleep(std::time::Duration::from_millis(4000));
    Ok(())
}

#[derive(Debug)]
pub enum Waveform {
    Sine,
    Square,
    Saw,
    Triangle,
}

pub struct Oscillator {
    pub sample_rate: f32,
    pub waveform: Waveform,
    pub current_sample_index: f32,
    pub frequency_hz: f32,
}

impl Oscillator {
    fn advance_sample(&mut self) {
        self.current_sample_index = (self.current_sample_index + 1.0) % self.sample_rate;
    }

    fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    fn calculate_sine_output_from_freq(&self, freq: f32) -> f32 {
        let two_pi = 2.0 * std::f32::consts::PI;
        (self.current_sample_index * freq * two_pi / self.sample_rate).sin()
    }

    fn is_multiple_of_freq_above_nyquist(&self, multiple: f32) -> bool {
        self.frequency_hz * multiple > self.sample_rate / 2.0
    }

    fn sine_wave(&mut self) -> f32 {
        self.advance_sample();
        self.calculate_sine_output_from_freq(self.frequency_hz)
    }

    fn generative_waveform(&mut self, harmonic_index_increment: i32, gain_exponent: f32) -> f32 {
        self.advance_sample();
        let mut output = 0.0;
        let mut i = 1;
        while !self.is_multiple_of_freq_above_nyquist(i as f32) {
            let gain = 1.0 / (i as f32).powf(gain_exponent);
            output += gain * self.calculate_sine_output_from_freq(self.frequency_hz * i as f32);
            i += harmonic_index_increment;
        }
        output
    }

    fn square_wave(&mut self) -> f32 {
        self.generative_waveform(2, 1.0)
    }

    fn saw_wave(&mut self) -> f32 {
        self.generative_waveform(1, 1.0)
    }

    fn triangle_wave(&mut self) -> f32 {
        self.generative_waveform(2, 2.0)
    }

    fn tick(&mut self) -> f32 {
        match self.waveform {
            Waveform::Sine => self.sine_wave(),
            Waveform::Square => self.square_wave(),
            Waveform::Saw => self.saw_wave(),
            Waveform::Triangle => self.triangle_wave(),
        }
    }
}

fn host_device_setup() -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig)> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device : {}", device.name()?);

    let config = device.default_output_config()?;

    println!("Default output config : {:?}", config);

    Ok((host, device, config))
}
