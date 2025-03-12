#![allow(dead_code)]
use anyhow::Result;
use egui::{self, Color32, Ui};
use rn_core::cpu::{Cpu, Disassembler};
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
}

impl DisasmWidget {
    /// Create a new DisasmWidget
    pub fn new() -> Self {
        Self {
            start_address: 0x8000,   // Default to common program start
            disasm_length: 64,       // Show reasonable number of bytes
            program_address: 0x8000, // Default program address
            program_size: 0,         // No program loaded yet
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

    /// Display the disassembly widget
    pub fn ui(&mut self, ui: &mut Ui, cpu: &Cpu) -> Result<()> {
        ui.heading("Disassembly");

        // If no program is loaded, show a message instead of disassembly
        if self.program_size == 0 {
            ui.label("No program loaded. Assemble code to see disassembly.");
            return Ok(());
        }

        // Get current PC
        let current_pc = cpu.pc;

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

        // Create a scrolling area for the disassembly
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Set display properties
            let text_color = ui.style().visuals.text_color();
            let highlight_color = Color32::YELLOW;

            // Display with monospace font
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

            // Split the disassembly into lines
            for line in formatted_disassembly.lines() {
                // Only highlight when a program is loaded
                let should_highlight = self.program_size > 0;

                // Check if this line contains the current program counter
                let is_current_line = if should_highlight {
                    let line_addr_str = line.split(':').next().unwrap_or("").trim();
                    if let Ok(line_addr) = u16::from_str_radix(line_addr_str, 16) {
                        line_addr == current_pc
                    } else {
                        false
                    }
                } else {
                    false // Don't highlight anything if no program is loaded
                };

                // Highlight the current instruction
                let color = if is_current_line { highlight_color } else { text_color };

                ui.colored_label(color, line);
            }
        });

        Ok(())
    }
}
