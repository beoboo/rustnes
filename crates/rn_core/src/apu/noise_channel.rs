use super::{envelope::Envelope, length_counter::LengthCounter};

/// Represents the noise channel in the APU
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NoiseChannel {
    // Registers
    control: u8,
    timer_lo: u8,
    timer_hi: u8,

    // Internal state
    enabled: bool,

    // Audio generation state
    timer: u16,
    timer_value: u16,
    shift_register: u16,
    mode: bool,

    // Envelope generator
    envelope: Envelope,

    // Length counter
    length_counter: LengthCounter,
}

/// Timer period lookup table for noise channel
const TIMER_PERIOD: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

impl NoiseChannel {
    /// Create a new noise channel
    pub fn new() -> Self {
        Self {
            // Initialize all registers to 0
            control: 0,
            timer_lo: 0,
            timer_hi: 0,

            // Channel initially disabled
            enabled: false,

            // Initialize audio generation state
            timer: 0,
            timer_value: 0,
            shift_register: 1, // Initialize to 1 (non-zero)
            mode: false,

            // Initialize envelope generator
            envelope: Envelope::new(),

            // Initialize length counter
            length_counter: LengthCounter::new(),
        }
    }

    /// Reset the noise channel to initial state
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
        self.shift_register = 1; // Initialize to 1 (non-zero)
        self.mode = false;

        // Reset envelope generator
        self.envelope.reset();

