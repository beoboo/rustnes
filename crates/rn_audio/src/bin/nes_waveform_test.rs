use anyhow::Result;
use rn_audio::CpalAudioOutput;
use rn_core::audio::AudioOutput;
use std::{
    io::{self, BufRead, Write},
    thread::{self, JoinHandle},
    time::Duration
};

/// Configuration for NES pulse wave
struct PulseConfig {
    enabled: bool,
    duty_cycle: f32,  // 0.125, 0.25, 0.5, 0.75 (NES values)
    volume: f32,      // 0.0 - 1.0
    frequency: f32,   // Hz
}

impl Default for PulseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            duty_cycle: 0.5,    // 50% duty cycle
            volume: 0.3,        // 30% volume to avoid clipping when mixed
            frequency: 440.0,   // A4 note
        }
    }
}

/// Configuration for NES triangle wave
struct TriangleConfig {
    enabled: bool,
    volume: f32,      // 0.0 - 1.0  
    frequency: f32,   // Hz
}

impl Default for TriangleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 0.3,        // 30% volume to avoid clipping when mixed
            frequency: 220.0,   // An octave below the pulse default
        }
    }
}

fn main() -> Result<()> {
    println!("NES Waveform Test");
    println!("=================");
    println!("This program generates NES-like audio waveforms");
    
    // Create audio output
    let mut audio = CpalAudioOutput::new();
    audio.set_volume(1.0);
    
    // Initialize configurations
    let mut pulse_config = PulseConfig::default();
    let mut triangle_config = TriangleConfig::default();
    let mut running = true;
    
    // Buffer for continuous playback
    let mut buffer_thread: Option<JoinHandle<()>> = None;
    let mut stop_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    
    while running {
        // Show menu
        println!("\nOptions:");
        println!("1. Toggle pulse (currently: {})", if pulse_config.enabled { "ON" } else { "OFF" });
        println!("2. Set pulse duty cycle (currently: {}%)", pulse_config.duty_cycle * 100.0);
        println!("3. Set pulse volume (currently: {}%)", pulse_config.volume * 100.0);
        println!("4. Set pulse frequency (currently: {} Hz)", pulse_config.frequency);
        println!("5. Toggle triangle (currently: {})", if triangle_config.enabled { "ON" } else { "OFF" });
        println!("6. Set triangle volume (currently: {}%)", triangle_config.volume * 100.0);
        println!("7. Set triangle frequency (currently: {} Hz)", triangle_config.frequency);
        println!("8. Play waveforms");
        println!("9. Stop playback");
        println!("0. Exit");
        print!("\nEnter choice: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().lock().read_line(&mut input)?;
        
        match input.trim() {
            "1" => {
                pulse_config.enabled = !pulse_config.enabled;
                println!("Pulse channel is now {}", if pulse_config.enabled { "enabled" } else { "disabled" });
            },
            "2" => {
                println!("Enter duty cycle percentage (12.5, 25, 50, or 75): ");
                let mut input = String::new();
                io::stdin().lock().read_line(&mut input)?;
                match input.trim().parse::<f32>() {
                    Ok(value) => {
                        // Convert percentage to decimal
                        let duty = value / 100.0;
                        // Validate duty cycle is one of the NES values
                        let valid_duties = [0.125, 0.25, 0.5, 0.75];
                        let closest = valid_duties.iter()
                            .min_by(|a, b| {
                                let diff_a = (duty - **a).abs();
                                let diff_b = (duty - **b).abs();
                                diff_a.partial_cmp(&diff_b).unwrap()
                            })
                            .unwrap();
                        
                        pulse_config.duty_cycle = *closest;
                        println!("Set duty cycle to {}%", pulse_config.duty_cycle * 100.0);
                    },
                    Err(_) => println!("Invalid input. Please enter a number."),
                }
            },
            "3" => {
                println!("Enter volume percentage (0-100): ");
                let mut input = String::new();
                io::stdin().lock().read_line(&mut input)?;
                match input.trim().parse::<f32>() {
                    Ok(value) if value >= 0.0 && value <= 100.0 => {
                        pulse_config.volume = value / 100.0;
                        println!("Set pulse volume to {}%", pulse_config.volume * 100.0);
                    },
                    _ => println!("Invalid input. Please enter a number between 0 and 100."),
                }
            },
            "4" => {
                println!("Enter frequency in Hz (50-5000): ");
                let mut input = String::new();
                io::stdin().lock().read_line(&mut input)?;
                match input.trim().parse::<f32>() {
                    Ok(value) if value >= 50.0 && value <= 5000.0 => {
                        pulse_config.frequency = value;
                        println!("Set pulse frequency to {} Hz", pulse_config.frequency);
                    },
                    _ => println!("Invalid input. Please enter a number between 50 and 5000."),
                }
            },
            "5" => {
                triangle_config.enabled = !triangle_config.enabled;
                println!("Triangle channel is now {}", if triangle_config.enabled { "enabled" } else { "disabled" });
            },
            "6" => {
                println!("Enter volume percentage (0-100): ");
                let mut input = String::new();
                io::stdin().lock().read_line(&mut input)?;
                match input.trim().parse::<f32>() {
                    Ok(value) if value >= 0.0 && value <= 100.0 => {
                        triangle_config.volume = value / 100.0;
                        println!("Set triangle volume to {}%", triangle_config.volume * 100.0);
                    },
                    _ => println!("Invalid input. Please enter a number between 0 and 100."),
                }
            },
            "7" => {
                println!("Enter frequency in Hz (50-5000): ");
                let mut input = String::new();
                io::stdin().lock().read_line(&mut input)?;
                match input.trim().parse::<f32>() {
                    Ok(value) if value >= 50.0 && value <= 5000.0 => {
                        triangle_config.frequency = value;
                        println!("Set triangle frequency to {} Hz", triangle_config.frequency);
                    },
                    _ => println!("Invalid input. Please enter a number between 50 and 5000."),
                }
            },
            "8" => {
                // Stop previous playback if any
                if let Some(handle) = buffer_thread.take() {
                    stop_signal.store(true, std::sync::atomic::Ordering::SeqCst);
                    handle.join().ok();
                }
                
                // Reset stop signal
                stop_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                let stop = stop_signal.clone();
                
                // Clear audio buffer
                audio.clear();
                
                // Clone configurations for the thread
                let p_config = pulse_config.clone();
                let t_config = triangle_config.clone();
                
                println!("Starting playback. Press 9 to stop.");
                
                // Start a thread for continuous audio generation
                buffer_thread = Some(thread::spawn(move || {
                    // Generate and queue samples continuously
                    let sample_rate = 44100.0;
                    let mut t = 0.0;
                    let time_step = 1.0 / sample_rate;
                    
                    // Initialize audio output
                    let mut audio_out = CpalAudioOutput::new();
                    audio_out.set_volume(1.0);
                    
                    while !stop.load(std::sync::atomic::Ordering::SeqCst) {
                        let mut sample = 0.0;
                        
                        // Generate pulse wave if enabled
                        if p_config.enabled {
                            let cycle_pos = (t * p_config.frequency) % 1.0;
                            let pulse_val = if cycle_pos < p_config.duty_cycle { 1.0 } else { -1.0 };
                            sample += pulse_val * p_config.volume;
                        }
                        
                        // Generate triangle wave if enabled
                        if t_config.enabled {
                            let cycle_pos = (t * t_config.frequency) % 1.0;
                            let tri_val = if cycle_pos < 0.5 {
                                // Rising part of triangle - 15 steps up in NES
                                4.0 * cycle_pos - 1.0
                            } else {
                                // Falling part of triangle - 15 steps down in NES
                                3.0 - 4.0 * cycle_pos
                            };
                            sample += tri_val * t_config.volume;
                        }
                        
                        // Queue the sample
                        audio_out.queue_sample(sample);
                        
                        // Increment time
                        t += time_step;
                        
                        // Small sleep to avoid consuming too much CPU
                        if t % 0.1 < time_step {
                            thread::sleep(Duration::from_millis(1));
                        }
                    }
                    
                    // Clear audio when stopped
                    audio_out.clear();
                }));
            },
            "9" => {
                if let Some(handle) = buffer_thread.take() {
                    println!("Stopping playback...");
                    stop_signal.store(true, std::sync::atomic::Ordering::SeqCst);
                    handle.join().ok();
                    audio.clear();
                    println!("Playback stopped.");
                } else {
                    println!("No playback is active.");
                }
            },
            "0" => {
                println!("Exiting...");
                running = false;
                // Stop playback if active
                if let Some(handle) = buffer_thread.take() {
                    stop_signal.store(true, std::sync::atomic::Ordering::SeqCst);
                    handle.join().ok();
                }
                audio.clear();
            },
            _ => println!("Invalid choice."),
        }
    }
    
    Ok(())
}

// Add Clone trait to configurations for thread usage
impl Clone for PulseConfig {
    fn clone(&self) -> Self {
        Self {
            enabled: self.enabled,
            duty_cycle: self.duty_cycle,
            volume: self.volume,
            frequency: self.frequency,
        }
    }
}

impl Clone for TriangleConfig {
    fn clone(&self) -> Self {
        Self {
            enabled: self.enabled,
            volume: self.volume,
            frequency: self.frequency,
        }
    }
} 