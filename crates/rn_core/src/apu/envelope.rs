/// Represents the envelope generator for sound channels
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Envelope {
    // State fields
    pub start: bool,
    pub divider: u8,
    pub counter: u8,
    pub loop_flag: bool,
    pub constant_volume: bool,
    pub period: u8,
    pub volume: u8,
}

impl Envelope {
    /// Create a new envelope generator
    pub fn new() -> Self {
        Self {
            start: false,
            divider: 0,
            counter: 0,
            loop_flag: false,
            constant_volume: false,
            period: 0,
            volume: 0,
        }
    }

    /// Reset the envelope generator to initial state
    pub fn reset(&mut self) {
        self.start = false;
        self.divider = 0;
        self.counter = 0;
        self.loop_flag = false;
        self.constant_volume = false;
        self.period = 0;
        self.volume = 0;
    }

    /// Process a quarter frame tick (240Hz)
    pub fn tick(&mut self) {
        // If the start flag is set, clear it and reload the counter and divider
        if self.start {
            self.start = false;
            self.counter = 15;
            self.divider = self.period;
            self.volume = 15; // Start at maximum volume
        } else {
            // If the divider is zero, clock the counter and reload the divider
            if self.divider == 0 {
                self.divider = self.period;

                // If the counter is not zero, decrement it
                if self.counter > 0 {
                    self.counter -= 1;
                } else if self.loop_flag {
                    // If we're at zero and looping is enabled, wrap to 15
                    self.counter = 15;
                }

                // Update envelope volume based on counter
                self.volume = self.counter;
            } else {
                // Otherwise, decrement the divider
                self.divider -= 1;
            }
        }
    }

    /// Update envelope parameters from a control register value
    pub fn update_from_register(&mut self, value: u8) {
        // Extract loop flag (bit 5)
        self.loop_flag = (value & 0x20) != 0;

        // Extract constant volume flag (bit 4)
        self.constant_volume = (value & 0x10) != 0;

        // Extract volume/envelope period (bits 0-3)
        self.period = value & 0x0F;
    }

    /// Restart the envelope generator
    pub fn restart(&mut self) {
        self.start = true;
    }

    /// Get the current output volume
    pub fn get_volume(&self) -> u8 {
        if self.constant_volume {
            self.period
        } else {
            self.volume
        }
    }

    /// Check if constant volume mode is enabled
    pub fn is_constant_volume(&self) -> bool {
        self.constant_volume
    }

    /// Get the current envelope volume
    pub fn get_envelope_volume(&self) -> u8 {
        self.volume
    }
}
