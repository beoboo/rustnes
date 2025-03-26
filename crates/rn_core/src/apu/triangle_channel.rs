use super::length_counter::LengthCounter;

/// Represents the triangle channel in the APU
#[derive(Debug)]
pub struct TriangleChannel {
    // Registers
    control: u8,  // Register 0 ($4008) - Control register
    timer_lo: u8, // Register 2 ($400A) - Timer low byte
    timer_hi: u8, // Register 3 ($400B) - Timer high byte and length counter

    // Internal state
    enabled: bool,

    // Audio generation state
    timer: u16,       // Timer value
    timer_value: u16, // Current timer countdown
    sequence_pos: u8, // Position in the triangle wave sequence
    volume: u8,       // Output volume (always max for triangle)

    // Linear counter
    linear_counter: u8,               // Linear counter value
    linear_counter_reload: u8,        // Linear counter reload value
    linear_counter_halt: bool,        // Linear counter halt flag
    linear_counter_reload_flag: bool, // Flag indicating if linear counter should be reloaded

    // Length counter
    length_counter: LengthCounter,
}

impl TriangleChannel {
    /// Create a new triangle channel
    pub fn new() -> Self {
        Self {
            // Initialize all registers to 0
            control: 0,
            timer_lo: 0,
            timer_hi: 0,

            // Disabled initially
            enabled: false,

            // Initialize audio generation state
            timer: 0,
            timer_value: 0,
            sequence_pos: 0,
            volume: 0,

            // Initialize linear counter
            linear_counter: 0,
            linear_counter_reload: 0,
            linear_counter_halt: false,
            linear_counter_reload_flag: false,

            // Initialize length counter
            length_counter: LengthCounter::new(),
        }
    }

    /// Reset the triangle channel to initial state
    pub fn reset(&mut self) {
        // Reset all registers to 0
        self.control = 0;
        self.timer_lo = 0;
        self.timer_hi = 0;

        // Disable channel
        self.enabled = false;

        // Reset audio generation state
        self.timer = 0;
        self.timer_value = 0;
        self.sequence_pos = 0;
        self.volume = 0;

        // Reset linear counter
        self.linear_counter = 0;
        self.linear_counter_reload = 0;
        self.linear_counter_halt = false;
        self.linear_counter_reload_flag = false;

        // Reset length counter
        self.length_counter.reset();
    }

    /// Process a single triangle channel cycle
    pub fn tick(&mut self) {
        if self.enabled && self.length_counter.is_active() && self.linear_counter > 0 {
            // Check if timer_value is zero first
            if self.timer_value == 0 {
                // Reset timer to the configured value and advance sequence position
                self.timer_value = self.timer;
                self.sequence_pos = (self.sequence_pos + 1) % 32;
            } else {
                // Decrement timer_value
                self.timer_value -= 1;
            }
        }
    }

    /// Process a quarter frame for linear counter (called at 240Hz rate)
    pub fn tick_linear_counter(&mut self) {
        if self.linear_counter_reload_flag {
            // Reload counter with reload value
            self.linear_counter = self.linear_counter_reload;
            // Clear the reload flag
            self.linear_counter_reload_flag = false;
        } else if self.linear_counter > 0 {
            // Only decrement if greater than 0
            self.linear_counter -= 1;
        }
    }

    /// Process a half frame for length counter (called at 120Hz rate)
    pub fn tick_length_counter(&mut self) {
        // Let the length counter handle the tick
        self.length_counter.tick();
    }

    /// Update the triangle channel timer from register values
    pub fn update_timer(&mut self) {
        // Timer value is a combination of timer_lo and timer_hi
        let timer_value = ((self.timer_hi as u16 & 0x07) << 8) | (self.timer_lo as u16);
        self.timer = timer_value;
    }

    /// Update triangle channel properties from control register
    pub fn update_properties(&mut self) {
        // Extract linear counter reload value (bits 0-6)
        self.linear_counter_reload = self.control & 0x7F;

        // Update length counter halt flag (bit 7)
        self.linear_counter_halt = (self.control & 0x80) != 0;
        self.length_counter.set_halt(self.linear_counter_halt);

        // Always reload linear counter when writing to control register
        self.linear_counter = self.linear_counter_reload;
    }

    /// Load length counter from timer high register (called when writing to register 3)
    pub fn load_length_counter(&mut self, value: u8) {
        self.length_counter.load(value);
    }

