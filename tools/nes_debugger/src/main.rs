mod frame_dump;

use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

use clap::Parser;
use eframe::{egui, App, Frame};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
#[macro_use]
extern crate log;
use rn_audio::{AudioControls, ChannelBuilder, CpalAudioBuilder, CpalAudioConsumer, Multiplexer};
use rn_core::{
    cartridge::load_rom,
    cpu::CpuWrapper,
    errors::NesError,
    memory::Addressable,
    system::{NesSystem, SystemState},
};
use rn_core::input::ControllerButton;
use rn_input::{controller_profile::ControllerProfile, key_mapping::KeyMappingManager};
use rn_ui::widgets::{
    convert_egui_key,
    AsmWidget,
    AudioStats,
    AudioWidget,
    ControllerWidget,
    CpuWidget,
    DisasmWidget,
    DmaControllerWidget,
    KeyboardMappingsWidget,
    MemoryPixelAdapter,
    MemoryWidget,
    NametableMapAdapter,
    PatternTableWidget,
    PixelDisplay,
    PpuPixelAdapter,
    PpuWidget,
    WaveformWidget,
};
use anyhow::{Context, Result};

/// One NTSC frame: the NES runs at 60.0988 Hz, not exactly 60.
const NTSC_FRAME_PERIOD: std::time::Duration = std::time::Duration::from_nanos(16_639_267);

/// The same rate as a number, for deciding whether the display has a cadence worth following.
const NES_FRAME_RATE: f32 = 60.0988;

/// How many repaints one emulated frame should occupy, or zero to pace by the wall clock instead.
///
/// Locking emulation to the display's cadence removes the beat between two nearly-equal rates, but
/// only when there genuinely is a cadence. The tolerance has to be tight: it was once 0.15, which
/// accepted anything from 51 to 69 repaints a second as "one frame per repaint", so a machine
/// managing only 55 had its emulation locked to 55 — running the game nine percent slow and
/// starving the sound card by the same fraction, heard as the audio dragging. Worse, the effect
/// deepened the more the display struggled, which is the opposite of what a fallback should do.
///
/// Locking is refused outright when the display cannot manage a repaint per frame. There is no
/// cadence to follow below that, and the wall clock can make the deficit up where this cannot.
fn cadence_lock(repaint_fps: f32) -> u32 {
    let ratio = repaint_fps / NES_FRAME_RATE;
    let rounded = ratio.round();

    let is_a_multiple = (1.0..=4.0).contains(&rounded) && (ratio - rounded).abs() < 0.02;
    let can_keep_up = repaint_fps >= NES_FRAME_RATE - 1.0;

    if is_a_multiple && can_keep_up {
        rounded as u32
    } else {
        0
    }
}

/// Most frames to run in one repaint while catching up.
const MAX_CATCH_UP_FRAMES: u32 = 2;

/// Command line arguments for the NesDebugger
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to load on startup: either an iNES ROM (.nes) or 6502 assembly.
    ///
    /// Detected by content, not extension.
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Start running as soon as the file is loaded, instead of waiting for Run to be pressed.
    #[arg(long, short = 'p')]
    play: bool,
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
#[derive(Default)]
enum DisplayMode {
    #[default]
    Memory,
    Ppu,
    /// All four nametables at once, with the viewport outlined.
    Nametables,
}


/// Available tabs for the dock
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DockTab {
    AssembledCode,
    Assembly,
    Audio,
    Controller,
    Cpu,
    Disassembly,
    Display,
    Dma,
    Memory,
    PatternTable,
    Ppu,
    WaveformVisualizer,
}

impl DockTab {
    fn title(&self) -> &'static str {
        match self {
            DockTab::AssembledCode => "Assembled Code",
            DockTab::Assembly => "Assembly",
            DockTab::Audio => "Audio Controls",
            DockTab::Controller => "Controller State",
            DockTab::Cpu => "CPU State",
            DockTab::Disassembly => "Disassembly",
            DockTab::Display => "Display",
            DockTab::Dma => "DMA State",
            DockTab::Memory => "Memory",
            DockTab::PatternTable => "Pattern Tables",
            DockTab::Ppu => "PPU State",
            DockTab::WaveformVisualizer => "Audio Waveform",
        }
    }
}

/// Shared application state
#[derive(Default)]
struct AppContext {
    display_mode: DisplayMode,
    /// Whether to hide the scanlines a television kept behind the bezel. On by default, because
    /// that margin is where games park what nobody was meant to see — including the attribute
    /// garbage an out-of-range vertical scroll produces.
    overscan: bool,
}

/// Main debugger application
struct NesDebugger {
    // Command line arguments
    args: Args,

    // Components
    asm_widget: AsmWidget,
    cpu_widget: CpuWidget,
    disasm_widget: DisasmWidget,
    dma_widget: DmaControllerWidget,
    controller_widget: ControllerWidget,
    keyboard_mappings_widget: KeyboardMappingsWidget,
    memory_widget: MemoryWidget,
    pattern_table_widget: PatternTableWidget,
    pixel_display: PixelDisplay,
    ppu_widget: PpuWidget,
    audio_widget: AudioWidget,
    waveform_visualizer: WaveformWidget,
    audio_output: CpalAudioConsumer,
    /// Buffer fill level and underrun/drop counts for the running audio stream.
    audio_controls: AudioControls,
    /// Whether the audio stream is running; emulation paces itself against it when it is.
    audio_running: bool,

