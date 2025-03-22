use serde::{Serialize, Deserialize};
use crate::error::InputError;
use rn_core::input::controller::ControllerState;
use crate::controller_profile::ControllerProfile;

/// Cross-platform keyboard key identifiers
/// 
/// These are designed to be platform-agnostic and easily mappable
/// to both desktop (egui/eframe) and web (WASM) key events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    // Arrow keys
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // Special keys
    Enter,
    Space,
    Escape,
    Tab,
    Backspace,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    
    // Modifiers
    ShiftLeft,
    ShiftRight,
    ControlLeft,
    ControlRight,
    AltLeft,
    AltRight,
    MetaLeft,
    MetaRight,
    
    // Numbers
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    
    // Numpad
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadAdd, NumpadSubtract, NumpadMultiply, NumpadDivide, NumpadDecimal, NumpadEnter,
    
    // Others
    Unknown,
}

impl KeyCode {
    /// Convert from string representation of a key (useful for config files)
    pub fn from_str(key_str: &str) -> Self {
        match key_str.to_lowercase().as_str() {
            "up" | "arrowup" => Self::ArrowUp,
            "down" | "arrowdown" => Self::ArrowDown,
            "left" | "arrowleft" => Self::ArrowLeft,
            "right" | "arrowright" => Self::ArrowRight,
            
            // Letters
            "a" => Self::A, "b" => Self::B, "c" => Self::C, "d" => Self::D, 
            "e" => Self::E, "f" => Self::F, "g" => Self::G, "h" => Self::H,
            "i" => Self::I, "j" => Self::J, "k" => Self::K, "l" => Self::L,
            "m" => Self::M, "n" => Self::N, "o" => Self::O, "p" => Self::P,
            "q" => Self::Q, "r" => Self::R, "s" => Self::S, "t" => Self::T,
            "u" => Self::U, "v" => Self::V, "w" => Self::W, "x" => Self::X,
            "y" => Self::Y, "z" => Self::Z,
            
            // Function keys
            "f1" => Self::F1, "f2" => Self::F2, "f3" => Self::F3, "f4" => Self::F4,
            "f5" => Self::F5, "f6" => Self::F6, "f7" => Self::F7, "f8" => Self::F8,
            "f9" => Self::F9, "f10" => Self::F10, "f11" => Self::F11, "f12" => Self::F12,
            
            // Special keys
            "enter" | "return" => Self::Enter,
            "space" => Self::Space,
            "escape" | "esc" => Self::Escape,
            "tab" => Self::Tab,
            "backspace" => Self::Backspace,
            "home" => Self::Home,
            "end" => Self::End,
            "pageup" => Self::PageUp,
            "pagedown" => Self::PageDown,
            "insert" => Self::Insert,
            "delete" => Self::Delete,
            
            // Modifiers
            "shiftleft" | "lshift" => Self::ShiftLeft,
            "shiftright" | "rshift" => Self::ShiftRight,
            "controlleft" | "lcontrol" | "lctrl" => Self::ControlLeft,
            "controlright" | "rcontrol" | "rctrl" => Self::ControlRight,
            "altleft" | "lalt" => Self::AltLeft,
            "altright" | "ralt" => Self::AltRight,
            "metaleft" | "lmeta" | "lcommand" | "lwin" => Self::MetaLeft,
            "metaright" | "rmeta" | "rcommand" | "rwin" => Self::MetaRight,
            
            // Numbers
            "0" => Self::Num0, "1" => Self::Num1, "2" => Self::Num2, "3" => Self::Num3, 
            "4" => Self::Num4, "5" => Self::Num5, "6" => Self::Num6, "7" => Self::Num7, 
            "8" => Self::Num8, "9" => Self::Num9,
            
            // Numpad
            "numpad0" => Self::Numpad0, "numpad1" => Self::Numpad1,
            "numpad2" => Self::Numpad2, "numpad3" => Self::Numpad3,
            "numpad4" => Self::Numpad4, "numpad5" => Self::Numpad5,
            "numpad6" => Self::Numpad6, "numpad7" => Self::Numpad7,
            "numpad8" => Self::Numpad8, "numpad9" => Self::Numpad9,
            "numpadadd" | "numpad+" => Self::NumpadAdd,
            "numpadsubtract" | "numpad-" => Self::NumpadSubtract,
            "numpadmultiply" | "numpad*" => Self::NumpadMultiply,
            "numpaddivide" | "numpad/" => Self::NumpadDivide,
            "numpaddecimal" | "numpad." => Self::NumpadDecimal,
            "numpadenter" => Self::NumpadEnter,
            
            _ => Self::Unknown,
        }
    }
    
