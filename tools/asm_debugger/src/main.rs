use std::{cell::RefCell, path::PathBuf, rc::Rc};

use clap::Parser;
use eframe::{egui, App, Frame};
use rn_core::{cpu::Cpu, errors::NesError, memory::Addressable, system::NesSystem};
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
    system: Rc<RefCell<NesSystem>>,

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
        // Create the NES system
        let system = Rc::new(RefCell::new(NesSystem::new()));

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
            system,
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
                    let mut system_borrow = self.system.borrow_mut();
                    let _ = self.asm_widget.assemble_code(&mut *system_borrow);

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
            // Show the CPU widget with CPU from the system
            let mut system = self.system.borrow_mut();
            self.cpu_widget.ui(ui, system.cpu_mut());
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

                        // Create a memory pixel adapter using the system's CPU
                        // We need to clone the system reference for the closure
                        let system_ref = self.system.clone();
                        let memory_adapter = MemoryPixelAdapter::new(
                            move |addr| system_ref.borrow().cpu().read_byte(addr),
                            0x0200,
                            0x05FF,
                            32,
                        );

                        // Update zoom and show the memory visualization
                        self.pixel_display.set_zoom(auto_zoom);
                        let _ = self.pixel_display.ui(ui, &memory_adapter);
                    },
                    DisplayMode::Ppu => {
                        // Calculate auto-zoom based on panel width
                        let available_width = ui.available_width();
                        let ppu_width = 256; // Width of the PPU display in pixels
                        let auto_zoom = (available_width / (ppu_width as f32 * 2.0)).max(0.5);

                        // Create a PPU pixel adapter using the system's PPU
                        // We need to clone the system reference for the closure
                        let system_ref = self.system.clone();
                        let ppu_adapter = PpuPixelAdapter::new(move || {
                            let system = system_ref.borrow();
                            // Create a copy of the frame buffer to avoid borrowing issues
                            let frame_buffer = system.ppu().frame_buffer().to_vec();
                            frame_buffer
                        });

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
                // Show the assembly widget with CPU from the system
                let mut system_borrow = self.system.borrow_mut();
                self.asm_widget.ui(ui, &mut *system_borrow);
            });

            ui.add_space(10.0);

            // Disassembly Section (if enabled)
            if self.show_disasm {
                egui::ScrollArea::vertical()
                    .id_salt("disassembly_scroll")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        let system_ref = self.system.borrow();
                        let _ = self.disasm_widget.ui(ui, system_ref.cpu());
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
                        let mut system_borrow = self.system.borrow_mut();
                        let mut adapter = CpuMemoryAdapter::new(system_borrow.cpu_mut());

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