    /// Emulated and repaint rates, measured over the last second.
    ///
    /// If emulation reads ~60 and the picture still stutters, the problem is in presentation
    /// rather than pacing — which is a different place to look, and worth knowing before guessing.
    fps_window_start: std::time::Instant,
    frames_in_window: u32,
    repaints_in_window: u32,
    emulated_fps: f32,
    repaint_fps: f32,

    /// Repaints per emulated frame, when the display's rate is a clean multiple of 60.
    ///
    /// Zero means no lock has been established and the wall clock is used instead.
    /// Where snapshots go, derived from the loaded file so each game has its own.
    save_state_path: Option<PathBuf>,
    /// Whether the window is filling the screen. Held here rather than asked of the windowing
    /// system, which reports it only after the change has taken effect.
    fullscreen: bool,
    /// Result of the most recent frame dump, shown beside the button.
    last_dump: Option<String>,
    repaints_per_frame: u32,
    /// Repaints since the last emulated frame, for the locked cadence.
    repaints_since_frame: u32,

    /// When the next emulated frame is due.
    ///
    /// Emulation is paced by the wall clock rather than by repaints. A ProMotion display repaints
    /// at 120 Hz, and running a frame per repaint made the emulator sprint at double speed until
    /// the audio buffer filled, then stall — which is seen as animation jumping and freezing.
    next_frame_at: std::time::Instant,

    // Emulation state
    system: Rc<RefCell<NesSystem>>,

    // Input handling
    key_mapping_manager: KeyMappingManager,

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
    controller_widget: &'a mut ControllerWidget,
    disasm_widget: &'a mut DisasmWidget,
    memory_widget: &'a mut MemoryWidget,
    pattern_table_widget: &'a mut PatternTableWidget,
    audio_widget: &'a mut AudioWidget,
    audio_stats: AudioStats,
    /// The active controller mapping, so the Controller tab can show what is bound.
    controller_profile: &'a ControllerProfile,
    waveform_visualizer: &'a mut WaveformWidget,
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
                let mut system_borrow = self.system.borrow_mut();
                self.asm_widget.ui(ui, &mut system_borrow);
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
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
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
                let system_ref = self.system.borrow();
                let _ = self.disasm_widget.ui(ui, system_ref.cpu());
            },
            DockTab::Memory => {
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
                            },
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
            DockTab::Controller => {
                // Controller Tab content
                let system = self.system.borrow();
                self.controller_widget.ui(ui, &system.controller_handler());

                // Show what is actually bound. Without this the only way to find out is to press
                // keys until something happens.
                ui.separator();
                {
                    let profile = self.controller_profile;
                    ui.label(format!("Key mapping: {}", profile.name()));
                    ui.add_space(2.0);

                    egui::Grid::new("controller_key_mapping").striped(true).show(ui, |ui| {
                        for button in ControllerButton::ALL {
                            let keys = profile.keys_for(button);
                            ui.label(format!("{button:?}"));
                            ui.label(if keys.is_empty() {
                                "— not bound —".to_string()
                            } else {
                                keys.iter().map(|key: &rn_input::key_mapping::KeyCode| key.to_str()).collect::<Vec<_>>().join("  or  ")
                            });
                            ui.end_row();
                        }
                    });
                }
            },
            DockTab::Display => {
                // Display Tab content

                // Display mode selector
                ui.horizontal(|ui| {
                    ui.label("Display Mode:");
                    ui.radio_value(&mut self.context.display_mode, DisplayMode::Memory, "Memory");
                    ui.radio_value(&mut self.context.display_mode, DisplayMode::Ppu, "PPU");
                    ui.radio_value(&mut self.context.display_mode, DisplayMode::Nametables, "Nametables");
                });

                ui.add_space(8.0);

                // Display content based on mode
                match self.context.display_mode {
                    DisplayMode::Memory => {
                        // Width only: the memory view scrolls vertically.
                        let auto_zoom = self.pixel_display.fit_zoom_width(ui, 32);

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
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.context.overscan, "Overscan");
                            ui.label("hides the 8 scanlines a television kept behind the bezel");
                        });

                        let overscan = if self.context.overscan { 8 } else { 0 };
                        let auto_zoom = self.pixel_display.fit_zoom(ui, 256, 240 - overscan * 2);

                        // Create a PPU pixel adapter using the system's PPU
                        let system_ref = self.system.clone();
                        let ppu_adapter = PpuPixelAdapter::new(move || {
                            let system = system_ref.borrow();
                            // Create a copy of the frame buffer to avoid borrowing issues
                            
                            system.ppu().frame_buffer().to_vec()
                        });

                        // Update zoom and show the PPU display
                        self.pixel_display.set_zoom(auto_zoom);
                        let _ = self.pixel_display.ui(ui, &ppu_adapter.with_overscan(overscan));
                    },
                    DisplayMode::Nametables => {
                        let system = self.system.borrow();
                        let ppu = system.ppu();

                        // State first, so it is readable even before finding it in the picture.
                        let (viewport_x, viewport_y) = ppu.viewport_origin();
                        let active = ppu.active_nametable();

                        ui.horizontal(|ui| {
                            ui.label(format!("Mirroring: {:?}", ppu.mirroring()));
                            ui.separator();
                            ui.label(format!("Viewport: ({viewport_x}, {viewport_y})"));
                            ui.separator();
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 96, 96),
                                format!("Showing nametable {active}"),
                            );
                        });

                        // Which of the four is aliased onto which, so a screen appearing twice is
                        // explained rather than surprising.
                        ui.label(match ppu.mirroring() {
                            rn_core::ppu::Mirroring::Horizontal => {
                                "Horizontal mirroring: 0/1 share memory, 2/3 share memory"
                            },
                            rn_core::ppu::Mirroring::Vertical => {
                                "Vertical mirroring: 0/2 share memory, 1/3 share memory"
                            },
                            rn_core::ppu::Mirroring::SingleScreenLower => {
                                "Single-screen (lower): all four show the same table"
                            },
                            rn_core::ppu::Mirroring::SingleScreenUpper => {
                                "Single-screen (upper): all four show the same table"
                            },
                        });
                        ui.add_space(4.0);

                        let auto_zoom = self.pixel_display.fit_zoom(ui, 512, 480);

                        let system_ref = self.system.clone();
                        let map_adapter = NametableMapAdapter::new(move || {
                            let system = system_ref.borrow();
                            system.ppu().render_nametable_map()
                        });

                        drop(system);
                        self.pixel_display.set_zoom(auto_zoom);
                        let _ = self.pixel_display.ui(ui, &map_adapter);
                    },
                }
            },
            DockTab::Audio => {
                // Create a mutable system reference for the audio widget
                let system = self.system.borrow_mut();

                // Use the audio widget
                self.audio_widget.ui(ui, system.apu(), self.audio_stats);
            },
            DockTab::WaveformVisualizer => {
                // Waveform Visualizer Tab content
                self.waveform_visualizer.ui(ui);
            },
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        // Don't allow closing tabs in this basic implementation
        false
    }
}

