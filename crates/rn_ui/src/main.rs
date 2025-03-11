use eframe::egui;
use rn_core::{
    cpu::Cpu,
    memory::{Addressable, Ram},
};
use anyhow::Result;
// Import widgets module
mod widgets;
use widgets::{CpuWidget, MemoryWidget};

fn main() -> Result<(), eframe::Error> {
    // Set up logging
    tracing_subscriber::fmt::init();

    // Create native window options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]), // Larger window for both widgets
        ..Default::default()
    };

    // Run the app
    eframe::run_native(
        "RustNES Debugger",
        options,
        Box::new(|_cc| Ok(Box::new(RustNESApp::default()))),
    )
}

/// Main application state
struct RustNESApp {
    cpu: Cpu,
    memory: Ram, // Store memory separately from CPU for UI purposes
    cpu_widget: CpuWidget,
    memory_widget: MemoryWidget,
    selected_tab: Tab,
}

#[derive(PartialEq)]
enum Tab {
    Cpu,
    Memory,
}

impl Default for RustNESApp {
    fn default() -> Self {
        Self::new().expect("Failed to create RustNESApp")
    }
}

impl eframe::App for RustNESApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("RustNES Debugger");
                ui.add_space(32.0);

                ui.selectable_value(&mut self.selected_tab, Tab::Cpu, "CPU");
                ui.selectable_value(&mut self.selected_tab, Tab::Memory, "Memory");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| -> Result<()> {
            match self.selected_tab {
                Tab::Cpu => {
                    // Show CPU state and controls
                    self.cpu_widget.ui(ui, &mut self.cpu);

                    ui.add_space(16.0);

                    // We don't need these manual controls anymore since the registers are directly editable
                    ui.heading("CPU Instructions");
                    ui.horizontal(|ui| -> Result<()> {
                        if ui.button("Reset CPU").clicked() {
                            self.cpu.reset()?;
                        }

                        Ok(())
                    });
                },
                Tab::Memory => {
                    // Show memory viewer
                    self.memory_widget.ui(ui, &mut self.memory);

                    ui.add_space(16.0);

                    // Add some example memory operations
                    ui.heading("Memory Operations");
                    ui.horizontal(|ui| -> Result<()> {
                        if ui.button("Clear Current Page").clicked() {
                            let page_start = self.memory_widget.start_address() & 0xFF00;
                            for i in 0..256 {
                                self.memory.write_byte(page_start + i, 0)?;
                            }
                        }

                        Ok(())
                    });
                },
            }

            Ok(())
        });
    }
}

impl RustNESApp {
    fn new() -> Result<Self> {
        // Create memory with some test values
        let mut memory = Ram::default();

        // Fill memory with some test patterns
        for i in 0..256 {
            memory.write_byte(i, i as u8)?; // 0x00-0xFF in zero page
            memory.write_byte(0x0100 + i, 0xFF - i as u8)?; // Descending pattern in stack page
            memory.write_byte(0x0200 + i, (i * 2) as u8)?; // Multiples of 2 in next page
        }

        // Write some test program at 0x8000 (common starting point for NES ROMs)
        memory.write_byte(0x8000, 0xA9)?; // LDA immediate
        memory.write_byte(0x8001, 0x42)?; // #$42
        memory.write_byte(0x8002, 0x8D)?; // STA absolute
        memory.write_byte(0x8003, 0x00)?; // $0200 (low byte)
        memory.write_byte(0x8004, 0x02)?; // $0200 (high byte)
        memory.write_byte(0x8005, 0xA9)?; // LDA immediate
        memory.write_byte(0x8006, 0xFF)?; // #$FF

        // Create CPU with initialized memory
        let mut cpu = Cpu::new(Box::new(Ram::default())); // This RAM won't be used directly

        // Set some example values
        cpu.a = 0x42; // Accumulator
        cpu.x = 0x10; // X register
        cpu.y = 0x20; // Y register
        cpu.pc = 0x8000; // Program counter
        cpu.sp = 0xFD; // Stack pointer
        cpu.status = 0x24; // Status flags

        Ok(Self {
            cpu,
            memory,
            cpu_widget: CpuWidget::new(),
            memory_widget: MemoryWidget::new()
                .with_start_address(0x8000) // Start at ROM entry point
                .with_rows(16)
                .with_bytes_per_row(16)
                .with_editable(true),
            selected_tab: Tab::Cpu,
        })
    }
}