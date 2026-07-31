use super::{envelope::Envelope, length_counter::LengthCounter, sweep::Sweep};

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

    // Envelope generator
    envelope: Envelope,

    // Sweep unit
    sweep_unit: Sweep,

    // Length counter
    length_counter: LengthCounter,
}

impl PulseChannel {
    /// Create a new pulse channel
    pub fn new(is_pulse1: bool) -> Self {
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

            // Initialize envelope generator
            envelope: Envelope::new(),

            // Initialize sweep unit
            sweep_unit: Sweep::new(is_pulse1),

            // Initialize length counter
            length_counter: LengthCounter::new(),
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

        // Reset envelope generator
        self.envelope.reset();

        // Reset sweep unit
        self.sweep_unit.reset();

        // Reset length counter
        self.length_counter.reset();
    }

    /// Process a single pulse channel cycle
    pub fn tick(&mut self) {
        if self.enabled && self.length_counter.is_active() {
            // Only output sound if the sweep unit is not muting the channel
            let target_period = self.sweep_unit.calculate_target_period(self.timer);
            let should_mute = self.sweep_unit.should_mute(self.timer, target_period);

            if !should_mute {
                // Check if timer_value is zero first
                if self.timer_value == 0 {
                    // Reset timer to the configured value and advance duty position
                    self.timer_value = self.timer;
                    self.duty_pos = (self.duty_pos + 1) % 8;
                } else {
                    // Decrement timer_value
                    self.timer_value -= 1;
                }
            }
        }
    }

    /// Process a quarter frame for envelope (called at 240Hz rate)
    pub fn tick_envelope(&mut self) {
        // Let the envelope handle the tick
        self.envelope.tick();

        // Update the volume if in envelope mode
        if !self.envelope.is_constant_volume() {
            self.volume = self.envelope.get_envelope_volume();
        }
    }

    /// Process a half frame for sweep and length counter (called at 120Hz rate)
    pub fn tick_sweep(&mut self) {
        // Let the sweep unit handle the tick
        if let Some(new_period) = self.sweep_unit.tick(self.timer) {
            // Update the timer with the new period
            self.timer = new_period;

            // Update timer registers to match (for accurate emulation)
            self.timer_lo = (self.timer & 0xFF) as u8;
            self.timer_hi = ((self.timer >> 8) & 0x07) as u8;
        }
    }

    /// Process a half frame for length counter (called at 120Hz rate)
    pub fn tick_length_counter(&mut self) {
        // Let the length counter handle the tick
        self.length_counter.tick();
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

        // Update envelope generator with control register
        self.envelope.update_from_register(self.control);

        // Update length counter halt flag (bit 5)
        self.length_counter.set_halt((self.control & 0x20) != 0);

        // Update volume from envelope
        self.volume = self.envelope.get_volume();
    }

    /// Restart the envelope generator
    pub fn restart_envelope(&mut self) {
        self.envelope.restart();
    }

    /// Load length counter from timer high register (called when writing to register 3)
    pub fn load_length_counter(&mut self, value: u8) {
        self.length_counter.load(value);
    }

    /// The channel's current DAC level, 0..=15.
    ///
    /// This is the raw value the hardware feeds to its mixer, not a normalised float: the NES
    /// mixes non-linearly, so scaling here would make a correct mix impossible. See
    /// [`Apu::mix`](super::Apu) for where these levels are combined.
    pub fn output(&self) -> u8 {
        if !self.enabled || !self.length_counter.is_active() {
            return 0;
        }

        if self
            .sweep_unit
            .should_mute(self.timer, self.sweep_unit.calculate_target_period(self.timer))
        {
            return 0;
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

        // Output the current volume when the duty pattern is high, silence otherwise.
        // `get_volume` handles both constant-volume and envelope modes.
        if is_active {
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
            self.volume = 0;
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
                // Update volume immediately when control register changes
                self.volume = self.envelope.get_volume();
            },
            1 => {
                // Sweep register
                self.sweep = value;
                self.sweep_unit.update_from_register(value);
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

                // Writing to the timer high register restarts the envelope generator
                self.restart_envelope();

                // Writing to the timer high register also loads the length counter
                self.load_length_counter(value);
            },
            _ => panic!("Invalid pulse channel register offset: {}", register_offset),
        }
    }