impl NesDebugger {
    fn new(_cc: &eframe::CreationContext<'_>, args: Args) -> Result<Self> {
        // Create the NES system
        let system = Rc::new(RefCell::new(NesSystem::new()));

        let (audio_producer, audio_consumer) = CpalAudioBuilder::build_default()?;

        // The APU must resample to whatever the device actually asked for, not to an assumed rate.
        let sample_rate = audio_producer.sample_rate() as f64;

        // Buffer telemetry, kept after the producer is handed to the APU: emulation paces itself
        // against this, and the audio widget displays it.
        let audio_controls = audio_producer.controls();

        // The visualiser taps the same stream the speakers get, through a bounded channel that
        // drops rather than blocks — a stalled UI must never stall audio.
        let (waveform_producer, waveform_consumer) = ChannelBuilder::<f32>::build(8192);

        // One stream, two destinations. The multiplexer is a SampleProducer itself, so it needs no
        // thread and nothing has to remember to pump it.
        let audio_fanout = Multiplexer::new()
            .with_producer(Box::new(audio_producer))
            .with_producer(Box::new(waveform_producer));

        // Create audio widget
        let audio_widget = AudioWidget::new();

        // Create and connect waveform visualizer widget
        let waveform = WaveformWidget::new(Box::new(waveform_consumer));

        // Connect the audio output to the system
        system
            .borrow_mut()
            .connect_audio_output(Box::new(audio_fanout), sample_rate);

        // Create input manager with default and WASD profiles
        let mut key_mapping_manager = KeyMappingManager::new();
        key_mapping_manager.add_profile(ControllerProfile::create_default_profile("Default"));
        key_mapping_manager.add_profile(ControllerProfile::create_wasd_profile());
        key_mapping_manager.add_profile(ControllerProfile::create_combined_profile());

        // Accept both layouts by default. Activating WASD alone meant the arrow keys silently did
        // nothing, which reads as broken input rather than as a profile choice.
        if let Err(error) = key_mapping_manager.set_controller1_profile("Arrows + WASD") {
            warn!("Could not select the default controller profile: {error}");
        }

        // Create initial dock state with all our tabs
        let mut dock_state = DockState::new(vec![
            DockTab::Assembly,
            DockTab::Memory,
            DockTab::PatternTable,
            DockTab::Audio,
        ]);

        // Create layout with Assembly/Memory/PatternTable in center, and CPU/PPU on the left
        let [center, left] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.2, // 20% width for CPU
            vec![DockTab::Cpu],
        );

        // Split the left panel vertically to hold CPU, PPU, DMA, and Controller
        dock_state.main_surface_mut().split_below(
            left,
            0.4, // CPU takes 40% of height
            vec![DockTab::Ppu, DockTab::Dma, DockTab::Controller],
        );