    /// Convert to string representation
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::ArrowUp => "ArrowUp",
            Self::ArrowDown => "ArrowDown",
            Self::ArrowLeft => "ArrowLeft",
            Self::ArrowRight => "ArrowRight",
            
            // Letters
            Self::A => "A", Self::B => "B", Self::C => "C", Self::D => "D",
            Self::E => "E", Self::F => "F", Self::G => "G", Self::H => "H",
            Self::I => "I", Self::J => "J", Self::K => "K", Self::L => "L",
            Self::M => "M", Self::N => "N", Self::O => "O", Self::P => "P",
            Self::Q => "Q", Self::R => "R", Self::S => "S", Self::T => "T",
            Self::U => "U", Self::V => "V", Self::W => "W", Self::X => "X",
            Self::Y => "Y", Self::Z => "Z",
            
            // Function keys
            Self::F1 => "F1", Self::F2 => "F2", Self::F3 => "F3", Self::F4 => "F4",
            Self::F5 => "F5", Self::F6 => "F6", Self::F7 => "F7", Self::F8 => "F8",
            Self::F9 => "F9", Self::F10 => "F10", Self::F11 => "F11", Self::F12 => "F12",
            
            // Special keys
            Self::Enter => "Enter",
            Self::Space => "Space",
            Self::Escape => "Escape",
            Self::Tab => "Tab",
            Self::Backspace => "Backspace",
            Self::Home => "Home",
            Self::End => "End",
            Self::PageUp => "PageUp",
            Self::PageDown => "PageDown",
            Self::Insert => "Insert",
            Self::Delete => "Delete",
            
            // Modifiers
            Self::ShiftLeft => "ShiftLeft",
            Self::ShiftRight => "ShiftRight",
            Self::ControlLeft => "ControlLeft",
            Self::ControlRight => "ControlRight",
            Self::AltLeft => "AltLeft",
            Self::AltRight => "AltRight",
            Self::MetaLeft => "MetaLeft",
            Self::MetaRight => "MetaRight",
            
            // Numbers
            Self::Num0 => "0", Self::Num1 => "1", Self::Num2 => "2", Self::Num3 => "3",
            Self::Num4 => "4", Self::Num5 => "5", Self::Num6 => "6", Self::Num7 => "7",
            Self::Num8 => "8", Self::Num9 => "9",
            
            // Numpad
            Self::Numpad0 => "Numpad0", Self::Numpad1 => "Numpad1", 
            Self::Numpad2 => "Numpad2", Self::Numpad3 => "Numpad3",
            Self::Numpad4 => "Numpad4", Self::Numpad5 => "Numpad5", 
            Self::Numpad6 => "Numpad6", Self::Numpad7 => "Numpad7",
            Self::Numpad8 => "Numpad8", Self::Numpad9 => "Numpad9",
            Self::NumpadAdd => "NumpadAdd",
            Self::NumpadSubtract => "NumpadSubtract",
            Self::NumpadMultiply => "NumpadMultiply",
            Self::NumpadDivide => "NumpadDivide",
            Self::NumpadDecimal => "NumpadDecimal",
            Self::NumpadEnter => "NumpadEnter",
            
            Self::Unknown => "Unknown",
        }
    }
    
    // Note: egui feature has been removed as per user request
}

/// Manages key mappings and active keyboard state for a controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMapping {
    /// The active controller profile
    pub profile: ControllerProfile,
    
    /// Currently pressed keys
    #[serde(skip)]
    pressed_keys: Vec<KeyCode>,
}

impl Default for KeyMapping {
    fn default() -> Self {
        Self {
            profile: ControllerProfile::default(),
            pressed_keys: Vec::new(),
        }
    }
}

impl KeyMapping {
    /// Create a new key mapping with the default profile
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a key mapping with a specific profile
    pub fn with_profile(profile: ControllerProfile) -> Self {
        Self {
            profile,
            pressed_keys: Vec::new(),
        }
    }
    
