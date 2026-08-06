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

    /// Ticks left before a `$4015`-started fetch may be requested: 2 if the write landed on an
    /// even CPU cycle, 3 if odd, so the request always surfaces on the same parity.
    ///
    /// Hardware inserts this delay, and the parity it normalises to is what makes the *start* of
    /// a sample stall the CPU 3 cycles where a mid-sample refill stalls it 4 — see the stall in
    /// `nes_system`. Without it, `dma_4016_read` reports the doubled read one CPU clock late, on
    /// iteration 4 of 5 against hardware's 3. Measured against tetanes' cycle ledger 2026-08-06:
    /// the two emulators' DMC refills agreed to the cycle (request at write+3406 in both) and the
    /// whole one-cycle error was this start-up stall.
    #[serde(default)]
    start_delay: u8,
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
            // Power-on `$4010` is zero, whose rate is 428 — the free-running timer needs its real
            // period from the first cycle, not a zero that would expire every tick.
            timer: 428,
            timer_value: 0,

            // Initialize length counter
            length_counter: LengthCounter::new(),
            irq_pending: false,
            pending_fetch: None,
            start_delay: 0,
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
        self.timer = 428;
        self.timer_value = 0;
        self.start_delay = 0;

        // Reset length counter
        self.length_counter.reset();
    }

    /// Process a single DMC channel cycle
    pub fn tick(&mut self) {
        // No early return when disabled: the timer and output unit free-run from power-on, as
        // hardware's do — `$4015` gates only the memory reader, through `bytes_remaining`. This
        // is not audible (a disabled channel shifts silence), but it is *visible* in the DMA's
        // timing: the timer's period is even, so its phase fixes which CPU-cycle parity every
        // shifter reload — and so every mid-sample refill DMA — lands on, for the whole run.
        // A timer that only ran while enabled took its phase from when the game last enabled the
        // channel, and `sync_dmc`'s calibrated loops hung whenever that phase came out wrong.

        // A `$4015`-started fetch waits out the parity-normalising delay before it may be
        // requested. Mid-sample refills never arm this, so they pass straight through.
        if self.start_delay > 0 {
            self.start_delay -= 1;
        }

        // The memory reader runs independently of the output unit: the moment the buffer is empty
        // and the sample is unfinished, it asks for the next byte. Asking only once the output
        // unit had run dry meant the *first* byte was never requested — a channel that has just
        // started has no bits to shift, so the branch that would have asked was never reached and
        // a sample never began at all.
        if self.start_delay == 0
            && self.sample_buffer_empty
            && self.bytes_remaining > 0
            && self.pending_fetch.is_none()
        {
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
                    // The refill is requested in this same cycle, not left for the reader check
                    // on the next one. The cycle matters because the stall's length is decided by
                    // the halt's parity: the timer's period is even, so every reload shares one
                    // parity and every mid-sample refill stalls the same 4 cycles — which is the
                    // constant `sync_dmc` is calibrated around. Requested a tick later, refills
                    // land on the other parity, stall 3, and that loop never converges.
                    if self.bytes_remaining > 0 && self.pending_fetch.is_none() {
                        self.load_next_byte();
                    }
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

    /// Whether the channel is waiting on a byte, leaving the request where it is.
    pub fn wants_fetch(&self) -> bool {
        self.pending_fetch.is_some()
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

    /// Set the enabled state. `on_even_cycle` is the parity of the CPU cycle the write landed on,
    /// which decides the start-up delay — see [`Self::start_delay`].
    pub fn set_enabled(&mut self, enabled: bool, on_even_cycle: bool) {
        self.enabled = enabled;

        if enabled {
            // Enabling starts a fetch, unless one is already under way — which is how a game loops
            // a sample without restarting it.
            if self.bytes_remaining == 0 {
                self.restart();
            }

            // And the buffer fills if it is empty, rather than waiting for the output unit —
            // `apu_test/7-dmc_basics` says "there should be a one-byte buffer that's filled
            // immediately if empty". "Immediately" is within a couple of cycles, not on the write
            // itself: the request surfaces 2 cycles later from an even write and 3 from an odd
            // one, landing on a fixed parity either way. `dma_4016_read` is what tells the two
            // apart — an instant request stalls its timed `LDA $4016` one CPU clock late.
            if self.sample_buffer_empty && self.bytes_remaining > 0 && self.pending_fetch.is_none() {
                self.start_delay = if on_even_cycle { 2 } else { 3 };
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
        // 428 — rate index 0, what a zeroed `$4010` selects — because the timer free-runs from
        // power-on and needs its real period, not a zero that would expire every cycle.
        assert_eq!(channel.timer, 428);
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
        // Reset restores the power-on period, same as `new` — see above.
        assert_eq!(channel.timer, 428);
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

        // Enable the channel. The parity only times a fetch's request, which this test never gets
        // to — either value works here.
        channel.set_enabled(true, true);

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

    /// The three behaviours that place a DMC DMA on the right CPU cycle. Each was measured
    /// against tetanes' cycle ledger on 2026-08-06, after `dma_4016_read` reported our doubled
    /// read one CPU clock late — iteration 4 of its five runs against hardware's 3 — and eight
    /// reasoned attempts had been reverted. The ledger showed the two emulators' DMC refills
    /// agreeing to the cycle and the whole error in how a *starting* sample's stall lands; these
    /// pin the mechanism that fixed it, because the ROMs that measure it cannot run in CI.
    mod dma_placement {
        use super::*;

        /// A channel with a one-byte sample programmed, as `sync_dmc` sets one up.
        fn programmed() -> DmcChannel {
            let mut channel = DmcChannel::new();
            channel.write_register(0, 0x00); // rate 428, no loop, no IRQ
            channel.write_register(2, 0x00); // sample at $C000
            channel.write_register(3, 0x00); // length 1 byte
            channel
        }

        /// A `$4015`-started fetch surfaces 2 cycles after an even write and 3 after an odd one —
        /// never on the write itself. Both delays land the request on the same parity, which is
        /// what lets a starting sample's stall be deterministically one cycle shorter than a
        /// refill's. Requested instantly — as this code did before 2026-08-06 — every run of
        /// `dma_4016_read`'s timed `LDA $4016` sees the halt one CPU clock late.
        #[test]
        fn a_started_fetch_surfaces_two_cycles_after_an_even_write_and_three_after_an_odd() {
            for (even, expected) in [(true, 2u32), (false, 3u32)] {
                let mut channel = programmed();
                channel.set_enabled(true, even);
                assert!(
                    !channel.wants_fetch(),
                    "the request must not surface on the write cycle itself"
                );
                let mut ticks = 0;
                while !channel.wants_fetch() {
                    channel.tick();
                    ticks += 1;
                    assert!(ticks < 10, "the started fetch never surfaced");
                }
                assert_eq!(
                    ticks, expected,
                    "an {} write should surface its fetch after exactly {expected} cycles",
                    if even { "even" } else { "odd" }
                );
            }
        }

        /// The timer free-runs from power-on: `$4015` gates only the memory reader. Two channels
        /// whose enables differ by one idle cycle must reload — and so request their refills — on
        /// cycles one apart, because the phase accumulated while disabled. A timer that only ran
        /// while enabled erased that difference, which let the refill's parity float with when
        /// the game last enabled the channel — and `sync_dmc`'s loops, calibrated around a
        /// constant 4-cycle refill stall, hung whenever it floated wrong.
        #[test]
        fn the_timer_keeps_its_phase_while_the_channel_is_disabled() {
            let refill_tick = |idle: u32| {
                let mut channel = programmed();
                channel.write_register(3, 0x01); // 17 bytes, so a refill follows the first fetch
                for _ in 0..idle {
                    channel.tick(); // disabled: only the free-running timer moves
                }
                channel.set_enabled(true, true);
                while !channel.wants_fetch() {
                    channel.tick();
                }
                channel.take_pending_fetch();
                channel.supply_byte(0xAA);
                let mut ticks = 0u32;
                while !channel.wants_fetch() {
                    channel.tick();
                    ticks += 1;
                    assert!(ticks < 5000, "no refill was ever requested");
                }
                ticks
            };

            let (base, shifted) = (refill_tick(0), refill_tick(1));
            assert_eq!(
                base,
                shifted + 1,
                "one idle cycle before enabling must move the refill by exactly one cycle; if it \
                 moves by zero the timer only ran while enabled"
            );
        }

        /// The refill is requested in the same cycle the shifter takes the buffer, not on the
        /// next one. There must be no cycle in which the buffer sits empty with bytes remaining
        /// and no request out — one such cycle flips the parity every refill DMA lands on.
        #[test]
        fn the_refill_is_requested_in_the_cycle_the_shifter_reloads() {
            let mut channel = programmed();
            channel.write_register(3, 0x01); // 17 bytes
            channel.set_enabled(true, true);
            while !channel.wants_fetch() {
                channel.tick();
            }
            channel.take_pending_fetch();
            channel.supply_byte(0xAA);

            for tick in 0..5000 {
                channel.tick();
                if channel.sample_buffer_empty {
                    assert!(
                        channel.wants_fetch(),
                        "tick {tick}: the buffer emptied into the shifter but the refill was not \
                         requested in the same cycle"
                    );
                    return;
                }
            }
            panic!("the shifter never reloaded");
        }
    }
}
