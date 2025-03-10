use std::{cell::RefCell, rc::Rc};

use eframe::{egui, App, Frame};
use rn_core::{
    cpu::Cpu,
    memory::{Addressable, Ram},
};
use rn_ui::widgets::{AsmWidget, CpuWidget, DisasmWidget, MemoryVisualizer, MemoryWidget};

/// Adapter to use CPU's memory with the memory editor
struct CpuMemoryAdapter<'a> {
    cpu: &'a mut Cpu,
}

impl<'a> CpuMemoryAdapter<'a> {
    fn new(cpu: &'a mut Cpu) -> Self {
        Self { cpu }
    }
}

impl<'a> Addressable for CpuMemoryAdapter<'a> {
    fn handles_address(&self, _address: u16) -> bool {
        true
    }

    fn read_byte(&self, address: u16) -> u8 {
        self.cpu.read_byte(address)
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.cpu.write_byte(address, value);
    }
}

/// Main debugger application
struct AsmDebugger {
    // Components
    memory_visualizer: MemoryVisualizer,
    asm_widget: AsmWidget,
    cpu_widget: CpuWidget,
    disasm_widget: DisasmWidget,
    memory_widget: MemoryWidget,

    // Emulation state
    cpu: Rc<RefCell<Cpu>>,

    // UI state
    show_memory_viz: bool,
    show_cpu: bool,
    show_disasm: bool,
    show_memory_editor: bool,
}

impl AsmDebugger {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create a shared CPU instance with RAM
        let ram = Ram::default(); // Use the default 64KB RAM
        let cpu = Rc::new(RefCell::new(Cpu::new(Box::new(ram))));

        Self {
            memory_visualizer: MemoryVisualizer::new(),
            asm_widget: AsmWidget::new(),
            cpu_widget: CpuWidget::new(),
            disasm_widget: DisasmWidget::new(),
            memory_widget: MemoryWidget::new()
                .with_start_address(0x0000)
                .with_rows(16)
                .with_bytes_per_row(16)
                .with_editable(true),
            cpu,
            show_memory_viz: true,
            show_cpu: true,
            show_disasm: true,
            show_memory_editor: false,
        }
    }
}

impl App for AsmDebugger {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Update DisasmWidget with program information
        if self.asm_widget.is_loaded() {
            self.disasm_widget.set_program_info(
                self.asm_widget.load_address(),
                self.asm_widget.assembled_bytes().len() as u16,
            );
        } else {
            // When program is not loaded (including after reset), show empty region
            self.disasm_widget.set_program_info(0x8000, 0);
        }

        // Left panel for controls and assembly editor
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.heading("RustNES Assembly Debugger");
            ui.add_space(10.0);

            // Show the assembly widget in the side panel
            let mut cpu_borrow = self.cpu.borrow_mut();
            self.asm_widget.ui(ui, &mut *cpu_borrow);

            ui.separator();

            // Simple controls for display options
            ui.checkbox(&mut self.show_memory_viz, "Show Memory Visualization");
            ui.checkbox(&mut self.show_cpu, "Show CPU State");
            ui.checkbox(&mut self.show_disasm, "Show Disassembly");
            ui.checkbox(&mut self.show_memory_editor, "Show Memory Editor");
        });

        // Right panel for disassembly
        egui::SidePanel::right("right_panel").show_animated(ctx, self.show_disasm, |ui| {
            // Show the disassembly widget
            let cpu_ref = self.cpu.borrow();
            self.disasm_widget.ui(ui, &cpu_ref);
        });

        // Main central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            // Memory visualization
            if self.show_memory_viz {
                ui.add_space(5.0);

                // Create a buffer of memory from the CPU's memory for visualization
                let cpu = self.cpu.borrow();
                let mut memory_buffer = Vec::with_capacity(0x0400); // Only need 0x0200-0x05FF range

                // We only care about the visualization range (0x0200-0x05FF)
                for addr in 0x0200..=0x05FF {
                    memory_buffer.push(cpu.read_byte(addr));
                }

                // Show the memory visualization
                self.memory_visualizer.ui(ui, &memory_buffer);

                ui.separator();
            }

            // Memory editor
            if self.show_memory_editor {
                ui.add_space(5.0);

                // Create an adapter to access CPU memory with the memory editor
                let mut cpu_borrow = self.cpu.borrow_mut();
                let mut adapter = CpuMemoryAdapter::new(&mut *cpu_borrow);

                // Show the memory editor widget with access to CPU memory
                self.memory_widget.ui(ui, &mut adapter);

                ui.separator();
            }

            // Display CPU state
            if self.show_cpu {
                ui.add_space(10.0);

                // Show the CPU widget
                self.cpu_widget.ui(ui, &mut self.cpu.borrow_mut());

                ui.separator();
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