    /// Change the active profile
    pub fn set_profile(&mut self, profile: ControllerProfile) {
        self.profile = profile;
    }
    
    /// Process a key press event
    pub fn process_key_press(&mut self, key: KeyCode) -> Result<ControllerState, InputError> {
        // Add to pressed keys if not already pressed
        if !self.pressed_keys.contains(&key) {
            self.pressed_keys.push(key);
        }
        
        // Update controller state based on currently pressed keys
        self.get_controller_state()
    }
    
    /// Process a key release event
    pub fn process_key_release(&mut self, key: KeyCode) -> Result<ControllerState, InputError> {
        // Remove from pressed keys
        self.pressed_keys.retain(|&k| k != key);
        
        // Update controller state based on currently pressed keys
        self.get_controller_state()
    }
    
    /// Get current controller state based on pressed keys
    pub fn get_controller_state(&self) -> Result<ControllerState, InputError> {
        let mut state = ControllerState::new();
        
        // For each pressed key, set the corresponding controller button state
        for key in &self.pressed_keys {
            if let Some(button) = self.profile.get_button_for_key(*key) {
                state.set_button(button, true);
            }
        }
        
        Ok(state)
    }
    
    /// Clear all pressed keys
    pub fn clear_pressed_keys(&mut self) {
        self.pressed_keys.clear();
    }
    
    /// Check if a specific key is currently pressed
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }
    
    /// Get a list of all currently pressed keys
    pub fn get_pressed_keys(&self) -> &[KeyCode] {
        &self.pressed_keys
    }
}

/// Manages key mappings for both controllers along with available profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMappingManager {
    /// Key mapping for controller 1
    pub controller1_mapping: KeyMapping,
    
    /// Key mapping for controller 2
    pub controller2_mapping: KeyMapping,
    
    /// Available profiles that can be selected
    pub available_profiles: Vec<ControllerProfile>,
}

impl Default for KeyMappingManager {
    fn default() -> Self {
        let default_profile = ControllerProfile::default();
        let wasd_profile = ControllerProfile::create_wasd_profile();
        
        Self {
            controller1_mapping: KeyMapping::with_profile(default_profile.clone()),
            controller2_mapping: KeyMapping::with_profile(wasd_profile.clone()),
            available_profiles: vec![default_profile, wasd_profile],
        }
    }
}

impl KeyMappingManager {
    /// Create a new key mapping manager with default configurations
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a new profile to the available profiles
    pub fn add_profile(&mut self, profile: ControllerProfile) {
        // Don't add duplicate names
        if !self.available_profiles.iter().any(|p| p.name == profile.name) {
            self.available_profiles.push(profile);
        }
    }
    
    /// Get a profile by name
    pub fn get_profile(&self, name: &str) -> Option<ControllerProfile> {
        self.available_profiles.iter()
            .find(|p| p.name == name)
            .cloned()
    }
    
    /// Set the active profile for controller 1
    pub fn set_controller1_profile(&mut self, profile_name: &str) -> Result<(), InputError> {
        if let Some(profile) = self.get_profile(profile_name) {
            self.controller1_mapping.set_profile(profile);
            Ok(())
        } else {
            Err(InputError::ProfileNotFound(profile_name.to_string()))
        }
    }
    
    /// Set the active profile for controller 2
    pub fn set_controller2_profile(&mut self, profile_name: &str) -> Result<(), InputError> {
        if let Some(profile) = self.get_profile(profile_name) {
            self.controller2_mapping.set_profile(profile);
            Ok(())
        } else {
            Err(InputError::ProfileNotFound(profile_name.to_string()))
        }
    }
    
    /// Process a key press for controller 1
    pub fn process_controller1_key_press(&mut self, key: KeyCode) -> Result<ControllerState, InputError> {
        self.controller1_mapping.process_key_press(key)
    }
    
    /// Process a key release for controller 1
    pub fn process_controller1_key_release(&mut self, key: KeyCode) -> Result<ControllerState, InputError> {
        self.controller1_mapping.process_key_release(key)
    }
    