        // Add Display on the right side of the center area
        let [center_main, right] = dock_state.main_surface_mut().split_right(
            center, // Split the center node, not the root
            0.7,    // Central area takes 70% of remaining width
            vec![DockTab::Display],
        );

        // Split right area to add WaveformVisualizer below Display
        dock_state.main_surface_mut().split_below(
            right,
            0.6, // Display takes 60% of height
            vec![DockTab::WaveformVisualizer],
        );

        // Create a bottom area for Disassembly and Assembled Code
        dock_state.main_surface_mut().split_below(
            center_main, // Split the center_main node, not the root
            0.7,         // Top takes 70% of height
            vec![DockTab::Disassembly, DockTab::AssembledCode],
        );

        // Create an instance with all components
        Ok(Self {
            args,
            asm_widget: AsmWidget::new(),
            audio_widget,
            cpu_widget: CpuWidget::new(),
            ppu_widget: PpuWidget::new(),
            dma_widget: DmaControllerWidget::new(),
            controller_widget: ControllerWidget::new(),
            disasm_widget: DisasmWidget::new(),
            keyboard_mappings_widget: KeyboardMappingsWidget::new(),
            memory_widget: MemoryWidget::new()
                .with_start_address(0x0000)
                .with_rows(16)
                .with_bytes_per_row(16)
                .with_editable(true),
            pattern_table_widget: PatternTableWidget::new(),
            pixel_display: PixelDisplay::new().with_pixel_size(2.0).with_zoom(1.0),
            audio_output: audio_consumer,
            audio_controls,
            audio_running: false,
            next_frame_at: std::time::Instant::now(),
            save_state_path: None,
            fullscreen: false,
            last_dump: None,
            repaints_per_frame: 0,
            repaints_since_frame: 0,
            fps_window_start: std::time::Instant::now(),
            frames_in_window: 0,
            repaints_in_window: 0,
            emulated_fps: 0.0,
            repaint_fps: 0.0,
            waveform_visualizer: waveform,
            system,
            key_mapping_manager,
            dock_state,
            context: AppContext {
                display_mode: DisplayMode::Memory,
                overscan: true,
            },
            initial_file_loaded: false,
        })
    }
}

impl NesDebugger {
    /// Write the machine's state beside the file it was loaded from.
    fn save_state(&mut self) {
        let Some(path) = self.save_state_path.clone() else {
            self.last_dump = Some("nothing loaded to save".into());
            return;
        };

        let result = serde_json::to_string(&self.system.borrow().save_state())
            .map_err(|e| e.to_string())
            .and_then(|encoded| std::fs::write(&path, encoded).map_err(|e| e.to_string()));

        // Reported to the terminal as well as to the panel, because the panel is not on screen in
        // fullscreen — which is exactly when someone is playing and wants to save.
        match &result {
            Ok(()) => info!("saved state to {}", path.display()),
            Err(error) => error!("saving state: {error}"),
        }

        self.last_dump = Some(match result {
            Ok(()) => format!("saved state to {}", path.display()),
            Err(error) => format!("saving state: {error}"),
        });
    }

    /// Restore the machine from the snapshot beside the loaded file.
    fn load_state(&mut self) {
        let Some(path) = self.save_state_path.clone() else {
            self.last_dump = Some("nothing loaded to restore into".into());
            return;
        };

        let result = std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|text| serde_json::from_str(&text).map_err(|e| e.to_string()))
            .and_then(|state: rn_core::system::SaveState| {
                self.system.borrow_mut().load_state(&state).map_err(|e| e.to_string())
            });

        match &result {
            Ok(()) => info!("restored state from {}", path.display()),
            Err(error) => error!("restoring state: {error}"),
        }

