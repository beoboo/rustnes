use egui::{Grid, TextEdit, Ui};
use rn_core::cpu::Cpu;

use crate::widgets::{HexEditText, ValueType};

/// Widget for displaying CPU state
pub struct CpuWidget {
    // Register edit widgets
    a_register: HexEditText,
    x_register: HexEditText,
    y_register: HexEditText,
    sp_register: HexEditText,
    pc_register: HexEditText,
}

impl Default for CpuWidget {
    fn default() -> Self {
        Self {
            a_register: HexEditText::new(),
            x_register: HexEditText::new(),
            y_register: HexEditText::new(),
            sp_register: HexEditText::new(),
            pc_register: HexEditText::new(),
        }
    }
}

impl CpuWidget {
    /// Create a new CPU widget
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the CPU widget using the given UI and CPU
    pub fn ui(&mut self, ui: &mut Ui, cpu: &mut Cpu) {
        ui.heading("CPU State");

        Grid::new("cpu_registers_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                // A register
                let mut a_value = cpu.a as u16;
                if self.a_register.ui(
                    ui,
                    "A (Accumulator):",
                    &mut a_value,
                    ValueType::Bit8,
                    Some("Accumulator Register"),
                ) {
                    cpu.a = a_value as u8;
                }
                ui.end_row();

                // X register
                let mut x_value = cpu.x as u16;
                if self.x_register.ui(
                    ui,
                    "X (Index X):",
                    &mut x_value,
                    ValueType::Bit8,
                    Some("X Index Register"),
                ) {
                    cpu.x = x_value as u8;
                }
                ui.end_row();

                // Y register
                let mut y_value = cpu.y as u16;
                if self.y_register.ui(
                    ui,
                    "Y (Index Y):",
                    &mut y_value,
                    ValueType::Bit8,
                    Some("Y Index Register"),
                ) {
                    cpu.y = y_value as u8;
                }
                ui.end_row();

                // Stack Pointer
                let mut sp_value = cpu.sp as u16;
                if self.sp_register.ui(
                    ui,
                    "SP (Stack Pointer):",
                    &mut sp_value,
                    ValueType::Bit8,
                    Some("Stack Pointer Register"),
                ) {
                    cpu.sp = sp_value as u8;
                }
                ui.end_row();

                // Program Counter
                if self.pc_register.ui(
                    ui,
                    "PC (Program Counter):",
                    &mut cpu.pc,
                    ValueType::Bit16,
                    Some("Program Counter Register"),
                ) {
                    // PC updated directly
                }
                ui.end_row();

                // Status register - this is shown as read-only with individual flags below
                ui.label("Status (P):");
                ui.label(format!("${:02X}", cpu.status));
                ui.end_row();

                // Cycles
                ui.label("Cycles:");
                ui.label(format!("{}", cpu.cycles));
                ui.end_row();
            });

        // Display Status flags with checkboxes
        ui.heading("Status Flags");

        // Define flags with their masks, labels, and tooltips
        let flags = [
            (0x80, "N", "Negative"),
            (0x40, "V", "Overflow"),
            (0x20, "-", "Unused"),
            (0x10, "B", "Break"),
            (0x08, "D", "Decimal"),
            (0x04, "I", "Interrupt Disable"),
            (0x02, "Z", "Zero"),
            (0x01, "C", "Carry"),
        ];

        // Ultra-compact grid layout
        ui.horizontal(|ui| {
            // Reduce UI scale for this section to make everything smaller
            let original_spacing = ui.spacing().clone();

            // Minimize spacing between elements
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

            for &(mask, label, tooltip) in &flags {
                ui.vertical(|ui| {
                    // Set very small minimum width
                    ui.set_min_width(16.0);
                    ui.set_max_width(16.0);

                    // Centered small label
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(label).text_style(egui::TextStyle::Small));
                    });

                    // Tight checkbox placement
                    let mut checked = (cpu.status & mask) != 0;
                    let response = ui.checkbox(&mut checked, "").on_hover_text(tooltip);

                    if response.changed() {
                        if checked {
                            cpu.status |= mask;
                        } else {
                            cpu.status &= !mask;
                        }
                    }
                });
            }

            // Restore original spacing
            *ui.spacing_mut() = original_spacing;
        });
    }
}
