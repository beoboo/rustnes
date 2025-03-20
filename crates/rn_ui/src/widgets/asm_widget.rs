#![allow(dead_code)]
use std::collections::HashMap;

use anyhow::Result;
use egui::{self, Color32, Ui};
use rn_core::{
    cpu::{Assembler, Cpu},
    system::{NesSystem, SystemState},
};

use crate::widgets::{HexEditText, ValueType};

/// A widget for editing and executing 6502 assembly code
pub struct AsmWidget {
    /// The assembly code being edited
    pub code: String,
    /// Flag indicating if the code has been assembled
    pub assembled: bool,
    /// Assembled bytes (from the default segment)
    pub assembled_bytes: Vec<u8>,
    /// All assembled segments
    pub assembled_segments: HashMap<String, Vec<u8>>,
    /// Error message from assembly process
    pub error_message: Option<String>,
    /// Assembler for 6502 code
    pub assembler: Assembler,
    /// Load address editor widget
    load_address_editor: HexEditText,
    /// Maximum number of cycles to run
    max_cycles: usize,
    /// Whether to run with no cycle limit
    no_cycle_limit: bool,
    /// Whether to run continuously (in the background)
    continuous_run: bool,
    /// Number of cycles to run per frame in continuous mode
    cycles_per_frame: usize,
}

impl AsmWidget {
    /// Create a new AsmWidget
    pub fn new() -> Self {
        // Create assembler with standard NES segments automatically added
        let assembler = Assembler::new(0x8000).with_nes_segments();

        Self {
            code: String::from("; Enter your 6502 assembly code here\n\nLDA #$01\nSTA $0200\nBRK"),
            assembled: false,
            assembled_bytes: Vec::new(),
            assembled_segments: HashMap::new(),
            error_message: None,
            assembler,
            load_address_editor: HexEditText::new(),
            max_cycles: 1_000_000, // Default to 1 million cycles
            no_cycle_limit: false, // Default to using a limit
            continuous_run: false, // Not running continuously by default
            cycles_per_frame: 100,  // Default to 100 cycles per frame
        }
    }

    /// Create a new AsmWidget with custom initial code
    pub fn with_code(code: &str) -> Self {
        let mut widget = Self::new();
        widget.code = code.to_string();
        widget
    }

    /// Reset the system and load the assembled program
    fn reset_and_load(&mut self, system: &mut NesSystem) -> Result<()> {
        if !self.assembled || self.assembled_segments.is_empty() {
            return Ok(());
        }

        // Reset the system first
        system.reset()?;

        // Load the program into the system
        system.load_program(&self.assembled_bytes, self.assembler.load_address)?;

        // Load CHR ROM data if available
        if let Some(chr_data) = self.assembled_segments.get("CHARS") {
            if !chr_data.is_empty() {
                // Get the cartridge and load the CHR ROM data
                if let Err(err) = system.load_chr_rom(&chr_data) {
                    self.error_message = Some(format!("Error loading CHR ROM: {}", err));
                    log::error!("Error loading CHR ROM: {}", err);
                } else {
                    log::info!("Loaded CHR ROM data: {} bytes", chr_data.len());
                }
            }
        }

        Ok(())
    }

    /// Step one instruction in the system
    pub fn step(&mut self, system: &mut NesSystem) -> Result<()> {
        // Only step if the system is in the right state
        if system.state() != SystemState::Loaded && system.state() != SystemState::Running {
            return Ok(());
        }

        // Step the system and capture any error
        if let Err(err) = system.step() {
            // Error already set in NesSystem, just ensure we have it in the widget too
            if self.error_message.is_none() {
                self.error_message = Some(format!("Execution error: {}", err));
                log::error!("Execution error: {}", err);
            }
        }

        Ok(())
    }

    /// Attempt to assemble the current code and immediately load it
    pub fn assemble_code(&mut self, system: &mut NesSystem) -> Result<()> {
        self.assembled_bytes.clear();
        self.assembled_segments.clear();
        self.error_message = None;

        // Use the assembler's assemble_program method to handle multiple lines and comments
        match self.assembler.assemble_program(&self.code) {
            Ok(segments) => {
                self.assembled_segments = segments;

                // Use the STARTUP segment as the default if it exists
                if let Some(startup_bytes) = self.assembled_segments.get("STARTUP") {
                    self.assembled_bytes = startup_bytes.clone();
                } else if let Some((_, bytes)) = self.assembled_segments.iter().next() {
                    // Otherwise use the first segment
                    self.assembled_bytes = bytes.clone();
                }

                self.assembled = true;

                // Immediately reset and load the program
                self.reset_and_load(system)?;
                log::info!(
                    "Program assembled and loaded at ${:04X}, {} bytes",
                    self.assembler.load_address,
                    self.assembled_bytes.len()
                );
            },
            Err(err) => {
                self.error_message = Some(format!("Assembly error: {}", err));
                log::error!("Assembly error: {}", err);
                self.assembled = false;
            },
        }

        Ok(())
    }

