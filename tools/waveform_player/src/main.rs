use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use eframe::{egui, CreationContext, Frame, NativeOptions};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use rn_audio::{ChannelBuilder, CpalAudioBuilder, CpalAudioConsumer, Multiplexer, Oscillator, Waveform};
use rn_ui::widgets::WaveformWidget;
use anyhow::Result;

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
    Stop,
    Play,
    Pause,
}

/// Main waveform player application
struct WaveformPlayer {
    // Components
    waveform: WaveformWidget,

    // Audio state
    audio_player: CpalAudioConsumer,
    audio_thread: Option<thread::JoinHandle<()>>,
    audio_command_sender: mpsc::Sender<AudioCommand>,

    // Dock state
    dock_state: DockState<DockTab>,

    // Shared context
    context: AppContext,
}

/// Tab viewer for the dock area
struct WaveformTabViewer<'a> {
    waveform: &'a mut WaveformWidget,
    audio_command_sender: &'a mpsc::Sender<AudioCommand>,
    audio_player: &'a mut CpalAudioConsumer,
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
                        let _ = self.audio_player.play().ok();

                        let _ = self.audio_command_sender.send(AudioCommand::Play);
                    } else if was_enabled && !self.context.audio_enabled {
                        // Stop audio playback
                        let _ = self.audio_player.pause().ok();

                        let _ = self.audio_command_sender.send(AudioCommand::Pause);
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
                        let _ = self.audio_command_sender.send(AudioCommand::SetWaveform(new_waveform));
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
                    let _ = self.audio_command_sender.send(AudioCommand::SetFrequency(self.context.frequency));
                }

                // Duty cycle control (only for square wave)
                if self.context.selected_waveform == WaveformType::Square {
                    if ui
                        .add(egui::Slider::new(&mut self.context.duty_cycle, 0.0..=1.0).text("Duty Cycle"))
                        .changed()
                    {
                        // Update oscillator duty cycle via command
                        let _ = self.audio_command_sender.send(AudioCommand::SetWaveform(Waveform::Square(self.context.duty_cycle)));
                    }
                }

                // Volume control
                if ui
                    .add(egui::Slider::new(&mut self.context.volume, 0.0..=1.0).text("Volume"))
                    .changed()
                {
                    // Update audio output volume via command
                    self.audio_player.set_volume(self.context.volume);
                }
            },
            DockTab::Waveform => {
                // Display the waveform visualizer
                self.waveform.ui(ui);
            },
        }
    }
}

impl WaveformPlayer {
    fn new(_cc: &CreationContext<'_>) -> Result<Self> {
        // Create dock state with the tabs
        let mut dock_state = DockState::new(vec![DockTab::Controls]);

        // Add tabs to the root node
        dock_state.main_surface_mut().split_below(
            NodeIndex::root(),
            0.5,
            vec![DockTab::Waveform],
        );
        
        // Set up initial context
        let context = AppContext {
            audio_enabled: false,
            selected_waveform: WaveformType::Sine,
            frequency: 440.0, // A4 note
            duty_cycle: 0.5,  // 50% duty cycle for square waves
            volume: 0.8,      // 80% volume
        };

        // Create a command channel
        let (audio_command_sender, audio_command_receiver) = mpsc::channel();

        let (audio_producer, audio_consumer) = CpalAudioBuilder::build_default()?;
        let (multiplexer_producer, multiplexer_consumer) = ChannelBuilder::build(1024);
        let (waveform_producer, waveform_consumer) = ChannelBuilder::build(1024);
        
        let sample_rate = audio_consumer.sample_rate();

        let mut multiplexer = Multiplexer::new(multiplexer_consumer);
        multiplexer.add_producer(Box::new(audio_producer));
        multiplexer.add_producer(Box::new(waveform_producer));

        let mut oscillator = Oscillator::new(Box::new(multiplexer_producer), sample_rate, Waveform::Sine, 440.0);

        let waveform = WaveformWidget::new(Box::new(waveform_consumer));

        // Start with the audio queue directly owned by the audio thread
        let audio_thread = thread::spawn(move || {
            // Create oscillator owned by this thread

            // Calculate time per sample in microseconds for sample rate
            let sample_time_us = (1.0 / sample_rate * 1_000_000.0) as u64;
            let mut next_sample_time = Instant::now();

            // Flag to track if we're actively generating audio
            let mut active = false;

            // High-resolution timer for audio generation
            let mut running = true;
            while running {
                // Process any pending commands
                while let Ok(command) = audio_command_receiver.try_recv() {
                    match command {
                        AudioCommand::SetWaveform(waveform) => {
                            oscillator.set_waveform(waveform);
                        },
                        AudioCommand::SetFrequency(freq) => {
                            oscillator.set_frequency(freq);
                        },
                        AudioCommand::Play => {
                            active = true;
                        },
                        AudioCommand::Pause => {
                            active = false;
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
                    oscillator.tick();
                    multiplexer.tick();
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
        });

        Ok(Self {
            waveform,
            audio_player: audio_consumer, 
            audio_thread: Some(audio_thread),
            audio_command_sender,
            dock_state,
            context,
        })
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
                    waveform: &mut self.waveform,
                    audio_command_sender: &self.audio_command_sender,
                    audio_player: &mut self.audio_player,
                    context: &mut self.context,
                },
            );

        // Request continuous updates for the UI and waveform visualization
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Stop audio thread
        let _ = self.audio_command_sender.send(AudioCommand::Stop);

        // If audio is playing, pause it
        if self.context.audio_enabled {
            let _ = self.audio_player.pause();
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
        Box::new(|cc| Ok(Box::new(WaveformPlayer::new(cc)?))),
    )
}
