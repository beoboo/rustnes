use egui::{Grid, TextEdit, Ui};
use rn_core::cpu::Cpu;

/// Widget for displaying CPU state
pub struct CpuWidget {
    // Single edit buffer
    edit_buffer: String,
    // Currently editing register (if any)
    editing: Option<EditTarget>,
}

/// Identifies what is being edited
#[derive(PartialEq, Copy, Clone)]
enum EditTarget {
    RegA,
    RegX,
    RegY,
    RegSP,
    RegPC,
}

impl Default for CpuWidget {
    fn default() -> Self {
        Self {
            edit_buffer: String::new(),
            editing: None,
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
                ui.label("A (Accumulator):");
                if self.editing == Some(EditTarget::RegA) {
                    let response = ui.add(
                        TextEdit::singleline(&mut self.edit_buffer)
                            .desired_width(50.0)
                            .hint_text(format!("${:02X}", cpu.a)),
                    );

                    if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Ok(value) =
                            u8::from_str_radix(self.edit_buffer.trim_start_matches("$"), 16)
                        {
                            cpu.a = value;
                        }
                        self.editing = None;
                    }
                } else if ui.button(format!("${:02X}", cpu.a)).clicked() {
                    self.edit_buffer = format!("{:02X}", cpu.a);
                    self.editing = Some(EditTarget::RegA);
                }
                ui.end_row();

                // X register
                ui.label("X (Index X):");
                if self.editing == Some(EditTarget::RegX) {
                    let response = ui.add(
                        TextEdit::singleline(&mut self.edit_buffer)
                            .desired_width(50.0)
                            .hint_text(format!("${:02X}", cpu.x)),
                    );

                    if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Ok(value) =
                            u8::from_str_radix(self.edit_buffer.trim_start_matches("$"), 16)
                        {
                            cpu.x = value;
                        }
                        self.editing = None;
                    }
                } else if ui.button(format!("${:02X}", cpu.x)).clicked() {
                    self.edit_buffer = format!("{:02X}", cpu.x);
                    self.editing = Some(EditTarget::RegX);
                }
                ui.end_row();

                // Y register
                ui.label("Y (Index Y):");
                if self.editing == Some(EditTarget::RegY) {
                    let response = ui.add(
                        TextEdit::singleline(&mut self.edit_buffer)
                            .desired_width(50.0)
                            .hint_text(format!("${:02X}", cpu.y)),
                    );

                    if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Ok(value) =
                            u8::from_str_radix(self.edit_buffer.trim_start_matches("$"), 16)
                        {
                            cpu.y = value;
                        }
                        self.editing = None;
                    }
                } else if ui.button(format!("${:02X}", cpu.y)).clicked() {
                    self.edit_buffer = format!("{:02X}", cpu.y);
                    self.editing = Some(EditTarget::RegY);
                }
                ui.end_row();

                // Stack Pointer
                ui.label("SP (Stack Pointer):");
                if self.editing == Some(EditTarget::RegSP) {
                    let response = ui.add(
                        TextEdit::singleline(&mut self.edit_buffer)
                            .desired_width(50.0)
                            .hint_text(format!("${:02X}", cpu.sp)),
                    );

                    if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Ok(value) =
                            u8::from_str_radix(self.edit_buffer.trim_start_matches("$"), 16)
                        {
                            cpu.sp = value;
                        }
                        self.editing = None;
                    }
                } else if ui.button(format!("${:02X}", cpu.sp)).clicked() {
                    self.edit_buffer = format!("{:02X}", cpu.sp);
                    self.editing = Some(EditTarget::RegSP);
                }
                ui.end_row();

                // Program Counter
                ui.label("PC (Program Counter):");
                if self.editing == Some(EditTarget::RegPC) {
                    let response = ui.add(
                        TextEdit::singleline(&mut self.edit_buffer)
                            .desired_width(50.0)
                            .hint_text(format!("${:04X}", cpu.pc)),
                    );

                    if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Ok(value) =
                            u16::from_str_radix(self.edit_buffer.trim_start_matches("$"), 16)
                        {
                            cpu.pc = value;
                        }
                        self.editing = None;
                    }
                } else if ui.button(format!("${:04X}", cpu.pc)).clicked() {
                    self.edit_buffer = format!("{:04X}", cpu.pc);
                    self.editing = Some(EditTarget::RegPC);
                }
                ui.end_row();

                // Status register
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
        Grid::new("cpu_flags_grid")
            .num_columns(8)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                // Flag names
                ui.label("N");
                ui.label("V");
                ui.label("-");
                ui.label("B");
                ui.label("D");
                ui.label("I");
                ui.label("Z");
                ui.label("C");
                ui.end_row();

                // Flag checkboxes for editing
                let flag_masks = [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01];
                let flag_names = [
                    "Negative",
                    "Overflow",
                    "Unused",
                    "Break",
                    "Decimal",
                    "Interrupt Disable",
                    "Zero",
                    "Carry",
                ];

                // Check and update each flag separately
                for (i, &mask) in flag_masks.iter().enumerate() {
                    let mut checked = (cpu.status & mask) != 0;
                    if ui
                        .checkbox(&mut checked, "")
                        .on_hover_text(flag_names[i])
                        .changed()
                    {
                        if checked {
                            cpu.status |= mask;
                        } else {
                            cpu.status &= !mask;
                        }
                    }
                }
                ui.end_row();

                // Flag descriptions in a new row
                ui.label("Sign");
                ui.label("Overflow");
                ui.label("Unused");
                ui.label("Break");
                ui.label("BCD");
                ui.label("IRQ Dis");
                ui.label("Zero");
                ui.label("Carry");
                ui.end_row();
            });
    }
}