        self.last_dump = Some(match result {
            Ok(()) => format!("restored state from {}", path.display()),
            Err(error) => format!("restoring state: {error}"),
        });
    }

    /// Load a `.nes` ROM or 6502 assembly, from the command line or the File menu.
    ///
    /// Detected by content rather than by extension: an iNES image starts with the four bytes
    /// `NES\x1A`. Reading a ROM as text fails with "stream did not contain valid UTF-8", which
    /// says nothing useful about what the user actually passed.
    fn load_file(&mut self, path: &Path) -> Result<()> {
        let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;

        // One snapshot slot per file, beside it, so loading a different game cannot restore the
        // wrong machine into it.
        self.save_state_path = Some(path.with_extension("state.json"));

        if bytes.starts_with(b"NES\x1A") {
            info!("Loading iNES ROM: {}", path.display());
            let rom = load_rom(path).map_err(|e| anyhow::anyhow!("{e}"))?;

            self.system
                .borrow_mut()
                .load_rom(&rom)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            // No mapper warning here: `load_rom` refuses a ROM whose mapper is not implemented,
            // with a message naming what is supported. Warning separately meant maintaining a
            // second, independent idea of what works — which is exactly how it came to claim that
            // MMC3 was unsupported long after it was implemented.

            // A ROM has graphics to show, so open on the PPU view.
            self.context.display_mode = DisplayMode::Ppu;
            return Ok(());
        }

        let source = String::from_utf8(bytes)
            .with_context(|| format!("{} is neither an iNES ROM nor valid UTF-8 assembly", path.display()))?;

        info!("Loading assembly: {}", path.display());
        self.asm_widget = AsmWidget::with_code(&source);

        let mut system = self.system.borrow_mut();
        self.asm_widget
            .assemble_code(&mut system)
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        self.context.display_mode = DisplayMode::Ppu;
        Ok(())
    }

    /// Start or stop the audio stream to match whether the emulator is running.
    ///
    /// Idempotent, so it is safe to call every frame: it only acts on a transition.
    fn sync_audio_to_run_state(&mut self) {
        let running = self.asm_widget.is_continuous_run();
        if running == self.audio_running {
            return;
        }

        if running {
            // Reset telemetry so the counters describe this run, not the last one.
            self.audio_controls.reset_stats();
            if let Err(error) = self.audio_output.play() {
                warn!("Could not start the audio stream: {error}");
                return;
            }
        } else if let Err(error) = self.audio_output.pause() {
            warn!("Could not pause the audio stream: {error}");
            return;
        }

        self.audio_running = running;
    }

    /// Current audio pipeline health, for display in the audio widget.
    fn audio_stats(&self) -> AudioStats {
        AudioStats {
            running: self.audio_running,
            sample_rate: self.audio_output.sample_rate(),
            queued: self.audio_controls.queued(),
            capacity: self.audio_controls.capacity(),
            fill_level: self.audio_controls.fill_level(),
            underruns: self.audio_controls.underruns(),
            dropped: self.audio_controls.dropped(),
        }
    }

    /// Update the emulated and repaint rates once a second.
    fn measure_rates(&mut self) {
        self.repaints_in_window += 1;

        let elapsed = self.fps_window_start.elapsed();
        if elapsed < std::time::Duration::from_secs(1) {
            return;
        }

        let seconds = elapsed.as_secs_f32();
        self.emulated_fps = self.frames_in_window as f32 / seconds;
        self.repaint_fps = self.repaints_in_window as f32 / seconds;

        // If the display refreshes at a clean multiple of the NES's rate, lock emulation to it.
        //
        // Pacing by the wall clock produces the right *average* rate, but the display refreshes on
        // its own schedule: at 120 Hz a frame should occupy exactly two refreshes, and because
        // 60.0988 is not exactly half of 120 it occasionally occupies one or three instead. That
        // beat is visible as periodic judder even though the average is correct. Counting refreshes
        // instead makes every frame occupy the same number of them.
        self.repaints_per_frame = cadence_lock(self.repaint_fps);

        self.fps_window_start = std::time::Instant::now();
        self.frames_in_window = 0;
        self.repaints_in_window = 0;
    }

    /// How many emulated frames are due, by the wall clock.
    ///
    /// The NES runs at 60.0988 Hz, which is unrelated to how often the UI repaints — a ProMotion
    /// display does so at 120 Hz. Running a frame per repaint therefore ran the emulator at double
    /// speed until the audio buffer filled and the gate closed, then stalled until it drained:
    /// visible as animation jumping ahead and freezing. Deciding by elapsed time instead makes the
    /// rate independent of the display.
    fn frames_due(&mut self) -> u32 {
        // Prefer the display's cadence when it has one, so each frame occupies the same number of
        // refreshes and there is no beat against the wall clock.
        if self.repaints_per_frame > 0 {
            self.repaints_since_frame += 1;
            if self.repaints_since_frame < self.repaints_per_frame {
                return 0;
            }
            self.repaints_since_frame = 0;

            // Keep the wall clock aligned, so falling back mid-run does not produce a burst.
            self.next_frame_at = std::time::Instant::now() + NTSC_FRAME_PERIOD;

            if self.audio_ahead() {
                return 0;
            }
            return 1;
        }

        let now = std::time::Instant::now();
        if now < self.next_frame_at {
            return 0;
        }

        // Cap the catch-up. Falling behind is normal on a slow repaint; sprinting to make it up
        // is exactly the behaviour being fixed.
        let mut frames = 0;
        while self.next_frame_at <= now && frames < MAX_CATCH_UP_FRAMES {
            self.next_frame_at += NTSC_FRAME_PERIOD;
            frames += 1;
        }

        // If the deficit is large — the window was hidden, or the machine stalled — resynchronise
        // rather than trying to replay the missing seconds.
        if now > self.next_frame_at + NTSC_FRAME_PERIOD * 8 {
            self.next_frame_at = now + NTSC_FRAME_PERIOD;
        }

        if self.audio_ahead() {
            return 0;
        }

        frames
    }

    /// Whether the emulator is already ahead of the sound card.
    ///
    /// A safety valve only: at this point another frame's samples would be dropped rather than
    /// played, so running it would gain nothing and cost accuracy.
    fn audio_ahead(&self) -> bool {
        let capacity = self.audio_controls.capacity();
        self.audio_running && capacity > 0 && self.audio_controls.queued() > capacity * 9 / 10
    }

}