    /// Process a key press for controller 2
    pub fn process_controller2_key_press(&mut self, key: KeyCode) -> Result<ControllerState, InputError> {
        self.controller2_mapping.process_key_press(key)
    }
    
    /// Process a key release for controller 2
    pub fn process_controller2_key_release(&mut self, key: KeyCode) -> Result<ControllerState, InputError> {
        self.controller2_mapping.process_key_release(key)
    }
    
    /// Get current state for controller 1
    pub fn get_controller1_state(&self) -> Result<ControllerState, InputError> {
        self.controller1_mapping.get_controller_state()
    }
    
    /// Get current state for controller 2
    pub fn get_controller2_state(&self) -> Result<ControllerState, InputError> {
        self.controller2_mapping.get_controller_state()
    }
    
    /// Clear all pressed keys for both controllers
    pub fn clear_all_pressed_keys(&mut self) {
        self.controller1_mapping.clear_pressed_keys();
        self.controller2_mapping.clear_pressed_keys();
    }
    
    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, InputError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| InputError::SerializationError(e.to_string()))
    }
    
    /// Deserialize from JSON
    pub fn from_json(json_data: &str) -> Result<Self, InputError> {
        serde_json::from_str(json_data)
            .map_err(|e| InputError::DeserializationError(e.to_string()))
    }
    
    /// Save to a file
    pub fn save_to_file(&self, path: &str) -> Result<(), InputError> {
        let json = self.to_json()?;
        std::fs::write(path, json)
            .map_err(|e| InputError::SerializationError(format!("Failed to write to file: {}", e)))
    }
    
    /// Load from a file
    pub fn load_from_file(path: &str) -> Result<Self, InputError> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| InputError::DeserializationError(format!("Failed to read file: {}", e)))?;
        Self::from_json(&json)
    }
}

#[cfg(test)]
mod tests {
    use rn_core::input::ControllerButton;

    use super::*;
    
    #[test]
    fn test_key_mapping_creation() -> Result<(), InputError> {
        let mapping = KeyMapping::new();
        
        // Default mappings should be set
        assert_eq!(mapping.profile.get_button_for_key(KeyCode::Z), Some(ControllerButton::A));
        assert_eq!(mapping.profile.get_button_for_key(KeyCode::X), Some(ControllerButton::B));
        assert_eq!(mapping.profile.get_button_for_key(KeyCode::Tab), Some(ControllerButton::Select));
        assert_eq!(mapping.profile.get_button_for_key(KeyCode::Enter), Some(ControllerButton::Start));
        assert_eq!(mapping.profile.get_button_for_key(KeyCode::ArrowUp), Some(ControllerButton::Up));
        assert_eq!(mapping.profile.get_button_for_key(KeyCode::ArrowDown), Some(ControllerButton::Down));
        assert_eq!(mapping.profile.get_button_for_key(KeyCode::ArrowLeft), Some(ControllerButton::Left));
        assert_eq!(mapping.profile.get_button_for_key(KeyCode::ArrowRight), Some(ControllerButton::Right));
        
        // Unknown key should have no mapping
        assert_eq!(mapping.profile.get_button_for_key(KeyCode::A), None);
        
        Ok(())
    }
    
    #[test]
    fn test_profiles() -> Result<(), InputError> {
        // Check default profile
        let default_profile = ControllerProfile::default();
        assert_eq!(default_profile.name, "Default");
        assert_eq!(default_profile.get_button_for_key(KeyCode::Z), Some(ControllerButton::A));
        
        // Check WASD profile
        let wasd_profile = ControllerProfile::create_wasd_profile();
        assert_eq!(wasd_profile.name, "WASD Layout");
        assert_eq!(wasd_profile.get_button_for_key(KeyCode::W), Some(ControllerButton::Up));
        assert_eq!(wasd_profile.get_button_for_key(KeyCode::K), Some(ControllerButton::A));
        
        Ok(())
    }
    
