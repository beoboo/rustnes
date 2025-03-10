use egui::{self, Color32, Ui};
use rn_core::cpu::Assembler;

/// A simple widget for editing and executing assembly code
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
}

impl AsmWidget {
    /// Create a new AsmWidget
    pub fn new() -> Self {
        Self {
            code: String::from("; Enter your 6502 assembly code here\n\nLDA #$01\nSTA $0200\nJMP $F000"),
            assembled: false,
            assembled_bytes: Vec::new(),
            error_message: None,
            assembler: Assembler::new(),
        }
    }
    
    /// Attempt to assemble the current code
    fn assemble_code(&mut self) {
        self.assembled_bytes.clear();
        self.error_message = None;
        
        // Use the assembler's assemble_program method to handle multiple lines and comments
        match self.assembler.assemble_program(&self.code) {
            Ok(bytes) => {
                self.assembled_bytes = bytes;
                self.assembled = true;
            },
            Err(err) => {
                self.error_message = Some(format!("Assembly error: {}", err));
                self.assembled = false;
            }
        }
    }
    
    /// Display the assembled bytes as hex
    fn show_assembled_bytes(&self, ui: &mut Ui) {
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
    pub fn show(&mut self, ui: &mut Ui) {
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
            if ui.button("Assemble").clicked() {
                self.assemble_code();
            }
            
            ui.add_enabled_ui(self.assembled, |ui| {
                if ui.button("Run").clicked() {
                    // TODO: Implement run logic
                    println!("Running assembled code");
                }
                
                if ui.button("Reset").clicked() {
                    // TODO: Implement reset logic
                    println!("Resetting execution");
                }
                
                if ui.button("Step").clicked() {
                    // TODO: Implement step logic
                    println!("Stepping execution");
                }
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
} 