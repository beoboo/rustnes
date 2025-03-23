#![allow(dead_code)]
use eframe::egui;
use rn_input::key_mapping::KeyMappingManager;

/// Widget for viewing and configuring controller keyboard mappings
pub struct KeyboardMappingsWidget {
    /// Whether the widget is currently visible
    visible: bool,
    /// Position for the window (can be saved between sessions)
    position: egui::Pos2,
}

impl Default for KeyboardMappingsWidget {
    fn default() -> Self {
        Self {
            visible: false,
            position: egui::pos2(50.0, 300.0),
        }
    }
}

impl KeyboardMappingsWidget {
    /// Create a new keyboard mappings widget
    pub fn new() -> Self {
        Self::default()
    }

    /// Show or hide the widget
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Toggle the visibility of the widget
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Whether the widget is currently visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Render the widget as a window
    pub fn ui(&mut self, ctx: &egui::Context, key_mapping_manager: &KeyMappingManager) {
        if !self.visible {
            return;
        }

        // Open window with close button
        let mut is_open = self.visible;
        egui::Window::new("Controller Keyboard Mappings")
            .open(&mut is_open)
            .resizable(true)
            .default_pos(self.position)
            .show(ctx, |ui| {
                self.show_mappings_ui(ui, key_mapping_manager);
            });

        // Update visibility based on window close button
        self.visible = is_open;

        // Save position for next time
        if let Some(rect) = ctx.memory(|mem| mem.area_rect(egui::Id::new("Controller Keyboard Mappings"))) {
            self.position = rect.left_top();
        }
    }

    /// Draw the mapping content within the UI
    fn show_mappings_ui(&self, ui: &mut egui::Ui, key_mapping_manager: &KeyMappingManager) {
        // Show the active key mapping information
        let profile_name = &key_mapping_manager.controller1_mapping.profile.name;

        ui.heading(format!("Active Profile: {}", profile_name));

        // Add hint text at the top
        ui.label("Press keyboard keys to control the NES controller.");
        ui.label("Change profiles in System → Controller Profile menu.");

        ui.add_space(10.0);
        ui.separator();

        // Create a grid to display key mappings
        egui::Grid::new("key_mappings")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.strong("NES Button");
                ui.strong("Keyboard Key");
                ui.end_row();

                // Get all mappings and sort them for consistent display
                let mappings = key_mapping_manager.controller1_mapping.profile.get_all_mappings();
                let mut button_keys: Vec<_> = mappings.iter().collect();
                button_keys.sort_by_key(|(_, &btn)| btn as u8);

                // Display each mapping
                for (&key, &button) in button_keys {
                    ui.label(format!("{:?}", button));
                    ui.label(format!("{:?}", key));
                    ui.end_row();
                }
            });
    }
}