    #[test]
    fn test_key_press_and_release() -> Result<(), InputError> {
        let mut mapping = KeyMapping::new();
        
        // Initially no keys are pressed
        let state = mapping.get_controller_state()?;
        assert_eq!(state.is_button_pressed(ControllerButton::A), false);
        
        // Press Z key which is mapped to A button
        mapping.process_key_press(KeyCode::Z)?;
        let state = mapping.get_controller_state()?;
        assert_eq!(state.is_button_pressed(ControllerButton::A), true);
        assert_eq!(state.is_button_pressed(ControllerButton::B), false);
        
        // Press X key which is mapped to B button
        mapping.process_key_press(KeyCode::X)?;
        let state = mapping.get_controller_state()?;
        assert_eq!(state.is_button_pressed(ControllerButton::A), true);
        assert_eq!(state.is_button_pressed(ControllerButton::B), true);
        
        // Release Z key
        mapping.process_key_release(KeyCode::Z)?;
        let state = mapping.get_controller_state()?;
        assert_eq!(state.is_button_pressed(ControllerButton::A), false);
        assert_eq!(state.is_button_pressed(ControllerButton::B), true);
        
        Ok(())
    }
    
    #[test]
    fn test_key_mapping_manager() -> Result<(), InputError> {
        let mut manager = KeyMappingManager::new();
        
        // Default profile for controller 1
        let c1_state = manager.controller1_mapping.get_controller_state()?;
        assert_eq!(c1_state.is_button_pressed(ControllerButton::A), false);
        
        // WASD profile for controller 2 by default
        assert_eq!(manager.controller2_mapping.profile.name, "WASD Layout");
        
        // Test controller 1 with default profile
        manager.process_controller1_key_press(KeyCode::Z)?;
        let state = manager.get_controller1_state()?;
        assert_eq!(state.is_button_pressed(ControllerButton::A), true);
        
        // Test controller 2 with WASD profile
        manager.process_controller2_key_press(KeyCode::K)?;
        let state = manager.get_controller2_state()?;
        assert_eq!(state.is_button_pressed(ControllerButton::A), true);
        
        // Switch controller 1 to WASD profile
        manager.set_controller1_profile("WASD Layout")?;
        assert_eq!(manager.controller1_mapping.profile.name, "WASD Layout");
        
        // Now Z key should no longer trigger A button for controller 1
        let state = manager.get_controller1_state()?;
        assert_eq!(state.is_button_pressed(ControllerButton::A), false);
        
        // But K key should
        manager.process_controller1_key_press(KeyCode::K)?;
        let state = manager.get_controller1_state()?;
        assert_eq!(state.is_button_pressed(ControllerButton::A), true);
        
        Ok(())
    }
    
    #[test]
    fn test_custom_profile() -> Result<(), InputError> {
        let mut manager = KeyMappingManager::new();
        
        // Create custom profile
        let mut custom = ControllerProfile::create_default_profile("Custom");
        custom.map_key(KeyCode::Space, ControllerButton::A);
        custom.map_key(KeyCode::ControlLeft, ControllerButton::B);
        
        // Add to manager
        manager.add_profile(custom);
        
        // Set as active for controller 1
        manager.set_controller1_profile("Custom")?;
        
        // Test the custom mappings
        manager.process_controller1_key_press(KeyCode::Space)?;
        let state = manager.get_controller1_state()?;
        assert_eq!(state.is_button_pressed(ControllerButton::A), true);
        
        Ok(())
    }
    
    #[test]
    fn test_serialization() -> Result<(), InputError> {
        let mut manager = KeyMappingManager::new();
        
        // Create and add a custom profile
        let mut custom = ControllerProfile::create_default_profile("Custom");
        custom.map_key(KeyCode::Space, ControllerButton::A);
        manager.add_profile(custom.clone());
        
        // Switch to the custom profile
        manager.set_controller1_profile("Custom")?;
        
        // Serialize to JSON
        let json = manager.to_json()?;
        
        // Deserialize from JSON
        let deserialized_manager = KeyMappingManager::from_json(&json)?;
        
        // Check if the deserialized manager is equivalent
        assert_eq!(deserialized_manager.available_profiles.len(), manager.available_profiles.len());
        assert!(deserialized_manager.available_profiles.iter().any(|p| p.name == "Custom"));
        assert_eq!(deserialized_manager.controller1_mapping.profile.name, "Custom");
        
        // Verify custom mapping is preserved
        let custom_profile = deserialized_manager.get_profile("Custom").unwrap();
        assert_eq!(custom_profile.get_button_for_key(KeyCode::Space), Some(ControllerButton::A));
        
        Ok(())
    }
} 