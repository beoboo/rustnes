use std::{
    sync::{mpsc, Arc},
    thread,
    time::{Duration, Instant},
};

use eframe::{egui, CreationContext, Frame, NativeOptions};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use ringbuf::{traits::{Consumer, Producer, Split}, CachingCons, HeapRb};
use rn_audio::{CpalAudioBuilder, CpalAudioOutput, Oscillator, Waveform};
use rn_core::audio::AudioOutput;
use rn_ui::widgets::WaveformVisualizerWidget;

// Size for the visualization ring buffer
const VIS_BUFFER_SIZE: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DockTab {
    Controls,
    Waveform,
}

impl DockTab {
    fn title(&self) -> &'static str {
        match self {
            DockTab::Controls => "Audio Controls",
            DockTab::Waveform => "Waveform Visualizer",
        }
    }
}

/// Shared application state
#[derive(Default)]
struct AppContext {
    audio_enabled: bool,
    selected_waveform: WaveformType,
    frequency: f32,
    duty_cycle: f32,
    volume: f32,
}

/// Application-specific waveform type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WaveformType {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

impl Default for WaveformType {
    fn default() -> Self {
        WaveformType::Sine
    }
}

// Command channel to communicate with the audio thread
enum AudioCommand {
    SetWaveform(Waveform),
    SetFrequency(f32),
    SetVolume(f32),
    Stop,
    Play,
    Pause,
}

/// Main waveform player application
struct WaveformPlayer {
    // Components
    waveform_visualizer: WaveformVisualizerWidget,

    // Audio state
    sample_consumer: CachingCons<Arc<HeapRb<f32>>>, // Lock-free ringbuffer consumer for visualization
    audio_output: Option<CpalAudioOutput>,
    audio_thread: Option<thread::JoinHandle<()>>,
    audio_command_sender: Option<mpsc::Sender<AudioCommand>>,

    // Dock state
    dock_state: DockState<DockTab>,

    // Shared context
    context: AppContext,
}

/// Tab viewer for the dock area
struct WaveformTabViewer<'a> {
    waveform_visualizer: &'a mut WaveformVisualizerWidget,
    sample_consumer: &'a mut CachingCons<Arc<HeapRb<f32>>>,
    audio_command_sender: &'a Option<mpsc::Sender<AudioCommand>>,
    audio_output: &'a mut Option<CpalAudioOutput>,
    context: &'a mut AppContext,
}