    /// Getter for the assembled bytes
    pub fn assembled_bytes(&self) -> &[u8] {
        &self.assembled_bytes
    }

    /// Getter for the load address
    pub fn load_address(&self) -> u16 {
        self.assembler.load_address
    }

    /// Getter for loaded state
    pub fn is_loaded(&self) -> bool {
        self.assembled
    }

    /// Run the program until completion or error, using the configured cycle limit
    pub fn run_program(&mut self, system: &mut NesSystem) -> Result<()> {
        // Check if continuous mode is enabled - if so, we'll just toggle the flag
        if self.continuous_run {
            self.continuous_run = false;
            return Ok(());
        }
        
        // Only run if the system is in a valid state (Ready, Loaded, Running, or Finished)
        if !matches!(system.state(), SystemState::Ready | SystemState::Loaded | SystemState::Running | SystemState::Finished) {
            return Ok(());
        }

        // If system is in Finished state, reset it first
        if system.state() == SystemState::Finished {
            // Reset the system
            log::info!("System in Finished state, resetting before run");
            system.reset()?;
            
            // Reload the program if assembled
            if self.assembled {
                system.load_program(&self.assembled_bytes, self.assembler.load_address)?;
                
                // Also load CHR ROM data if available
                if let Some(chr_data) = self.assembled_segments.get("CHARS") {
                    if !chr_data.is_empty() {
                        system.load_chr_rom(&chr_data)?;
                    }
                }
            }
        }

        // Start continuous running
        self.continuous_run = true;
        
        // Make sure cycles_per_frame has a reasonable value
        if self.cycles_per_frame == 0 {
            self.cycles_per_frame = 100;
        }

        // The actual continuous running happens in the run_continuous method
        // which is called by the main app loop
        Ok(())
    }

    /// Set a custom load address
    pub fn set_load_address(&mut self, address: u16) {
        self.assembler.load_address = address;
    }

