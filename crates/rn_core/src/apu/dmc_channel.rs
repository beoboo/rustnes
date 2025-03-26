use super::length_counter::LengthCounter;

/// Represents the DMC (Delta Modulation Channel) in the APU
#[derive(Debug)]
pub struct DmcChannel {
    // Registers
    control: u8,     // $4010: IRQ enable, loop, frequency
    direct_load: u8, // $4011: Direct load value
    address: u8,     // $4012: Sample address
    length: u8,      // $4013: Sample length

    // Internal state
    enabled: bool,
    irq_enabled: bool,
    loop_flag: bool,
    sample_buffer: u8,
    sample_buffer_empty: bool,
    bits_remaining: u8,
    shift_register: u8,
    silence_flag: bool,
    current_address: u16,
    bytes_remaining: u16,
    timer: u16,
    timer_value: u16,

    // Length counter
    length_counter: LengthCounter,
}

impl DmcChannel {
    /// Create a new DMC channel
    pub fn new() -> Self {
        Self {
            // Initialize all registers to 0
            control: 0,
            direct_load: 0,
            address: 0,
            length: 0,

            // Channel initially disabled
            enabled: false,
            irq_enabled: false,
            loop_flag: false,
            sample_buffer: 0,
            sample_buffer_empty: true,
            bits_remaining: 0,
            shift_register: 0,
            silence_flag: true,
            current_address: 0,
            bytes_remaining: 0,
            timer: 0,
            timer_value: 0,

            // Initialize length counter
            length_counter: LengthCounter::new(),
        }
    }

    /// Reset the DMC channel to initial state
    pub fn reset(&mut self) {
        // Reset all registers to 0
        self.control = 0;
        self.direct_load = 0;
        self.address = 0;
        self.length = 0;

        // Reset internal state
        self.enabled = false;
        self.irq_enabled = false;
        self.loop_flag = false;
        self.sample_buffer = 0;
        self.sample_buffer_empty = true;
        self.bits_remaining = 0;
        self.shift_register = 0;
        self.silence_flag = true;
        self.current_address = 0;
        self.bytes_remaining = 0;
        self.timer = 0;
        self.timer_value = 0;

        // Reset length counter
        self.length_counter.reset();
    }

