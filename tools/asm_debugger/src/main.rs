use eframe::{egui, App, Frame};
use rn_core::cpu::Cpu;
use rn_core::memory::Ram;
use rn_ui::widgets::{AsmWidget, CpuWidget, MemoryVisualizer};
use std::cell::RefCell;
use std::rc::Rc;

/// Main debugger application
struct AsmDebugger {
    // Components
    memory_visualizer: MemoryVisualizer,
    asm_widget: AsmWidget,
    cpu_widget: CpuWidget,

    // Emulation state
    cpu: Rc<RefCell<Cpu>>,

    // UI state
    show_memory_viz: bool,
    show_cpu: bool,
}

impl AsmDebugger {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create a shared CPU instance with RAM
        let ram = Ram::default(); // Use the default 64KB RAM
        let cpu = Rc::new(RefCell::new(Cpu::new(Box::new(ram))));

        Self {
            memory_visualizer: MemoryVisualizer::new(),
            asm_widget: AsmWidget::new(cpu.clone()),
            cpu_widget: CpuWidget::new(),
            cpu,
            show_memory_viz: true,
            show_cpu: true,
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
            ui.checkbox(&mut self.show_cpu, "Show CPU State");
        });

        // Main central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("RustNES Assembly Debugger");

            // Memory visualization
            if self.show_memory_viz {
                ui.add_space(10.0);
                ui.heading("Memory Visualization (0x0200-0x05FF)");
                ui.label("Each byte is represented as a pixel with grayscale value");
                
                // Create a buffer of memory from the CPU's memory for visualization
                let cpu = self.cpu.borrow();
                let mut memory_buffer = Vec::with_capacity(0x0400); // Only need 0x0200-0x05FF range
                
                // We only care about the visualization range (0x0200-0x05FF)
                for addr in 0x0200..=0x05FF {
                    memory_buffer.push(cpu.read_byte(addr));
                }
                
                // Show the memory visualization
                self.memory_visualizer.show(ui, &memory_buffer);
            }

            // Display CPU state if show_cpu is true and the assembler is running
            if self.show_cpu {
                ui.add_space(10.0);

                // Show the CPU widget
                self.cpu_widget.ui(ui, &mut self.cpu.borrow_mut());
            }
        });
    }
}

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Set up the native options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    // Run the app
    eframe::run_native(
        "RustNES Assembly Debugger",
        options,
        Box::new(|cc| Ok(Box::new(AsmDebugger::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("Application error: {}", e))?;

    Ok(())
}
