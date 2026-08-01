#![allow(dead_code)]
use egui::{Slider, SliderClamping, Ui};
use rn_core::{
    apu::{ApuWrapper, Channel},
    memory::Addressable,
};

/// A snapshot of the audio pipeline's health, for display.
///
/// Passed in rather than read directly so `rn_ui` stays independent of the host audio backend.
/// The fill level is the useful one: emulation is paced to hold it near 50%, so a value that
/// drifts towards 0 or 1 is the visible symptom of a timing problem.
#[derive(Debug, Clone, Copy, Default)]
pub struct AudioStats {
    pub running: bool,
    /// Emulated frames per second; should sit at ~60 for NTSC.
    pub emulated_fps: f32,
    /// UI repaints per second, which is the display's rate and unrelated to the above.
    pub repaint_fps: f32,
    pub sample_rate: f32,
    pub queued: usize,
    pub capacity: usize,
    pub fill_level: f32,
    pub underruns: u64,
    pub dropped: u64,
}

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
    pub fn ui(&mut self, ui: &mut Ui, apu: ApuWrapper, stats: AudioStats) {
        self.controls_ui(ui, apu);
        ui.separator();
        self.stats_ui(ui, stats);
    }

    /// Buffer health and error counters.
    fn stats_ui(&mut self, ui: &mut Ui, stats: AudioStats) {
        ui.collapsing("Output Pipeline", |ui| {
            if !stats.running {
                ui.label("Stream paused");
                return;
            }

            // Emulation should hold ~60 regardless of the display's rate. A gap between these two
            // is expected; emulation drifting from 60 is not.
            ui.horizontal(|ui| {
                let off_rate = (stats.emulated_fps - 60.0).abs() > 3.0;
                ui.label("Emulated:");
                ui.colored_label(
                    if off_rate { egui::Color32::YELLOW } else { egui::Color32::GRAY },
                    format!("{:.1} fps", stats.emulated_fps),
                );
                ui.separator();
                ui.label(format!("Repaints: {:.0} fps", stats.repaint_fps));
            });

            ui.label(format!("Device rate: {:.0} Hz", stats.sample_rate));
            ui.label(format!("Buffered: {} / {} samples", stats.queued, stats.capacity));

            // Emulation tops the buffer back up to ~50%, so the bar should sit near the middle.
            // Pinned at either end means production and consumption have come apart.
            ui.add(
                egui::ProgressBar::new(stats.fill_level)
                    .text(format!("{:.0}% full", stats.fill_level * 100.0)),
            );

            ui.horizontal(|ui| {
                ui.label("Underruns:");
                if stats.underruns > 0 {
                    ui.colored_label(egui::Color32::YELLOW, stats.underruns.to_string());
                } else {
                    ui.label("0");
                }

                ui.label("Dropped:");
                if stats.dropped > 0 {
                    ui.colored_label(egui::Color32::YELLOW, stats.dropped.to_string());
                } else {
                    ui.label("0");
                }
            });

            if stats.underruns > 0 {
                ui.small("Underruns mean the emulator is not keeping up with the sound card.");
            }
            if stats.dropped > 0 {
                ui.small("Drops mean the emulator is running ahead of the sound card.");
            }
        });
    }

    fn controls_ui(&mut self, ui: &mut Ui, mut apu: ApuWrapper) {
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

        // Channel enables, read back from the APU each frame so they show what the running
        // program actually asked for rather than only what was last clicked here.
        ui.collapsing("Channel Controls", |ui| {
            let mut enabled: Vec<bool> = Channel::ALL.iter().map(|&c| apu.channel_enabled(c)).collect();
            let mut changed = false;

            for (index, channel) in Channel::ALL.iter().enumerate() {
                if ui.checkbox(&mut enabled[index], channel.label()).changed() {
                    changed = true;
                }
            }

            if changed {
                let status = Channel::ALL
                    .iter()
                    .zip(&enabled)
                    .filter(|(_, &on)| on)
                    .fold(0u8, |acc, (channel, _)| acc | channel.status_bit());

                if let Err(error) = apu.write_byte(0x4015, status) {
                    log::warn!("Failed to update APU channel enables: {error}");
                }
            }
        });
    }
}
