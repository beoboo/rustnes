use std::{cell::RefCell, path::PathBuf, rc::Rc};

use clap::Parser;
use eframe::{egui, App, Frame};
use rn_core::{
    cpu::Cpu, errors::NesError, memory::{Addressable, Ram}, ppu::{registers::PpuRegisters, Ppu}, system::Bus
};
use rn_ui::widgets::{
    AsmWidget,
    CpuWidget,
    DisasmWidget,
    MemoryPixelAdapter,
    MemoryWidget,
    PixelDisplay,
    PpuPixelAdapter,
};

/// Command line arguments for the AsmDebugger
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optional assembly file to load on startup
    #[arg(value_name = "FILE")]
    asm_file: Option<PathBuf>,
}

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

    fn read_byte(&self, address: u16) -> Result<u8, NesError> {
        self.cpu.read_byte(address)
    }

    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError> {
        self.cpu.write_byte(address, value)
    }
}

/// Display mode enum for the pixel display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisplayMode {
    Memory,
    Ppu,
}

/// Main debugger application
struct AsmDebugger {
    // Command line arguments
    args: Args,

    // Components
    pixel_display: PixelDisplay,
    asm_widget: AsmWidget,
    cpu_widget: CpuWidget,
    disasm_widget: DisasmWidget,
    memory_widget: MemoryWidget,

    // Emulation state
    cpu: Rc<RefCell<Cpu>>,
    ppu: Rc<RefCell<Ppu>>,

    // UI state
    show_pixel_display: bool,
    display_mode: DisplayMode,
    show_cpu: bool,
    show_disasm: bool,
    show_memory_editor: bool,

    // Startup file loading flag
    initial_file_loaded: bool,
}

impl AsmDebugger {
    fn new(_cc: &eframe::CreationContext<'_>, args: Args) -> Self {
        // Create a PPU instance
        let ppu = Rc::new(RefCell::new(Ppu::new()));

        // Create a bus for the CPU
        let mut cpu_bus = Bus::new();

        // Attach PPU registers to the CPU bus
        let ppu_regs = Box::new(PpuRegisters::new(ppu.clone()));
        cpu_bus.attach_component(ppu_regs);

        // Add program memory (ROM) area (0x8000-0xFFFF)
        // For the debugger, we need to be able to load code here
        let program_ram = Box::new(Ram::with_range(0x8000, 0xFFFF));
        cpu_bus.attach_component(program_ram);

        // Create the CPU with its own bus
        let cpu = Rc::new(RefCell::new(Cpu::new(Box::new(cpu_bus))));

        Self {
            args,
            pixel_display: PixelDisplay::new().with_pixel_size(2.0).with_zoom(1.0),
            asm_widget: AsmWidget::new(),
            cpu_widget: CpuWidget::new(),
            disasm_widget: DisasmWidget::new(),
            memory_widget: MemoryWidget::new()
                .with_start_address(0x0000)
                .with_rows(16)
                .with_bytes_per_row(16)
                .with_editable(true),
            cpu,
            ppu,
            show_pixel_display: true,
            display_mode: DisplayMode::Memory, // Default to memory visualization
            show_cpu: true,
            show_disasm: true,
            show_memory_editor: false,
            initial_file_loaded: false,
        }
    }
}

impl App for AsmDebugger {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Load the initial file if specified and not yet loaded
        if !self.initial_file_loaded {
            if let Some(file_path) = &self.args.asm_file {
                if let Ok(file_content) = std::fs::read_to_string(file_path) {
                    // Update the code in the AsmWidget
                    self.asm_widget = AsmWidget::with_code(&file_content);

                    // Assemble and load the code
                    let mut cpu_borrow = self.cpu.borrow_mut();
                    let _ = self.asm_widget.assemble_code(&mut cpu_borrow);

                    println!("Loaded assembly file: {}", file_path.display());

                    // Switch to PPU view by default for test files
                    self.display_mode = DisplayMode::Ppu;
                    self.show_pixel_display = true;
                } else {
                    eprintln!("Error reading file: {}", file_path.display());
                }
            }
            self.initial_file_loaded = true;
        }