    /// Show the widget in the given UI
    pub fn ui(&mut self, ui: &mut Ui, system: &mut NesSystem) {
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
                    .interactive(system.state() == SystemState::Ready); // Only editable when system is ready

                ui.add(text_edit);
            });

        ui.add_space(10.0);

        // Load address editor - always visible
        ui.horizontal(|ui| {
            ui.label("Load address:");

            // Only make it editable if not loaded
            if system.state() == SystemState::Ready {
                let mut load_address = self.assembler.load_address;
                if self.load_address_editor.ui(
                    ui,
                    "", // Skip the label since we added it above
                    &mut load_address,
                    ValueType::Bit16,
                    Some("Program load address in memory"),
                ) {
                    // Update the assembler with the new load address
                    self.set_load_address(load_address);
                }
            } else {
                ui.label(format!("${:04X}", self.assembler.load_address));
            }
        });

        ui.add_space(5.0);

        // Status indicator
        ui.horizontal(|ui| {
            ui.label("Status: ");
            match system.state() {
                SystemState::Ready => ui.colored_label(Color32::WHITE, "Ready"),
                SystemState::Loaded => ui.colored_label(Color32::CYAN, "Program loaded"),
                SystemState::Running => {
                    if self.continuous_run {
                        ui.colored_label(Color32::GOLD, "Running continuously")
                    } else {
                        ui.colored_label(Color32::YELLOW, "Running")
                    }
                },
                SystemState::Finished => ui.colored_label(Color32::GREEN, "Finished"),
                SystemState::Error(pc) => ui.colored_label(Color32::RED, format!("Error at ${:04X}", pc)),
            };
        });

        ui.add_space(5.0);

        // Buttons
        ui.horizontal(|ui| -> Result<()> {
            // Assemble button - only enabled when system is ready
            if ui
                .add_enabled(system.state() == SystemState::Ready, egui::Button::new("Assemble"))
                .clicked()
            {
                self.assemble_code(system)?;
            }

            // Run button - enabled when system is loaded, running, or finished
            let can_run = matches!(system.state(), SystemState::Loaded | SystemState::Running | SystemState::Finished);
            let run_text = if self.continuous_run { "Stop" } else { "Run" };
            if ui.add_enabled(can_run, egui::Button::new(run_text)).clicked() {
                self.run_program(system)?;
            }

            // Step button - enabled when system is loaded or running
            let can_step = matches!(system.state(), SystemState::Loaded | SystemState::Running);
            if ui.add_enabled(can_step, egui::Button::new("Step")).clicked() {
                self.step(system)?;
            }

            // Reset button - enabled when not in ready state
            if ui
                .add_enabled(system.state() != SystemState::Ready, egui::Button::new("Reset"))
                .clicked()
            {
                system.reset()?;
                // Make sure continuous run is stopped on reset
                self.continuous_run = false;
            }

            // Run to Next Frame button - enabled when system is loaded, running, or finished
            if ui.add_enabled(can_run, egui::Button::new("Run to Next Frame")).clicked() {
                // If system is in Finished state, reset it first
                if system.state() == SystemState::Finished {
                    // Reset the system
                    log::info!("System in Finished state, resetting before running to next frame");
                    system.reset()?;
                    
                    // Reload the program if assembled
                    if self.assembled {
                        system.load_program(&self.assembled_bytes, self.assembler.load_address)?;
                        
                        // Also load CHR ROM data if available
                        if let Some(chr_data) = self.assembled_segments.get("CHARS") {
                            if !chr_data.is_empty() {
                                system.load_chr_rom(&chr_data)?;
                            }
                        }
                    }
                }
                
                // Run up to PPU frame completion
                let current_frame = system.ppu().frame_count();
                let target_frame = current_frame + 1;
                
                // Run until we reach the next frame - no safety limit
                while system.ppu().frame_count() < target_frame {
                    if let Err(e) = system.step() {
                        self.error_message = Some(format!("Error stepping to next frame: {}", e));
                        break;
                    }
                    
                    // Also check if we've hit the Finished state
                    if system.state() == SystemState::Finished {
                        break; // Stop running if we hit a BRK
                    }
                }
                
                // Force a frame render
                system.ppu().force_render_frame();
            }

            Ok(())
        });

        ui.add_space(5.0);

        // Add the cycle limit controls in a single row
        ui.horizontal(|ui| {
            // Only show the cycle limit field if "No cycle limit" is unchecked
            if !self.no_cycle_limit {
                ui.label("Run for: ");
                // Use a DragValue for easy adjustment
                ui.add(
                    egui::DragValue::new(&mut self.max_cycles)
                        .speed(10_000)
                        .range(1..=100_000_000),
                );
                ui.label("cycles");
            }

            // "No cycle limit" checkbox
            ui.checkbox(&mut self.no_cycle_limit, "No limit");
            
            ui.separator();
            
            // Cycles per frame input
            ui.label("Cycles/frame:");
            ui.add(
                egui::DragValue::new(&mut self.cycles_per_frame)
                    .speed(10)
                    .range(10..=10000)
            );
        });

        // Display any error message
        if let Some(err_msg) = system.error_message() {
            ui.add_space(5.0);
            ui.colored_label(Color32::RED, err_msg);
        } else if let Some(err_msg) = &self.error_message {
            ui.add_space(5.0);
            ui.colored_label(Color32::RED, err_msg);
        }
    }

    /// Run a fixed number of cycles in continuous mode
    /// Returns true if we should continue running, false if we've stopped
    pub fn run_continuous(&mut self, system: &mut NesSystem) -> bool {
        if !self.continuous_run {
            return false;
        }
        
        // Check if system is in Finished state and reset if needed
        if system.state() == SystemState::Finished {
            // Reset the system
            log::info!("System in Finished state, resetting before continuous run");
            if let Err(e) = system.reset() {
                self.error_message = Some(format!("Error resetting system: {}", e));
                self.continuous_run = false;
                return false;
            }
            
            // Reload the program if assembled
            if self.assembled {
                match system.load_program(&self.assembled_bytes, self.assembler.load_address) {
                    Ok(_) => {
                        // Also load CHR ROM data if available
                        if let Some(chr_data) = self.assembled_segments.get("CHARS") {
                            if !chr_data.is_empty() {
                                if let Err(e) = system.load_chr_rom(&chr_data) {
                                    self.error_message = Some(format!("Error loading CHR ROM: {}", e));
                                    self.continuous_run = false;
                                    return false;
                                }
                            }
                        }
                    },
                    Err(e) => {
                        self.error_message = Some(format!("Error loading program: {}", e));
                        self.continuous_run = false;
                        return false;
                    }
                }
            }
        }
        
        // If no cycle limit, run a much larger number of cycles
        let cycles_to_run = if self.no_cycle_limit {
            1_000_000 // Run a lot of cycles per frame when unlimited
        } else {
            self.cycles_per_frame
        };
        
        // Run a fixed number of cycles
        let mut cycles_run = 0;
        while cycles_run < cycles_to_run {
            match system.step() {
                Ok(_) => {
                    cycles_run += 1;
                },
                Err(e) => {
                    self.error_message = Some(format!("Error during continuous run: {}", e));
                    self.continuous_run = false; // Stop on error
                    return false;
                }
            }
            
            // Check if we've hit a terminal state
            if system.state() == SystemState::Finished {
                // Don't stop the continuous run - we'll reset on next frame
                break;
            }
        }
        
        // Force a frame render periodically
        if cycles_run > 0 {
            if self.no_cycle_limit || cycles_run % 1000 == 0 {
                system.ppu().force_render_frame();
            }
        }
        
        // Continue running
        true
    }
    
    /// Check if continuous run is enabled
    pub fn is_continuous_run(&self) -> bool {
        self.continuous_run
    }
}
