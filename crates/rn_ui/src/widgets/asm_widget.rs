use egui::{self, Color32, Ui};
use rn_core::cpu::{Assembler, Cpu};
use std::rc::Rc;
use std::cell::RefCell;

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
    /// CPU for execution
    cpu: Rc<RefCell<Cpu>>,
    /// Default load address for the program
    load_address: u16,
    /// Whether the program is loaded into the CPU
    is_loaded: bool,
    /// Whether the CPU is actually executing
    is_executing: bool,
}

impl AsmWidget {
    /// Create a new AsmWidget
    pub fn new(cpu: Rc<RefCell<Cpu>>) -> Self {
        Self {
            code: String::from("; Enter your 6502 assembly code here\n\nLDA #$01\nSTA $0200\nBRK"),
            assembled: false,
            assembled_bytes: Vec::new(),
            error_message: None,
            assembler: Assembler::new(),
            cpu,
            load_address: 0x8000, // Default load address
            is_loaded: false,
            is_executing: false,
        }
    }

    /// Create a new AsmWidget with custom initial code
    pub fn with_code(cpu: Rc<RefCell<Cpu>>, code: &str) -> Self {
        let mut widget = Self::new(cpu);
        widget.code = code.to_string();
        widget
    }
    
    /// Reset the CPU and load the assembled program
    pub fn reset_and_load(&mut self) {
        if !self.assembled || self.assembled_bytes.is_empty() {
            return;
        }
        
        // Load the program into the CPU
        let mut cpu = self.cpu.borrow_mut();
        cpu.load_program(&self.assembled_bytes, self.load_address);
        
        // Program is loaded but not executing yet
        self.is_loaded = true;
        self.is_executing = false;
    }
    
    /// Run the program until it reaches a halt condition or max steps
    pub fn run_program(&mut self) {
        if !self.is_loaded {
            self.reset_and_load();
        }
        
        self.is_executing = true;
        
        // Define a max number of steps to avoid infinite loops
        let max_steps = 100;
        let mut step_count = 0;
        
        // Execute instructions until we reach a halt or max steps
        let mut cpu = self.cpu.borrow_mut();
        while step_count < max_steps {
            match cpu.step() {
                Ok(_) => {
                    step_count += 1;
                    
                    // Check if we've hit a halt condition (JMP to self or BRK)
                    let pc = cpu.pc;
                    let opcode = cpu.read_byte(pc);
                    
                    // BRK instruction (0x00) or we reached our stop address
                    if opcode == 0x00 || pc >= 0xF000 {
                        println!("Program halted at ${:04X} after {} steps", pc, step_count);
                        if opcode == 0x00 {
                            println!("BRK instruction encountered - program terminated normally");
                        }
                        break;
                    }
                },
                Err(err) => {
                    // Handle error
                    self.error_message = Some(format!("Execution error: {}", err));
                    self.is_executing = false;
                    break;
                }
            }
        }
        
        if step_count >= max_steps {
            println!("Program reached maximum steps ({})", max_steps);
            self.error_message = Some(format!("Program reached maximum of {} steps - possible infinite loop", max_steps));
        }
    }
    
    /// Execute a single instruction
    pub fn step(&mut self) {
        if !self.is_loaded {
            self.reset_and_load();
        }
        
        self.is_executing = true;
        
        let mut cpu = self.cpu.borrow_mut();
        match cpu.step() {
            Ok(_) => {
                // Instruction executed successfully
            },
            Err(err) => {
                // Handle error
                self.error_message = Some(format!("Execution error: {}", err));
                self.is_executing = false;
            }
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
                
                // Load the program but don't run it
                self.reset_and_load();
            },
            Err(err) => {
                self.error_message = Some(format!("Assembly error: {}", err));
                self.assembled = false;
                self.is_loaded = false;
                self.is_executing = false;
            }
        }
    }
    
    /// Check if the CPU is running (legacy compatibility)
    pub fn is_running(&self) -> bool {
        self.is_executing
    }
    
    /// Check if the program is loaded
    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }
    
    /// Check if the CPU is executing
    pub fn is_executing(&self) -> bool {
        self.is_executing
    }
    
    /// Get the assembled bytes
    pub fn assembled_bytes(&self) -> &[u8] {
        &self.assembled_bytes
    }
    
    /// Get the load address
    pub fn load_address(&self) -> u16 {
        self.load_address
    }
    
    /// Set the load address
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
            ui.label("Load address: ");
            let mut addr_text = format!("{:04X}", self.load_address);
            if ui.text_edit_singleline(&mut addr_text).changed() {
                if let Ok(addr) = u16::from_str_radix(&addr_text, 16) {
                    self.load_address = addr;
                }
            }
        });
    }
    
    /// Show the widget in the given UI
    pub fn ui(&mut self, ui: &mut Ui) {
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
            // "Assemble" enabled when not loaded
            ui.add_enabled_ui(!self.is_loaded, |ui| {
                if ui.button("Assemble").clicked() {
                    self.assemble_code();
                }
            });
            
            // Other buttons enabled when assembled
            ui.add_enabled_ui(self.assembled, |ui| {
                // "Run" enabled when loaded but not executing
                ui.add_enabled_ui(self.is_loaded && !self.is_executing, |ui| {
                    if ui.button("Run").clicked() {
                        self.run_program();
                    }
                });
                
                // "Reset" enabled when loaded
                ui.add_enabled_ui(self.is_loaded, |ui| {
                    if ui.button("Reset").clicked() {
                        self.full_reset();
                    }
                });
                
                // "Step" enabled when loaded
                ui.add_enabled_ui(self.is_loaded, |ui| {
                    if ui.button("Step").clicked() {
                        self.step();
                    }
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
    pub fn full_reset(&mut self) {
        // Get CPU
        let mut cpu = self.cpu.borrow_mut();
        
        // Clear the memory where the program was loaded
        if self.assembled {
            // Zero out the memory where the program was loaded
            let end_address = self.load_address + self.assembled_bytes.len() as u16;
            for addr in self.load_address..end_address {
                cpu.write_byte(addr, 0);
            }
            
            // Also clear the reset vector to prevent auto-loading
            cpu.write_word(0xFFFC, 0);
        }
        
        // Reset CPU to initial state
        cpu.reset();
        
        // Explicitly set PC to 0 - this will be outside most disassembly views
        // This is needed because reset reads from the reset vector which might have 
        // been set by a previous program
        cpu.pc = 0;
        
        // Reset widget state
        self.is_loaded = false;
        self.is_executing = false;
    }
} 