        // Tick the PPU a few times to ensure it renders frames properly
        {
            let mut ppu = self.ppu.borrow_mut();
            // Tick PPU for an entire frame to ensure it renders
            for _ in 0..10000 {
                ppu.tick();
            }
        }

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

        // Top menu bar for show/hide controls
        egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_pixel_display, "Show Display");
                    ui.checkbox(&mut self.show_cpu, "Show CPU State");
                    ui.checkbox(&mut self.show_disasm, "Show Disassembly");
                    ui.checkbox(&mut self.show_memory_editor, "Show Memory Editor");
                });

                ui.menu_button("Display Mode", |ui| {
                    ui.radio_value(&mut self.display_mode, DisplayMode::Memory, "Memory");
                    ui.radio_value(&mut self.display_mode, DisplayMode::Ppu, "PPU");
                });
            });
        });

        // Left panel for CPU state
        egui::SidePanel::left("left_panel").show_animated(ctx, self.show_cpu, |ui| {
            // Show the CPU widget
            self.cpu_widget.ui(ui, &mut self.cpu.borrow_mut());
        });

        // Right panel for pixel display
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .show_animated(ctx, self.show_pixel_display, |ui| {
                match self.display_mode {
                    DisplayMode::Memory => {
                        // Calculate auto-zoom based on panel width
                        let available_width = ui.available_width();
                        let memory_width = 32; // Width of the memory display in pixels
                        let auto_zoom = (available_width / (memory_width as f32 * 2.0)).max(1.0);

                        // Create a memory pixel adapter
                        let cpu_ref = self.cpu.clone();
                        let memory_adapter = MemoryPixelAdapter::new(cpu_ref, 0x0200, 0x05FF, 32);

                        // Update zoom and show the memory visualization
                        self.pixel_display.set_zoom(auto_zoom);
                        let _ = self.pixel_display.ui(ui, &memory_adapter);
                    },
                    DisplayMode::Ppu => {
                        // Calculate auto-zoom based on panel width
                        let available_width = ui.available_width();
                        let ppu_width = 256; // Width of the PPU display in pixels
                        let auto_zoom = (available_width / (ppu_width as f32 * 2.0)).max(0.5);

                        // Create a PPU pixel adapter
                        let ppu_adapter = PpuPixelAdapter::new(self.ppu.clone());

                        // Update zoom and show the PPU display
                        self.pixel_display.set_zoom(auto_zoom);
                        let _ = self.pixel_display.ui(ui, &ppu_adapter);
                    },
                }
            });

        // Main central panel with editor and disassembly
        egui::CentralPanel::default().show(ctx, |ui| {
            // Editor Section
            ui.vertical(|ui| {
                // Show the assembly widget directly - it handles its own scrolling internally
                let mut cpu_borrow = self.cpu.borrow_mut();
                self.asm_widget.ui(ui, &mut *cpu_borrow);
            });

            ui.add_space(10.0);

            // Disassembly Section (if enabled)
            if self.show_disasm {
                egui::ScrollArea::vertical()
                    .id_salt("disassembly_scroll")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        let cpu_ref = self.cpu.borrow();
                        let _ = self.disasm_widget.ui(ui, &cpu_ref);
                    });

                ui.add_space(5.0);
            }

            // Memory Editor (if enabled)
            if self.show_memory_editor {
                egui::ScrollArea::vertical()
                    .id_salt("memory_editor_scroll")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        // Create an adapter to access CPU memory with the memory editor
                        let mut cpu_borrow = self.cpu.borrow_mut();
                        let mut adapter = CpuMemoryAdapter::new(&mut *cpu_borrow);

                        // Show the memory editor widget with access to CPU memory
                        self.memory_widget.ui(ui, &mut adapter);
                    });
            }
        });
    }
}

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args = Args::parse();

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
        Box::new(|cc| Ok(Box::new(AsmDebugger::new(cc, args)))),
    )
    .map_err(|e| anyhow::anyhow!("Application error: {}", e))?;

    Ok(())
}
