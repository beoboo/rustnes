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
    /// Whether to use authentic NES timing (29,780 cycles/frame)
    use_authentic_timing: bool,
    /// Target frames per second (independent of cycle limit)
    target_fps: f32,
    /// Whether to limit FPS
    limit_fps: bool,
    /// Last timestamp for FPS limiting
    last_frame_time: std::time::Instant,
    /// CPU cycles executed since continuous run started, for the optional cycle limit.
    total_cycles_run: usize,
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
            max_cycles: 1_000_000,      // Default to 1 million cycles
            no_cycle_limit: true,       // Default to no cycle limit
            continuous_run: false,      // Not running continuously by default
            cycles_per_frame: 29780,    // Default to authentic NES cycles per frame
            use_authentic_timing: true, // Default to authentic timing
            target_fps: 60.0,           // Default to 60 FPS
            limit_fps: true,            // Limit FPS by default
            last_frame_time: std::time::Instant::now(),
            total_cycles_run: 0,
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
                if let Err(err) = system.load_chr_rom(chr_data) {
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
                // Format a more detailed error message
                let error_message = format!("Assembly error: {}", err);
                self.error_message = Some(error_message);
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
        if !matches!(
            system.state(),
            SystemState::Ready | SystemState::Loaded | SystemState::Running | SystemState::Finished
        ) {
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
                        system.load_chr_rom(chr_data)?;
                    }
                }
            }
        }

        // Start continuous running
        self.continuous_run = true;

        // Make sure cycles_per_frame has a reasonable value
        if self.cycles_per_frame == 0 {
            self.cycles_per_frame = 29780;
        }

        // The actual continuous running happens in the run_continuous method
        // which is called by the main app loop
        Ok(())
    }

    /// Set a custom load address
    pub fn set_load_address(&mut self, address: u16) {
        self.assembler.load_address = address;
    }

    /// Reset and reload the program
    pub fn reset_program(&mut self, system: &mut NesSystem) -> Result<()> {
        // Always reset the system first
        system.reset()?;

        // If we have assembled code, reload it
        if self.assembled && !self.assembled_segments.is_empty() {
            // Load the program into the system
            system.load_program(&self.assembled_bytes, self.assembler.load_address)?;

            // Load CHR ROM data if available
            if let Some(chr_data) = self.assembled_segments.get("CHARS") {
                if !chr_data.is_empty() {
                    // Get the cartridge and load the CHR ROM data
                    if let Err(err) = system.load_chr_rom(chr_data) {
                        self.error_message = Some(format!("Error loading CHR ROM: {}", err));
                        log::error!("Error loading CHR ROM: {}", err);
                    } else {
                        log::info!("Loaded CHR ROM data: {} bytes", chr_data.len());
                    }
                }
            }
        } else {
            // Clear any error message when fully resetting with no code
            self.error_message = None;
            log::info!("System reset to Ready state");
        }

        // When user explicitly resets, set assembled flag to false to allow new code
        // This ensures the system truly goes back to Ready state
        self.assembled = false;
        self.assembled_bytes.clear();
        self.assembled_segments.clear();

        Ok(())
    }

    /// Run until the next frame is rendered
    pub fn run_to_next_frame(&mut self, system: &mut NesSystem) -> Result<()> {
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
                        system.load_chr_rom(chr_data)?;
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

        Ok(())
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
        if let Some(error) = &self.error_message {
            ui.colored_label(Color32::RED, error);
        } else {
            match system.state() {
                SystemState::Ready => ui.label("Ready to assemble"),
                SystemState::Loaded => ui.label("Program loaded and ready to run"),
                SystemState::Running => ui.label("Program is running"),
                SystemState::Finished => ui.label("Program execution finished (hit BRK)"),
                SystemState::Error(pc) => ui.colored_label(Color32::RED, format!("Error at ${:04X}", pc)),
            };
        }

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

            // "No cycle limit" checkbox with proper description
            ui.checkbox(&mut self.no_cycle_limit, "No cycle limit")
                .on_hover_text("When checked, program will run indefinitely.\nWhen unchecked, program will stop after reaching the specified number of cycles.");
        });

        // Cycles per frame and authentic timing controls
        ui.horizontal(|ui| {
            // Authentic timing checkbox
            if ui
                .checkbox(&mut self.use_authentic_timing, "Use authentic NES timing")
                .clicked()
            {
                // When checked, set to authentic NES timing (29,780 cycles per frame)
                if self.use_authentic_timing {
                    self.cycles_per_frame = 29780;
                }
            }

            ui.separator();

            // Cycles per frame input
            ui.label("Cycles/frame:");

            // Update widget to show the cycles per frame
            // Use add_enabled to disable the widget when using authentic timing
            let response = ui.add_enabled(
                !self.use_authentic_timing,
                egui::DragValue::new(&mut self.cycles_per_frame)
                    .speed(100)
                    .range(10..=100_000),
            );

            // If user changed the cycles manually, turn off authentic timing
            if response.changed() {
                self.use_authentic_timing = false;
            }
        });

        // Add FPS control in a new row
        ui.horizontal(|ui| {
            // FPS limit checkbox
            ui.checkbox(&mut self.limit_fps, "Limit FPS");

            if self.limit_fps {
                ui.label("Target FPS:");
                ui.add(egui::Slider::new(&mut self.target_fps, 1.0..=240.0).step_by(1.0));
            }
        });
    }

    /// Run a batch of cycles in continuous mode, paced by the wall clock.
    ///
    /// Returns true if we should continue running, false if we've stopped.
    pub fn run_continuous(&mut self, system: &mut NesSystem) -> bool {
        self.run_continuous_with_budget(system, None)
    }

    /// Run a batch of cycles in continuous mode.
    ///
    /// `cycle_budget` is how many CPU cycles to execute this call. When `Some`, the caller is
    /// pacing emulation itself — normally against the audio clock, which is the only source of
    /// timing that cannot drift relative to what you hear — and the frame/FPS logic is bypassed.
    /// When `None` the widget falls back to its own wall-clock pacing, which is what happens when
    /// audio is paused or unavailable.
    pub fn run_continuous_with_budget(&mut self, system: &mut NesSystem, cycle_budget: Option<usize>) -> bool {
        if !self.continuous_run {
            return false;
        }

        let now = std::time::Instant::now();

        // Apply FPS control - determines both timing and cycles per update
        if cycle_budget.is_some() {
            // Externally paced: the caller decides how much work to do, so skip the FPS gate but
            // keep the timestamp current for when pacing hands back to the wall clock.
            self.last_frame_time = now;
        } else if self.limit_fps {
            // Calculate the target frame duration based on the desired FPS
            let target_frame_duration = std::time::Duration::from_secs_f32(1.0 / self.target_fps);
            let elapsed = now.duration_since(self.last_frame_time);

            // For slowing down (target FPS < natural speed):
            // If not enough time has passed, skip this frame to maintain lower FPS
            if elapsed < target_frame_duration && self.target_fps <= 60.0 {
                return true; // Skip processing but keep running
            }

            // For both speeding up and slowing down:
            // Only update the timestamp when we actually process a frame
            self.last_frame_time = now;
        } else {
            // When FPS is not limited, still update last_frame_time for next calculation
            self.last_frame_time = now;
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
                                if let Err(e) = system.load_chr_rom(chr_data) {
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
                    },
                }
            }
        }

        // Determine how many cycles to run this frame.
        // An externally supplied budget wins; otherwise scale by the target FPS.
        let cycles_to_run = if let Some(budget) = cycle_budget {
            budget
        } else if self.limit_fps && self.target_fps != 60.0 {
            // Calculate cycles based on target FPS
            // At 60 FPS we run exactly one frame's worth of cycles (29780)
            // At higher FPS we run fewer cycles per frame
            // At lower FPS we run more cycles per frame
            // This gives us proper speed control in both directions
            let speed_ratio = 60.0 / self.target_fps;
            (self.cycles_per_frame as f32 / speed_ratio) as usize
        } else {
            // Normal case - run exactly one frame's worth of cycles
            self.cycles_per_frame
        };

        // Run the calculated number of cycles.
        //
        // `step()` executes one instruction and returns how many CPU cycles it took, so the budget
        // must be charged that many cycles — counting calls instead made a "frame" run roughly 3x
        // too much work, since the average 6502 instruction is about 3 cycles.
        let mut cycles_run = 0usize;
        while cycles_run < cycles_to_run {
            match system.step() {
                Ok(cycles) => {
                    cycles_run += cycles.max(1) as usize;
                },
                Err(e) => {
                    self.error_message = Some(format!("Error during continuous run: {}", e));
                    self.continuous_run = false; // Stop on error
                    return false;
                },
            }

            // Check if we've hit a terminal state
            if system.state() == SystemState::Finished {
                // Don't stop the continuous run - we'll reset on next frame
                break;
            }
        }

        // Check if we've hit the cycle limit when no_cycle_limit is false.
        // Held per widget rather than in a `static mut`, which was both unsound and shared across
        // every AsmWidget in the process.
        if !self.no_cycle_limit {
            self.total_cycles_run += cycles_run;

            if self.total_cycles_run >= self.max_cycles {
                log::info!("Reached cycle limit of {} cycles", self.max_cycles);
                self.continuous_run = false;
                self.total_cycles_run = 0; // Reset for next run
                return false;
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

impl Default for AsmWidget {
    fn default() -> Self {
        Self::new()
    }
}
