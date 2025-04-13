#![allow(dead_code)]
use egui::{Slider, SliderClamping, Ui};
use rn_core::{apu::ApuWrapper, memory::Addressable, system::nes_system::NesSystem};

use super::{AudioCaptureOutput, WaveformWidget};

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
        }
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
            let mut changed = false;
            // Pulse Channel 1
            if ui.checkbox(&mut self.pulse1_enabled, "Pulse Channel 1").changed() {
                changed = true;
            }

            // Pulse Channel 2
            if ui.checkbox(&mut self.pulse2_enabled, "Pulse Channel 2").changed() {
                changed = true;
            }

            // Triangle Channel
            if ui.checkbox(&mut self.triangle_enabled, "Triangle Channel").changed() {
                changed = true;
            }

            // Noise Channel
            if ui.checkbox(&mut self.noise_enabled, "Noise Channel").changed() {
            }

            // DMC Channel
            if ui.checkbox(&mut self.dmc_enabled, "DMC Channel").changed() {
                changed = true;
            }

            if changed {
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
    }
}
