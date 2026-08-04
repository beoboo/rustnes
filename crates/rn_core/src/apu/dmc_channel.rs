use super::length_counter::LengthCounter;

/// Represents the DMC (Delta Modulation Channel) in the APU
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    /// The 7-bit level the channel presents to the mixer.
    ///
    /// Distinct from the sample buffer, which the code used to conflate it with. The buffer holds
    /// a *fetched byte* waiting to enter the shifter; this is the running level that each shifted
    /// bit nudges up or down by two.
    output_level: u8,

    /// The one-byte buffer between the memory reader and the output unit.
    ///
    /// Hardware really does have exactly one byte here, and `apu_test/7-dmc_basics` checks for it:
    /// it is refilled the moment it empties, independently of whether the output unit is ready for
    /// it, which is what lets a sample play without gaps.
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

    /// Set when a sample finishes with the IRQ enabled, until a game clears it.
    ///
    /// This is how a game learns a sample has ended without polling for it, and it is reported by
    /// bit 7 of `$4015`. It is cleared by reading that register, by writing it, and by clearing
    /// the enable bit in `$4010`.
    irq_pending: bool,

    /// A sample byte the channel wants fetched, and has not been given yet.
    ///
    /// The channel cannot read memory itself: the fetch is a DMA that stalls the CPU, so it has to
    /// be performed by whatever owns the bus and the cycle count. The channel asks, and carries on
    /// once it is answered.
    pending_fetch: Option<u16>,
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
            output_level: 0,
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
            irq_pending: false,
            pending_fetch: None,
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

        // The memory reader runs independently of the output unit: the moment the buffer is empty
        // and the sample is unfinished, it asks for the next byte. Asking only once the output
        // unit had run dry meant the *first* byte was never requested — a channel that has just
        // started has no bits to shift, so the branch that would have asked was never reached and
        // a sample never began at all.
        if self.sample_buffer_empty && self.bytes_remaining > 0 && self.pending_fetch.is_none() {
            self.load_next_byte();
        }

        // Check if timer_value is zero
        if self.timer_value == 0 {
            // One less than the period, not the period itself. Acting at zero and *then* reloading
            // with the full value puts one extra cycle between one action and the next, which
            // `apu_test/8-dmc_rates` reports as "rate 0's period is too long".
            self.timer_value = self.timer.saturating_sub(1);

            // A bit leaves the shifter and nudges the level, unless the channel is silent.
            if !self.silence_flag && self.bits_remaining > 0 {
                let bit = (self.shift_register & 0x01) != 0;
                self.shift_register >>= 1;

                if bit {
                    if self.output_level <= 0x7D {
                        self.output_level += 2;
                    }
                } else if self.output_level >= 0x02 {
                    self.output_level -= 2;
                }
            }

            if self.bits_remaining > 0 {
                self.bits_remaining -= 1;
            }

            // An emptied shifter takes whatever the buffer holds. If the buffer is empty too the
            // channel goes silent — it keeps its level and stops changing it — until the memory
            // reader refills it.
            if self.bits_remaining == 0 {
                self.bits_remaining = 8;
                if self.sample_buffer_empty {
                    self.silence_flag = true;
                } else {
                    self.silence_flag = false;
                    self.shift_register = self.sample_buffer;
                    self.sample_buffer_empty = true;
                }
            }
        } else {
            // Decrement timer_value
            self.timer_value -= 1;
        }
    }

    /// Ask for the next sample byte, or fall silent if the sample is finished.
    ///
    /// The read itself is not done here. On hardware it is a DMA: the CPU is halted for a few
    /// cycles while the channel takes the bus, and those cycles belong to the CPU's count. Only
    /// the part of the system that owns both can do that, so this records the address wanted and
    /// [`supply_byte`](Self::supply_byte) finishes the job.
    fn load_next_byte(&mut self) {
        if self.bytes_remaining > 0 {
            self.pending_fetch = Some(self.current_address);
        } else {
            // No more bytes to load
            self.sample_buffer_empty = true;
            self.silence_flag = true;
        }
    }

    /// Whether the channel is holding an interrupt.
    pub fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    /// Drop any interrupt the channel is holding.
    pub fn acknowledge_irq(&mut self) {
        self.irq_pending = false;
    }

    /// The address this channel wants read, if it is waiting on one. Clears the request.
    pub fn take_pending_fetch(&mut self) -> Option<u16> {
        self.pending_fetch.take()
    }

    /// Hand the channel the byte it asked for.
    pub fn supply_byte(&mut self, value: u8) {
        // Into the buffer only. The output unit takes it when its own shifter empties, which is
        // what makes this a buffer rather than a hand-off.
        self.sample_buffer = value;
        self.sample_buffer_empty = false;

        // The sample address runs up through the top of memory and wraps to $8000, not to zero:
        // the whole sample lives in cartridge space. Masking it into the low half instead pointed
        // the channel at work RAM, which is why the byte fetched had to be faked.
        self.current_address =
            if self.current_address == 0xFFFF { 0x8000 } else { self.current_address + 1 };
        self.bytes_remaining -= 1;

        // The end of a sample either restarts it or raises an interrupt, depending on how the
        // game set $4010 — and never both.
        if self.bytes_remaining == 0 {
            if self.loop_flag {
                self.restart();
            } else if self.irq_enabled {
                self.irq_pending = true;
            }
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

    /// The channel's current DAC level, 0..=127 (the DMC has a 7-bit DAC).
    ///
    /// See [`PulseChannel::output`](super::PulseChannel) for why this is a raw level. Note that
    /// silence here means level 0, not "the midpoint": the DC offset that leaves is removed by the
    /// high-pass filters on the mixed output, exactly as on hardware.
    pub fn output(&self) -> u8 {
        if !self.enabled {
            return 0;
        }
        // The level persists through silence rather than dropping to zero: silence means the
        // shifter has nothing to change it *with*, not that the channel stops driving the mixer.
        self.output_level
    }

    /// Set the enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        if enabled {
            // Enabling starts a fetch, unless one is already under way — which is how a game loops
            // a sample without restarting it.
            if self.bytes_remaining == 0 {
                self.restart();
            }
        } else {
            // Disabling stops the sample immediately rather than letting it finish. Leaving the
            // count standing kept the channel reporting itself as busy through $4015 forever, so a
            // game waiting for a sample to end before queueing the next one would wait for good.
            self.bytes_remaining = 0;
        }

        // Update length counter enabled state
        self.length_counter.set_enabled(enabled);
    }

    /// Check if the channel is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Whether the channel still has sample bytes to fetch.
    ///
    /// This is what bit 4 of `$4015` reports for the DMC. The other four channels report a length
    /// counter; the DMC has none, and reporting one meant the bit described something the hardware
    /// does not have — it would read as silent while a sample was still playing, and as playing
    /// after one had finished.
    pub fn has_bytes_remaining(&self) -> bool {
        self.bytes_remaining > 0
    }

    /// Write to a channel register
    pub fn write_register(&mut self, register_offset: u16, value: u8) {
        match register_offset {
            0 => {
                // Control register ($4010)
                self.control = value;
                self.irq_enabled = (value & 0x80) != 0;
                // Clearing the enable bit also clears any interrupt it had already raised.
                if !self.irq_enabled {
                    self.irq_pending = false;
                }
                self.loop_flag = (value & 0x40) != 0;
                self.update_timer();
            },
            1 => {
                // Direct load register ($4011)
                self.direct_load = value;
                // $4011 writes the DAC level straight out, which is how a game plays PCM by hand.
                self.output_level = value & 0x7F;
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
            // Writes to registers the hardware does not use are simply ignored — every offset 0-3 is decoded, so this is unreachable via the bus.
            // A ROM writing one must not bring the emulator down: blargg's instr_test-v5 does
            // exactly this, and it used to panic.
            _ => {},
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
        assert!(!channel.enabled);
        assert!(!channel.irq_enabled);
        assert!(!channel.loop_flag);
        assert_eq!(channel.sample_buffer, 0);
        assert!(channel.sample_buffer_empty);
        assert_eq!(channel.bits_remaining, 0);
        assert_eq!(channel.shift_register, 0);
        assert!(channel.silence_flag);
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
        assert!(!channel.enabled);
        assert!(!channel.irq_enabled);
        assert!(!channel.loop_flag);
        assert_eq!(channel.sample_buffer, 0);
        assert!(channel.sample_buffer_empty);
        assert_eq!(channel.bits_remaining, 0);
        assert_eq!(channel.shift_register, 0);
        assert!(channel.silence_flag);
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

        // $4011 writes the DAC level straight out, which is how a game plays PCM by hand. It
        // does not touch the sample buffer: that holds a byte fetched from memory, which is a
        // different thing that this code used to conflate it with.
        channel.write_register(1, 0x42);
        assert_eq!(channel.output_level, 0x42);

        // The level is seven bits, so the top bit of the write is dropped rather than kept.
        channel.write_register(1, 0x80);
        assert_eq!(channel.output_level, 0x00);
    }

    #[test]
    fn test_sample_generation() {
        let mut channel = DmcChannel::new();

        // Channel should be silent when disabled
        assert_eq!(channel.output(), 0);

        // Enable the channel
        channel.set_enabled(true);

        // The output is the DAC level, wherever the level came from.
        channel.output_level = 0x40;
        assert_eq!(channel.output(), 64); // Mid-scale: the DAC centre

        channel.output_level = 0x7F;
        assert_eq!(channel.output(), 127); // Full scale

        channel.output_level = 0x00;
        assert_eq!(channel.output(), 0); // Bottom of the DAC range

        // Silence does not pull the level to zero. It means the shifter has nothing to change the
        // level *with* — the channel goes on driving the mixer at whatever it last reached, which
        // is why a stopped sample does not click.
        channel.output_level = 0x40;
        channel.silence_flag = true;
        assert_eq!(channel.output(), 64);
    }

    #[test]
    fn test_register_writing() {
        let mut channel = DmcChannel::new();

        // Test control register ($4010)
        channel.write_register(0, 0xC0); // Set IRQ enable and loop flag
        assert!(channel.irq_enabled);
        assert!(channel.loop_flag);

        // Test direct load register ($4011), which sets the DAC level
        channel.write_register(1, 0x42);
        assert_eq!(channel.output_level, 0x42);

        // Test address register ($4012)
        channel.write_register(2, 0x30);
        assert_eq!(channel.address, 0x30);

        // Test length register ($4013)
        channel.write_register(3, 0x20);
        assert_eq!(channel.length, 0x20);
    }
}
