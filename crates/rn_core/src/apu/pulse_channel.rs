/// Represents a single pulse channel in the APU
#[derive(Debug)]
pub struct PulseChannel {
    // Registers
    control: u8,
    sweep: u8,
    timer_lo: u8,
    timer_hi: u8,
    
    // Internal state
    enabled: bool,
    
    // Audio generation state
    timer: u16,
    timer_value: u16,
    duty_cycle: u8,
    duty_pos: u8,
    volume: u8,
}

impl PulseChannel {
    /// Create a new pulse channel
    pub fn new() -> Self {
        Self {
            // Initialize all registers to 0
            control: 0,
            sweep: 0,
            timer_lo: 0,
            timer_hi: 0,
            
            // Channel initially disabled
            enabled: false,
            
            // Initialize audio generation state
            timer: 0,
            timer_value: 0,
            duty_cycle: 0,
            duty_pos: 0,
            volume: 0,
        }
    }
    
    /// Reset the pulse channel to initial state
    pub fn reset(&mut self) {
        // Reset all registers to 0
        self.control = 0;
        self.sweep = 0;
        self.timer_lo = 0;
        self.timer_hi = 0;
        
        // Disable channel
        self.enabled = false;
        
        // Reset audio generation state
        self.timer = 0;
        self.timer_value = 0;
        self.duty_cycle = 0;
        self.duty_pos = 0;
        self.volume = 0;
    }
    
    /// Process a single pulse channel cycle
    pub fn tick(&mut self) {
        if self.enabled {
            if self.timer_value == 0 {
                // Reset timer
                self.timer_value = self.timer;
                
                // Update duty cycle position
                self.duty_pos = (self.duty_pos + 1) % 8;
            } else {
                // Decrement timer
                self.timer_value -= 1;
            }
        }
    }
    
    /// Update the pulse channel timer from register values
    pub fn update_timer(&mut self) {
        // Timer value is a combination of timer_lo and timer_hi
        let timer_value = ((self.timer_hi as u16 & 0x07) << 8) | (self.timer_lo as u16);
        self.timer = timer_value;
    }
    
    /// Update pulse channel properties from control register
    pub fn update_properties(&mut self) {
        // Extract duty cycle from control register (bits 6-7)
        self.duty_cycle = (self.control >> 6) & 0x03;
        
        // Extract volume from control register (bits 0-3)
        self.volume = self.control & 0x0F;
    }
    
    /// Generate a single audio sample
    pub fn generate_sample(&self) -> f32 {
        if !self.enabled || self.volume == 0 {
            return 0.0;
        }
        
        // Determine if the waveform is high or low based on duty cycle
        // Duty cycle patterns (indexed by duty_cycle value):
        // 0: 12.5% (1/8) - 00000001
        // 1: 25.0% (2/8) - 00000011
        // 2: 50.0% (4/8) - 00001111
        // 3: 75.0% (6/8) - 11111100
        let duty_table: [u8; 4] = [0b00000001, 0b00000011, 0b00001111, 0b11111100];
        
        // Make sure duty_cycle is valid
        let duty_idx = (self.duty_cycle & 0x03) as usize;
        let duty_pattern = duty_table[duty_idx];
        
        // Make sure duty_pos is valid (0-7)
        let pos = (self.duty_pos & 0x07) as usize;
        
        // Check if the current duty position bit is set in the pattern
        let is_active = ((duty_pattern >> pos) & 0x01) != 0;
        
        // Output sample if active
        if is_active {
            // Convert volume (0-15) to sample (0.0 to 1.0)
            (self.volume as f32) / 15.0
        } else {
            0.0
        }
    }
    