impl App for NesDebugger {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Shortcuts are read here, before the controller block below empties the event queue.
        //
        // They used to be read after it, with a comment claiming otherwise, so none of them ever
        // fired while the window had focus — which is the only time anyone would press one.
        let fullscreen_pressed = ctx.input(|i| i.key_pressed(egui::Key::F11));
        let leave_fullscreen = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        let save_pressed = ctx.input(|i| i.key_pressed(egui::Key::F5));
        let load_pressed = ctx.input(|i| i.key_pressed(egui::Key::F9));

        // Handle keyboard input for controller
        if ctx.input(|i| i.focused) {
            // Process key events
            ctx.input_mut(|input| {
                // Handle key presses
                for event in &input.events {
                    let egui::Event::Key {
                        key,
                        pressed,
                        repeat: false, // Ignore key repeats
                        ..
                    } = event
                    else {
                        continue;
                    };

                    let Some(our_key) = convert_egui_key(*key) else {
                        continue;
                    };

                    // Update key state in our mapping manager, then push it to the NES system.
                    let state = if *pressed {
                        self.key_mapping_manager.process_controller1_key_press(our_key)
                    } else {
                        self.key_mapping_manager.process_controller1_key_release(our_key)
                    };

                    if let Ok(state) = state {
                        let system = self.system.borrow_mut();
                        system.controller_handler().set_controller1_state(state);
                    }
                }

                // Consume key events to avoid them being processed multiple times.
                //
                // Anything wanting a keyboard shortcut has to look before this point, or the
                // event is gone by the time it runs.
                input.events.clear();
            });
        }