    #[cfg(test)]
    /// Set the duty position for testing
    pub fn set_duty_pos(&mut self, pos: u8) {
        self.duty_pos = pos;
    }

    #[cfg(test)]
    /// Current position in the 8-step duty sequence, for timing tests.
    pub fn duty_pos(&self) -> u8 {
        self.duty_pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pulse_channel() {
        let channel = PulseChannel::new(true);
        assert_eq!(channel.control, 0);
        assert_eq!(channel.sweep, 0);
        assert_eq!(channel.timer_lo, 0);
        assert_eq!(channel.timer_hi, 0);
        assert!(!channel.enabled);
        assert_eq!(channel.timer, 0);
        assert_eq!(channel.timer_value, 0);
        assert_eq!(channel.duty_cycle, 0);
        assert_eq!(channel.duty_pos, 0);
        assert_eq!(channel.volume, 0);
    }

    #[test]
    fn test_reset() {
        let mut channel = PulseChannel::new(true);

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
        assert!(!channel.enabled);
        assert_eq!(channel.timer, 0);
        assert_eq!(channel.timer_value, 0);
        assert_eq!(channel.duty_cycle, 0);
        assert_eq!(channel.duty_pos, 0);
        assert_eq!(channel.volume, 0);
    }

    #[test]
    fn test_duty_cycle_extraction() {
        let mut channel = PulseChannel::new(true);

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
        let mut channel = PulseChannel::new(true);

        // Test volume 0
        channel.write_register(0, 0b01010000);
        assert_eq!(channel.envelope.period, 0);
        assert_eq!(channel.volume, 0);

        // Test volume 7
        channel.write_register(0, 0b01010111);
        assert_eq!(channel.envelope.period, 7);
        assert_eq!(channel.volume, 7);

        // Test volume 15 (maximum)
        channel.write_register(0, 0b01011111);
        assert_eq!(channel.envelope.period, 15);
        assert_eq!(channel.volume, 15);
    }

    #[test]
    fn test_enable_disable() {
        let mut channel = PulseChannel::new(true);

        // Default is disabled
        assert!(!channel.is_enabled());

        // Enable
        channel.set_enabled(true);
        assert!(channel.is_enabled());

        // Disable
        channel.set_enabled(false);
        assert!(!channel.is_enabled());
    }

    #[test]
    fn test_timer_update() {
        let mut channel = PulseChannel::new(true);

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
        let mut channel = PulseChannel::new(true);

        // Set up channel but leave it disabled
        channel.write_register(2, 0x10); // Timer low
        channel.write_register(3, 0x00); // Timer high
        channel.timer_value = 5;

        // Tick the channel - nothing should change since it's disabled
        channel.tick();

        assert_eq!(channel.timer_value, 5); // Should remain unchanged
        assert_eq!(channel.duty_pos, 0); // Should remain unchanged
    }

    #[test]
    fn test_tick_with_enabled_channel() {
        let mut channel = PulseChannel::new(true);

        // Configure the channel with proper NES timing
        channel.enabled = true;
        channel.timer = 8; // Timer value = 8 (minimum value to avoid sweep muting)
        channel.timer_value = 0; // Start with timer_value = 0 to test reload
        channel.duty_pos = 7;

        // Set up length counter with a non-zero value
        channel.length_counter.set_enabled(true);
        channel.length_counter.load(1 << 3); // Load length counter with value 1

        // Configure sweep unit to never mute
        // Set sweep disabled (bit 7=0), period=0, negate=0, shift=0
        channel.sweep_unit.update_from_register(0x00);

        // First tick - timer_value is 0, so reload from timer and advance duty
        channel.tick();
        assert_eq!(channel.timer_value, 8, "Timer value should reload from timer (8)");
        assert_eq!(channel.duty_pos, 0, "Duty position should advance from 7 to 0");

        // Second tick - timer decrements from 8 to 7
        channel.tick();
        assert_eq!(channel.timer_value, 7, "Timer value should decrement from 8 to 7");
        assert_eq!(channel.duty_pos, 0, "Duty position unchanged during decrement");

        // Third tick - timer decrements from 7 to 6
        channel.tick();
        assert_eq!(channel.timer_value, 6, "Timer value should decrement from 7 to 6");
        assert_eq!(channel.duty_pos, 0, "Duty position unchanged during decrement");

        // Fourth tick - timer decrements from 6 to 5
        channel.tick();
        assert_eq!(channel.timer_value, 5, "Timer value should decrement from 6 to 5");
        assert_eq!(channel.duty_pos, 0, "Duty position unchanged during decrement");
    }

    #[test]
    fn test_sample_generation_disabled() {
        let channel = PulseChannel::new(true);

        // When disabled, should always return 0
        assert_eq!(channel.output(), 0);
    }

    #[test]
    fn test_sample_generation_12_5_percent_duty() {
        let mut channel = PulseChannel::new(true);

        // Enable the channel for testing
        channel.set_enabled(true);

        // Set up length counter for testing
        channel.length_counter.set_enabled(true);
        channel.length_counter.load(0 << 3); // Load length counter value

        // A period below 8 is muted by the sweep unit on real hardware, so give it a valid one.
        channel.write_register(2, 0x40);

        // Configure for 12.5% duty cycle and maximum volume
        channel.write_register(0, 0b00011111); // Duty 0 (12.5%), constant volume (15)

        // Setup for positions - 12.5% duty cycle should output a non-zero value
        // only at position 0 (one out of eight positions)

        // Test position 0 (should output sound)
        channel.duty_pos = 0;
        assert_eq!(channel.output(), 15);

        // Test position 1-7 (should be silent)
        for pos in 1..8 {
            channel.duty_pos = pos;
            assert_eq!(channel.output(), 0);
        }
    }

    #[test]
    fn test_sample_generation_volume_scaling() {
        let mut channel = PulseChannel::new(true);

        // Enable the channel
        channel.set_enabled(true);

        // Set up length counter for testing
        channel.length_counter.set_enabled(true);
        channel.length_counter.load(0 << 3); // Load length counter value

        // A period below 8 is muted by the sweep unit on real hardware, so give it a valid one.
        channel.write_register(2, 0x40);

        // Set duty position where output is active
        channel.duty_pos = 0;

        // Test with volume 0
        channel.write_register(0, 0b00010000); // Constant volume mode, volume 0
        assert_eq!(channel.output(), 0);

        // Test with volume 1
        channel.write_register(0, 0b00010001); // Constant volume mode, volume 1
        assert_eq!(channel.output(), 1);

        // Test with volume 7
        channel.write_register(0, 0b00010111); // Constant volume mode, volume 7
        assert_eq!(channel.output(), 7);

        // Test with maximum volume (15)
        channel.write_register(0, 0b00011111); // Constant volume mode, volume 15
        assert_eq!(channel.output(), 15);
    }

    #[test]
    fn test_envelope_initialization() {
        let channel = PulseChannel::new(true);

        // Initial values should all be zeroed
        assert!(!channel.envelope.start);
        assert_eq!(channel.envelope.divider, 0);
        assert_eq!(channel.envelope.counter, 0);
        assert!(!channel.envelope.loop_flag);
        assert!(!channel.envelope.constant_volume);
        assert_eq!(channel.envelope.period, 0);
        assert_eq!(channel.envelope.volume, 0);
    }

    #[test]
    fn test_envelope_control_register_decoding() {
        let mut channel = PulseChannel::new(true);

        // Test constant volume mode with volume level 7
        channel.write_register(0, 0b00010111); // Volume 7, constant volume, no loop
        assert!(channel.envelope.constant_volume);
        assert!(!channel.envelope.loop_flag);
        assert_eq!(channel.envelope.period, 7);
        assert_eq!(channel.volume, 7); // Volume should match period in constant volume mode

        // Test envelope mode with decay rate 10
        channel.write_register(0, 0b00001010); // Rate 10, envelope mode, no loop
        assert!(!channel.envelope.constant_volume);
        assert!(!channel.envelope.loop_flag);
        assert_eq!(channel.envelope.period, 10);
        // Volume should be from envelope counter, initially 0
        assert_eq!(channel.volume, 0);

        // Test loop mode
        channel.write_register(0, 0b00101100); // Rate 12, loop mode
        assert!(channel.envelope.loop_flag);
        assert_eq!(channel.envelope.period, 12);
    }

    #[test]
    fn test_envelope_restart() {
        let mut channel = PulseChannel::new(true);

        // Configure channel with envelope mode and period 5
        channel.write_register(0, 0b00000101); // Rate 5, envelope mode, no loop

        // Initially the envelope counter and volume should be 0
        assert_eq!(channel.envelope.counter, 0);
        assert_eq!(channel.envelope.volume, 0);

        // Restart envelope by writing to timer high
        channel.write_register(3, 0x01);

        // Envelope start flag should be set, but no change to counter yet
        assert!(channel.envelope.start);

        // Tick envelope once - should initialize counter to 15 and divider to period
        channel.tick_envelope();
        assert!(!channel.envelope.start);
        assert_eq!(channel.envelope.counter, 15);
        assert_eq!(channel.envelope.divider, 5);
        assert_eq!(channel.envelope.volume, 15);

        // Volume should be envelope volume since we're in envelope mode
        assert_eq!(channel.volume, 15);
    }

    #[test]
    fn test_envelope_decay() {
        let mut channel = PulseChannel::new(true);

        // Configure envelope mode with period 1 (fast decay)
        channel.write_register(0, 0b00000001);

        // Restart envelope
        channel.restart_envelope();
        channel.tick_envelope();

        // Should now have counter=15, divider=1
        assert_eq!(channel.envelope.counter, 15);
        assert_eq!(channel.envelope.divider, 1);

        // First tick: divider becomes 0
        channel.tick_envelope();
        assert_eq!(channel.envelope.divider, 0);
        assert_eq!(channel.envelope.counter, 15);

        // Second tick: divider resets, counter decrements
        channel.tick_envelope();
        assert_eq!(channel.envelope.divider, 1);
        assert_eq!(channel.envelope.counter, 14);

        // Continue decay
        for _ in 0..14 {
            channel.tick_envelope();
            channel.tick_envelope();
        }

        // After 14 more decrements, counter should be 0
        assert_eq!(channel.envelope.counter, 0);
        assert_eq!(channel.envelope.volume, 0);

        // Once at 0, it should stay there without loop mode
        channel.tick_envelope();
        channel.tick_envelope();
        assert_eq!(channel.envelope.counter, 0);
    }

    #[test]
    fn test_envelope_loop() {
        let mut channel = PulseChannel::new(true);

        // Configure envelope mode with loop and period 1
        channel.write_register(0, 0b00100001);

        // Restart and initialize
        channel.restart_envelope();
        channel.tick_envelope();

        // Verify initial state
        assert_eq!(channel.envelope.counter, 15);
        assert_eq!(channel.envelope.volume, 15);

        // Tick enough times to wrap around with loop
        for _ in 0..32 {
            channel.tick_envelope();
        }

        // With loop mode, counter should wrap back to 15 rather than stay at 0
        assert_eq!(channel.envelope.counter, 15);
        assert_eq!(channel.envelope.volume, 15);
    }

    #[test]
    fn test_constant_volume_vs_envelope() {
        let mut channel = PulseChannel::new(true);

        // Configure with constant volume 12
        channel.write_register(0, 0b00011100);

        // Envelope counter is 0, but volume should be 12 due to constant volume mode
        assert_eq!(channel.envelope.counter, 0);
        assert_eq!(channel.volume, 12);

        // Switch to envelope mode, same value (12)
        channel.write_register(0, 0b00001100);

        // Now volume should match envelope counter (0) not the period value
        assert_eq!(channel.envelope.period, 12);
        assert_eq!(channel.volume, 0);

        // Restart envelope and tick it
        channel.restart_envelope();
        channel.tick_envelope();

        // Now volume should be 15 from envelope counter
        assert_eq!(channel.volume, 15);
    }
    #[test]
    fn test_sweep_update_period() {
        let mut channel = PulseChannel::new(true);

        // Set up initial state
        channel.write_register(2, 0x40); // Timer low = 0x40
        channel.write_register(3, 0x01); // Timer high = 0x01
        channel.set_enabled(true);

        // Verify initial timer value
        assert_eq!(channel.timer, 0x0140); // 320

        // Configure sweep with enabled=false so no frequency change occurs
        // This is for initial testing
        channel.write_register(1, 0b00000001);

        // First tick just loads the divider counter
        channel.tick_sweep();
        assert_eq!(channel.timer, 0x0140); // Still 320

        // More ticks shouldn't change anything (sweep disabled)
        channel.tick_sweep();
        assert_eq!(channel.timer, 0x0140); // Still 320

        // Now enable sweep with period=1, negate=false, shift=1
        // This will add period >> 1 to period
        channel.write_register(1, 0b10010001);

        // First tick reloads divider (period=1)
        channel.tick_sweep();

        // Second tick should process divider=0 and update the period
        channel.tick_sweep();

        // Period should change: 320 + (320 >> 1) = 320 + 160 = 480
        assert_eq!(channel.timer, 480);

        // Now test with negate=true for pulse channel 1
        channel.write_register(1, 0b10011001);

        // First tick reloads divider
        channel.tick_sweep();

        // Second tick updates period with ones' complement negation
        channel.tick_sweep();

        // For pulse 1, change = -(period >> 1) - 1 = -(480 >> 1) - 1 = -240 - 1 = -241
        // New period = 480 + (-241) = 239
        assert_eq!(channel.timer, 239);
    }

    #[test]
    fn test_length_counter_initialization() {
        let channel = PulseChannel::new(true);

        // Initial state should have inactive length counter
        assert!(!channel.is_length_counter_active());
    }

    #[test]
    fn test_length_counter_load() {
        let mut channel = PulseChannel::new(true);

        // Enable the channel
        channel.set_enabled(true);

        // Load a value into the length counter via register 3
        // Use length index 7 (value should be 6)
        channel.write_register(3, 7 << 3);

        // Length counter should be active
        assert!(channel.is_length_counter_active());

        // Check if sound is produced
        channel.write_register(0, 0b01011111); // 25% duty, constant volume (15)
        channel.write_register(2, 0x40); // Period >= 8, or the sweep unit mutes the channel
        channel.duty_pos = 0; // Position where output is active
        assert!(channel.output() > 0);

        // Disable the channel
        channel.set_enabled(false);

        // Length counter should now be inactive
        assert!(!channel.is_length_counter_active());

        // Sound should be muted
        assert_eq!(channel.output(), 0);
    }

    #[test]
    fn test_length_counter_decrement() {
        let mut channel = PulseChannel::new(true);

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
        channel.write_register(0, 0b01011111); // 25% duty, constant volume (15)
        channel.duty_pos = 0;
        assert_eq!(channel.output(), 0);
    }

    #[test]
    fn test_length_counter_halt() {
        let mut channel = PulseChannel::new(true);

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

    #[test]
    fn test_length_counter_channel_silencing() {
        let mut channel = PulseChannel::new(true);

        // Enable channel and set up for sound output
        channel.set_enabled(true);
        channel.write_register(0, 0b01011111); // 25% duty, constant volume (15)
        channel.write_register(2, 0x40); // Period >= 8, or the sweep unit mutes the channel

        // Set duty position for sound generation
        channel.duty_pos = 0;

        // No length counter loaded yet, so no sound
        assert_eq!(channel.output(), 0);

        // Load a length value
        channel.write_register(3, 0 << 3); // index 0 = value 10

        // Now sound should be generated
        assert!(channel.output() > 0);

        // Tick length counter until it runs out
        for _ in 0..10 {
            channel.tick_length_counter();
        }

        // Sound should now be muted due to length counter
        assert_eq!(channel.output(), 0);
    }

    #[test]
    fn test_pulse2_sweep_behavior() {
        let mut channel = PulseChannel::new(false); // Create pulse channel 2

        // Set up initial state
        channel.write_register(2, 0x40); // Timer low = 0x40
        channel.write_register(3, 0x01); // Timer high = 0x01
        channel.set_enabled(true);

        // Verify initial timer value
        assert_eq!(channel.timer, 0x0140); // 320

        // Configure sweep with enabled=false so no frequency change occurs
        // This is for initial testing
        channel.write_register(1, 0b00000001);

        // First tick just loads the divider counter
        channel.tick_sweep();
        assert_eq!(channel.timer, 0x0140); // Still 320

        // More ticks shouldn't change anything (sweep disabled)
        channel.tick_sweep();
        assert_eq!(channel.timer, 0x0140); // Still 320

        // Now enable sweep with period=1, negate=true, shift=1
        // This will subtract period >> 1 from period (two's complement)
        channel.write_register(1, 0b10011001);

        // First tick reloads divider
        channel.tick_sweep();

        // Second tick updates period with two's complement negation
        channel.tick_sweep();

        // For pulse 2, change = -(period >> 1) = -(320 >> 1) = -160
        // New period = 320 + (-160) = 160
        assert_eq!(channel.timer, 160);
    }

    #[test]
    fn test_pulse2_register_handling() {
        let mut channel = PulseChannel::new(false); // Create pulse channel 2

        // Enable the channel first
        channel.set_enabled(true);

        // Test duty cycle setting (bits 6-7 of control register)
        channel.write_register(0, 0b11000000); // Duty cycle 3 (75%)
        assert_eq!(channel.duty_cycle, 3);

        // Test volume setting (bits 0-3 of control register)
        channel.write_register(0, 0b00011111); // Volume 15, constant volume mode (bit 4 set)
        assert_eq!(channel.volume, 15);

        // Test sweep register (period=1, negate=true, shift=2)
        // 0b10010010: bit 7=1 (enabled), bits 4-6=001 (period=1), bit 3=0 (negate=false), bits 0-2=010 (shift=2)
        channel.write_register(1, 0b10010010);
        assert_eq!(channel.sweep_unit.get_period(), 1);
        assert!(!channel.sweep_unit.get_negate());
        assert_eq!(channel.sweep_unit.get_shift(), 2);

        // Test timer low register
        channel.write_register(2, 0x42);
        assert_eq!(channel.timer_lo, 0x42);

        // Test timer high register
        channel.write_register(3, 0x01);
        assert_eq!(channel.timer_hi, 0x01);
        assert_eq!(channel.timer, 0x0142); // Combined timer value
    }

    #[test]
    fn test_pulse2_length_counter() {
        let mut channel = PulseChannel::new(false); // Create pulse channel 2

        // Enable the channel
        channel.set_enabled(true);

        // Load a value into the length counter via register 3
        // Use length index 7 (value should be 6)
        channel.write_register(3, 7 << 3);

        // Length counter should be active
        assert!(channel.is_length_counter_active());

        // Check if sound is produced
        channel.write_register(0, 0b01011111); // 25% duty, constant volume (15)
        channel.write_register(2, 0x40); // Period >= 8, or the sweep unit mutes the channel
        channel.duty_pos = 0; // Position where output is active
        assert!(channel.output() > 0);

        // Disable the channel
        channel.set_enabled(false);

        // Length counter should now be inactive
        assert!(!channel.is_length_counter_active());

        // Sound should be muted
        assert_eq!(channel.output(), 0);
    }

    #[test]
    fn test_pulse2_envelope() {
        let mut channel = PulseChannel::new(false); // Create pulse channel 2

        // Configure envelope mode with period 1 (fast decay)
        channel.write_register(0, 0b00000001);

        // Restart envelope
        channel.restart_envelope();
        channel.tick_envelope();

        // Should now have counter=15, divider=1
        assert_eq!(channel.envelope.counter, 15);
        assert_eq!(channel.envelope.divider, 1);

        // First tick: divider becomes 0
        channel.tick_envelope();
        assert_eq!(channel.envelope.divider, 0);
        assert_eq!(channel.envelope.counter, 15);

        // Second tick: divider resets, counter decrements
        channel.tick_envelope();
        assert_eq!(channel.envelope.divider, 1);
        assert_eq!(channel.envelope.counter, 14);

        // Continue decay
        for _ in 0..14 {
            channel.tick_envelope();
            channel.tick_envelope();
        }

        // After 14 more decrements, counter should be 0
        assert_eq!(channel.envelope.counter, 0);
        assert_eq!(channel.envelope.volume, 0);

        // Once at 0, it should stay there without loop mode
        channel.tick_envelope();
        channel.tick_envelope();
        assert_eq!(channel.envelope.counter, 0);
    }
}
