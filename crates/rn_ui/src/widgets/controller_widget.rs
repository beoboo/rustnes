#![allow(dead_code)]
use egui::{Color32, Grid, RichText, Ui};
use rn_core::input::{ControllerButton, ControllerHandlerWrapper, ControllerState};

// Define the emoji characters for the controller buttons
const ARROW_UP: &str = "⬆"; // UP ARROW
const ARROW_DOWN: &str = "⬇"; // DOWN ARROW
const ARROW_LEFT: &str = "⬅"; // LEFT ARROW
const ARROW_RIGHT: &str = "➡"; // RIGHT ARROW

/// Widget for displaying controller state
#[derive(Default)]
pub struct ControllerWidget {}


impl ControllerWidget {
    /// Create a new controller widget
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the controller widget using the given UI and controller handler
    pub fn ui(&mut self, ui: &mut Ui, controller: &ControllerHandlerWrapper) {
        ui.heading("Controller State");

        // Add a help text
        ui.label("Click on buttons to toggle controller state");

        // Split into two columns for the two controllers
        ui.columns(2, |columns| {
            // Controller 1
            self.render_controller(&mut columns[0], controller, 1);

            // Controller 2
            self.render_controller(&mut columns[1], controller, 2);
        });
    }

    /// Render a single controller
    fn render_controller(&self, ui: &mut Ui, controller: &ControllerHandlerWrapper, controller_num: u8) {
        ui.heading(RichText::new(format!("Controller {}", controller_num)).size(16.0));

        // Get controller state for this controller
        let state = if controller_num == 1 {
            controller_state_1(controller)
        } else {
            controller_state_2(controller)
        };

        // Create a visual representation of the NES controller
        ui.vertical_centered(|ui| {
            // D-Pad visualization in a grid layout
            Grid::new(format!("controller_{}_dpad", controller_num))
                .num_columns(3)
                .spacing([5.0, 5.0])
                .show(ui, |ui| {
                    // Empty space for top-left
                    ui.label("");

                    // Up button
                    if ui
                        .add(interactive_button(
                            ARROW_UP,
                            state.is_button_pressed(ControllerButton::Up),
                        ))
                        .clicked()
                    {
                        toggle_button(controller, controller_num, ControllerButton::Up);
                    }

                    // Empty space for top-right
                    ui.label("");
                    ui.end_row();

                    // Left button
                    if ui
                        .add(interactive_button(
                            ARROW_LEFT,
                            state.is_button_pressed(ControllerButton::Left),
                        ))
                        .clicked()
                    {
                        toggle_button(controller, controller_num, ControllerButton::Left);
                    }

                    // Center (empty)
                    ui.label("");

                    // Right button
                    if ui
                        .add(interactive_button(
                            ARROW_RIGHT,
                            state.is_button_pressed(ControllerButton::Right),
                        ))
                        .clicked()
                    {
                        toggle_button(controller, controller_num, ControllerButton::Right);
                    }
                    ui.end_row();

                    // Empty space for bottom-left
                    ui.label("");

                    // Down button
                    if ui
                        .add(interactive_button(
                            ARROW_DOWN,
                            state.is_button_pressed(ControllerButton::Down),
                        ))
                        .clicked()
                    {
                        toggle_button(controller, controller_num, ControllerButton::Down);
                    }

                    // Empty space for bottom-right
                    ui.label("");
                    ui.end_row();
                });

            ui.add_space(10.0);

            // Action buttons
            ui.horizontal(|ui| {
                if ui
                    .add(interactive_button("B", state.is_button_pressed(ControllerButton::B)))
                    .clicked()
                {
                    toggle_button(controller, controller_num, ControllerButton::B);
                }

                ui.add_space(10.0);

                if ui
                    .add(interactive_button("A", state.is_button_pressed(ControllerButton::A)))
                    .clicked()
                {
                    toggle_button(controller, controller_num, ControllerButton::A);
                }
            });

            ui.add_space(10.0);

            // Select and Start buttons
            ui.horizontal(|ui| {
                if ui
                    .add(interactive_button(
                        "SELECT",
                        state.is_button_pressed(ControllerButton::Select),
                    ))
                    .clicked()
                {
                    toggle_button(controller, controller_num, ControllerButton::Select);
                }

                ui.add_space(10.0);

                if ui
                    .add(interactive_button(
                        "START",
                        state.is_button_pressed(ControllerButton::Start),
                    ))
                    .clicked()
                {
                    toggle_button(controller, controller_num, ControllerButton::Start);
                }
            });

            ui.add_space(10.0);

            // Button state summary
            Grid::new(format!("controller_{}_state", controller_num))
                .num_columns(2)
                .spacing([10.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    for button in [
                        ControllerButton::A,
                        ControllerButton::B,
                        ControllerButton::Select,
                        ControllerButton::Start,
                        ControllerButton::Up,
                        ControllerButton::Down,
                        ControllerButton::Left,
                        ControllerButton::Right,
                    ] {
                        ui.label(format!("{:?}:", button));
                        ui.label(
                            RichText::new(if state.is_button_pressed(button) {
                                "Pressed"
                            } else {
                                "Released"
                            })
                            .color(if state.is_button_pressed(button) {
                                Color32::GREEN
                            } else {
                                Color32::GRAY
                            }),
                        );
                        ui.end_row();
                    }
                });

            // Add reset button
            if ui.button("Reset All Buttons").clicked() {
                reset_controller(controller, controller_num);
            }
        });
    }
}

// Helper to create an interactive button that shows pressed state
fn interactive_button(text: &str, is_pressed: bool) -> egui::Button<'_> {
    let mut button = egui::Button::new(RichText::new(text).size(18.0).strong().color(if is_pressed {
        Color32::BLACK
    } else {
        Color32::WHITE
    }));

    if is_pressed {
        button = button.fill(Color32::from_rgb(100, 220, 100));
    } else {
        button = button.fill(Color32::from_rgb(70, 70, 70));
    }

    button = button.min_size(egui::vec2(30.0, 30.0));

    button
}

// Helper function to toggle a controller button state
fn toggle_button(controller: &ControllerHandlerWrapper, controller_num: u8, button: ControllerButton) {
    // Get current state
    let current_state = if controller_num == 1 {
        controller.get_controller1_state()
    } else {
        controller.get_controller2_state()
    };

    // Create a new state with the button toggled
    let mut new_state = current_state;
    new_state.set_button(button, !current_state.is_button_pressed(button));

    // Update the controller
    if controller_num == 1 {
        controller.set_controller1_state(new_state);
    } else {
        controller.set_controller2_state(new_state);
    }
}

// Helper function to reset all buttons on a controller
fn reset_controller(controller: &ControllerHandlerWrapper, controller_num: u8) {
    let empty_state = ControllerState::new();

    if controller_num == 1 {
        controller.set_controller1_state(empty_state);
    } else {
        controller.set_controller2_state(empty_state);
    }
}

// Helper to get controller 1 state
fn controller_state_1(controller: &ControllerHandlerWrapper) -> ControllerState {
    controller.get_controller1_state()
}

// Helper to get controller 2 state
fn controller_state_2(controller: &ControllerHandlerWrapper) -> ControllerState {
    controller.get_controller2_state()
}