        // Fullscreen and save states, from the presses sampled at the top of this function.
        if fullscreen_pressed {
            self.fullscreen = !self.fullscreen;
        }
        // Escape only leaves, never enters: a key that toggles is a key that can strand someone
        // in a mode they cannot see the way out of.
        if self.fullscreen && leave_fullscreen {
            self.fullscreen = false;
        }
        if save_pressed {
            self.save_state();
        }
        if load_pressed {
            self.load_state();
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));

        // Load the initial file if specified and not yet loaded
        if !self.initial_file_loaded {
            if let Some(file_path) = self.args.file.clone() {
                match self.load_file(&file_path) {
                    Ok(()) => {
                        if self.args.play {
                            let mut system = self.system.borrow_mut();
                            if let Err(error) = self.asm_widget.run_program(&mut system) {
                                error!("starting playback: {error}");
                            }
                        }
                    },
                    Err(err) => error!("Failed to load {}: {err:#}", file_path.display()),
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

        self.measure_rates();

        // Keep the audio stream in step with whether the emulator is actually running.
        //
        // This used to be toggled inside the Run button's handler, which meant any other route
        // into continuous execution left the stream paused and the emulator silent — and the
        // silence looked like an audio bug rather than a missing call.
        self.sync_audio_to_run_state();

        // Advance the emulator a whole frame at a time, so every frame the PPU completes is drawn
        // exactly once. Sizing the work from the audio buffer instead meant a call could span two
        // frames — publishing one that was replaced before it was ever displayed, which shows up
        // as an animation skipping — or none, redisplaying the previous frame.
        if self.asm_widget.is_continuous_run() {
            let frames = self.frames_due();
            if frames > 0 {
                let mut system = self.system.borrow_mut();
                for _ in 0..frames {
                    if !self.asm_widget.run_one_frame(&mut system) {
                        break;
                    }
                }
                self.frames_in_window += frames;
            }

            // Ask to be woken when the next frame is due, rather than spinning at the display's
            // refresh rate and deciding to do nothing.
            let now = std::time::Instant::now();
            ctx.request_repaint_after(self.next_frame_at.saturating_duration_since(now));
        }

        // The menu and toolbar are part of the furniture fullscreen is meant to remove, so they
        // are skipped rather than merely made smaller. F11 and Escape still work: both are read
        // from the input queue before any panel draws.
        if !self.fullscreen {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open...").clicked() {
                        ui.close_menu();
                        // The same loader the command line uses, so a file behaves identically
                        // whichever way it arrives — including the content sniffing that tells a
                        // ROM from assembly.
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("NES ROM or 6502 assembly", &["nes", "asm", "s", "txt"])
                            .add_filter("All files", &["*"])
                            .pick_file()
                        {
                            match self.load_file(&path) {
                                Ok(()) => self.last_dump = Some(format!("loaded {}", path.display())),
                                Err(error) => {
                                    error!("loading {}: {error:#}", path.display());
                                    self.last_dump = Some(format!("failed to load: {error}"));
                                },
                            }
                        }
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

                    ui.separator();

                    // Controller profile submenu
                    ui.menu_button("Controller Profile", |ui| {
                        // Default profile
                        if ui.button("Default").clicked() {
                            if let Err(e) = self.key_mapping_manager.set_controller1_profile("Default") {
                                error!("Failed to switch to Default profile: {}", e);
                            } else {
                                info!("Switched to Default controller profile");
                            }
                            ui.close_menu();
                        }

                        // WASD profile
                        if ui.button("WASD Layout").clicked() {
                            if let Err(e) = self.key_mapping_manager.set_controller1_profile("WASD Layout") {
                                error!("Failed to switch to WASD Layout profile: {}", e);
                            } else {
                                info!("Switched to WASD Layout controller profile");
                            }
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Keyboard Mappings").clicked() {
                        self.keyboard_mappings_widget.toggle_visibility();
                        ui.close_menu();
                    }
                });
            });
        });

        // Add a toolbar for emulation controls
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);

                // System state for enabling/disabling buttons
                let system_state = self.system.borrow().state();

                // Double-check if we can assemble - enabled if system is Ready OR
                // if we've specifically cleared the assembly but system state hasn't updated yet
                let can_assemble = system_state == SystemState::Ready || !self.asm_widget.is_loaded();

                // Assemble button
                if ui.add_enabled(can_assemble, egui::Button::new("🔨 Assemble")).clicked() {
                    let mut system = self.system.borrow_mut();
                    if let Err(e) = self.asm_widget.assemble_code(&mut system) {
                        error!("Error assembling code: {}", e);
                    } else {
                        info!("Code assembled successfully");
                    }
                }

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                // Capturing the picture together with the registers that produced it, so a fault
                // seen while playing can be diagnosed afterwards instead of described.
                let fullscreen_label = if self.fullscreen { "🗗 Windowed" } else { "⛶ Fullscreen" };
                if ui.button(fullscreen_label).on_hover_text("F11").clicked() {
                    self.fullscreen = !self.fullscreen;
                }

                ui.add_space(4.0);

                if ui.button("💾 Save state").on_hover_text("F5").clicked() {
                    self.save_state();
                }
                if ui.button("📂 Load state").on_hover_text("F9").clicked() {
                    self.load_state();
                }

                ui.add_space(4.0);

                if ui.button("📷 Dump frame").clicked() {
                    let system = self.system.borrow();
                    match frame_dump::dump(&system, std::path::Path::new("frame-dumps")) {
                        Ok(path) => {
                            info!("dumped frame to {}", path.display());
                            self.last_dump = Some(format!("saved {}", path.display()));
                        },
                        Err(error) => {
                            error!("dumping the frame: {error}");
                            self.last_dump = Some(format!("failed: {error}"));
                        },
                    }
                }
                if let Some(message) = &self.last_dump {
                    ui.label(message);
                }

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                // Run/Stop button
                if self.asm_widget.is_continuous_run() {
                    if ui.button("⏹ Stop").clicked() {
                        // Toggle continuous run mode off
                        let mut system = self.system.borrow_mut();
                        let _ = self.asm_widget.run_program(&mut system);

                    }
                } else {
                    // Only enable Run when loaded, running, or finished
                    let can_run = matches!(
                        system_state,
                        SystemState::Loaded | SystemState::Running | SystemState::Finished
                    );
                    if ui.add_enabled(can_run, egui::Button::new("▶ Run")).clicked() {
                        // Start continuous execution
                        let mut system = self.system.borrow_mut();
                        let _ = self.asm_widget.run_program(&mut system);

                    }
                }

                // Step button - only enabled when loaded or running
                let can_step = matches!(system_state, SystemState::Loaded | SystemState::Running);
                if ui.add_enabled(can_step, egui::Button::new("⏯ Step")).clicked() {
                    let mut system = self.system.borrow_mut();
                    let _ = self.asm_widget.step(&mut system);
                }

                // Run to next frame - only enabled when loaded, running, or finished
                let can_run_frame = matches!(
                    system_state,
                    SystemState::Loaded | SystemState::Running | SystemState::Finished
                );
                if ui
                    .add_enabled(can_run_frame, egui::Button::new("⏭ Next Frame"))
                    .clicked()
                {
                    let mut system = self.system.borrow_mut();
                    let _ = self.asm_widget.run_to_next_frame(&mut system);
                }

                // Reset/Clear button - only enabled when not in ready state
                if ui
                    .add_enabled(system_state != SystemState::Ready, egui::Button::new("🗑️ Clear"))
                    .clicked()
                {
                    let mut system = self.system.borrow_mut();
                    info!("Clearing program and resetting system to Ready state");
                    if let Err(e) = self.asm_widget.reset_program(&mut system) {
                        error!("Error resetting system: {}", e);
                    }

                    // Request a repaint to immediately update the UI state
                    ctx.request_repaint();

                    // Log the system state after reset for debugging
                    info!("System state after reset: {:?}", system.state());
                }

                // Current address/instruction display
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(4.0);

                let system = self.system.borrow();
                let pc = system.cpu().pc();

                // Emulation rate, beside the other run-state indicators. Kept always visible
                // rather than inside a panel: it is the first thing to check when the picture
                // looks wrong, and a metric you cannot find is a metric you do not have.
                if self.emulated_fps > 0.0 {
                    let off_rate = (self.emulated_fps - 60.0).abs() > 3.0;
                    ui.colored_label(
                        if off_rate { egui::Color32::YELLOW } else { egui::Color32::GRAY },
                        format!("{:.1} fps", self.emulated_fps),
                    )
                    .on_hover_text(format!(
                        "Emulated frames per second; NTSC runs at 60.1.\nUI repaints: {:.0} fps \
                         (the display's rate, unrelated).",
                        self.repaint_fps
                    ));
                    ui.separator();
                }

                // Display current position
                ui.label(format!("PC: ${:04X}", pc));

                // Display the current instruction if we can read it
                if let Ok(opcode) = system.cpu().read_byte(pc) {
                    ui.label(format!("Current: ${:02X}", opcode));
                }

                // System state
                ui.add_space(8.0);

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

            ui.add_space(4.0);
        });
        }

        // Snapshot the audio pipeline's health for this frame's UI.
        let audio_stats = self.audio_stats();

        // Main central panel with dock area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Fullscreen shows the picture and nothing else.
            //
            // Making the window fill the screen while the debugger's panels still surround the
            // display just enlarges the furniture. What is wanted is the game at the size of the
            // screen, so the docked layout is set aside entirely and the display drawn on its own.
            if self.fullscreen {
                let overscan = if self.context.overscan { 8 } else { 0 };
                let zoom = self.pixel_display.fit_zoom(ui, 256, 240 - overscan * 2);
                self.pixel_display.set_zoom(zoom);
                self.pixel_display.set_show_grid(false);

                let system_ref = self.system.clone();
                let adapter = PpuPixelAdapter::new(move || {
                    system_ref.borrow().ppu().frame_buffer()
                })
                .with_overscan(overscan);

                ui.vertical_centered(|ui| {
                    ui.add_space((ui.available_height() - 240.0 * zoom).max(0.0) / 2.0);
                    let _ = self.pixel_display.ui(ui, &adapter);
                });
                return;
            }

            // Create a tab viewer with references to all components
            let mut tab_viewer = NesTabViewer {
                pixel_display: &mut self.pixel_display,
                asm_widget: &mut self.asm_widget,
                cpu_widget: &mut self.cpu_widget,
                ppu_widget: &mut self.ppu_widget,
                dma_widget: &mut self.dma_widget,
                controller_widget: &mut self.controller_widget,
                disasm_widget: &mut self.disasm_widget,
                memory_widget: &mut self.memory_widget,
                pattern_table_widget: &mut self.pattern_table_widget,
                audio_widget: &mut self.audio_widget,
                audio_stats,
                controller_profile: self.key_mapping_manager.controller1_profile(),
                waveform_visualizer: &mut self.waveform_visualizer,
                system: self.system.clone(),
                context: &mut self.context,
            };

            // Render the dock area
            DockArea::new(&mut self.dock_state)
                .style(Style::from_egui(ui.style().as_ref()))
                .show(ctx, &mut tab_viewer);

            // Render keyboard mappings widget (if visible)
            self.keyboard_mappings_widget.ui(ctx, &self.key_mapping_manager);
        });
    }
}


