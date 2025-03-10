#![allow(dead_code)]
use egui::{self, Color32, Ui};
use rn_core::cpu::{Assembler, Cpu};

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
    is_executing: bool,
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
            is_executing: false,
        }
    }

    /// Create a new AsmWidget with custom initial code
    pub fn with_code(code: &str) -> Self {
        let mut widget = Self::new();
        widget.code = code.to_string();
        widget
    }

    /// Reset the CPU and load the assembled program
    pub fn reset_and_load(&mut self, cpu: &mut Cpu) {
        if !self.assembled || self.assembled_bytes.is_empty() {
            return;
        }

        // Load the program into the CPU
        cpu.load_program(&self.assembled_bytes, self.load_address);

        // Update state
        self.is_loaded = true;
        self.is_executing = true;
    }

    /// Step one instruction in the CPU
    pub fn step(&mut self, cpu: &mut Cpu) {
        if !self.is_loaded {
            return;
        }

        self.is_executing = true;

        match cpu.step() {
            Ok(_) => {
                // Instruction executed successfully
            },
            Err(err) => {
                // Handle error
                self.error_message = Some(format!("Execution error: {}", err));
                self.is_executing = false;
            },
        }
    }

    /// Attempt to assemble the current code
    pub fn assemble_code(&mut self) {
        self.assembled_bytes.clear();
        self.error_message = None;

        // Use the assembler's assemble_program method to handle multiple lines and comments
        match self.assembler.assemble_program(&self.code) {
            Ok(bytes) => {
                self.assembled_bytes = bytes;
                self.assembled = true;

                // Reset states but wait for reset_and_load to actually load
                self.is_loaded = false;
                self.is_executing = false;
            },
            Err(err) => {
                self.error_message = Some(format!("Assembly error: {}", err));
                self.assembled = false;
                self.is_loaded = false;
                self.is_executing = false;
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

    /// Fully reset the system
    pub fn run_program(&mut self, cpu: &mut Cpu) {
        // First reset and load the program
        self.reset_and_load(cpu);

        if !self.is_loaded {
            return;
        }

        // Now start executing instructions until we hit a BRK or error
        // Include a safety limit to prevent infinite loops
        let max_steps = 1000;
        let mut steps = 0;

        println!("Running program from ${:04X}", cpu.pc);

        while self.is_executing && steps < max_steps {
            match cpu.step() {
                Ok(_) => {
                    steps += 1;

                    // Check if we've hit a BRK instruction
                    if cpu.read_byte(cpu.pc) == 0x00 {
                        println!("BRK instruction encountered at ${:04X}, halting", cpu.pc);
                        break;
                    }
                },
                Err(err) => {
                    self.error_message = Some(format!("Execution error at step {}: {}", steps, err));
                    self.is_executing = false;
                    println!("Error at step {}: {}", steps, err);
                    break;
                },
            }
        }

        if steps >= max_steps {
            self.error_message = Some(format!("Program reached maximum step limit of {}", max_steps));
            println!("Program reached maximum step limit of {}", max_steps);
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

        // Show total size and load address
        ui.add_space(5.0);
        ui.label(format!("Total size: {} bytes", self.assembled_bytes.len()));
        ui.label(format!("Load address: ${:04X}", self.load_address));

        // Show input for changing load address
        ui.horizontal(|ui| {
            ui.label("Load address:");
            let mut addr_string = format!("{:04X}", self.load_address);
            if ui.text_edit_singleline(&mut addr_string).changed() {
                // Try to parse as hex
                if let Ok(addr) = u16::from_str_radix(&addr_string, 16) {
                    self.load_address = addr;
                }
            }
        });
    }

    /// Show the widget in the given UI
    pub fn ui(&mut self, ui: &mut Ui, cpu: &mut Cpu) {
        // Code editor
        ui.heading("Assembly Code");

        let text_edit = egui::TextEdit::multiline(&mut self.code)
            .code_editor()
            .desired_rows(20)
            .lock_focus(true)
            .desired_width(f32::INFINITY);

        ui.add(text_edit);

        // Buttons
        ui.horizontal(|ui| {
            // Assemble button
            if ui.button("Assemble").clicked() {
                self.assemble_code();
            }

            // Execute buttons (only enabled after assembly)
            ui.add_enabled_ui(self.assembled, |ui| {
                ui.horizontal(|ui| {
                    // "Reset & Load" enabled when assembled
                    if ui.button("Reset & Load").clicked() {
                        self.reset_and_load(cpu);
                    }

                    // "Run" enabled when assembled (will reset and run the program)
                    if ui.button("Run").clicked() {
                        self.run_program(cpu);
                    }

                    // "Reset" enabled when loaded
                    ui.add_enabled_ui(self.is_loaded, |ui| {
                        if ui.button("Reset").clicked() {
                            self.full_reset(cpu);
                        }
                    });

                    // "Step" enabled when loaded
                    ui.add_enabled_ui(self.is_loaded, |ui| {
                        if ui.button("Step").clicked() {
                            self.step(cpu);
                        }
                    });
                });
            });
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
        self.is_executing = false;
    }
}