impl<'a> TabViewer for WaveformTabViewer<'a> {
    type Tab = DockTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            DockTab::Controls => {
                ui.heading("Waveform Controls");

                // Play/Stop controls
                let was_enabled = self.context.audio_enabled;
                if ui.button(if was_enabled { "Stop" } else { "Play" }).clicked() {
                    self.context.audio_enabled = !self.context.audio_enabled;

                    if !was_enabled && self.context.audio_enabled {
                        // Start audio playback
                        if let Some(output) = &mut self.audio_output {
                            // Start the audio stream
                            output.play().ok();

                            // Notify the audio thread to start processing
                            if let Some(sender) = &self.audio_command_sender {
                                sender.send(AudioCommand::Play).ok();
                            }
                        }
                    } else if was_enabled && !self.context.audio_enabled {
                        // Stop audio playback
                        if let Some(output) = &mut self.audio_output {
                            // Pause the audio stream
                            output.pause().ok();

                            // Notify the audio thread
                            if let Some(sender) = &self.audio_command_sender {
                                sender.send(AudioCommand::Pause).ok();
                            }
                        }
                    }
                }

                // Waveform selection
                ui.horizontal(|ui| {
                    ui.label("Waveform: ");
                    let mut new_waveform = None;

                    if ui
                        .radio_value(&mut self.context.selected_waveform, WaveformType::Sine, "Sine")
                        .clicked()
                    {
                        new_waveform = Some(Waveform::Sine);
                    }
                    if ui
                        .radio_value(&mut self.context.selected_waveform, WaveformType::Square, "Square")
                        .clicked()
                    {
                        new_waveform = Some(Waveform::Square(self.context.duty_cycle));
                    }
                    if ui
                        .radio_value(&mut self.context.selected_waveform, WaveformType::Triangle, "Triangle")
                        .clicked()
                    {
                        new_waveform = Some(Waveform::Triangle);
                    }
                    if ui
                        .radio_value(&mut self.context.selected_waveform, WaveformType::Sawtooth, "Sawtooth")
                        .clicked()
                    {
                        new_waveform = Some(Waveform::Saw);
                    }

                    // Send command to audio thread if changed
                    if let Some(new_waveform) = new_waveform {
                        if let Some(sender) = &self.audio_command_sender {
                            sender.send(AudioCommand::SetWaveform(new_waveform)).ok();
                        }
                    }
                });

                // Frequency control
                if ui
                    .add(
                        egui::Slider::new(&mut self.context.frequency, 20.0..=2000.0)
                            .text("Frequency (Hz)")
                            .logarithmic(true),
                    )
                    .changed()
                {
                    // Update oscillator frequency via command
                    if let Some(sender) = &self.audio_command_sender {
                        sender.send(AudioCommand::SetFrequency(self.context.frequency)).ok();
                    }
                }

                // Duty cycle control (only for square wave)
                if self.context.selected_waveform == WaveformType::Square {
                    if ui
                        .add(egui::Slider::new(&mut self.context.duty_cycle, 0.0..=1.0).text("Duty Cycle"))
                        .changed()
                    {
                        // Update oscillator duty cycle via command
                        if let Some(sender) = &self.audio_command_sender {
                            sender
                                .send(AudioCommand::SetWaveform(Waveform::Square(self.context.duty_cycle)))
                                .ok();
                        }
                    }
                }

                // Volume control
                if ui
                    .add(egui::Slider::new(&mut self.context.volume, 0.0..=1.0).text("Volume"))
                    .changed()
                {
                    // Update audio output volume via command
                    if let Some(sender) = &self.audio_command_sender {
                        sender.send(AudioCommand::SetVolume(self.context.volume)).ok();
                    }
                }
            },
            DockTab::Waveform => {
                // Process samples from the ring buffer into the waveform visualizer
                // Read a batch of samples from our lock-free consumer
                let mut samples = Vec::with_capacity(64);
                for _ in 0..64 {
                    if let Some(sample) = self.sample_consumer.try_pop() {
                        samples.push(sample);
                    } else {
                        break;
                    }
                }

                // If we have samples, add them to the visualizer
                if !samples.is_empty() {
                    // Add the samples to the visualizer
                    self.waveform_visualizer.add_samples(&samples);
                }

                // Display the waveform visualizer
                self.waveform_visualizer.ui(ui);
            },
        }
    }
}

