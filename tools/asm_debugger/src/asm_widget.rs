use egui::{self, Ui};

/// A simple widget for editing and executing assembly code
pub struct AsmWidget {
    /// The assembly code being edited
    pub code: String,
    /// Flag indicating if the code has been assembled
    pub assembled: bool,
}

impl AsmWidget {
    /// Create a new AsmWidget
    pub fn new() -> Self {
        Self {
            code: String::from("; Enter your 6502 assembly code here\n\nLDA #$01\nSTA $0200\nJMP $F000"),
            assembled: false,
        }
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
                // TODO: Implement assembly logic
                self.assembled = true;
                println!("Assembling code: {}", self.code);
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
    }
} 