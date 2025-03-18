use std::{cell::RefCell, path::PathBuf, rc::Rc};

use clap::Parser;
use eframe::{egui, App, Frame};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
#[macro_use]
extern crate log;
use rn_core::{
    cpu::CpuWrapper,
    errors::NesError,
    memory::Addressable,
    system::{NesSystem, SystemState},
};
use rn_ui::widgets::{
    AsmWidget,
    CpuWidget,
    DisasmWidget,
    DmaControllerWidget,
    MemoryPixelAdapter,
    MemoryWidget,
    PatternTableWidget,
    PixelDisplay,
    PpuPixelAdapter,
    PpuWidget,
};

/// Command line arguments for the NesDebugger
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optional assembly file to load on startup
    #[arg(value_name = "FILE")]
    asm_file: Option<PathBuf>,
}

/// Adapter to use CPU's memory with the memory editor
#[derive(Debug)]
struct CpuMemoryAdapter {
    cpu: CpuWrapper,
}

impl CpuMemoryAdapter {
    fn new(cpu: CpuWrapper) -> Self {
        Self { cpu }
    }
}

impl Addressable for CpuMemoryAdapter {
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

impl Default for DisplayMode {
    fn default() -> Self {
        DisplayMode::Memory
    }
}

/// Available tabs for the dock
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DockTab {
    Assembly,
    Disassembly,
    Memory,
    PatternTable,
    Cpu,
    Ppu,
    Dma,
    Display,
    AssembledCode,
}

impl DockTab {
    fn title(&self) -> &'static str {
        match self {
            DockTab::Assembly => "Assembly",
            DockTab::Disassembly => "Disassembly",
            DockTab::Memory => "Memory",
            DockTab::PatternTable => "Pattern Tables",
            DockTab::Cpu => "CPU State",
            DockTab::Ppu => "PPU State",
            DockTab::Dma => "DMA State",
            DockTab::Display => "Display",
            DockTab::AssembledCode => "Assembled Code",
        }
    }
}

/// Shared application state
#[derive(Default)]
struct AppContext {
    display_mode: DisplayMode,
}

/// Main debugger application
struct NesDebugger {
    // Command line arguments
    args: Args,

    // Components
    pixel_display: PixelDisplay,
    asm_widget: AsmWidget,
    cpu_widget: CpuWidget,
    ppu_widget: PpuWidget,
    dma_widget: DmaControllerWidget,
    disasm_widget: DisasmWidget,
    memory_widget: MemoryWidget,
    pattern_table_widget: PatternTableWidget,

    // Emulation state
    system: Rc<RefCell<NesSystem>>,

    // Dock state
    dock_state: DockState<DockTab>,

    // Shared context
    context: AppContext,

    // Startup file loading flag
    initial_file_loaded: bool,
}

/// Tab viewer for the dock area
struct NesTabViewer<'a> {
    pixel_display: &'a mut PixelDisplay,
    asm_widget: &'a mut AsmWidget,
    cpu_widget: &'a mut CpuWidget,
    ppu_widget: &'a mut PpuWidget,
    dma_widget: &'a mut DmaControllerWidget,
    disasm_widget: &'a mut DisasmWidget,
    memory_widget: &'a mut MemoryWidget,
    pattern_table_widget: &'a mut PatternTableWidget,
    system: Rc<RefCell<NesSystem>>,
    context: &'a mut AppContext,
}

