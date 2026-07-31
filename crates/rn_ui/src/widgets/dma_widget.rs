#![allow(dead_code)]
use egui::{Grid, Ui};
use rn_core::system::{dma::DmaControllerWrapper, DmaController};

/// Widget for displaying DMA controller state
#[derive(Default)]
pub struct DmaControllerWidget {}


impl DmaControllerWidget {
    /// Create a new DMA controller widget
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the DMA widget using the given UI and DMA controller
    pub fn ui<C, P>(&mut self, ui: &mut Ui, dma: &DmaControllerWrapper<C, P>)
    where
        C: rn_core::cpu::CpuInterface,
        P: rn_core::ppu::PpuInterface,
    {
        ui.heading("DMA Controller State");

        Grid::new("dma_state_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                // Active status
                ui.label("Active:");
                ui.label(format!("{}", dma.is_active()));
                ui.end_row();

                // Can't directly access internal state like source_high_byte, cycles_remaining
                // since they're private. We only have access to what's exposed through the wrapper.

                // Add a visual indicator for active state
                ui.label("Status:");
                if dma.is_active() {
                    ui.colored_label(egui::Color32::RED, "⏳ TRANSFER IN PROGRESS");
                } else {
                    ui.colored_label(egui::Color32::GREEN, "✓ IDLE");
                }
                ui.end_row();

                // Display cycle information when active
                if dma.is_active() {
                    // Cycles remaining
                    ui.label("Cycles Remaining:");
                    ui.label(format!("{} of 513", dma.cycles_remaining()));
                    ui.end_row();

                    // Cycles elapsed
                    ui.label("Cycles Elapsed:");
                    ui.label(format!("{}", dma.cycles_elapsed()));
                    ui.end_row();

                    // Progress percentage
                    ui.label("Progress:");
                    let progress = (dma.cycles_elapsed() as f32 / 513.0) * 100.0;
                    ui.label(format!("{:.1}%", progress));
                    ui.end_row();

                    // Progress bar
                    ui.label("Transfer:");
                    let progress_bar = egui::ProgressBar::new(progress / 100.0).show_percentage().animate(true);
                    ui.add(progress_bar);
                    ui.end_row();

                    ui.label("Note:");
                    ui.label("CPU execution is suspended during DMA transfer (513 cycles)");
                    ui.end_row();
                }
            });
    }
}
