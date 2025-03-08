use egui::{Color32, Grid, RichText, TextEdit, Ui};
use rn_core::memory::Memory;

/// Widget for displaying and editing memory contents
pub struct MemoryWidget {
    /// Starting address for memory display
    start_address: u16,
    /// Number of rows to display
    rows: u8,
    /// Bytes per row
    bytes_per_row: u8,
    /// Whether editing is allowed
    editable: bool,
    /// Temporary buffer for editing
    edit_buffer: Option<(u16, String)>,
}

impl Default for MemoryWidget {
    fn default() -> Self {
        Self {
            start_address: 0x0000,
            rows: 16,
            bytes_per_row: 16,
            editable: true,
            edit_buffer: None,
        }
    }
}

impl MemoryWidget {
    /// Create a new memory widget
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure the starting address
    pub fn with_start_address(mut self, addr: u16) -> Self {
        self.start_address = addr;
        self
    }

    /// Configure the number of rows
    pub fn with_rows(mut self, rows: u8) -> Self {
        self.rows = rows;
        self
    }

    /// Configure the bytes per row
    pub fn with_bytes_per_row(mut self, bytes: u8) -> Self {
        self.bytes_per_row = bytes;
        self
    }

    /// Set whether memory is editable
    pub fn with_editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    /// Get the current start address
    pub fn start_address(&self) -> u16 {
        self.start_address
    }

    /// Set the start address
    pub fn set_start_address(&mut self, addr: u16) {
        self.start_address = addr;
    }

    /// Render the memory widget using the given UI and memory
    pub fn ui<M: Memory>(&mut self, ui: &mut Ui, memory: &mut M) {
        ui.heading("Memory Viewer");
        
        // Navigation controls
        ui.horizontal(|ui| {
            // Address input field
            ui.label("Address:");
            let mut addr_str = format!("{:04X}", self.start_address);
            if ui.text_edit_singleline(&mut addr_str).changed() {
                if let Ok(addr) = u16::from_str_radix(&addr_str, 16) {
                    self.start_address = addr;
                }
            }
            
            // Navigation buttons
            if ui.button("⏮️ Start").clicked() {
                self.start_address = 0x0000;
            }
            if ui.button("⬅️ -256").clicked() {
                self.start_address = self.start_address.saturating_sub(0x0100);
            }
            if ui.button("⬅️ -16").clicked() {
                self.start_address = self.start_address.saturating_sub(0x0010);
            }
            if ui.button("➡️ +16").clicked() {
                self.start_address = self.start_address.saturating_add(0x0010);
            }
            if ui.button("➡️ +256").clicked() {
                self.start_address = self.start_address.saturating_add(0x0100);
            }
            if ui.button("⏭️ End").clicked() {
                // Go to last possible page of memory (64KB - display rows)
                // We need to be careful with u16 overflow
                let bytes_to_display = (self.rows as u16) * (self.bytes_per_row as u16);
                self.start_address = u16::MAX - bytes_to_display + 1;
            }
        });
        
        ui.separator();
        
        // Memory contents display
        Grid::new("memory_grid")
            .num_columns(self.bytes_per_row as usize + 1) // Address + bytes
            .striped(true)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                // Header row with column numbers
                ui.label(RichText::new("Addr").strong());
                for col in 0..self.bytes_per_row {
                    ui.label(RichText::new(format!("{:X}", col)).strong());
                }
                ui.end_row();
                
                // Memory rows
                for row in 0..self.rows {
                    let row_addr = self.start_address.saturating_add(row as u16 * self.bytes_per_row as u16);
                    
                    // Row address
                    ui.label(RichText::new(format!("{:04X}", row_addr)).monospace().color(Color32::GOLD));
                    
                    // Bytes in this row
                    for col in 0..self.bytes_per_row {
                        let addr = row_addr.saturating_add(col as u16);
                        let byte = memory.read_byte(addr);
                        
                        // Check if this byte is being edited
                        if let Some((edit_addr, ref mut buf)) = self.edit_buffer {
                            if edit_addr == addr {
                                // Show text edit field
                                let response = ui.add(
                                    TextEdit::singleline(buf)
                                        .desired_width(24.0)
                                        .font(egui::TextStyle::Monospace)
                                );
                                
                                if response.lost_focus() {
                                    // Try to parse and update the value
                                    if let Ok(value) = u8::from_str_radix(buf, 16) {
                                        memory.write_byte(addr, value);
                                    }
                                    self.edit_buffer = None;
                                }
                                continue;
                            }
                        }
                        
                        // Normal display
                        let byte_text = RichText::new(format!("{:02X}", byte)).monospace();
                        
                        if self.editable {
                            // Clickable if editable
                            if ui.add(egui::Label::new(byte_text).sense(egui::Sense::click())).clicked() {
                                self.edit_buffer = Some((addr, format!("{:02X}", byte)));
                            }
                        } else {
                            // Just display if not editable
                            ui.label(byte_text);
                        }
                    }
                    ui.end_row();
                }
                
                // ASCII representation (as a future enhancement)
                // This could be added as an extra column or separate grid
            });
        
        // Memory region selector
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Jump to region:");
            if ui.button("Zero Page").clicked() {
                self.start_address = 0x0000;
            }
            if ui.button("Stack").clicked() {
                self.start_address = 0x0100;
            }
            if ui.button("RAM").clicked() {
                self.start_address = 0x0200;
            }
            if ui.button("PPU Regs").clicked() {
                self.start_address = 0x2000;
            }
            if ui.button("APU Regs").clicked() {
                self.start_address = 0x4000;
            }
            if ui.button("Cart").clicked() {
                self.start_address = 0x8000;
            }
        });
    }
} 