#![allow(dead_code)]
use eframe::egui;
use rn_input::key_mapping::KeyCode;

/// Convert egui key to our KeyCode
pub fn convert_egui_key(key: egui::Key) -> Option<KeyCode> {
    match key {
        // Arrow keys
        egui::Key::ArrowUp => Some(KeyCode::ArrowUp),
        egui::Key::ArrowDown => Some(KeyCode::ArrowDown),
        egui::Key::ArrowLeft => Some(KeyCode::ArrowLeft),
        egui::Key::ArrowRight => Some(KeyCode::ArrowRight),

        // Letters
        egui::Key::A => Some(KeyCode::A),
        egui::Key::B => Some(KeyCode::B),
        egui::Key::C => Some(KeyCode::C),
        egui::Key::D => Some(KeyCode::D),
        egui::Key::E => Some(KeyCode::E),
        egui::Key::F => Some(KeyCode::F),
        egui::Key::G => Some(KeyCode::G),
        egui::Key::H => Some(KeyCode::H),
        egui::Key::I => Some(KeyCode::I),
        egui::Key::J => Some(KeyCode::J),
        egui::Key::K => Some(KeyCode::K),
        egui::Key::L => Some(KeyCode::L),
        egui::Key::M => Some(KeyCode::M),
        egui::Key::N => Some(KeyCode::N),
        egui::Key::O => Some(KeyCode::O),
        egui::Key::P => Some(KeyCode::P),
        egui::Key::Q => Some(KeyCode::Q),
        egui::Key::R => Some(KeyCode::R),
        egui::Key::S => Some(KeyCode::S),
        egui::Key::T => Some(KeyCode::T),
        egui::Key::U => Some(KeyCode::U),
        egui::Key::V => Some(KeyCode::V),
        egui::Key::W => Some(KeyCode::W),
        egui::Key::X => Some(KeyCode::X),
        egui::Key::Y => Some(KeyCode::Y),
        egui::Key::Z => Some(KeyCode::Z),

        // Special keys
        egui::Key::Enter => Some(KeyCode::Enter),
        egui::Key::Space => Some(KeyCode::Space),
        egui::Key::Escape => Some(KeyCode::Escape),
        egui::Key::Tab => Some(KeyCode::Tab),
        egui::Key::Backspace => Some(KeyCode::Backspace),

        // Not mapped
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rn_core::input::controller::ControllerButton;
    use rn_input::controller_profile::ControllerProfile;

    /// A binding is only real if the window toolkit can deliver the key it names.
    ///
    /// Select used to be bound to Right Shift, which egui reports as a modifier flag on other
    /// events and never as a `Key` of its own — so `convert_egui_key` could not produce it and no
    /// keypress ever set the button. The profile tests still passed, because they only check that
    /// every button has *a* key. Super Mario Bros 3 chooses between one and two players with
    /// Select, so the whole two-player mode was unreachable while the binding looked correct.
    #[test]
    fn every_button_is_reachable_through_egui() {
        let reachable: Vec<KeyCode> = egui::Key::ALL.iter().filter_map(|k| convert_egui_key(*k)).collect();

        for profile in [
            ControllerProfile::create_default_profile("Default"),
            ControllerProfile::create_combined_profile(),
            ControllerProfile::create_wasd_profile(),
        ] {
            for button in ControllerButton::ALL {
                let keys = profile.keys_for(button);
                assert!(
                    keys.iter().any(|k| reachable.contains(k)),
                    "the {} profile binds {button:?} only to {keys:?}, none of which egui can deliver",
                    profile.name
                );
            }
        }
    }
}
