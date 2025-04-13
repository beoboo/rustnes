use cpal::traits::{DeviceTrait, HostTrait};
use rn_audio::{CpalAudioBuilder, Oscillator, Waveform};
use anyhow::Result;


fn main() -> Result<()> {
    let (_host, device, config) = host_device_setup()?;
    let sample_rate = config.sample_rate().0 as f32;

    let (audio_queue, mut audio_output) = CpalAudioBuilder::build(device, config)?;

    let mut oscillator = Oscillator::new(Box::new(audio_queue), sample_rate, Waveform::Sine, 440.0);

    let time_at_start = std::time::Instant::now();
    println!("Time at start: {:?}", time_at_start);

    // Pre-fill buffer with initial samples before starting playback
    let pre_fill_samples = (sample_rate * 0.1) as usize; // 100ms worth of samples
    println!("Pre-filling buffer with {} samples", pre_fill_samples);
    for _ in 0..pre_fill_samples {
        oscillator.tick();
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
                oscillator.set_waveform(Waveform::Square(0.5));
            } else if time_since_start < 4.0 {
                oscillator.set_waveform(Waveform::Saw);
            } else {
                oscillator.set_waveform(Waveform::Sine);
            }

            // Generate the next sample
            oscillator.tick();

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
