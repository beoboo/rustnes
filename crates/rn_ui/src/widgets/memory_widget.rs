use egui::{Color32, Grid, RichText, TextEdit, Ui};
use rn_core::memory::Addressable;
use anyhow::Result;
use crate::widgets::{HexEditText, ValueType};

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
    /// Register edit widgets for each cell (created on demand)
    cell_editors: Vec<HexEditText>,
    /// Start address editor widget
    start_address_editor: HexEditText,
}

impl Default for MemoryWidget {
    fn default() -> Self {
        Self {
            start_address: 0x0000,
            rows: 16,
            bytes_per_row: 16,
            editable: true,
            cell_editors: Vec::new(),
            start_address_editor: HexEditText::new(),
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

    /// Configure whether memory is editable
    pub fn with_editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    /// Get the current start address
    pub fn start_address(&self) -> u16 {
        self.start_address
    }

    /// Show the memory widget UI
    pub fn ui<A: Addressable>(&mut self, ui: &mut Ui, addressable: &mut A) {
        // Controls for navigation
        ui.horizontal(|ui| {
            // Use HexEditText for the start address
            if self.start_address_editor.ui(
                ui,
                "Start Address:",
                &mut self.start_address,
                ValueType::Bit16,
                Some("First memory address to display"),
            ) {
                // Value already updated in start_address
            }

            // Address navigation buttons
            if ui.button("◄").clicked() {
                // Go back one page
                self.start_address = self
                    .start_address
                    .saturating_sub((self.rows as u16) * (self.bytes_per_row as u16));
            }

            if ui.button("◼").clicked() {
                // Go to 0
                self.start_address = 0;
            }

            if ui.button("►").clicked() {
                // Go forward one page
                self.start_address = self
                    .start_address
                    .saturating_add((self.rows as u16) * (self.bytes_per_row as u16));
            }

            // Row configuration
            ui.separator();
            ui.label("Rows:");
            let mut rows_str = self.rows.to_string();
            if ui.text_edit_singleline(&mut rows_str).changed() {
                if let Ok(rows) = rows_str.parse::<u8>() {
                    if rows > 0 {
                        self.rows = rows;
                    }
                }
            }
        });

        // Memory display grid
        Grid::new("memory_grid")
            .striped(true)
            .spacing([4.0, 4.0])
            .show(ui, |ui| -> Result<()> {
                // Header row
                ui.label(""); // Empty cell for address column
                for col in 0..self.bytes_per_row {
                    ui.label(
                        RichText::new(format!("+{:X}", col))
                            .monospace()
                            .color(Color32::LIGHT_BLUE),
                    );
                }
                ui.end_row();

                // Ensure we have enough cell editors
                let total_cells = self.rows as usize * self.bytes_per_row as usize;
                if self.cell_editors.len() < total_cells {
                    self.cell_editors.resize_with(total_cells, HexEditText::new);
                }

                // Memory rows
                for row in 0..self.rows {
                    let row_addr = self
                        .start_address
                        .saturating_add(row as u16 * self.bytes_per_row as u16);

                    // Row address
                    ui.label(
                        RichText::new(format!("{:04X}", row_addr))
                            .monospace()
                            .color(Color32::GOLD),
                    );

                    // Bytes in this row
                    for col in 0..self.bytes_per_row {
                        let addr = row_addr.saturating_add(col as u16);
                        let editor_idx = row as usize * self.bytes_per_row as usize + col as usize;

                        // Get the byte value and create a mutable copy for the widget
                        let byte = addressable.read_byte(addr).expect("Failed to read byte");
                        let mut byte_value = byte as u16;

                        // Use the HexEditText to edit the byte
                        if self.editable {
                            if self.cell_editors[editor_idx].ui(
                                ui,
                                "", // No label for memory cells
                                &mut byte_value,
                                ValueType::Bit8,
                                Some(&format!("Address: ${:04X}", addr)),
                            ) {
                                // Value changed, update memory
                                addressable.write_byte(addr, byte_value as u8)?;
                            }
                        } else {
                            // Just display the value without editing
                            ui.label(RichText::new(format!("{:02X}", byte)).monospace());
                        }
                    }
                    ui.end_row();
                }

                Ok(())
            });
    }
}
