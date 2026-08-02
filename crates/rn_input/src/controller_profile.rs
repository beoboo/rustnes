use std::collections::HashMap;

use rn_core::input::controller::ControllerButton;
use serde::{Deserialize, Serialize};

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
        // Not Tab, however conventional that is for Select. The debugger is a GUI, and its
        // toolkit claims Tab to move focus between widgets before the application sees any input —
        // so the key never reaches the game. Super Mario Bros 3's title screen uses Select to
        // choose between one and two players, which is simply impossible with Tab bound here.
        profile.key_to_button.insert(KeyCode::ShiftRight, ControllerButton::Select);
        profile.key_to_button.insert(KeyCode::Enter, ControllerButton::Start);
        profile.key_to_button.insert(KeyCode::ArrowUp, ControllerButton::Up);
        profile.key_to_button.insert(KeyCode::ArrowDown, ControllerButton::Down);
        profile.key_to_button.insert(KeyCode::ArrowLeft, ControllerButton::Left);
        profile
            .key_to_button
            .insert(KeyCode::ArrowRight, ControllerButton::Right);

        profile
    }

    /// A profile binding both common layouts at once.
    ///
    /// Arrow keys *and* WASD for the d-pad, Z/X *and* K/L for A/B. A player should not have to
    /// discover which of two conventions a build happens to use — binding both means whichever
    /// they reach for works, and nothing is lost by accepting the other.
    pub fn create_combined_profile() -> Self {
        let mut profile = Self::create_default_profile("Arrows + WASD");

        profile.key_to_button.insert(KeyCode::W, ControllerButton::Up);
        profile.key_to_button.insert(KeyCode::A, ControllerButton::Left);
        profile.key_to_button.insert(KeyCode::S, ControllerButton::Down);
        profile.key_to_button.insert(KeyCode::D, ControllerButton::Right);

        profile.key_to_button.insert(KeyCode::K, ControllerButton::A);
        profile.key_to_button.insert(KeyCode::L, ControllerButton::B);

        // Shift is a common alternative for Select, which Tab alone makes awkward in a windowed
        // application where Tab may move focus.
        profile.key_to_button.insert(KeyCode::ShiftRight, ControllerButton::Select);
        profile.key_to_button.insert(KeyCode::Space, ControllerButton::Start);

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
        // Not Tab, however conventional that is for Select. The debugger is a GUI, and its
        // toolkit claims Tab to move focus between widgets before the application sees any input —
        // so the key never reaches the game. Super Mario Bros 3's title screen uses Select to
        // choose between one and two players, which is simply impossible with Tab bound here.
        profile.key_to_button.insert(KeyCode::ShiftRight, ControllerButton::Select);
        profile.key_to_button.insert(KeyCode::Enter, ControllerButton::Start);

        profile
    }

    /// Every key bound to `button`, sorted so the listing is stable between frames.
    ///
    /// Used to show the player what is actually bound rather than making them guess.
    pub fn keys_for(&self, button: ControllerButton) -> Vec<KeyCode> {
        let mut keys: Vec<KeyCode> = self
            .key_to_button
            .iter()
            .filter(|(_, mapped)| **mapped == button)
            .map(|(key, _)| *key)
            .collect();
        keys.sort_by_key(|key| key.to_str());
        keys
    }

    /// This profile's name.
    pub fn name(&self) -> &str {
        &self.name
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
    /// No profile may bind Tab.
    ///
    /// Tab is the conventional key for Select, and it does not work: a windowed application's
    /// toolkit takes Tab to move focus between widgets before the application is given any input,
    /// so the press never reaches the game. The symptom is specific and baffling — Super Mario
    /// Bros 3's title screen uses Select to choose between one and two players, so two-player mode
    /// simply cannot be selected, while every other button behaves.
    #[test]
    fn no_profile_binds_tab() {
        let profiles = [
            ("default", ControllerProfile::create_default_profile("default")),
            ("combined", ControllerProfile::create_combined_profile()),
            ("wasd", ControllerProfile::create_wasd_profile()),
        ];

        for (name, profile) in profiles {
            assert!(
                !profile.key_to_button.contains_key(&KeyCode::Tab),
                "the {name} profile binds Tab, which the window toolkit consumes first"
            );
        }
    }

    /// Every button must be reachable, or part of a game becomes unplayable without it being
    /// obvious which key is missing.
    #[test]
    fn every_button_is_bound_in_every_profile() {
        use ControllerButton::*;
        let wanted = [A, B, Select, Start, Up, Down, Left, Right];

        for (name, profile) in [
            ("default", ControllerProfile::create_default_profile("default")),
            ("combined", ControllerProfile::create_combined_profile()),
            ("wasd", ControllerProfile::create_wasd_profile()),
        ] {
            for button in wanted {
                assert!(
                    profile.key_to_button.values().any(|bound| *bound == button),
                    "the {name} profile has no key for {button:?}"
                );
            }
        }
    }

    use super::*;

    #[test]
    fn test_profiles() {
        // Check default profile
        let default_profile = ControllerProfile::default();
        assert_eq!(default_profile.name, "Default");
        assert_eq!(
            default_profile.get_button_for_key(KeyCode::Z),
            Some(ControllerButton::A)
        );

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
        assert_eq!(
            custom.get_button_for_key(KeyCode::ControlLeft),
            Some(ControllerButton::B)
        );

        // Test clone with name
        let cloned = custom.clone_with_name("Cloned");
        assert_eq!(cloned.name, "Cloned");
        assert_eq!(cloned.get_button_for_key(KeyCode::Space), Some(ControllerButton::A));
    }

    /// Every button must be reachable, or a game becomes unplayable in a way that looks like an
    /// emulation bug rather than a missing binding.
    #[test]
    fn the_combined_profile_binds_every_button() {
        let profile = ControllerProfile::create_combined_profile();

        for button in [
            ControllerButton::Up,
            ControllerButton::Down,
            ControllerButton::Left,
            ControllerButton::Right,
            ControllerButton::A,
            ControllerButton::B,
            ControllerButton::Start,
            ControllerButton::Select,
        ] {
            assert!(
                !profile.keys_for(button).is_empty(),
                "{button:?} has no key bound to it"
            );
        }
    }

    /// The point of the combined profile: a player should not have to discover which of two
    /// conventions this build uses.
    #[test]
    fn the_combined_profile_accepts_arrows_and_wasd() {
        let profile = ControllerProfile::create_combined_profile();

        let up = profile.keys_for(ControllerButton::Up);
        assert!(up.contains(&KeyCode::ArrowUp), "arrow keys should work");
        assert!(up.contains(&KeyCode::W), "WASD should work too");

        let a = profile.keys_for(ControllerButton::A);
        assert!(a.contains(&KeyCode::Z) && a.contains(&KeyCode::K));
    }

    #[test]
    fn both_stock_profiles_bind_every_button() {
        for profile in [
            ControllerProfile::create_default_profile("Default"),
            ControllerProfile::create_wasd_profile(),
        ] {
            for button in [
                ControllerButton::Up,
                ControllerButton::Down,
                ControllerButton::Left,
                ControllerButton::Right,
                ControllerButton::A,
                ControllerButton::B,
                ControllerButton::Start,
                ControllerButton::Select,
            ] {
                assert!(
                    !profile.keys_for(button).is_empty(),
                    "{} leaves {button:?} unbound",
                    profile.name()
                );
            }
        }
    }

}
