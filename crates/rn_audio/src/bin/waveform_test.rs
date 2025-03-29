use anyhow::Result;
use rn_audio::CpalAudioOutput;
use rn_core::audio::AudioOutput;
use std::{f32::consts::PI, thread, time::Duration};

/// Waveform types to generate
enum Waveform {
    Sine,
    Square,
    Triangle,
    Sawtooth,
    PulseAndTriangle,
}

fn main() -> Result<()> {
    // Create a new audio output
    let mut audio = CpalAudioOutput::new();
    
    // Set audio parameters
    audio.set_volume(0.5); // 50% volume to avoid clipping
    
    // List of waveforms to test
    let waveforms = [
        Waveform::Sine,
        Waveform::Square,
        Waveform::Triangle,
        Waveform::Sawtooth,
        Waveform::PulseAndTriangle,
    ];
    
    for waveform in &waveforms {
        println!("Playing {:?} waveform for 3 seconds...", waveform_name(waveform));
        
        // Generate samples for 3 seconds
        let sample_rate = 44100.0;
        let duration_secs = 3.0;
        let frequency = 440.0; // A4 note
        
        // Generate the waveform
        match waveform {
            Waveform::Sine => {
                for i in 0..(sample_rate * duration_secs) as usize {
                    let t = i as f32 / sample_rate;
                    let sample = (t * frequency * 2.0 * PI).sin();
                    audio.queue_sample(sample);
                }
            },
            Waveform::Square => {
                for i in 0..(sample_rate * duration_secs) as usize {
                    let t = i as f32 / sample_rate;
                    let cycle_position = (t * frequency) % 1.0;
                    let sample = if cycle_position < 0.5 { 0.8 } else { -0.8 };
                    audio.queue_sample(sample);
                }
            },
            Waveform::Triangle => {
                for i in 0..(sample_rate * duration_secs) as usize {
                    let t = i as f32 / sample_rate;
                    let cycle_position = (t * frequency) % 1.0;
                    let sample = if cycle_position < 0.5 {
                        // Rising part of triangle
                        4.0 * cycle_position - 1.0
                    } else {
                        // Falling part of triangle
                        3.0 - 4.0 * cycle_position
                    };
                    audio.queue_sample(sample);
                }
            },
            Waveform::Sawtooth => {
                for i in 0..(sample_rate * duration_secs) as usize {
                    let t = i as f32 / sample_rate;
                    let cycle_position = (t * frequency) % 1.0;
                    let sample = 2.0 * cycle_position - 1.0;
                    audio.queue_sample(sample);
                }
            },
            Waveform::PulseAndTriangle => {
                // Mixed pulse (square) and triangle waves
                let pulse_freq = 440.0;   // Pulse frequency
                let tri_freq = 220.0;     // Triangle frequency (half of pulse)
                
                for i in 0..(sample_rate * duration_secs) as usize {
                    let t = i as f32 / sample_rate;
                    
                    // Generate pulse wave
                    let pulse_pos = (t * pulse_freq) % 1.0;
                    let pulse_sample = if pulse_pos < 0.5 { 0.4 } else { -0.4 };
                    
                    // Generate triangle wave
                    let tri_pos = (t * tri_freq) % 1.0;
                    let tri_sample = if tri_pos < 0.5 {
                        2.0 * tri_pos - 0.5
                    } else {
                        1.5 - 2.0 * tri_pos
                    };
                    
                    // Mix the two waveforms
                    let mixed_sample = pulse_sample + tri_sample;
                    audio.queue_sample(mixed_sample);
                }
            },
        }
        
        // Wait for the sound to play
        thread::sleep(Duration::from_secs_f32(duration_secs + 0.5));
        
        // Clear the buffer before the next waveform
        audio.clear();
        thread::sleep(Duration::from_millis(500));
    }
    
    println!("Test completed.");
    Ok(())
}

fn waveform_name(waveform: &Waveform) -> String {
    match waveform {
        Waveform::Sine => "Sine".to_string(),
        Waveform::Square => "Square".to_string(),
        Waveform::Triangle => "Triangle".to_string(),
        Waveform::Sawtooth => "Sawtooth".to_string(),
        Waveform::PulseAndTriangle => "Pulse and Triangle mix".to_string(),
    }
} 