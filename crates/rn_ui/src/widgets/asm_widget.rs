#![allow(dead_code)]
use egui::{self, Color32, Ui};
use rn_core::cpu::{Assembler, Cpu};

use crate::widgets::{HexEditText, ValueType};

/// A widget for editing and executing 6502 assembly code
pub struct AsmWidget {
    /// The assembly code being edited
    pub code: String,
    /// Flag indicating if the code has been assembled
    pub assembled: bool,
    /// Assembled bytes
    pub assembled_bytes: Vec<u8>,
    /// Error message from assembly process
    pub error_message: Option<String>,
    /// Assembler for 6502 code
    assembler: Assembler,
    /// Default load address for the program
    load_address: u16,
    /// Whether the program is loaded into the CPU
    is_loaded: bool,
    /// Whether the CPU is actually executing
    is_running: bool,
    /// Whether the program has finished execution (hit BRK)
    is_finished: bool,
    /// Load address editor widget
    load_address_editor: HexEditText,
}

impl AsmWidget {
    /// Create a new AsmWidget
    pub fn new() -> Self {
        Self {
            code: String::from("; Enter your 6502 assembly code here\n\nLDA #$01\nSTA $0200\nBRK"),
            assembled: false,
            assembled_bytes: Vec::new(),
            error_message: None,
            assembler: Assembler::new(),
            load_address: 0x8000, // Default load address
            is_loaded: false,
            is_running: false,
            is_finished: false,
            load_address_editor: HexEditText::new(),
        }
    }

    /// Create a new AsmWidget with custom initial code
    pub fn with_code(code: &str) -> Self {
        let mut widget = Self::new();
        widget.code = code.to_string();
        widget
    }

    /// Reset the CPU and load the assembled program
    fn reset_and_load(&mut self, cpu: &mut Cpu) {
        if !self.assembled || self.assembled_bytes.is_empty() {
            return;
        }

        // Load the program into the CPU
        cpu.load_program(&self.assembled_bytes, self.load_address);

        // Update state
        self.is_loaded = true;
        self.is_running = false;
        self.is_finished = false;
    }

    /// Step one instruction in the CPU
    pub fn step(&mut self, cpu: &mut Cpu) {
        if !self.is_loaded || self.is_running || self.is_finished {
            return;
        }

        match cpu.step() {
            Ok(_) => {
                // Check if we've hit a BRK instruction (end of program)
                if cpu.read_byte(cpu.pc) == 0x00 {
                    println!("BRK instruction encountered at ${:04X}, halting", cpu.pc);
                    self.is_finished = true;
                }
            },
            Err(err) => {
                // Handle error
                self.error_message = Some(format!("Execution error: {}", err));
                self.is_finished = true;
            },
        }
    }

    /// Attempt to assemble the current code and immediately load it
    pub fn assemble_code(&mut self, cpu: &mut Cpu) {
        self.assembled_bytes.clear();
        self.error_message = None;

        // Use the assembler's assemble_program method to handle multiple lines and comments
        match self.assembler.assemble_program(&self.code) {
            Ok(bytes) => {
                self.assembled_bytes = bytes;
                self.assembled = true;

                // Immediately reset and load the program
                self.reset_and_load(cpu);
                println!(
                    "Program assembled and loaded at ${:04X}, {} bytes",
                    self.load_address,
                    self.assembled_bytes.len()
                );
            },
            Err(err) => {
                self.error_message = Some(format!("Assembly error: {}", err));
                self.assembled = false;
                self.is_loaded = false;
                self.is_running = false;
                self.is_finished = false;
            },
        }
    }

    /// Getter for the assembled bytes
    pub fn assembled_bytes(&self) -> &[u8] {
        &self.assembled_bytes
    }

    /// Getter for the load address
    pub fn load_address(&self) -> u16 {
        self.load_address
    }

