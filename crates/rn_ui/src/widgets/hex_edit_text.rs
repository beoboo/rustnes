use egui::{TextEdit, Ui};

/// A widget for editing hexadecimal values
pub struct HexEditText {
    /// Current edit buffer for when editing is active
    edit_buffer: String,
    /// Whether we're currently editing this value
    is_editing: bool,
    /// Track if we need to request focus
    needs_focus: bool,
}

/// Value type for type-safe operations
pub enum ValueType {
    /// 8-bit value (e.g., CPU registers, memory bytes)
    Bit8,
    /// 16-bit value (e.g., memory addresses, PC)
    Bit16,
}

impl HexEditText {
    /// Create a new hex edit text widget
    pub fn new() -> Self {
        Self {
            edit_buffer: String::new(),
            is_editing: false,
            needs_focus: false,
        }
    }

    /// Display and edit a hex value
    ///
    /// Returns true if the value was changed
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        label: &str,
        value: &mut u16,
        value_type: ValueType,
        tooltip: Option<&str>,
    ) -> bool {
        let mut value_changed = false;

        // Display label if provided
        if !label.is_empty() {
            ui.label(label);
        }

        // Format the current value based on value type
        let formatted_value = match value_type {
            ValueType::Bit8 => format!("${:02X}", *value as u8),
            ValueType::Bit16 => format!("${:04X}", *value),
        };

        if self.is_editing {
            // Create a unique ID for the text edit field
            let text_edit_id = ui.make_persistent_id(format!("hex_edit_{}", label));

            // Request focus if needed (when we just started editing)
            if self.needs_focus {
                ui.memory_mut(|mem| mem.request_focus(text_edit_id));
                self.needs_focus = false;
            }

            // Show text edit field
            let response = ui.add(
                TextEdit::singleline(&mut self.edit_buffer)
                    .desired_width(50.0)
                    .hint_text(&formatted_value)
                    .id(text_edit_id),
            );

            // Check if editing is complete
            if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                // Try to parse the new value
                let parse_result = match value_type {
                    ValueType::Bit8 => {
                        u8::from_str_radix(self.edit_buffer.trim_start_matches("$"), 16).map(|v| v as u16)
                    },
                    ValueType::Bit16 => u16::from_str_radix(self.edit_buffer.trim_start_matches("$"), 16),
                };

                // Update value if parsing succeeded
                if let Ok(new_value) = parse_result {
                    *value = new_value;
                    value_changed = true;
                }

                self.is_editing = false;
            }
        } else {
            // Show button with current value
            let button = ui.button(&formatted_value);

            // Add tooltip if provided
            if let Some(tip) = tooltip {
                button.clone().on_hover_text(tip);
            }

            // Start editing if clicked
            if button.clicked() {
                self.edit_buffer = match value_type {
                    ValueType::Bit8 => format!("{:02X}", *value as u8),
                    ValueType::Bit16 => format!("{:04X}", *value),
                };
                self.is_editing = true;
                self.needs_focus = true; // Request focus on next frame
            }
        }

        value_changed
    }
}
