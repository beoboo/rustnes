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

        // Not mapped
        _ => None,
    }
}