    /// Set the enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if the channel is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Write to a channel register
    pub fn write_register(&mut self, register_offset: u16, value: u8) {
        match register_offset {
            0 => { // Control register
                self.control = value;
                self.update_properties();
            },
            1 => { // Sweep register
                self.sweep = value;
            },
            2 => { // Timer low register
                self.timer_lo = value;
                self.update_timer();
            },
            3 => { // Timer high register
                self.timer_hi = value;
                self.update_timer();
            },
            _ => panic!("Invalid pulse channel register offset: {}", register_offset),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pulse_channel() {
        let channel = PulseChannel::new();
        assert_eq!(channel.control, 0);
        assert_eq!(channel.sweep, 0);
        assert_eq!(channel.timer_lo, 0);
        assert_eq!(channel.timer_hi, 0);
        assert_eq!(channel.enabled, false);
        assert_eq!(channel.timer, 0);
        assert_eq!(channel.timer_value, 0);
        assert_eq!(channel.duty_cycle, 0);
        assert_eq!(channel.duty_pos, 0);
        assert_eq!(channel.volume, 0);
    }

    #[test]
    fn test_reset() {
        let mut channel = PulseChannel::new();
        
        // Set some values first
        channel.control = 0x40;
        channel.sweep = 0x80;
        channel.timer_lo = 0x42;
        channel.timer_hi = 0x03;
        channel.enabled = true;
        channel.timer = 800;
        channel.timer_value = 500;
        channel.duty_cycle = 2;
        channel.duty_pos = 3;
        channel.volume = 10;
        
        // Reset should clear all values
        channel.reset();
        
        assert_eq!(channel.control, 0);
        assert_eq!(channel.sweep, 0);
        assert_eq!(channel.timer_lo, 0);
        assert_eq!(channel.timer_hi, 0);
        assert_eq!(channel.enabled, false);
        assert_eq!(channel.timer, 0);
        assert_eq!(channel.timer_value, 0);
        assert_eq!(channel.duty_cycle, 0);
        assert_eq!(channel.duty_pos, 0);
        assert_eq!(channel.volume, 0);
    }

    #[test]
    fn test_duty_cycle_extraction() {
        let mut channel = PulseChannel::new();
        
        // Test duty cycle 0 (12.5%)
        channel.write_register(0, 0b00001111); // Bits 6-7 = 00
        assert_eq!(channel.duty_cycle, 0);
        
        // Test duty cycle 1 (25%)
        channel.write_register(0, 0b01001111); // Bits 6-7 = 01
        assert_eq!(channel.duty_cycle, 1);
        
        // Test duty cycle 2 (50%)
        channel.write_register(0, 0b10001111); // Bits 6-7 = 10
        assert_eq!(channel.duty_cycle, 2);
        
        // Test duty cycle 3 (75%)
        channel.write_register(0, 0b11001111); // Bits 6-7 = 11
        assert_eq!(channel.duty_cycle, 3);
    }

    #[test]
    fn test_volume_extraction() {
        let mut channel = PulseChannel::new();
        
        // Test volume 0
        channel.write_register(0, 0b01000000);
        assert_eq!(channel.volume, 0);
        
        // Test volume 7
        channel.write_register(0, 0b01000111);
        assert_eq!(channel.volume, 7);
        
        // Test volume 15 (maximum)
        channel.write_register(0, 0b01001111);
        assert_eq!(channel.volume, 15);
    }

    #[test]
    fn test_enable_disable() {
        let mut channel = PulseChannel::new();
        
        // Default is disabled
        assert_eq!(channel.is_enabled(), false);
        
        // Enable
        channel.set_enabled(true);
        assert_eq!(channel.is_enabled(), true);
        
        // Disable
        channel.set_enabled(false);
        assert_eq!(channel.is_enabled(), false);
    }

    #[test]
    fn test_timer_update() {
        let mut channel = PulseChannel::new();
        
        // Set timer to 1000 (0x03E8)
        channel.write_register(2, 0xE8); // Low byte
        channel.write_register(3, 0x03); // High byte (only lowest 3 bits used)
        
        assert_eq!(channel.timer_lo, 0xE8);
        assert_eq!(channel.timer_hi, 0x03);
        assert_eq!(channel.timer, 0x03E8); // 1000 in decimal
        
        // Set timer to 255 (0x00FF)
        channel.write_register(2, 0xFF); // Low byte
        channel.write_register(3, 0x00); // High byte
        
        assert_eq!(channel.timer_lo, 0xFF);
        assert_eq!(channel.timer_hi, 0x00);
        assert_eq!(channel.timer, 0xFF); // 255 in decimal
        
        // Test high byte masking (only lowest 3 bits used)
        channel.write_register(2, 0x42); // Low byte
        channel.write_register(3, 0xF8); // High byte (0xF8 = 0b11111000, only 0b000 should be used)
        
        assert_eq!(channel.timer_lo, 0x42);
        assert_eq!(channel.timer_hi, 0xF8);
        assert_eq!(channel.timer, 0x0042); // Only 0x0042, high bits are masked
    }

    #[test]
    fn test_tick_with_disabled_channel() {
        let mut channel = PulseChannel::new();
        
        // Set up channel but leave it disabled
        channel.write_register(2, 0x10); // Timer low
        channel.write_register(3, 0x00); // Timer high
        channel.timer_value = 5;
        
        // Tick the channel - nothing should change since it's disabled
        channel.tick();
        
        assert_eq!(channel.timer_value, 5); // Should remain unchanged
        assert_eq!(channel.duty_pos, 0);    // Should remain unchanged
    }

    #[test]
    fn test_tick_with_enabled_channel() {
        let mut channel = PulseChannel::new();
        
        // Set up channel and enable it
        channel.write_register(2, 0x10); // Timer low byte = 16
        channel.write_register(3, 0x00); // Timer high byte = 0
        channel.set_enabled(true);
        channel.timer_value = 3;
        
        // First tick - should decrement timer_value
        channel.tick();
        assert_eq!(channel.timer_value, 2);
        assert_eq!(channel.duty_pos, 0); // No change to duty position yet
        
        // Second tick
        channel.tick();
        assert_eq!(channel.timer_value, 1);
        
        // Third tick
        channel.tick();
        assert_eq!(channel.timer_value, 0);
        
        // Fourth tick - timer_value is 0, should reset to timer value and increment duty_pos
        channel.tick();
        assert_eq!(channel.timer_value, channel.timer); // Should reset to timer (16)
        assert_eq!(channel.duty_pos, 1); // Should increment
        
        // Continue ticking to see if duty_pos wraps around correctly
        for _ in 0..7 {
            while channel.timer_value > 0 {
                channel.tick();
            }
            channel.tick(); // This tick should increment duty_pos
        }
        
        assert_eq!(channel.duty_pos, 0); // Should have wrapped around to 0
    }

    #[test]
    fn test_sample_generation_disabled() {
        let channel = PulseChannel::new();
        
        // When disabled, should always return 0
        assert_eq!(channel.generate_sample(), 0.0);
    }

    #[test]
    fn test_sample_generation_12_5_percent_duty() {
        let mut channel = PulseChannel::new();
        
        // Set up a 12.5% duty cycle (duty_cycle = 0)
        channel.write_register(0, 0b00001111); // Duty cycle = 0, volume = 15
        channel.set_enabled(true);
        
        // Duty pattern for 12.5% duty cycle: 00000001
        
        // Test all 8 positions
        channel.duty_pos = 0;
        assert_eq!(channel.generate_sample(), 1.0); // Position 0 is on
        
        channel.duty_pos = 1;
        assert_eq!(channel.generate_sample(), 0.0); // Positions 1-7 are off
        
        channel.duty_pos = 2;
        assert_eq!(channel.generate_sample(), 0.0);
        
        channel.duty_pos = 3;
        assert_eq!(channel.generate_sample(), 0.0);
        
        channel.duty_pos = 4;
        assert_eq!(channel.generate_sample(), 0.0);
        
        channel.duty_pos = 5;
        assert_eq!(channel.generate_sample(), 0.0);
        
        channel.duty_pos = 6;
        assert_eq!(channel.generate_sample(), 0.0);
        
        channel.duty_pos = 7;
        assert_eq!(channel.generate_sample(), 0.0);
    }

    #[test]
    fn test_sample_generation_volume_scaling() {
        let mut channel = PulseChannel::new();
        
        // Set up a 25% duty cycle (duty_cycle = 1) with position where output is active
        channel.write_register(0, 0b01000000); // Duty cycle = 1, volume = 0
        channel.set_enabled(true);
        channel.duty_pos = 0; // Position where output is active
        
        // Test different volumes
        
        // Volume 0 (silent)
        channel.write_register(0, 0b01000000);
        assert_eq!(channel.generate_sample(), 0.0);
        
        // Volume 1 (minimum)
        channel.write_register(0, 0b01000001);
        assert_eq!(channel.generate_sample(), 1.0/15.0);
        
        // Volume 8 (mid)
        channel.write_register(0, 0b01001000);
        assert_eq!(channel.generate_sample(), 8.0/15.0);
        
        // Volume 15 (maximum)
        channel.write_register(0, 0b01001111);
        assert_eq!(channel.generate_sample(), 1.0);
    }
} 