        // Reset length counter
        self.length_counter.reset();
    }

    /// Process a single noise channel cycle
    pub fn tick(&mut self) {
        if self.enabled && self.length_counter.is_active() {
            // Check if timer_value is zero first
            if self.timer_value == 0 {
                // Reset timer to the configured value
                self.timer_value = self.timer;

                // Shift the register right by 1
                let bit0 = self.shift_register & 1;
                self.shift_register >>= 1;

                // XOR bit0 with bit1 (or bit6 in mode 1)
                let bit1 = if self.mode {
                    (self.shift_register >> 5) & 1
                } else {
                    (self.shift_register >> 1) & 1
                };
                let feedback = bit0 ^ bit1;

                // Set the feedback bit (bit 14)
                self.shift_register |= feedback << 14;
            } else {
                // Decrement timer_value
                self.timer_value -= 1;
            }
        }
    }

    /// Process a quarter frame for envelope (called at 240Hz rate)
    pub fn tick_envelope(&mut self) {
        // Let the envelope handle the tick
        self.envelope.tick();
    }

    /// Process a half frame for length counter (called at 120Hz rate)
    pub fn tick_length_counter(&mut self) {
        // Let the length counter handle the tick
        self.length_counter.tick();
    }

    /// Update the noise channel timer from register values
    pub fn update_timer(&mut self) {
        // Timer period is looked up from a table based on the low 4 bits of timer_lo
        let period_index = self.timer_lo & 0x0F;
        self.timer = TIMER_PERIOD[period_index as usize];
    }

    /// Update noise channel properties from control register
    pub fn update_properties(&mut self) {
        // Update envelope generator with control register
        self.envelope.update_from_register(self.control);

        // Update length counter halt flag (bit 5)
        self.length_counter.set_halt((self.control & 0x20) != 0);
    }

    /// Load length counter from timer high register (called when writing to register 3)
    pub fn load_length_counter(&mut self, value: u8) {
        self.length_counter.load(value);
    }

    /// The channel's current DAC level, 0..=15. See [`PulseChannel::output`](super::PulseChannel).
    pub fn output(&self) -> u8 {
        if !self.enabled || !self.length_counter.is_active() {
            return 0;
        }

        // Bit 0 of the shift register gates the envelope volume.
        if self.shift_register & 1 != 0 {
            self.envelope.get_volume()
        } else {
            0
        }
    }

    /// Set the enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        // If disabling the channel, reset envelope state
        if !enabled {
            self.envelope = Envelope::new();
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
            1 => {
                // Mode register
                self.mode = (value & 0x80) != 0;
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

                // Writing to the timer high register restarts the envelope generator
                self.envelope.restart();
            },
            // Writes to registers the hardware does not use are simply ignored — $400D is unused on hardware.
            // A ROM writing one must not bring the emulator down: blargg's instr_test-v5 does
            // exactly this, and it used to panic.
            _ => {},
        }
    }

    #[cfg(test)]
    /// Set the shift register value for testing
    pub fn set_shift_register(&mut self, value: u16) {
        self.shift_register = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_channel_new() {
        let channel = NoiseChannel::new();
        assert_eq!(channel.control, 0);
        assert_eq!(channel.timer_lo, 0);
        assert_eq!(channel.timer_hi, 0);
        assert!(!channel.enabled);
        assert_eq!(channel.timer, 0);
        assert_eq!(channel.timer_value, 0);
        assert_eq!(channel.shift_register, 1);
        assert!(!channel.mode);
    }

    #[test]
    fn test_noise_channel_reset() {
        let mut channel = NoiseChannel::new();

        // Set some non-default values
        channel.control = 0xFF;
        channel.timer_lo = 0xAA;
        channel.timer_hi = 0x55;
        channel.enabled = true;
        channel.timer = 0x1234;
        channel.timer_value = 0x5678;
        channel.shift_register = 0xABCD;
        channel.mode = true;

        // Reset the channel
        channel.reset();

        // Check that values are back to defaults
        assert_eq!(channel.control, 0);
        assert_eq!(channel.timer_lo, 0);
        assert_eq!(channel.timer_hi, 0);
        assert!(!channel.enabled);
        assert_eq!(channel.timer, 0);
        assert_eq!(channel.timer_value, 0);
        assert_eq!(channel.shift_register, 1);
        assert!(!channel.mode);
    }

    #[test]
    fn test_noise_channel_tick() {
        let mut channel = NoiseChannel::new();

        // Enable the channel
        channel.set_enabled(true);
        channel.length_counter.set_enabled(true);
        channel.length_counter.load(0 << 3); // Load length counter value

        // Set a short timer for testing
        channel.write_register(2, 0x01); // Timer low
        channel.write_register(3, 0x00); // Timer high

        // Set initial shift register value
        channel.set_shift_register(0x4001);

        // Tick the channel until the shift register is updated
        while channel.timer_value > 0 {
            channel.tick();
        }

        // Check that the shift register was updated correctly
        assert_eq!(channel.shift_register & 1, 1); // Original bit0
        assert_eq!((channel.shift_register >> 14) & 1, 1); // Feedback bit
    }

    #[test]
    fn test_noise_channel_mode_1() {
        let mut channel = NoiseChannel::new();

        // Enable the channel
        channel.set_enabled(true);
        channel.length_counter.set_enabled(true);
        channel.length_counter.load(0 << 3);

        // Set mode 1 (bit 7 of register 1)
        channel.write_register(1, 0x80);

        // Set a short timer for testing
        channel.write_register(2, 0x01);
        channel.write_register(3, 0x00);

        // Set initial shift register value
        channel.set_shift_register(0x4001);

        // Tick the channel until the shift register is updated
        while channel.timer_value > 0 {
            channel.tick();
        }

        // Check that the shift register was updated correctly with mode 1
        assert_eq!(channel.shift_register & 1, 1); // Original bit0
        assert_eq!((channel.shift_register >> 14) & 1, 1); // Feedback bit
    }

    #[test]
    fn test_noise_channel_sample_generation() {
        let mut channel = NoiseChannel::new();

        // Enable the channel
        channel.set_enabled(true);
        channel.length_counter.set_enabled(true);
        channel.length_counter.load(0 << 3);

        // Set maximum volume
        channel.write_register(0, 0b00011111); // Constant volume (15)

        // Set shift register to 1 for testing
        channel.set_shift_register(1);

        // Should generate maximum volume sample
        assert_eq!(channel.output(), 15);

        // Set shift register to 0
        channel.set_shift_register(0);

        // Should generate zero sample
        assert_eq!(channel.output(), 0);
    }

    #[test]
    fn test_noise_channel_length_counter() {
        let mut channel = NoiseChannel::new();

        // Enable the channel
        channel.set_enabled(true);

        // Load a value into the length counter via register 3
        // Use length index 7 (value should be 6)
        channel.write_register(3, 7 << 3);

        // Length counter should be active
        assert!(channel.is_length_counter_active());

        // Check if sound is produced
        channel.write_register(0, 0b00011111); // Constant volume (15)
        channel.set_shift_register(1); // Set for sound output
        assert!(channel.output() > 0);

        // Disable the channel
        channel.set_enabled(false);

        // Length counter should now be inactive
        assert!(!channel.is_length_counter_active());

        // Sound should be muted
        assert_eq!(channel.output(), 0);
    }

    #[test]
    fn test_noise_channel_length_counter_decrement() {
        let mut channel = NoiseChannel::new();

        // Enable the channel
        channel.set_enabled(true);

        // Load a short length (index 0 = value 10)
        channel.write_register(3, 0 << 3);
        assert!(channel.is_length_counter_active());

        // Tick length counter 9 times
        for _ in 0..9 {
            channel.tick_length_counter();
        }

        // Should still be active
        assert!(channel.is_length_counter_active());

        // One more tick should silence it
        channel.tick_length_counter();
        assert!(!channel.is_length_counter_active());

        // Sound should now be muted even if the channel is enabled
        channel.write_register(0, 0b00011111); // Constant volume (15)
        channel.set_shift_register(1);
        assert_eq!(channel.output(), 0);
    }

    #[test]
    fn test_noise_channel_length_counter_halt() {
        let mut channel = NoiseChannel::new();

        // Enable the channel
        channel.set_enabled(true);

        // Load a short length (index 0 = value 10)
        channel.write_register(3, 0 << 3);

        // Set length counter halt flag (bit 5 of control register)
        channel.write_register(0, 0b00100000);

        // Tick multiple times - length counter should not decrement due to halt
        for _ in 0..20 {
            channel.tick_length_counter();
        }

        // Should still be active because halt prevented decrement
        assert!(channel.is_length_counter_active());

        // Clear halt flag
        channel.write_register(0, 0b00000000);

        // Now ticking should decrement
        for _ in 0..10 {
            channel.tick_length_counter();
        }

        // Should now be inactive after enough ticks
        assert!(!channel.is_length_counter_active());
    }
}
