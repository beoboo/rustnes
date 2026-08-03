/// Length counter for NES APU channels
/// The length counter is used to limit the duration of a sound
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LengthCounter {
    // Length counter value (0 = disabled)
    counter: u8,

    // Whether the length counter is halted/disabled
    halt: bool,

    // Whether the channel is enabled at all
    enabled: bool,
}

/// The length counter load values
/// Index is the 5-bit value from the register (bits 3-7), value is the actual length
/// These values are hardcoded in the NES APU hardware
const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16,
    28, 32, 30,
];

impl LengthCounter {
    /// Create a new length counter
    pub fn new() -> Self {
        Self {
            counter: 0,
            halt: false,
            enabled: false,
        }
    }

    /// Reset the length counter to its initial state
    pub fn reset(&mut self) {
        self.counter = 0;
        self.halt = false;
        self.enabled = false;
    }

    /// Load a new length value from the timer high register (bits 3-7)
    /// This is used when writing to the 4th register of a channel ($4003, $4007, etc.)
    pub fn load(&mut self, value: u8) {
        // Only load if the channel is enabled
        if self.enabled {
            // Get the 5-bit value from the register (bits 3-7)
            let length_index = (value >> 3) & 0x1F;

            // Load the corresponding length value from the table
            self.counter = LENGTH_TABLE[length_index as usize];
        }
    }

    /// Set the halt flag (controls whether the length counter decrements)
    /// This is typically bit 5 of the first register for a channel ($4000, $4004, etc.)
    pub fn set_halt(&mut self, halt: bool) {
        self.halt = halt;
    }

    /// Set the enabled state for the channel
    /// This is controlled by the $4015 status register
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        // If disabling, clear the counter
        if !enabled {
            self.counter = 0;
        }
    }

    /// Process a half-frame clock tick (at 120Hz)
    /// Returns whether the counter reached zero after this tick
    pub fn tick(&mut self) -> bool {
        // Only decrement if counter > 0 and not halted
        if self.counter > 0 && !self.halt {
            self.counter -= 1;
        }

        // Return whether the counter is now zero
        self.counter == 0
    }

    /// Check if the length counter is non-zero
    /// This is used to determine if the channel should output sound
    pub fn is_active(&self) -> bool {
        self.counter > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_length_counter() {
        let counter = LengthCounter::new();
        assert_eq!(counter.counter, 0);
        assert!(!counter.halt);
        assert!(!counter.enabled);
    }

    #[test]
    fn test_load_length_counter() {
        let mut counter = LengthCounter::new();

        // Enable the counter
        counter.set_enabled(true);

        // Test loading different values
        // Test with index 0 (value should be 10)
        counter.load(0 << 3);
        assert_eq!(counter.counter, 10);

        // Test with index 7 (value should be 6)
        counter.load(7 << 3);
        assert_eq!(counter.counter, 6);

        // Test with index 31 (value should be 30)
        counter.load(31 << 3);
        assert_eq!(counter.counter, 30);
    }

    #[test]
    fn test_length_counter_tick() {
        let mut counter = LengthCounter::new();

        // Enable the counter and load a value
        counter.set_enabled(true);
        counter.load(0 << 3); // Value 10
        assert_eq!(counter.counter, 10);

        // Tick the counter and verify it decrements
        counter.tick();
        assert_eq!(counter.counter, 9);

        // Tick several more times
        for _ in 0..8 {
            counter.tick();
        }
        assert_eq!(counter.counter, 1);

        // One more tick should bring it to zero
        let is_zero = counter.tick();
        assert_eq!(counter.counter, 0);
        assert!(is_zero);

        // Further ticks shouldn't change it
        counter.tick();
        assert_eq!(counter.counter, 0);
    }

    #[test]
    fn test_length_counter_halt() {
        let mut counter = LengthCounter::new();

        // Enable the counter and load a value
        counter.set_enabled(true);
        counter.load(0 << 3); // Value 10

        // Set halt to true
        counter.set_halt(true);

        // Tick the counter and verify it doesn't decrement
        counter.tick();
        assert_eq!(counter.counter, 10);

        // Set halt to false
        counter.set_halt(false);

        // Now it should decrement
        counter.tick();
        assert_eq!(counter.counter, 9);
    }

    #[test]
    fn test_length_counter_disable() {
        let mut counter = LengthCounter::new();

        // Enable the counter and load a value
        counter.set_enabled(true);
        counter.load(0 << 3); // Value 10

        // Disable the counter
        counter.set_enabled(false);

        // Counter should be cleared
        assert_eq!(counter.counter, 0);

        // Re-enable and try to load
        counter.set_enabled(true);
        counter.load(0 << 3);

        // Should work now
        assert_eq!(counter.counter, 10);
    }
}
