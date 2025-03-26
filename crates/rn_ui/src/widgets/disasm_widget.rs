#![allow(dead_code)]
use std::cell::Ref;

use anyhow::Result;
use egui::{self, Color32, Ui};
use rn_core::{
    cpu::{Cpu, CpuWrapper, Disassembler},
    memory::Addressable,
};
/// A widget for disassembling and displaying 6502 machine code
pub struct DisasmWidget {
    /// Memory range to disassemble
    start_address: u16,
    /// Number of bytes to disassemble
    disasm_length: u16,
    /// Program load address (when program is loaded)
    program_address: u16,
    /// Program size in bytes (when program is loaded)
    program_size: u16,
    /// Auto-scroll to current instruction
    auto_scroll: bool,
    /// Scroll to this memory address (when auto-scroll is enabled)
    scroll_to_addr: Option<u16>,
}

impl DisasmWidget {
    /// Create a new DisasmWidget
    pub fn new() -> Self {
        Self {
            start_address: 0x8000,   // Default to common program start
            disasm_length: 64,       // Show reasonable number of bytes
            program_address: 0x8000, // Default program address
            program_size: 0,         // No program loaded yet
            auto_scroll: false,      // Auto-scroll disabled by default
            scroll_to_addr: None,    // No scroll target yet
        }
    }

    /// Update program information
    pub fn set_program_info(&mut self, address: u16, size: u16) {
        self.program_address = address;
        self.program_size = size;

        // Always show only the program, or a default empty region
        if size > 0 {
            // If program loaded, show it
            self.start_address = address;
            self.disasm_length = size;
        } else {
            // If no program or program was reset, show a default empty region
            self.start_address = 0x8000; // Default program start
            self.disasm_length = 16; // Just show a few bytes
        }
    }

    /// Enable or disable auto-scrolling
    pub fn set_auto_scroll(&mut self, enabled: bool) {
        self.auto_scroll = enabled;
    }

    /// Toggle auto-scrolling
    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll = !self.auto_scroll;
    }

    /// Get current auto-scroll state
    pub fn auto_scroll(&self) -> bool {
        self.auto_scroll
    }

    /// Display the disassembly widget
    pub fn ui(&mut self, ui: &mut Ui, cpu: CpuWrapper) -> Result<()> {
        ui.horizontal(|ui| {
            ui.heading("Disassembly");
            ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
        });

        ui.add_space(10.0);

        // If no program is loaded, show a message instead of disassembly
        if self.program_size == 0 {
            ui.label("No program loaded. Assemble code to see disassembly.");
            return Ok(());
        }

        // Get current PC
        let current_pc = cpu.pc();

        // Set scroll target if auto-scroll is enabled
        if self.auto_scroll {
            self.scroll_to_addr = Some(current_pc);
        }

        // Create disassembler
        let disassembler = Disassembler::new();

        // Collect memory to disassemble
        let mut memory = Vec::with_capacity(self.disasm_length as usize);
        for addr in self.start_address..self.start_address.wrapping_add(self.disasm_length) {
            memory.push(cpu.read_byte(addr)?);
        }

        // Disassemble the memory region
        let disassembly = disassembler.disassemble_program(&memory, 0, memory.len());

        // Convert relative offsets to actual memory addresses
        let addressed_disassembly: Vec<(usize, Vec<u8>, String)> = disassembly
            .into_iter()
            .map(|(offset, bytes, text)| (self.start_address as usize + offset, bytes, text))
            .collect();

        // Format the result
        let formatted_disassembly = disassembler.format_disassembly(&addressed_disassembly);

        // Create a scrollable ID for this disassembly view (needed for scroll-to-item)
        let scroll_area_id = ui.make_persistent_id("disasm_scroll_area");

        // Create a scrolling area for the disassembly
        egui::ScrollArea::vertical()
            .id_salt(scroll_area_id)
            // Make the scroll area take up the full width
            .auto_shrink([false, true])
            .show(ui, |ui| {
                // Set display properties
                let text_color = ui.style().visuals.text_color();
                let highlight_color = Color32::YELLOW;

                // Display with monospace font
                ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

                // Keep track of found PC line for auto-scrolling
                let mut current_line_idx = 0;
                let mut found_current_line = false;

                // Split the disassembly into lines
                let lines: Vec<&str> = formatted_disassembly.lines().collect();

                // First pass to find current line index (if auto-scroll enabled)
                if self.auto_scroll {
                    for (idx, line) in lines.iter().enumerate() {
                        let line_addr_str = line.split(':').next().unwrap_or("").trim();
                        if let Ok(line_addr) = u16::from_str_radix(line_addr_str, 16) {
                            if line_addr == current_pc {
                                current_line_idx = idx;
                                found_current_line = true;
                                break;
                            }
                        }
                    }
                }

                // Calculate index of the line to scroll to (one line above current if possible)
                let scroll_to_idx = if found_current_line && current_line_idx > 0 {
                    current_line_idx - 1  // One line above current instruction
                } else if found_current_line {
                    current_line_idx  // At current instruction if it's the first line
                } else {
                    0  // Default to top if not found
                };

                // Second pass to actually render the lines
                for (idx, line) in lines.iter().enumerate() {
                    // Parse the address from the start of the line
                    let line_addr_str = line.split(':').next().unwrap_or("").trim();
                    let is_current_line = if let Ok(line_addr) = u16::from_str_radix(line_addr_str, 16) {
                        line_addr == current_pc
                    } else {
                        false
                    };

                    // Highlight the current instruction
                    let color = if is_current_line { highlight_color } else { text_color };

                    // Create a label that takes up the full width available
                    ui.horizontal(|ui| {
                        // Force the horizontal layout to take the full width
                        ui.set_width(ui.available_width());
                        let response = ui.colored_label(color, *line);
                        ui.add_space(ui.available_width()); // Fill remaining space

                        // Auto-scroll to the target line (one above current instruction)
                        if self.auto_scroll && found_current_line && idx == scroll_to_idx {
                            ui.scroll_to_rect(response.rect, Some(egui::Align::Center));
                        }
                    });
                }
            });

        // Reset scroll target
        self.scroll_to_addr = None;

        Ok(())
    }
}