impl<'a> TabViewer for NesTabViewer<'a> {
    type Tab = DockTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            DockTab::Assembly => {
                // Assembly Tab content - just show the editor
                let mut system_borrow = self.system.borrow_mut();
                self.asm_widget.ui(ui, &mut *system_borrow);
            },
            DockTab::AssembledCode => {
                // Only show content if code is loaded
                if !self.asm_widget.is_loaded() {
                    ui.centered_and_justified(|ui| {
                        ui.label("No code assembled yet");
                    });
                    return;
                }

                // Show assembly information header
                ui.horizontal(|ui| {
                    ui.label(format!("Load Address: ${:04X}", self.asm_widget.load_address()));
                    ui.label(format!("Size: {} bytes", self.asm_widget.assembled_bytes().len()));
                });

                ui.add_space(8.0);

                // Display bytes in a formatted table
                let bytes = self.asm_widget.assembled_bytes();
                let load_addr = self.asm_widget.load_address();
                let mut addr = load_addr;
                let bytes_per_row = 16;

                egui::ScrollArea::vertical()
                    .id_salt("assembled_code_scroll")
                    .show(ui, |ui| {
                        let text_style = egui::TextStyle::Monospace;
                        let _row_height = ui.text_style_height(&text_style) + 4.0;

                        for chunk in bytes.chunks(bytes_per_row) {
                            ui.horizontal(|ui| {
                                // Show address
                                ui.label(format!("${:04X}:", addr));

                                // Show hex bytes
                                for byte in chunk {
                                    ui.label(format!("{:02X}", byte));
                                }
                            });
                            addr += chunk.len() as u16;
                        }
                    });
            },
            DockTab::Disassembly => {
                // Disassembly Tab content
                egui::ScrollArea::vertical()
                    .id_salt("disassembly_scroll")
                    .show(ui, |ui| {
                        let system_ref = self.system.borrow();
                        let _ = self.disasm_widget.ui(ui, system_ref.cpu());
                    });
            },
            DockTab::Memory => {
                // Memory Tab content

                // Use a ScrollArea with both horizontal and vertical scrolling
                egui::ScrollArea::both().id_salt("memory_editor_scroll").show(ui, |ui| {
                    // Use a fixed width for the content to ensure horizontal scrolling works
                    let available_width = ui.available_width();
                    let min_content_width: f32 = 800.0; // This should be enough for the memory widget

                    ui.allocate_ui(
                        egui::vec2(min_content_width.max(available_width), ui.available_height()),
                        |ui| {
                            // Create an adapter to access CPU memory with the memory editor
                            let system_borrow = self.system.borrow_mut();
                            let mut adapter = CpuMemoryAdapter::new(system_borrow.cpu());

                            // Show the memory editor widget with access to CPU memory
                            self.memory_widget.ui(ui, &mut adapter);
                        },
                    );
                });
            },
            DockTab::PatternTable => {
                egui::ScrollArea::vertical()
                    .id_salt("pattern_table_scroll")
                    .show(ui, |ui| {
                        let system_borrow = self.system.borrow();
                        // Get cartridge reference from the system and convert to the expected format
                        if let Some(cart_rc) = system_borrow.ppu().cartridge() {
                            // Pass a reference to the cloned Rc
                            let _ = self.pattern_table_widget.ui(ui, Some(&cart_rc));
                        } else {
                            // No cartridge
                            let _ = self.pattern_table_widget.ui(ui, None);
                        }
                    });
            },
            DockTab::Cpu => {
                // CPU Tab content
                let system = self.system.borrow_mut();
                self.cpu_widget.ui(ui, system.cpu());
            },
            DockTab::Ppu => {
                // PPU Tab content
                
                // Debug controls in a horizontal layout
                let mut needs_refresh = false;
                
                ui.horizontal(|ui| {
                    // Run button
                    if ui.button("Run 1000 cycles").clicked() {
                        let mut system = self.system.borrow_mut();
                        match system.run(1000) {
                            Ok(steps) => {
                                ui.label(format!("Ran for {} steps", steps));
                            },
                            Err(e) => {
                                ui.label(format!("Error: {}", e));
                            }
                        }
                        needs_refresh = true;
                    }
                    
                    // Direct MASK register write button
                    if ui.button("Set MASK=0x18").clicked() {
                        let system = self.system.borrow_mut();
                        system.ppu().write_register(0x2001, 0x18); // Enable sprites and background
                        log::info!("Direct write to MASK register: 0x18");
                        needs_refresh = true;
                    }

                    // Direct CTRL register write button
                    if ui.button("Set CTRL=0x80").clicked() {
                        let system = self.system.borrow_mut();
                        system.ppu().write_register(0x2000, 0x80); // Enable NMI
                        log::info!("Direct write to CTRL register: 0x80");
                        needs_refresh = true;
                    }
                });
                
                // After all potential modifications, render the widget once
                let system = self.system.borrow_mut();
                self.ppu_widget.ui(ui, system.ppu());
            },
            DockTab::Dma => {
                // DMA Tab content
                let system = self.system.borrow();
                self.dma_widget.ui(ui, &system.dma());
            },
            DockTab::Display => {
                // Display Tab content

                // Display mode selector
                ui.horizontal(|ui| {
                    ui.label("Display Mode:");
                    ui.radio_value(&mut self.context.display_mode, DisplayMode::Memory, "Memory");
                    ui.radio_value(&mut self.context.display_mode, DisplayMode::Ppu, "PPU");
                });

                ui.add_space(8.0);

                // Display content based on mode
                match self.context.display_mode {
                    DisplayMode::Memory => {
                        // Calculate auto-zoom based on panel width
                        let available_width = ui.available_width();
                        let memory_width = 32; // Width of the memory display in pixels
                        let auto_zoom = (available_width / (memory_width as f32 * 2.0)).max(1.0);

                        // Create a memory pixel adapter using the system's CPU
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
            },
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        // Don't allow closing tabs in this basic implementation
        false
    }
}

impl NesDebugger {
    fn new(_cc: &eframe::CreationContext<'_>, args: Args) -> Self {
        // Create the NES system
        let system = Rc::new(RefCell::new(NesSystem::new()));

        // Create initial dock state with all our tabs
        let mut dock_state = DockState::new(vec![DockTab::Assembly, DockTab::Memory, DockTab::PatternTable]);

        // Create layout with Assembly/Memory/PatternTable in center, and CPU/PPU on the left
        let [center, left] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.2, // 20% width for CPU
            vec![DockTab::Cpu],
        );
        