    /// Getter for loaded state
    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }

    /// Run the program until completion or error
    pub fn run_program(&mut self, cpu: &mut Cpu) {
        if !self.is_loaded || self.is_finished {
            return;
        }

        // Mark as running
        self.is_running = true;

        // Execute instructions until we hit a BRK or error
        // Include a safety limit to prevent infinite loops
        let max_steps = 1000;
        let mut steps = 0;

        println!("Running program from ${:04X}", cpu.pc);

        while self.is_running && steps < max_steps {
            match cpu.step() {
                Ok(_) => {
                    steps += 1;

                    // Check if we've hit a BRK instruction
                    if cpu.read_byte(cpu.pc) == 0x00 {
                        println!("BRK instruction encountered at ${:04X}, halting", cpu.pc);
                        self.is_finished = true;
                        self.is_running = false;
                        break;
                    }
                },
                Err(err) => {
                    self.error_message = Some(format!("Execution error at step {}: {}", steps, err));
                    self.is_running = false;
                    self.is_finished = true;
                    println!("Error at step {}: {}", steps, err);
                    break;
                },
            }
        }

        if steps >= max_steps {
            self.error_message = Some(format!("Program reached maximum step limit of {}", max_steps));
            println!("Program reached maximum step limit of {}", max_steps);
            self.is_running = false;
            self.is_finished = true;
        } else {
            println!("Program terminated after {} steps at ${:04X}", steps, cpu.pc);
        }
    }

    /// Set a custom load address
    pub fn set_load_address(&mut self, address: u16) {
        self.load_address = address;
    }

    /// Show the assembled bytes as hex
    fn show_assembled_bytes(&mut self, ui: &mut Ui) {
        if self.assembled_bytes.is_empty() {
            return;
        }

        ui.heading("Assembled Code");

        // Display in hex format
        ui.horizontal_wrapped(|ui| {
            for (i, &byte) in self.assembled_bytes.iter().enumerate() {
                if i > 0 && i % 8 == 0 {
                    ui.end_row();
                }

                let hex = format!("{:02X}", byte);
                ui.label(hex);
                ui.add_space(8.0);
            }
        });

        // Show total size
        ui.add_space(5.0);
        ui.label(format!("Total size: {} bytes", self.assembled_bytes.len()));
    }

    /// Show the widget in the given UI
    pub fn ui(&mut self, ui: &mut Ui, cpu: &mut Cpu) {
        // Code editor
        ui.heading("Assembly Code");

        // Wrap just the multi-line editor in a scroll area
        egui::ScrollArea::vertical()
            .id_salt("asm_code_editor_scroll")
            .max_height(300.0)
            .show(ui, |ui| {
                // Create text editor, disabled if program is loaded
                let text_edit = egui::TextEdit::multiline(&mut self.code)
                    .code_editor()
                    .desired_rows(20)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .interactive(!self.is_loaded);

                ui.add(text_edit);
            });

        ui.add_space(10.0);

        // Load address editor - always visible
        ui.horizontal(|ui| {
            ui.label("Load address:");

            // Only make it editable if not loaded
            if !self.is_loaded {
                if self.load_address_editor.ui(
                    ui,
                    "", // Skip the label since we added it above
                    &mut self.load_address,
                    ValueType::Bit16,
                    Some("Program load address in memory"),
                ) {
                    // Value already updated in load_address
                }
            } else {
                ui.label(format!("${:04X}", self.load_address));
            }
        });

        ui.add_space(5.0);

        // Buttons
        ui.horizontal(|ui| {
            // Assemble button - only enabled when not loaded and not running
            if ui
                .add_enabled(!self.is_running && !self.is_loaded, egui::Button::new("Assemble"))
                .clicked()
            {
                self.assemble_code(cpu);
            }

            // Run button - enabled when loaded and not running or finished
            if ui
                .add_enabled(
                    self.is_loaded && !self.is_running && !self.is_finished,
                    egui::Button::new("Run"),
                )
                .clicked()
            {
                self.run_program(cpu);
            }

            // Step button - enabled when loaded and not running or finished
            if ui
                .add_enabled(
                    self.is_loaded && !self.is_running && !self.is_finished,
                    egui::Button::new("Step"),
                )
                .clicked()
            {
                self.step(cpu);
            }

            // Reset button - enabled when loaded or finished
            if ui
                .add_enabled(self.is_loaded || self.is_finished, egui::Button::new("Reset"))
                .clicked()
            {
                self.full_reset(cpu);
            }
        });

        // Display any error message
        if let Some(error) = &self.error_message {
            ui.add_space(5.0);
            ui.colored_label(Color32::RED, error);
        }

        // Show assembled bytes if available
        if self.assembled {
            ui.add_space(10.0);
            self.show_assembled_bytes(ui);
        }
    }

    /// Fully reset the system and clear memory
    pub fn full_reset(&mut self, cpu: &mut Cpu) {
        // Reset the CPU
        cpu.reset();

        // Also reset our state
        self.is_loaded = false;
        self.is_running = false;
        self.is_finished = false;
    }
}
