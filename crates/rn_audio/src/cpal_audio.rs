use std::{
    collections::VecDeque,
    fmt,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat,
    Stream,
};
use log::{debug, error, info, warn};
use rn_core::audio::AudioOutput;

/// Audio output implementation that uses cpal to play audio on the system's audio device
pub struct CpalAudioOutput {
    sample_buffer: Arc<Mutex<VecDeque<f32>>>,
    sample_rate: f32,
    volume: f32,
    muted: bool,
    _stream: Option<Stream>,
    is_initialized: bool,
}

impl fmt::Debug for CpalAudioOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CpalAudioOutput")
            .field("sample_rate", &self.sample_rate)
            .field("volume", &self.volume)
            .field("muted", &self.muted)
            .field("is_initialized", &self.is_initialized)
            .field(
                "buffer_size",
                &self.sample_buffer.lock().map(|buf| buf.len()).unwrap_or(0),
            )
            .finish()
    }
}

impl CpalAudioOutput {
    /// Create a new CpalAudioOutput instance
    pub fn new() -> Self {
        Self {
            sample_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(8192))),
            sample_rate: 44100.0, // Default sample rate
            volume: 1.0,
            muted: false,
            _stream: None,
            is_initialized: false,
        }
    }

    /// Initialize the audio device and start playback
    pub fn initialize(&mut self) -> Result<()> {
        if self.is_initialized {
            // Already initialized
            return Ok(());
        }

        // Get the default host
        let host = cpal::default_host();

        // Get the default output device
        let device = match host.default_output_device() {
            Some(device) => device,
            None => {
                error!("No output device available");
                return Err(anyhow::anyhow!("No output device available"));
            },
        };

        info!("Using audio device: {}", device.name()?);

        // Get the default output config
        let default_config = device.default_output_config()?;
        info!("Default audio config: {:?}", default_config);

        // Create output config with our desired sample rate
        let config = cpal::StreamConfig {
            channels: 1, // Mono output
            sample_rate: cpal::SampleRate(self.sample_rate as u32),
            buffer_size: cpal::BufferSize::Default,
        };

        // Create buffer clone for stream closure
        let buffer = self.sample_buffer.clone();
        let volume = self.volume;
        let muted = self.muted;
        let err_fn = |err| error!("An error occurred on the audio stream: {}", err);

        // Build the stream with the appropriate sample format
        let stream = match default_config.sample_format() {
            SampleFormat::F32 => {
                let stream = device.build_output_stream(
                    &config,
                    move |output_buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        // Fill the output buffer with samples from our buffer
                        let mut guard = match buffer.lock() {
                            Ok(guard) => guard,
                            Err(_) => return, // Skip this callback if we can't lock the buffer
                        };

                        for sample in output_buffer.iter_mut() {
                            // Get a sample from our buffer or use silence
                            let raw_sample = guard.pop_front().unwrap_or(0.0);
                            *sample = if muted { 0.0 } else { raw_sample * volume };
                        }
                    },
                    err_fn.clone(),
                    None,
                )?;
                stream
            },
            SampleFormat::I16 => {
                let stream = device.build_output_stream(
                    &config,
                    move |output_buffer: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        // Fill the output buffer with samples from our buffer
                        let mut guard = match buffer.lock() {
                            Ok(guard) => guard,
                            Err(_) => return, // Skip this callback if we can't lock the buffer
                        };

                        for sample in output_buffer.iter_mut() {
                            // Get a sample from our buffer, scale to i16 range
                            let raw_sample = guard.pop_front().unwrap_or(0.0);
                            let value = if muted { 0.0 } else { raw_sample * volume };
                            *sample = (value * 32767.0) as i16;
                        }
                    },
                    err_fn.clone(),
                    None,
                )?;
                stream
            },
            SampleFormat::U16 => {
                let stream = device.build_output_stream(
                    &config,
                    move |output_buffer: &mut [u16], _: &cpal::OutputCallbackInfo| {
                        // Fill the output buffer with samples from our buffer
                        let mut guard = match buffer.lock() {
                            Ok(guard) => guard,
                            Err(_) => return, // Skip this callback if we can't lock the buffer
                        };

                        for sample in output_buffer.iter_mut() {
                            // Get a sample, scale from [-1.0, 1.0] to [0, 65535]
                            let raw_sample = guard.pop_front().unwrap_or(0.0);
                            let value = if muted { 0.0 } else { raw_sample * volume };
                            // Convert from [-1.0, 1.0] to [0, 65535]
                            *sample = ((value * 0.5 + 0.5) * 65535.0) as u16;
                        }
                    },
                    err_fn,
                    None,
                )?;
                stream
            },
            format => {
                return Err(anyhow::anyhow!("Unsupported sample format: {:?}", format));
            },
        };

        // Start the stream
        stream.play()?;

        // Store the stream to keep it alive
        self._stream = Some(stream);
        self.is_initialized = true;

        info!("Audio output initialized successfully");
        Ok(())
    }
}

impl Default for CpalAudioOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioOutput for CpalAudioOutput {
    fn set_volume(&mut self, volume: f32) {
        self.volume = volume.max(0.0).min(1.0);
    }

    fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    fn set_sample_rate(&mut self, rate: f32) {
        // Only update if not initialized yet, since changing sample rate
        // would require recreating the stream
        if !self.is_initialized {
            self.sample_rate = rate;
        } else {
            warn!("Cannot change sample rate after initialization");
        }
    }

    fn queue_sample(&mut self, sample: f32) {
        // Initialize on first sample if not already done
        if !self.is_initialized {
            match self.initialize() {
                Ok(_) => debug!("Audio initialized on first sample"),
                Err(e) => error!("Failed to initialize audio: {}", e),
            }
        }

        // Add sample to buffer
        if let Ok(mut buffer) = self.sample_buffer.lock() {
            buffer.push_back(sample);

            // Keep buffer at a reasonable size to avoid excessive latency
            if buffer.len() > 8192 {
                buffer.pop_front();
            }
        }
    }

    fn clear(&mut self) {
        if let Ok(mut buffer) = self.sample_buffer.lock() {
            buffer.clear();
        }
    }

    fn is_ready(&self) -> bool {
        true // Always ready to receive samples, even if not initialized yet
    }
}