        // Split the left panel vertically to hold CPU, PPU, and DMA
        dock_state.main_surface_mut().split_below(
            left,
            0.4, // CPU takes 40% of height
            vec![DockTab::Ppu, DockTab::Dma],
        );

        // Add Display on the right side of the center area
        let [center_main, _right] = dock_state.main_surface_mut().split_right(
            center, // Split the center node, not the root
            0.7,    // Central area takes 70% of remaining width
            vec![DockTab::Display],
        );

        // Create a bottom area for Disassembly and Assembled Code
        dock_state.main_surface_mut().split_below(
            center_main, // Split the center_main node, not the root
            0.7,         // Top takes 70% of height
            vec![DockTab::Disassembly, DockTab::AssembledCode],
        );

        Self {
            args,
            pixel_display: PixelDisplay::new().with_pixel_size(2.0).with_zoom(1.0),
            asm_widget: AsmWidget::new(),
            cpu_widget: CpuWidget::new(),
            ppu_widget: PpuWidget::new(),
            dma_widget: DmaControllerWidget::new(),
            disasm_widget: DisasmWidget::new(),
            memory_widget: MemoryWidget::new()
                .with_start_address(0x0000)
                .with_rows(16)
                .with_bytes_per_row(16)
                .with_editable(true),
            pattern_table_widget: PatternTableWidget::new(),
            system,
            dock_state,
            context: AppContext {
                display_mode: DisplayMode::Memory,
            },
            initial_file_loaded: false,
        }
    }
}