impl WaveformPlayer {
    fn new(_cc: &CreationContext<'_>) -> Self {
        // Create dock state with the tabs
        let mut dock_state = DockState::new(vec![DockTab::Controls]);

        // Add tabs to the root node
        dock_state.main_surface_mut().split_below(
            NodeIndex::root(),
            0.5,
            vec![DockTab::Waveform],
        );
        // Create ringbuffer for visualization
        let vis_buffer = HeapRb::<f32>::new(VIS_BUFFER_SIZE);
        let (vis_producer, vis_consumer) = vis_buffer.split();

        // Set up initial context
        let context = AppContext {
            audio_enabled: false,
            selected_waveform: WaveformType::Sine,
            frequency: 440.0, // A4 note
            duty_cycle: 0.5,  // 50% duty cycle for square waves
            volume: 0.8,      // 80% volume
        };

        // Create audio output using CpalAudioBuilder
        match CpalAudioBuilder::build_default() {
            Ok((mut audio_queue, audio_output)) => {
                let sample_rate = audio_output.sample_rate();
                // Create a command channel
                let (tx, rx) = mpsc::channel();

                // Create oscillator in the audio thread to avoid mutex locking
                // Start audio processing thread
                let mut vis_prod = vis_producer;

                // Start with the audio queue directly owned by the audio thread
                let audio_thread = thread::spawn(move || {
                    // Create oscillator owned by this thread
                    let mut oscillator = Oscillator::new(sample_rate, Waveform::Sine, 440.0);

                    // Set initial volume and unmute
                    audio_queue.set_volume(0.8);
                    audio_queue.set_muted(false);

                    // Pre-fill the buffer with silence to avoid initial scratches
                    for _ in 0..1024 {
                        audio_queue.queue_sample(0.0);
                    }

                    // Calculate time per sample in microseconds for sample rate
                    let sample_time_us = (1.0 / sample_rate * 1_000_000.0) as u64;
                    let mut next_sample_time = Instant::now();

                    // For rate limiting visualization updates
                    let mut vis_sample_counter = 0;

                    // Flag to track if we're actively generating audio
                    let mut active = false;

                    // High-resolution timer for audio generation
                    let mut running = true;
                    while running {
                        // Process any pending commands
                        while let Ok(command) = rx.try_recv() {
                            match command {
                                AudioCommand::SetWaveform(waveform) => {
                                    oscillator.set_waveform(waveform);
                                },
                                AudioCommand::SetFrequency(freq) => {
                                    oscillator.set_frequency(freq);
                                },
                                AudioCommand::SetVolume(vol) => {
                                    audio_queue.set_volume(vol);
                                },
                                AudioCommand::Play => {
                                    active = true;
                                },
                                AudioCommand::Pause => {
                                    active = false;
                                    // Fill with silence when paused
                                    for _ in 0..512 {
                                        audio_queue.queue_sample(0.0);
                                    }
                                },
                                AudioCommand::Stop => {
                                    running = false;
                                    break;
                                },
                            }
                        }

                        if !running {
                            break;
                        }

                        if active {
                            // Generate a sample from the oscillator
                            let sample = oscillator.tick() * 0.8; // Scale to 80% volume

                            // Only send every 4th sample to visualization to avoid overwhelming the UI
                            vis_sample_counter += 1;
                            if vis_sample_counter >= 4 {
                                // Add to visualization ring buffer (non-blocking)
                                let _ = vis_prod.try_push(sample);
                                vis_sample_counter = 0;
                            }

                            // Send directly to audio output (no locking needed)
                            audio_queue.queue_sample(sample);
                        } else {
                            // If not active, just sleep a bit to avoid busy-waiting
                            thread::sleep(Duration::from_millis(10));
                            continue;
                        }

                        // Precise timing for sample generation
                        next_sample_time += Duration::from_micros(sample_time_us);

                        let now = Instant::now();
                        if next_sample_time > now {
                            // Sleep until next sample is due
                            thread::sleep(next_sample_time.duration_since(now));
                        } else {
                            // We're falling behind, reset timing
                            next_sample_time = now;
                        }
                    }

                    // Clear audio queue before exiting
                    audio_queue.clear();
                });

                Self {
                    waveform_visualizer: WaveformVisualizerWidget::new(),
                    sample_consumer: vis_consumer,
                    audio_output: Some(audio_output), 
                    audio_thread: Some(audio_thread),
                    audio_command_sender: Some(tx),
                    dock_state,
                    context,
                }
            },
            Err(err) => {
                eprintln!("Failed to initialize audio: {}", err);

                // Create a dummy consumer since we don't have real audio
                let (_, vis_consumer) = HeapRb::<f32>::new(VIS_BUFFER_SIZE).split();

                Self {
                    waveform_visualizer: WaveformVisualizerWidget::new(),
                    sample_consumer: vis_consumer,
                    audio_output: None,
                    audio_thread: None,
                    audio_command_sender: None,
                    dock_state,
                    context,
                }
            },
        }
    }
}

impl eframe::App for WaveformPlayer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Create the dock UI
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(
                ctx,
                &mut WaveformTabViewer {
                    waveform_visualizer: &mut self.waveform_visualizer,
                    sample_consumer: &mut self.sample_consumer,
                    audio_command_sender: &self.audio_command_sender,
                    audio_output: &mut self.audio_output,
                    context: &mut self.context,
                },
            );

        // Request continuous updates for the UI and waveform visualization
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Stop audio thread
        if let Some(sender) = &self.audio_command_sender {
            sender.send(AudioCommand::Stop).ok();
        }

        // If audio is playing, pause it
        if self.context.audio_enabled {
            if let Some(output) = &mut self.audio_output {
                output.pause().ok();
            }
        }

        if let Some(thread) = self.audio_thread.take() {
            thread.join().ok();
        }
    }
}

fn main() -> eframe::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Run the application
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Waveform Player",
        options,
        Box::new(|cc| Ok(Box::new(WaveformPlayer::new(cc)))),
    )
}
