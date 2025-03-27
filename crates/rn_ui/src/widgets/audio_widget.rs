#![allow(dead_code)]
use egui::{Slider, SliderClamping, Ui};
use rn_core::{apu::ApuWrapper, memory::Addressable, system::nes_system::NesSystem};

use super::{AudioCaptureOutput, WaveformVisualizerWidget};

/// Widget for controlling audio settings
#[derive(Default)]
pub struct AudioWidget {
    volume: f32,
    muted: bool,
    pulse1_enabled: bool,
    pulse2_enabled: bool,
    triangle_enabled: bool,
    noise_enabled: bool,
    dmc_enabled: bool,

    // Waveform visualizer
    waveform_widget: WaveformVisualizerWidget,
    show_waveform: bool,
    visualizer: Option<AudioCaptureOutput>,
}

impl AudioWidget {
    /// Create a new audio control widget
    pub fn new() -> Self {
        Self {
            volume: 1.0, // Default to full volume
            muted: false,
            pulse1_enabled: true,
            pulse2_enabled: true,
            triangle_enabled: true,
            noise_enabled: true,
            dmc_enabled: true,

            // Initialize visualizer
            waveform_widget: WaveformVisualizerWidget::new(),
            show_waveform: true,
            visualizer: None,
        }
    }

    /// Connect an audio visualizer for real-time waveform display
    pub fn connect_visualizer(&mut self, visualizer: AudioCaptureOutput) {
        self.waveform_widget.connect_capture(&visualizer);
        self.visualizer = Some(visualizer);
    }

    /// Render the audio widget using the given UI
    pub fn ui(&mut self, ui: &mut Ui, mut apu: ApuWrapper) {
        ui.heading("Audio Controls");

        // Volume slider
        if ui
            .add(
                Slider::new(&mut self.volume, 0.0..=1.0)
                    .text("Volume")
                    .clamping(SliderClamping::Always)
                    .show_value(true),
            )
            .changed()
        {
            apu.set_volume(self.volume);
        }

        // Mute toggle
        if ui.checkbox(&mut self.muted, "Mute Audio").clicked() {
            // Apply mute state immediately
            apu.set_muted(self.muted);
        }

        // Show audio status
        ui.horizontal(|ui| {
            ui.label("Status:");
            if self.muted {
                ui.label("Muted");
            } else {
                ui.label(format!("Volume: {:.0}%", self.volume * 100.0));
            }
        });

        // UI section for individual channel controls
        ui.collapsing("Channel Controls", |ui| {
            // Pulse Channel 1
            if ui.checkbox(&mut self.pulse1_enabled, "Pulse Channel 1").changed() {
                let mut status = 0;
                if self.pulse1_enabled {
                    status |= 0x01;
                }
                if self.pulse2_enabled {
                    status |= 0x02;
                }
                if self.triangle_enabled {
                    status |= 0x04;
                }
                if self.noise_enabled {
                    status |= 0x08;
                }
                if self.dmc_enabled {
                    status |= 0x10;
                }
                apu.write_byte(0x4015, status).unwrap();
            }

            // Pulse Channel 2
            if ui.checkbox(&mut self.pulse2_enabled, "Pulse Channel 2").changed() {
                let mut status = 0;
                if self.pulse1_enabled {
                    status |= 0x01;
                }
                if self.pulse2_enabled {
                    status |= 0x02;
                }
                if self.triangle_enabled {
                    status |= 0x04;
                }
                if self.noise_enabled {
                    status |= 0x08;
                }
                if self.dmc_enabled {
                    status |= 0x10;
                }
                apu.write_byte(0x4015, status).unwrap();
            }

            // Triangle Channel
            if ui.checkbox(&mut self.triangle_enabled, "Triangle Channel").changed() {
                let mut status = 0;
                if self.pulse1_enabled {
                    status |= 0x01;
                }
                if self.pulse2_enabled {
                    status |= 0x02;
                }
                if self.triangle_enabled {
                    status |= 0x04;
                }
                if self.noise_enabled {
                    status |= 0x08;
                }
                if self.dmc_enabled {
                    status |= 0x10;
                }
                apu.write_byte(0x4015, status).unwrap();
            }

            // Noise Channel
            if ui.checkbox(&mut self.noise_enabled, "Noise Channel").changed() {
                let mut status = 0;
                if self.pulse1_enabled {
                    status |= 0x01;
                }
                if self.pulse2_enabled {
                    status |= 0x02;
                }
                if self.triangle_enabled {
                    status |= 0x04;
                }
                if self.noise_enabled {
                    status |= 0x08;
                }
                if self.dmc_enabled {
                    status |= 0x10;
                }
                apu.write_byte(0x4015, status).unwrap();
            }

            // DMC Channel
            if ui.checkbox(&mut self.dmc_enabled, "DMC Channel").changed() {
                let mut status = 0;
                if self.pulse1_enabled {
                    status |= 0x01;
                }
                if self.pulse2_enabled {
                    status |= 0x02;
                }
                if self.triangle_enabled {
                    status |= 0x04;
                }
                if self.noise_enabled {
                    status |= 0x08;
                }
                if self.dmc_enabled {
                    status |= 0x10;
                }
                apu.write_byte(0x4015, status).unwrap();
            }
        });

        // Toggle for waveform visualizer
        ui.checkbox(&mut self.show_waveform, "Show Audio Waveform");

        // Display waveform visualizer if enabled
        if self.show_waveform {
            ui.group(|ui| {
                ui.set_min_width(500.0);
                self.waveform_widget.ui(ui);
            });
        }

        // Information about APU
        ui.collapsing("APU Information", |ui| {
            ui.label("The NES Audio Processing Unit (APU) includes:");
            ui.label("• 2 Pulse channels for melody and effects");
            ui.label("• 1 Triangle channel for bass notes");
            ui.label("• 1 Noise channel for percussion");
            ui.label("• 1 DMC channel for digital samples");

            ui.label("\nCurrently implemented:");
            ui.label("• All channels with proper enable/disable control");
            ui.label("• Real-time waveform visualization");
        });
    }
}
