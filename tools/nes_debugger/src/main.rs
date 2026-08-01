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
                            
                            system.ppu().frame_buffer().to_vec()
                        });

                        // Update zoom and show the PPU display
                        self.pixel_display.set_zoom(auto_zoom);
                        let _ = self.pixel_display.ui(ui, &ppu_adapter);
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

                        let available_width = ui.available_width();
                        let auto_zoom = (available_width / 512.0).clamp(0.25, 4.0);

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
            waveform_visualizer: waveform,
            system,
            key_mapping_manager,
            dock_state,
            context: AppContext {
                display_mode: DisplayMode::Memory,
            },
            initial_file_loaded: false,
        })
    }
}

impl NesDebugger {
    /// Load the file given on the command line, which may be a `.nes` ROM or 6502 assembly.
    ///
    /// Detected by content rather than by extension: an iNES image starts with the four bytes
    /// `NES\x1A`. Reading a ROM as text fails with "stream did not contain valid UTF-8", which
    /// says nothing useful about what the user actually passed.
    fn load_initial_file(&mut self, path: &Path) -> Result<()> {
        let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;

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

    /// How many emulated frames are due, by the wall clock.
    ///
    /// The NES runs at 60.0988 Hz, which is unrelated to how often the UI repaints — a ProMotion
    /// display does so at 120 Hz. Running a frame per repaint therefore ran the emulator at double
    /// speed until the audio buffer filled and the gate closed, then stalled until it drained:
    /// visible as animation jumping ahead and freezing. Deciding by elapsed time instead makes the
    /// rate independent of the display.
    fn frames_due(&mut self) -> u32 {
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

        // The audio buffer is now only a safety valve: if it is nearly full the emulator is ahead
        // of the sound card despite the clock, and another frame would just be dropped samples.
        let capacity = self.audio_controls.capacity();
        if self.audio_running && capacity > 0 && self.audio_controls.queued() > capacity * 9 / 10 {
            return 0;
        }

        frames
    }

}

impl App for NesDebugger {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
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

                // Consume key events to avoid them being processed multiple times
                input.events.clear();
            });
        }

        // Load the initial file if specified and not yet loaded
        if !self.initial_file_loaded {
            if let Some(file_path) = self.args.file.clone() {
                if let Err(err) = self.load_initial_file(&file_path) {
                    error!("Failed to load {}: {err:#}", file_path.display());
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
            }

            // Ask to be woken when the next frame is due, rather than spinning at the display's
            // refresh rate and deciding to do nothing.
            let now = std::time::Instant::now();
            ctx.request_repaint_after(self.next_frame_at.saturating_duration_since(now));
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

        // Snapshot the audio pipeline's health for this frame's UI.
        let audio_stats = self.audio_stats();

        // Main central panel with dock area
        egui::CentralPanel::default().show(ctx, |ui| {
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