impl App for NesDebugger {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Load the initial file if specified and not yet loaded
        if !self.initial_file_loaded {
            if let Some(file_path) = &self.args.asm_file {
                debug!("Attempting to load assembly file: {}", file_path.display());
                match std::fs::read_to_string(file_path) {
                    Ok(file_content) => {
                        debug!("Successfully read file contents, creating assembly widget");
                        self.asm_widget = AsmWidget::with_code(&file_content);

                        // Assemble and load the code
                        let mut system_borrow = self.system.borrow_mut();
                        match self.asm_widget.assemble_code(&mut *system_borrow) {
                            Ok(_) => {
                                info!("Successfully assembled and loaded code from: {}", file_path.display());
                                // Switch to PPU view by default for test files
                                self.context.display_mode = DisplayMode::Ppu;
                            },
                            Err(err) => {
                                error!("Failed to assemble code from {}: {}", file_path.display(), err);
                            },
                        }
                    },
                    Err(err) => {
                        error!("Error reading file {}: {}", file_path.display(), err);
                    },
                }
                self.initial_file_loaded = true;
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

        // Run cycles if continuous mode is enabled in the AsmWidget
        if self.asm_widget.is_continuous_run() {
            let mut system = self.system.borrow_mut();
            
            // Run a batch of cycles via the AsmWidget
            if self.asm_widget.run_continuous(&mut system) {
                // If still running, request a redraw for the next frame
                ctx.request_repaint();
            }
        }

        // Top menu bar for show/hide controls
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open...").clicked() {
                        // File open code would go here - can be added later
                        info!("File Open clicked - functionality not yet implemented");
                        ui.close_menu();
                    }

                    if ui.button("Save As...").clicked() {
                        // File save code would go here - can be added later
                        info!("File Save As clicked - functionality not yet implemented");
                        ui.close_menu();
                    }

                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("System", |ui| {

                    if ui.button("Write Test Pattern").clicked() {
                        info!("Writing test pattern to PPU frame buffer");
                        let mut system_borrow = self.system.borrow_mut();
                        system_borrow.write_ppu_test_pattern();
                        // Also switch to PPU display mode to see it
                        self.context.display_mode = DisplayMode::Ppu;
                        ui.close_menu();
                    }

                    if ui.button("Write Test Sprite").clicked() {
                        info!("Writing test sprite to PPU OAM and rendering");
                        let mut system_borrow = self.system.borrow_mut();
                        system_borrow.write_ppu_test_sprite();
                        // Also switch to PPU display mode to see it
                        self.context.display_mode = DisplayMode::Ppu;
                        ui.close_menu();
                    }
                });

                // Add system state indicator
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let system = self.system.borrow();
                    match system.state() {
                        SystemState::Ready => ui.colored_label(egui::Color32::WHITE, "Ready"),
                        SystemState::Loaded => ui.colored_label(egui::Color32::CYAN, "Loaded"),
                        SystemState::Running => ui.colored_label(egui::Color32::YELLOW, "Running"),
                        SystemState::Finished => ui.colored_label(egui::Color32::GREEN, "Finished"),
                        SystemState::Error(pc) => ui.colored_label(egui::Color32::RED, format!("Error at ${:04X}", pc)),
                    };
                    ui.label("System: ");
                });
            });
        });

        // Main central panel with dock area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create a tab viewer with references to all components
            let mut tab_viewer = NesTabViewer {
                pixel_display: &mut self.pixel_display,
                asm_widget: &mut self.asm_widget,
                cpu_widget: &mut self.cpu_widget,
                ppu_widget: &mut self.ppu_widget,
                dma_widget: &mut self.dma_widget,
                disasm_widget: &mut self.disasm_widget,
                memory_widget: &mut self.memory_widget,
                pattern_table_widget: &mut self.pattern_table_widget,
                system: self.system.clone(),
                context: &mut self.context,
            };

            // Render the dock area
            DockArea::new(&mut self.dock_state)
                .style(Style::from_egui(ui.style().as_ref()))
                .show(ctx, &mut tab_viewer);
        });
    }
}

fn main() -> anyhow::Result<()> {
    // Initialize logging with tracing-subscriber
    // Set default log level to info, but allow override via RUST_LOG environment variable
    use tracing_subscriber::{fmt, EnvFilter};
    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    info!("Starting RustNES Debugger");

    // Parse command line arguments
    let args = Args::parse();

    if let Some(path) = &args.asm_file {
        info!("Assembly file specified: {}", path.display());
    }

    // Set up the native options with a maximized window
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0]) // Default size if maximizing fails
            .with_min_inner_size([800.0, 600.0])
            .with_maximized(true), // Start maximized but not fullscreen
        ..Default::default()
    };

    // Run the app
    info!("Launching UI");
    eframe::run_native(
        "RustNES Debugger",
        options,
        Box::new(|cc| {
            info!("Initializing application");
            Ok(Box::new(NesDebugger::new(cc, args)))
        }),
    )
    .map_err(|e| {
        error!("Application error: {}", e);
        anyhow::anyhow!("Application error: {}", e)
    })?;

    info!("Application closed normally");
    Ok(())
}