    /// Process a single DMC channel cycle
    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        // Check if timer_value is zero
        if self.timer_value == 0 {
            // Reset timer to the configured value
            self.timer_value = self.timer;

            // Process one bit if we have bits remaining
            if self.bits_remaining > 0 {
                // Get the next bit from the shift register
                let bit = (self.shift_register & 0x01) != 0;
                self.shift_register >>= 1;
                self.bits_remaining -= 1;

                // Update the sample buffer based on the bit
                if bit {
                    if self.sample_buffer <= 0x7D {
                        self.sample_buffer += 2;
                    }
                } else {
                    if self.sample_buffer >= 0x02 {
                        self.sample_buffer -= 2;
                    }
                }

                // If we've processed all bits, try to load the next byte
                if self.bits_remaining == 0 {
                    self.load_next_byte();
                }
            }
        } else {
            // Decrement timer_value
            self.timer_value -= 1;
        }
    }

    /// Load the next byte from memory
    fn load_next_byte(&mut self) {
        if self.bytes_remaining > 0 {
            // TODO: Implement memory reading through the bus
            // For now, we'll just simulate it with a dummy value
            self.sample_buffer = 0x80; // Dummy value
            self.sample_buffer_empty = false;
            self.bits_remaining = 8;
            self.shift_register = self.sample_buffer;
            self.silence_flag = false;

            // Update address and remaining bytes
            self.current_address = (self.current_address + 1) & 0x7FFF;
            self.bytes_remaining -= 1;

            // If we've reached the end and loop is enabled, restart
            if self.bytes_remaining == 0 && self.loop_flag {
                self.restart();
            }
        } else {
            // No more bytes to load
            self.sample_buffer_empty = true;
            self.silence_flag = true;
        }
    }

    /// Restart the DMC channel
    fn restart(&mut self) {
        // Reset address to the start
        self.current_address = (self.address as u16) << 6 | 0xC000;

        // Reset bytes remaining
        self.bytes_remaining = (self.length as u16) << 4 | 1;
    }

    /// Update the DMC channel timer from control register
    fn update_timer(&mut self) {
        // Timer value is based on the frequency bits (bits 0-3)
        let freq_bits = self.control & 0x0F;
        let timer_value = match freq_bits {
            0 => 428, // NTSC
            1 => 380,
            2 => 340,
            3 => 320,
            4 => 286,
            5 => 254,
            6 => 226,
            7 => 214,
            8 => 190,
            9 => 160,
            10 => 142,
            11 => 128,
            12 => 106,
            13 => 84,
            14 => 72,
            15 => 54,
            _ => 428, // Default to NTSC
        };
        self.timer = timer_value;
    }

    /// Generate a single audio sample
    pub fn generate_sample(&self) -> f32 {
        if !self.enabled || self.silence_flag {
            return 0.0;
        }
        // The DMC channel uses a 7-bit DAC, but we need to handle the full 8-bit range
        // Map 0x00-0xFF to -1.0 to 1.0, with 0x80 being 0.0
        let value = self.sample_buffer as f32;
        if value >= 128.0 {
            (value - 128.0) / 127.0
        } else {
            value / 128.0 - 1.0
        }
    }

    /// Set the enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        // If enabling the channel and no bytes remaining, restart
        if enabled && self.bytes_remaining == 0 {
            self.restart();
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
                // Control register ($4010)
                self.control = value;
                self.irq_enabled = (value & 0x80) != 0;
                self.loop_flag = (value & 0x40) != 0;
                self.update_timer();
            },
            1 => {
                // Direct load register ($4011)
                self.direct_load = value;
                self.sample_buffer = value;
                self.silence_flag = false;
            },
            2 => {
                // Address register ($4012)
                self.address = value;
            },
            3 => {
                // Length register ($4013)
                self.length = value;
                self.length_counter.load(value);
            },
            _ => panic!("Invalid DMC channel register offset: {}", register_offset),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dmc_channel() {
        let channel = DmcChannel::new();
        assert_eq!(channel.control, 0);
        assert_eq!(channel.direct_load, 0);
        assert_eq!(channel.address, 0);
        assert_eq!(channel.length, 0);
        assert_eq!(channel.enabled, false);
        assert_eq!(channel.irq_enabled, false);
        assert_eq!(channel.loop_flag, false);
        assert_eq!(channel.sample_buffer, 0);
        assert_eq!(channel.sample_buffer_empty, true);
        assert_eq!(channel.bits_remaining, 0);
        assert_eq!(channel.shift_register, 0);
        assert_eq!(channel.silence_flag, true);
        assert_eq!(channel.current_address, 0);
        assert_eq!(channel.bytes_remaining, 0);
        assert_eq!(channel.timer, 0);
        assert_eq!(channel.timer_value, 0);
    }

    #[test]
    fn test_reset() {
        let mut channel = DmcChannel::new();

        // Set some values first
        channel.control = 0x80;
        channel.direct_load = 0x42;
        channel.address = 0x30;
        channel.length = 0x20;
        channel.enabled = true;
        channel.irq_enabled = true;
        channel.loop_flag = true;
        channel.sample_buffer = 0x80;
        channel.sample_buffer_empty = false;
        channel.bits_remaining = 4;
        channel.shift_register = 0x0F;
        channel.silence_flag = false;
        channel.current_address = 0xC000;
        channel.bytes_remaining = 100;
        channel.timer = 428;
        channel.timer_value = 200;

        // Reset should clear all values
        channel.reset();

        assert_eq!(channel.control, 0);
        assert_eq!(channel.direct_load, 0);
        assert_eq!(channel.address, 0);
        assert_eq!(channel.length, 0);
        assert_eq!(channel.enabled, false);
        assert_eq!(channel.irq_enabled, false);
        assert_eq!(channel.loop_flag, false);
        assert_eq!(channel.sample_buffer, 0);
        assert_eq!(channel.sample_buffer_empty, true);
        assert_eq!(channel.bits_remaining, 0);
        assert_eq!(channel.shift_register, 0);
        assert_eq!(channel.silence_flag, true);
        assert_eq!(channel.current_address, 0);
        assert_eq!(channel.bytes_remaining, 0);
        assert_eq!(channel.timer, 0);
        assert_eq!(channel.timer_value, 0);
    }

    #[test]
    fn test_timer_update() {
        let mut channel = DmcChannel::new();

        // Test NTSC frequency (0)
        channel.write_register(0, 0x00);
        assert_eq!(channel.timer, 428);

        // Test frequency 7
        channel.write_register(0, 0x07);
        assert_eq!(channel.timer, 214);

        // Test frequency 15 (highest)
        channel.write_register(0, 0x0F);
        assert_eq!(channel.timer, 54);
    }

    #[test]
    fn test_direct_load() {
        let mut channel = DmcChannel::new();

        // Write a value to the direct load register
        channel.write_register(1, 0x80);

        // Check that the sample buffer was updated
        assert_eq!(channel.sample_buffer, 0x80);
        assert_eq!(channel.silence_flag, false);
    }

    #[test]
    fn test_sample_generation() {
        let mut channel = DmcChannel::new();

        // Channel should be silent when disabled
        assert_eq!(channel.generate_sample(), 0.0);

        // Enable the channel
        channel.set_enabled(true);

        // Should still be silent with silence flag set
        assert_eq!(channel.generate_sample(), 0.0);

        // Set a sample value and clear silence flag
        channel.sample_buffer = 0x80;
        channel.silence_flag = false;

        // Should output 0.0 (middle value)
        assert_eq!(channel.generate_sample(), 0.0);

        // Test maximum value (255)
        channel.sample_buffer = 0xFF;
        assert_eq!(channel.generate_sample(), 1.0);

        // Test minimum value (0)
        channel.sample_buffer = 0x00;
        assert_eq!(channel.generate_sample(), -1.0);
    }

    #[test]
    fn test_register_writing() {
        let mut channel = DmcChannel::new();

        // Test control register ($4010)
        channel.write_register(0, 0xC0); // Set IRQ enable and loop flag
        assert_eq!(channel.irq_enabled, true);
        assert_eq!(channel.loop_flag, true);

        // Test direct load register ($4011)
        channel.write_register(1, 0x42);
        assert_eq!(channel.sample_buffer, 0x42);
        assert_eq!(channel.silence_flag, false);

        // Test address register ($4012)
        channel.write_register(2, 0x30);
        assert_eq!(channel.address, 0x30);

        // Test length register ($4013)
        channel.write_register(3, 0x20);
        assert_eq!(channel.length, 0x20);
    }
}
