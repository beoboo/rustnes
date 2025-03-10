use eframe::{egui, App, Frame};
use rn_core::cpu::Cpu;

// Import our modules
mod memory_viz;
use memory_viz::MemoryVisualizer;

mod asm_widget;
use asm_widget::AsmWidget;

/// Main debugger application
struct AsmDebugger {
    // Components
    memory_visualizer: MemoryVisualizer,
    asm_widget: AsmWidget,
    
    // Emulation state
    cpu: Option<Cpu>,
    memory: Vec<u8>, // Mock memory for testing
    
    // UI state
    show_memory_viz: bool,
}

impl AsmDebugger {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create mock memory with some test data
        let mut memory = vec![0; 0x10000]; // Full 64KB address space
        
        // Add some test patterns to the video memory region (0x0200-0x05FF)
        for i in 0x0200..=0x05FF {
            let x = (i - 0x0200) % 32;
            let y = (i - 0x0200) / 32;
            
            // Create a checkerboard pattern
            if (x / 4 + y / 4) % 2 == 0 {
                memory[i] = 0xFF; // White
            } else {
                memory[i] = 0x00; // Black
            }
            
            // Add a gradient in another area
            if x > 16 && y > 16 {
                memory[i] = ((x + y) % 255) as u8;
            }
        }
        
        Self {
            memory_visualizer: MemoryVisualizer::new(),
            asm_widget: AsmWidget::new(),
            cpu: None,
            memory,
            show_memory_viz: true,
        }
    }
}

impl App for AsmDebugger {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Left panel for controls and settings
        egui::SidePanel::left("controls_panel").show(ctx, |ui| {
            ui.heading("Assembly");
            
            // Show the assembly widget in the side panel
            self.asm_widget.show(ui);
            
            ui.separator();
            
            // Simple controls for memory visualization
            ui.checkbox(&mut self.show_memory_viz, "Show Memory Visualization");
            
            if ui.button("Fill with test pattern").clicked() {
                // Regenerate test pattern
                for i in 0x0200..=0x05FF {
                    let x = (i - 0x0200) % 32;
                    let y = (i - 0x0200) / 32;
                    
                    // Create a different pattern
                    if (x / 2 + y / 4) % 2 == 0 {
                        self.memory[i] = 0xAA;
                    } else {
                        self.memory[i] = 0x55;
                    }
                }
            }
        });
        
        // Main central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("RustNES Assembly Debugger");
            
            // Memory visualization
            if self.show_memory_viz {
                ui.add_space(10.0);
                ui.heading("Memory Visualization (0x0200-0x05FF)");
                ui.label("Each byte is represented as a pixel with grayscale value");
                
                // Show the memory visualization
                self.memory_visualizer.show(ui, &self.memory);
            }
        });
    }
} 