fn main() -> anyhow::Result<()> {
    info!("Starting RustNES Debugger");

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args = Args::parse();

    if let Some(path) = &args.file {
        info!("File specified: {}", path.display());
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
            Ok(Box::new(NesDebugger::new(cc, args)?))
        }),
    )
    .map_err(|e| {
        error!("Application error: {}", e);
        anyhow::anyhow!("Application error: {}", e)
    })?;

    info!("Application closed normally");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A display that cannot manage a repaint per emulated frame must not capture the cadence.
    ///
    /// This is the case that was reported: 55 repaints a second, locked to one frame each, so the
    /// game ran at 55 Hz instead of 60.0988 — nine percent slow, with the sound card starved by
    /// the same fraction and dragging audibly. The old tolerance of 0.15 accepted it because
    /// 55/60 rounds to 1. Falling back to the wall clock instead lets the deficit be made up.
    #[test]
    fn a_display_that_cannot_keep_up_does_not_take_the_cadence() {
        assert_eq!(cadence_lock(55.0), 0, "55 Hz is not a cadence, it is a shortfall");
        assert_eq!(cadence_lock(51.0), 0);
        assert_eq!(cadence_lock(59.0), 0, "even one repaint short of the NES rate");
    }

    /// The rates that genuinely are a whole number of repaints per frame still lock.
    #[test]
    fn a_clean_multiple_of_the_nes_rate_locks() {
        assert_eq!(cadence_lock(60.0988), 1, "the NES's own rate");
        assert_eq!(cadence_lock(120.1976), 2, "a ProMotion display at twice it");
        assert_eq!(cadence_lock(240.3952), 4);
    }

    /// A rate close to a multiple but not close enough is left to the wall clock.
    ///
    /// 69 rounds to 1 and 65 to 1, and the old tolerance took both. Neither is a cadence: the
    /// beat between them and the NES's rate is exactly what locking is supposed to remove.
    #[test]
    fn a_rate_near_but_not_on_a_multiple_is_refused() {
        assert_eq!(cadence_lock(69.0), 0);
        assert_eq!(cadence_lock(65.0), 0);
        assert_eq!(cadence_lock(100.0), 0, "between one and two repaints a frame");
    }

    /// 144 Hz is not a multiple of 60.0988 and must not be treated as one.
    #[test]
    fn a_144_hz_display_paces_by_the_clock() {
        assert_eq!(cadence_lock(144.0), 0);
    }
}
