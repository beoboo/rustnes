/// Represents the sweep unit for pulse channels
#[derive(Debug, Clone)]
pub struct Sweep {
    // Sweep register state
    enabled: bool,
    period: u8,
    negate: bool,
    shift: u8,

    // Internal state
    divider: u8,
    reload_flag: bool,
    // Pulse 1 vs Pulse 2 differentiation
    is_pulse1: bool,
}

impl Sweep {
    /// Create a new sweep unit
    pub fn new(is_pulse1: bool) -> Self {
        Self {
            enabled: false,
            period: 0,
            negate: false,
            shift: 0,
            divider: 0,
            reload_flag: false,
            is_pulse1,
        }
    }

    /// Reset the sweep unit to initial state
    pub fn reset(&mut self) {
        self.enabled = false;
        self.period = 0;
        self.negate = false;
        self.shift = 0;
        self.divider = 0;
        self.reload_flag = false;
    }

    /// Update sweep parameters from a register value
    pub fn update_from_register(&mut self, value: u8) {
        // Extract the enabled flag (bit 7)
        self.enabled = (value & 0x80) != 0;

        // Extract the period (bits 6-4)
        self.period = (value >> 4) & 0x07;

        // Extract the negate flag (bit 3)
        self.negate = (value & 0x08) != 0;

        // Extract the shift count (bits 2-0)
        self.shift = value & 0x07;

        // Set the reload flag
        self.reload_flag = true;
    }

    /// Process a half frame tick (120Hz)
    pub fn tick(&mut self, current_period: u16) -> Option<u16> {
        let mut new_period = None;

        // If the divider outputs a clock (reaches zero) and sweep is enabled with non-zero shift
        if self.divider == 0 && self.enabled && self.shift > 0 {
            // Calculate the target period
            let target = self.calculate_target_period(current_period);

            // Only update the period if we're not muting the channel
            if !self.should_mute(current_period, target) {
                new_period = Some(target);
            }
        }

        // Clock the divider
        if self.divider == 0 || self.reload_flag {
            // Reset divider to period
            self.divider = self.period;
            self.reload_flag = false;
        } else {
            // Decrement divider
            self.divider -= 1;
        }

        new_period
    }

    /// Calculate the target period for the sweep unit
    pub fn calculate_target_period(&self, current_period: u16) -> u16 {
        // Calculate the change amount by shifting the current period right
        let change_amount = current_period >> self.shift;

        // Apply the change based on negate flag and which pulse channel we are
        if self.negate {
            if self.is_pulse1 {
                // Pulse 1 uses ones' complement negation (∼c + 1)
                current_period.wrapping_sub(change_amount).wrapping_sub(1)
            } else {
                // Pulse 2 uses two's complement negation (∼c)
                current_period.wrapping_sub(change_amount)
            }
        } else {
            // Addition mode (increase period, lower frequency)
            current_period.wrapping_add(change_amount)
        }
    }

    /// Check if the sweep unit should mute the channel
    pub fn should_mute(&self, current_period: u16, target_period: u16) -> bool {
        // Mute if current period is less than 8
        if current_period < 8 {
            return true;
        }

        // Mute if target period is greater than 0x7FF (11 bits)
        if target_period > 0x7FF {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sweep_register_decoding() {
        let mut sweep_unit = Sweep::new(true);

        // Test with enabled = 1, period = 2, negate = 0, shift = 3
        // Register value 0b10100011:
        // bit 7 = 1 (enabled), bits 6-4 = 010 (period=2), bit 3 = 0 (negate=false), bits 2-0 = 011 (shift=3)
        sweep_unit.update_from_register(0b10100011);
        assert_eq!(sweep_unit.enabled, true);
        assert_eq!(sweep_unit.period, 2);
        assert_eq!(sweep_unit.negate, false);
        assert_eq!(sweep_unit.shift, 3);
        assert_eq!(sweep_unit.reload_flag, true);

        // Test with enabled = 0, period = 2, negate = 1, shift = 7
        // Register value 0b00100111:
        // bit 7 = 0 (disabled), bits 6-4 = 010 (period=2), bit 3 = 1 (negate=true), bits 2-0 = 111 (shift=7)
        sweep_unit.reload_flag = false;
        sweep_unit.update_from_register(0b00101111);
        assert_eq!(sweep_unit.enabled, false);
        assert_eq!(sweep_unit.period, 2);
        assert_eq!(sweep_unit.negate, true);
        assert_eq!(sweep_unit.shift, 7);
        assert_eq!(sweep_unit.reload_flag, true);
    }

    #[test]
    fn test_sweep_target_period_calculation() {
        let mut sweep_pulse1 = Sweep::new(true);
        let mut sweep_pulse2 = Sweep::new(false);

        // Test with negate = false (addition mode), shift = 1
        sweep_pulse1.enabled = true;
        sweep_pulse1.negate = false;
        sweep_pulse1.shift = 1;

        // Given period = 100, shift = 1, change = 50, target = 150
        let target = sweep_pulse1.calculate_target_period(100);
        assert_eq!(target, 150);

        // Test with negate = true (subtraction mode), shift = 2
        sweep_pulse1.negate = true;
        sweep_pulse1.shift = 2;

        // Given period = 100, shift = 2, change = 25, target = 100 - 25 - 1 = 74 (ones' complement)
        let target = sweep_pulse1.calculate_target_period(100);
        assert_eq!(target, 74);

        // Test pulse 2's different negation behavior
        sweep_pulse2.enabled = true;
        sweep_pulse2.negate = true;
        sweep_pulse2.shift = 2;

        // Given period = 100, shift = 2, change = 25, target = 100 - 25 = 75 (two's complement)
        let target = sweep_pulse2.calculate_target_period(100);
        assert_eq!(target, 75);
    }

    #[test]
    fn test_sweep_muting_conditions() {
        let sweep = Sweep::new(true);

        // Mute if period < 8
        assert_eq!(sweep.should_mute(7, 10), true);
        assert_eq!(sweep.should_mute(8, 10), false);

        // Mute if target period > 0x7FF
        assert_eq!(sweep.should_mute(100, 0x7FF), false);
        assert_eq!(sweep.should_mute(100, 0x800), true);
    }
}