    /// Generate a single audio sample
    pub fn generate_sample(&self) -> f32 {
        if !self.enabled || !self.length_counter.is_active() || self.linear_counter == 0 {
            return 0.0;
        }

        // Triangle wave sequence (32 steps)
        // This is a simplified version - the actual NES has a more complex sequence
        // but this is sufficient for basic functionality
        let sequence: [u8; 32] = [
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        ];

        // Get the current sequence value
        let value = sequence[self.sequence_pos as usize];

        // Convert to sample value (0.0 to 1.0)
        (value as f32) / 15.0
    }

    /// Set the enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        // If disabling the channel, reset linear counter
        if !enabled {
            self.linear_counter = 0;
        }

        // Update length counter enabled state
        self.length_counter.set_enabled(enabled);
    }

    /// Check if the channel is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if the length counter is active (non-zero)
    pub fn is_length_counter_active(&self) -> bool {
        self.length_counter.is_active()
    }

    /// Write to a channel register
    pub fn write_register(&mut self, register_offset: u16, value: u8) {
        match register_offset {
            0 => {
                // Control register
                self.control = value;
                self.update_properties();
            },
            2 => {
                // Timer low register
                self.timer_lo = value;
                self.update_timer();
            },
            3 => {
                // Timer high register
                self.timer_hi = value;
                self.update_timer();

                // Writing to the timer high register loads the length counter
                self.load_length_counter(value);

                // Set flag to reload linear counter
                self.linear_counter_reload_flag = true;
            },
            _ => panic!("Invalid triangle channel register offset: {}", register_offset),
        }
    }

    #[cfg(test)]
    /// Set the sequence position for testing
    pub fn set_sequence_pos(&mut self, pos: u8) {
        self.sequence_pos = pos;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_channel_initialization() {
        let channel = TriangleChannel::new();

        // Initial values should all be zeroed
        assert_eq!(channel.control, 0);
        assert_eq!(channel.timer_lo, 0);
        assert_eq!(channel.timer_hi, 0);
        assert_eq!(channel.enabled, false);
        assert_eq!(channel.timer, 0);
        assert_eq!(channel.timer_value, 0);
        assert_eq!(channel.sequence_pos, 0);
        assert_eq!(channel.volume, 0);
        assert_eq!(channel.linear_counter, 0);
        assert_eq!(channel.linear_counter_reload, 0);
        assert_eq!(channel.linear_counter_halt, false);
    }

    #[test]
    fn test_triangle_channel_reset() {
        let mut channel = TriangleChannel::new();

        // Set some non-zero values
        channel.control = 0x80;
        channel.timer_lo = 0x42;
        channel.timer_hi = 0x01;
        channel.enabled = true;
        channel.timer = 0x0142;
        channel.timer_value = 0x42;
        channel.sequence_pos = 16;
        channel.volume = 15;
        channel.linear_counter = 7;
        channel.linear_counter_reload = 7;
        channel.linear_counter_halt = true;

        // Reset the channel
        channel.reset();

        // All values should be back to initial state
        assert_eq!(channel.control, 0);
        assert_eq!(channel.timer_lo, 0);
        assert_eq!(channel.timer_hi, 0);
        assert_eq!(channel.enabled, false);
        assert_eq!(channel.timer, 0);
        assert_eq!(channel.timer_value, 0);
        assert_eq!(channel.sequence_pos, 0);
        assert_eq!(channel.volume, 0);
        assert_eq!(channel.linear_counter, 0);
        assert_eq!(channel.linear_counter_reload, 0);
        assert_eq!(channel.linear_counter_halt, false);
    }

    #[test]
    fn test_triangle_channel_tick() {
        let mut channel = TriangleChannel::new();

        // Enable the channel
        channel.set_enabled(true);

        // Set up length counter for testing
        channel.length_counter.set_enabled(true);
        channel.length_counter.load(0 << 3); // Load length counter value

        // Set up linear counter
        channel.linear_counter = 1;
        channel.linear_counter_reload = 1;
        channel.linear_counter_halt = true;

        // Set up timer
        channel.timer = 8;
        channel.timer_value = 0;

        // First tick - timer_value is 0, so reload from timer and advance sequence
        channel.tick();
        assert_eq!(channel.timer_value, 8, "Timer value should reload from timer (8)");
        assert_eq!(channel.sequence_pos, 1, "Sequence position should advance from 0 to 1");

        // Second tick - timer decrements from 8 to 7
        channel.tick();
        assert_eq!(channel.timer_value, 7, "Timer value should decrement from 8 to 7");
        assert_eq!(channel.sequence_pos, 1, "Sequence position unchanged during decrement");

        // Third tick - timer decrements from 7 to 6
        channel.tick();
        assert_eq!(channel.timer_value, 6, "Timer value should decrement from 7 to 6");
        assert_eq!(channel.sequence_pos, 1, "Sequence position unchanged during decrement");
    }

    #[test]
    fn test_triangle_channel_linear_counter() {
        let mut channel = TriangleChannel::new();

        // Enable the channel
        channel.set_enabled(true);

        // Set up length counter for testing
        channel.length_counter.set_enabled(true);
        channel.length_counter.load(0 << 3);

        // Configure linear counter with reload value 7 and no halt
        channel.write_register(0, 0x07); // Linear counter reload = 7, halt = false
        channel.write_register(3, 0x01); // Reload linear counter

        // Linear counter should be loaded with reload value
        assert_eq!(channel.linear_counter, 7);

        // Tick linear counter - should reload from reload flag and remain at 7
        channel.tick_linear_counter();
        assert_eq!(channel.linear_counter, 7);

        // Tick again - now it should decrement
        channel.tick_linear_counter();
        assert_eq!(channel.linear_counter, 6);

        // Set halt flag
        channel.write_register(0, 0x87); // Linear counter reload = 7, halt = true

        // Tick linear counter - should still decrement
        channel.tick_linear_counter();
        assert_eq!(channel.linear_counter, 6);

        // Clear halt flag and reload
        channel.write_register(0, 0x07); // Linear counter reload = 7, halt = false
        channel.write_register(3, 0x01); // Reload linear counter

        // Linear counter should be reloaded
        assert_eq!(channel.linear_counter, 7);

        // Continue ticking until it reaches 0
        // First tick with reload flag
        channel.tick_linear_counter();

        for _ in 0..7 {
            channel.tick_linear_counter();
        }
        assert_eq!(channel.linear_counter, 0);

        // One more tick shouldn't change it
        channel.tick_linear_counter();
        assert_eq!(channel.linear_counter, 0);
    }

    #[test]
    fn test_triangle_channel_sample_generation() {
        let mut channel = TriangleChannel::new();

        // Enable the channel
        channel.set_enabled(true);

        // Set up length counter for testing
        channel.length_counter.set_enabled(true);
        channel.length_counter.load(0 << 3);

        // Set up linear counter
        channel.linear_counter = 1;
        channel.linear_counter_reload = 1;
        channel.linear_counter_halt = true;

        // Test sequence position 0 (should be 15/15)
        channel.set_sequence_pos(0);
        assert_eq!(channel.generate_sample(), 1.0);

        // Test sequence position 8 (should be 7/15)
        channel.set_sequence_pos(8);
        assert_eq!(channel.generate_sample(), 7.0 / 15.0);

        // Test sequence position 16 (should be 0/15)
        channel.set_sequence_pos(16);
        assert_eq!(channel.generate_sample(), 0.0);

        // Test sequence position 24 (should be 8/15)
        channel.set_sequence_pos(24);
        assert_eq!(channel.generate_sample(), 8.0 / 15.0);

        // Test with channel disabled
        channel.set_enabled(false);
        assert_eq!(channel.generate_sample(), 0.0);

        // Test with length counter inactive
        channel.set_enabled(true);
        channel.length_counter.set_enabled(false);
        assert_eq!(channel.generate_sample(), 0.0);

        // Test with linear counter at 0
        channel.length_counter.set_enabled(true);
        channel.linear_counter = 0;
        assert_eq!(channel.generate_sample(), 0.0);
    }

    #[test]
    fn test_triangle_channel_register_writing() {
        let mut channel = TriangleChannel::new();

        // Enable the channel
        channel.set_enabled(true);

        // Test control register (linear counter reload and halt)
        channel.write_register(0, 0x87); // Reload = 7, halt = true
        assert_eq!(channel.linear_counter_reload, 7);
        assert_eq!(channel.linear_counter_halt, true);

        // Test timer low register
        channel.write_register(2, 0x42);
        assert_eq!(channel.timer_lo, 0x42);
        assert_eq!(channel.timer, 0x0042);

        // Test timer high register
        channel.write_register(3, 0x01);
        assert_eq!(channel.timer_hi, 0x01);
        assert_eq!(channel.timer, 0x0142);

        // Test that writing to timer high reloads linear counter
        channel.write_register(0, 0x07); // Reload = 7, halt = false
        channel.write_register(3, 0x01);
        assert_eq!(channel.linear_counter, 7);
    }
}
