use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use rn_core::input::controller::ControllerButton;
use crate::key_mapping::KeyCode;

/// A keyboard layout profile that defines key mappings for a controller
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControllerProfile {
    /// Name of the profile
    pub name: String,
    
    /// Mapping from keyboard keys to controller buttons
    key_to_button: HashMap<KeyCode, ControllerButton>,
}

impl Default for ControllerProfile {
    fn default() -> Self {
        Self::create_default_profile("Default")
    }
}

impl ControllerProfile {
    /// Create a new profile with the given name and default key mappings
    pub fn create_default_profile(name: &str) -> Self {
        let mut profile = Self {
            name: name.to_string(),
            key_to_button: HashMap::new(),
        };
        
        // Default NES controller mapping - common configuration
        profile.key_to_button.insert(KeyCode::Z, ControllerButton::A);
        profile.key_to_button.insert(KeyCode::X, ControllerButton::B);
        profile.key_to_button.insert(KeyCode::Tab, ControllerButton::Select);
        profile.key_to_button.insert(KeyCode::Enter, ControllerButton::Start);
        profile.key_to_button.insert(KeyCode::ArrowUp, ControllerButton::Up);
        profile.key_to_button.insert(KeyCode::ArrowDown, ControllerButton::Down);
        profile.key_to_button.insert(KeyCode::ArrowLeft, ControllerButton::Left);
        profile.key_to_button.insert(KeyCode::ArrowRight, ControllerButton::Right);
        
        profile
    }
    
    /// Create an alternative keyboard layout
    pub fn create_wasd_profile() -> Self {
        let mut profile = Self {
            name: "WASD Layout".to_string(),
            key_to_button: HashMap::new(),
        };
        
        // WASD for movement
        profile.key_to_button.insert(KeyCode::W, ControllerButton::Up);
        profile.key_to_button.insert(KeyCode::A, ControllerButton::Left);
        profile.key_to_button.insert(KeyCode::S, ControllerButton::Down);
        profile.key_to_button.insert(KeyCode::D, ControllerButton::Right);
        
        // Action buttons
        profile.key_to_button.insert(KeyCode::K, ControllerButton::A);
        profile.key_to_button.insert(KeyCode::L, ControllerButton::B);
        profile.key_to_button.insert(KeyCode::Tab, ControllerButton::Select);
        profile.key_to_button.insert(KeyCode::Enter, ControllerButton::Start);
        
        profile
    }
    
    /// Map a key to a controller button
    pub fn map_key(&mut self, key: KeyCode, button: ControllerButton) {
        self.key_to_button.insert(key, button);
    }
    
    /// Remove a key mapping
    pub fn unmap_key(&mut self, key: KeyCode) {
        self.key_to_button.remove(&key);
    }
    
    /// Get the controller button mapped to a key
    pub fn get_button_for_key(&self, key: KeyCode) -> Option<ControllerButton> {
        self.key_to_button.get(&key).copied()
    }
    
    /// Get all current key mappings
    pub fn get_all_mappings(&self) -> &HashMap<KeyCode, ControllerButton> {
        &self.key_to_button
    }
    
    /// Replace all key mappings
    pub fn set_all_mappings(&mut self, mappings: HashMap<KeyCode, ControllerButton>) {
        self.key_to_button = mappings;
    }
    
    /// Get all keys mapped to a specific button
    pub fn get_keys_for_button(&self, button: ControllerButton) -> Vec<KeyCode> {
        self.key_to_button
            .iter()
            .filter_map(|(key, &btn)| if btn == button { Some(*key) } else { None })
            .collect()
    }
    
    /// Create a copy of this profile with a new name
    pub fn clone_with_name(&self, name: &str) -> Self {
        Self {
            name: name.to_string(),
            key_to_button: self.key_to_button.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_profiles() {
        // Check default profile
        let default_profile = ControllerProfile::default();
        assert_eq!(default_profile.name, "Default");
        assert_eq!(default_profile.get_button_for_key(KeyCode::Z), Some(ControllerButton::A));
        
        // Check WASD profile
        let wasd_profile = ControllerProfile::create_wasd_profile();
        assert_eq!(wasd_profile.name, "WASD Layout");
        assert_eq!(wasd_profile.get_button_for_key(KeyCode::W), Some(ControllerButton::Up));
        assert_eq!(wasd_profile.get_button_for_key(KeyCode::K), Some(ControllerButton::A));
    }
    
    #[test]
    fn test_custom_profile() {
        // Create custom profile
        let mut custom = ControllerProfile::create_default_profile("Custom");
        custom.map_key(KeyCode::Space, ControllerButton::A);
        custom.map_key(KeyCode::ControlLeft, ControllerButton::B);
        
        // Test the custom mappings
        assert_eq!(custom.get_button_for_key(KeyCode::Space), Some(ControllerButton::A));
        assert_eq!(custom.get_button_for_key(KeyCode::ControlLeft), Some(ControllerButton::B));
        
        // Test clone with name
        let cloned = custom.clone_with_name("Cloned");
        assert_eq!(cloned.name, "Cloned");
        assert_eq!(cloned.get_button_for_key(KeyCode::Space), Some(ControllerButton::A));
    }
} 