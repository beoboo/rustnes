#![allow(dead_code)]
use egui::{Slider, SliderClamping, Ui};
use rn_core::{apu::ApuWrapper, system::nes_system::NesSystem};

/// Widget for controlling audio settings
#[derive(Default)]
pub struct AudioWidget {
    volume: f32,
    muted: bool,
}

impl AudioWidget {
    /// Create a new audio control widget
    pub fn new() -> Self {
        Self {
            volume: 1.0, // Default to full volume
            muted: false,
        }
    }

    /// Render the audio widget using the given UI
    pub fn ui(&mut self, ui: &mut Ui, apu: ApuWrapper) {
        ui.heading("Audio Controls");

        // Volume slider
        ui.add(
            Slider::new(&mut self.volume, 0.0..=1.0)
                .text("Volume")
                .clamping(SliderClamping::Always)
                .show_value(true),
        );

        // Mute toggle
        if ui.checkbox(&mut self.muted, "Mute Audio").clicked() {
            // Apply mute state immediately

            // Future implementation: self.apu.set_audio_muted(self.muted);
        }

        // Apply volume (and mute if needed)
        let effective_volume = if self.muted { 0.0 } else { self.volume };
        
        // Future implementation: self.apu.set_audio_volume(effective_volume);
        // For now this is just a UI demo without actual functionality
        
        // Show audio status
        ui.horizontal(|ui| {
            ui.label("Status:");
            if self.muted {
                ui.label("Muted");
            } else {
                ui.label(format!("Volume: {:.0}%", self.volume * 100.0));
            }
        });
        
        // UI section for individual channel controls (placeholder for future expansion)
        ui.collapsing("Channel Controls", |ui| {
            ui.label("Pulse Channel 1");
            ui.checkbox(&mut true, "Enabled").on_hover_text("Enable/disable pulse channel 1");
            
            ui.label("Pulse Channel 2");
            ui.checkbox(&mut false, "Enabled").on_hover_text("Not implemented yet");
            
            ui.label("Triangle Channel");
            ui.checkbox(&mut false, "Enabled").on_hover_text("Not implemented yet");
            
            ui.label("Noise Channel");
            ui.checkbox(&mut false, "Enabled").on_hover_text("Not implemented yet");
        });
        
        // Information about APU
        ui.collapsing("APU Information", |ui| {
            ui.label("The NES Audio Processing Unit (APU) includes:");
            ui.label("• 2 Pulse channels for melody and effects");
            ui.label("• 1 Triangle channel for bass notes");
            ui.label("• 1 Noise channel for percussion");
            ui.label("• 1 DMC channel for digital samples");
            
            ui.label("\nCurrently implemented:");
            ui.label("• Basic Pulse Channel 1 with constant volume");
        });
    }
} 