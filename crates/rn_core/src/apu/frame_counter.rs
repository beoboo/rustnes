//! The APU frame counter (frame sequencer).
//!
//! A free-running divider off the CPU clock that clocks the envelopes, the triangle's linear
//! counter, the sweep units and the length counters at fixed points in a repeating sequence. It
//! has nothing to do with video frames despite the name.
//!
//! Two modes, selected by bit 7 of `$4017`:
//!
//! - **4-step** (bit clear): quarter frames at ~240 Hz, half frames at ~120 Hz, and an IRQ raised
//!   at the end of each sequence unless inhibited.
//! - **5-step** (bit set): the same clocks spread over a longer sequence, giving ~192 Hz and
//!   ~96 Hz, and no IRQ ever. Writing the bit also clocks quarter *and* half frames immediately,
//!   which music drivers rely on to get a deterministic starting point.
//!
//! Step boundaries are in CPU cycles, twice the APU-cycle figures usually quoted, because this is
//! driven once per CPU cycle.

/// What a step of the sequence should clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct FrameClock {
    /// Envelopes and the triangle's linear counter.
    pub quarter_frame: bool,
    /// Sweep units and length counters. Always accompanied by a quarter frame on hardware.
    pub half_frame: bool,
}

impl FrameClock {
    const NONE: Self = Self {
        quarter_frame: false,
        half_frame: false,
    };
    const QUARTER: Self = Self {
        quarter_frame: true,
        half_frame: false,
    };
    /// A half frame also clocks the quarter-frame units.
    const BOTH: Self = Self {
        quarter_frame: true,
        half_frame: true,
    };

    #[cfg(test)]
    pub fn is_idle(&self) -> bool {
        !self.quarter_frame && !self.half_frame
    }
}

/// 4-step sequence: (CPU cycle within the sequence, what to clock).
const FOUR_STEP: [(u64, FrameClock); 4] = [
    (7457, FrameClock::QUARTER),
    (14913, FrameClock::BOTH),
    (22371, FrameClock::QUARTER),
    (29829, FrameClock::BOTH),
];

/// Where the 4-step sequence wraps. One cycle past its last clock.
const FOUR_STEP_LENGTH: u64 = 29830;

/// The cycles across which the 4-step sequence holds its IRQ flag up.
///
/// Three of them, not one. The flag goes up at 29828, before the last clock rather than with it,
/// and is set again on each of the two cycles that follow — so a program that reads `$4015` in the
/// middle of that window clears it and finds it set again immediately.
///
/// Raising it only at the wrap put it two cycles late, which is what `apu_test/6-irq_flag_timing`
/// reports as "flag first set too late" and `4-jitter` as "frame irq is set too late".
const FOUR_STEP_IRQ: std::ops::RangeInclusive<u64> = 29828..=29830;

/// 5-step sequence. One step longer, and it never raises an IRQ.
const FIVE_STEP: [(u64, FrameClock); 4] = [
    (7457, FrameClock::QUARTER),
    (14913, FrameClock::BOTH),
    (22371, FrameClock::QUARTER),
    (37281, FrameClock::BOTH),
];

/// Where the 5-step sequence wraps.
const FIVE_STEP_LENGTH: u64 = 37282;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Mode {
    FourStep,
    FiveStep,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FrameCounter {
    mode: Mode,
    /// CPU cycles since the sequence last restarted.
    cycle: u64,
    /// Set by bit 6 of `$4017`. When set, the frame IRQ never fires and any pending one is cleared.
    irq_inhibit: bool,
    /// Latched IRQ status, reported by bit 6 of `$4015` and cleared when `$4015` is read.
    irq_pending: bool,
    /// Countdown to a pending `$4017` reset, in CPU cycles.
    ///
    /// A write to `$4017` does not take effect immediately, and how long it waits depends on *when*
    /// it lands: three CPU cycles if the write falls on an APU cycle, four if it falls between two.
    /// The APU is clocked every other CPU cycle, so this is the parity of the cycle the write
    /// happened on and nothing more.
    ///
    /// A fixed three loses the distinction, which `apu_test/4-jitter` exists to catch — its name is
    /// the jitter a program sees when it writes `$4017` on alternating cycles.
    pending_reset: Option<u8>,

    /// Whether the next CPU cycle is one the APU is clocked on.
    ///
    /// Free-running: never reset by a `$4017` write, because the APU's divider is not what the
    /// write restarts.
    ///
    /// The phase is not arbitrary. `apu_test/4-jitter` pins it, by measuring the `$4017` delay on
    /// alternating cycles — which makes this the one divider in the machine whose alignment is
    /// backed by evidence, and the reason [`Apu::apu_cycle`](super::Apu) is kept equal to it rather
    /// than counting to its own phase.
    apu_cycle: bool,

    
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameCounter {
    pub fn new() -> Self {
        Self {
            mode: Mode::FourStep,
            cycle: 0,
            irq_inhibit: false,
            irq_pending: false,
            pending_reset: None,
            apu_cycle: true,
        }
    }

    /// Press reset.
    ///
    /// The mode survives it. `apu_reset/4017_written` states the rule: "at power, $4017 = $00; at
    /// reset, $4017 mode is unchanged, but IRQ inhibit flag is sometimes cleared". A program that
    /// put the sequencer in 5-step mode and then pressed reset finds it still there.
    ///
    /// The sequence restarts, and it restarts *now*. The three-or-four cycle deferral belongs to a
    /// write to `$4017` landing between APU cycles and not to the button; the cycles the machine
    /// spends starting up are already run by the system, and charging both put `4017_timing`'s
    /// second stage out of range — it printed the right figure for power-on and then waited for an
    /// interrupt that came too late, forever.
    pub fn reset(&mut self) {
        self.irq_pending = false;
        self.irq_inhibit = false;
        self.cycle = 0;
    }


    /// Whether a frame IRQ is currently asserted.
    ///
    /// The CPU does not yet have an interrupt line, so nothing consumes this — but the flag is
    /// maintained correctly so `$4015` reads report it, and so wiring it up later is a connection
    /// rather than an implementation.
    pub fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    /// Read and clear the IRQ flag, as reading `$4015` does on hardware.
    pub fn take_irq(&mut self) -> bool {
        let pending = self.irq_pending;
        self.irq_pending = false;
        pending
    }

    /// Handle a write to `$4017`.
    ///
    /// Returns the clocks to apply immediately: writing the 5-step bit clocks quarter and half
    /// frames right away, which is how a music driver forces a known state.
    pub fn write(&mut self, value: u8) -> FrameClock {
        self.mode = if value & 0x80 != 0 { Mode::FiveStep } else { Mode::FourStep };
        self.irq_inhibit = value & 0x40 != 0;

        // Setting the inhibit flag clears any IRQ that is already pending.
        if self.irq_inhibit {
            self.irq_pending = false;
        }

        // Three cycles if the write lands on an APU cycle, four if it lands between two.
        self.pending_reset = Some(if self.apu_cycle { 3 } else { 4 });

        match self.mode {
            Mode::FiveStep => FrameClock::BOTH,
            Mode::FourStep => FrameClock::NONE,
        }
    }

    /// Which half of the divide-by-two the CPU cycle just ticked fell on.
    pub fn is_apu_cycle(&self) -> bool {
        self.apu_cycle
    }

    /// Advance one CPU cycle and report what should be clocked.
    pub fn tick(&mut self) -> FrameClock {
        // Apply a deferred $4017 reset before advancing, so the restarted sequence begins on the
        // correct cycle.
        if let Some(delay) = self.pending_reset {
            if delay == 0 {
                self.pending_reset = None;
                self.cycle = 0;
            } else {
                self.pending_reset = Some(delay - 1);
            }
        }

        self.cycle += 1;
        self.apu_cycle = !self.apu_cycle;

        let (sequence, length): (&[(u64, FrameClock)], u64) = match self.mode {
            Mode::FourStep => (&FOUR_STEP, FOUR_STEP_LENGTH),
            Mode::FiveStep => (&FIVE_STEP, FIVE_STEP_LENGTH),
        };

        let clock = sequence
            .iter()
            .find(|(cycle, _)| *cycle == self.cycle)
            .map(|(_, clock)| *clock)
            .unwrap_or(FrameClock::NONE);

        // The 4-step sequence holds its IRQ up across the last three cycles. The 5-step never
        // raises one at all.
        if matches!(self.mode, Mode::FourStep)
            && !self.irq_inhibit
            && FOUR_STEP_IRQ.contains(&self.cycle)
        {
            self.irq_pending = true;
        }

        if self.cycle == length {
            self.cycle = 0;
        }

        clock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run `cycles` ticks, returning every non-idle clock and the cycle it happened on.
    fn collect(counter: &mut FrameCounter, cycles: u64) -> Vec<(u64, FrameClock)> {
        let mut events = Vec::new();
        for cycle in 1..=cycles {
            let clock = counter.tick();
            if !clock.is_idle() {
                events.push((cycle, clock));
            }
        }
        events
    }

    #[test]
    fn four_step_sequence_has_the_documented_shape() {
        let mut counter = FrameCounter::new();
        let events = collect(&mut counter, 29830);

        assert_eq!(events.len(), 4, "4-step mode should clock four times per sequence");
        assert_eq!(events[0], (7457, FrameClock::QUARTER));
        assert_eq!(events[1], (14913, FrameClock::BOTH));
        assert_eq!(events[2], (22371, FrameClock::QUARTER));
        assert_eq!(events[3], (29829, FrameClock::BOTH), "the last clock is one cycle before the wrap");
    }

    #[test]
    fn five_step_sequence_has_the_documented_shape() {
        let mut counter = FrameCounter::new();
        counter.write(0x80);
        // Consume the deferred reset so the sequence starts cleanly.
        collect(&mut counter, 4);

        let events = collect(&mut counter, 37282);
        assert_eq!(events.len(), 4, "5-step mode clocks four times, over a longer sequence");
        assert_eq!(events.last().unwrap().1, FrameClock::BOTH);
    }

    #[test]
    fn quarter_frames_run_at_about_240hz_in_four_step_mode() {
        let mut counter = FrameCounter::new();
        // One second of CPU cycles.
        let events = collect(&mut counter, 1_789_773);
        let quarters = events.iter().filter(|(_, c)| c.quarter_frame).count();

        assert!(
            (239..=241).contains(&quarters),
            "expected ~240 quarter frames per second, got {quarters}"
        );
    }

    #[test]
    fn quarter_frames_run_at_about_192hz_in_five_step_mode() {
        let mut counter = FrameCounter::new();
        counter.write(0x80);
        collect(&mut counter, 4);

        let events = collect(&mut counter, 1_789_773);
        let quarters = events.iter().filter(|(_, c)| c.quarter_frame).count();

        assert!(
            (191..=193).contains(&quarters),
            "expected ~192 quarter frames per second, got {quarters}"
        );
    }

    #[test]
    fn half_frames_are_half_the_rate_of_quarter_frames() {
        let mut counter = FrameCounter::new();
        // A whole number of sequences: an arbitrary window would cut one mid-sequence and count a
        // quarter frame whose half frame has not happened yet.
        let events = collect(&mut counter, 29830 * 60);

        let quarters = events.iter().filter(|(_, c)| c.quarter_frame).count();
        let halves = events.iter().filter(|(_, c)| c.half_frame).count();

        assert_eq!(halves * 2, quarters, "every half frame also clocks a quarter frame");
    }

    /// The frame IRQ goes up three cycles before the sequence wraps, and stays up across all three.
    ///
    /// Not one cycle at the wrap, which is what this asserted before and what `6-irq_flag_timing`
    /// calls "flag first set too late". The width matters as much as the position: a program that
    /// reads `$4015` inside the window clears the flag and finds it set again on the next cycle,
    /// which is the behaviour `4-jitter` is built to catch.
    #[test]
    fn four_step_mode_raises_its_irq_across_the_last_three_cycles() {
        let mut counter = FrameCounter::new();

        assert!(!counter.irq_pending());
        collect(&mut counter, 29827);
        assert!(!counter.irq_pending(), "IRQ raised too early");

        counter.tick(); // cycle 29828
        assert!(counter.irq_pending(), "the flag goes up before the last clock, not with it");

        // Reading $4015 clears it, and the very next cycle puts it back.
        counter.take_irq();
        assert!(!counter.irq_pending());
        counter.tick(); // cycle 29829
        assert!(counter.irq_pending(), "still asserted on the second of the three");

        counter.take_irq();
        counter.tick(); // cycle 29830, the wrap
        assert!(counter.irq_pending(), "and on the third");

        // And then not again until the next sequence comes round.
        counter.take_irq();
        collect(&mut counter, 100);
        assert!(!counter.irq_pending(), "the window has passed");
    }

    #[test]
    fn reading_the_status_register_clears_the_irq() {
        let mut counter = FrameCounter::new();
        collect(&mut counter, 29830);

        assert!(counter.take_irq(), "first read should report the IRQ");
        assert!(!counter.irq_pending(), "reading must clear it");
        assert!(!counter.take_irq(), "second read should report nothing");
    }

    #[test]
    fn five_step_mode_never_raises_an_irq() {
        let mut counter = FrameCounter::new();
        counter.write(0x80);

        collect(&mut counter, 1_789_773);
        assert!(!counter.irq_pending(), "5-step mode must not raise a frame IRQ");
    }

    #[test]
    fn the_inhibit_flag_suppresses_and_clears_the_irq() {
        let mut counter = FrameCounter::new();
        collect(&mut counter, 29830);
        assert!(counter.irq_pending());

        // Setting the inhibit flag clears whatever is already pending...
        counter.write(0x40);
        assert!(!counter.irq_pending(), "inhibit should clear a pending IRQ");

        // ...and stops new ones.
        collect(&mut counter, 29830 * 2);
        assert!(!counter.irq_pending(), "inhibit should suppress new IRQs");
    }

    #[test]
    fn writing_the_five_step_bit_clocks_immediately() {
        let mut counter = FrameCounter::new();
        assert_eq!(counter.write(0x80), FrameClock::BOTH);
    }

    #[test]
    fn writing_four_step_mode_does_not_clock_immediately() {
        let mut counter = FrameCounter::new();
        assert_eq!(counter.write(0x00), FrameClock::NONE);
    }

    #[test]
    fn writing_restarts_the_sequence() {
        let mut counter = FrameCounter::new();

        // Get most of the way to the first quarter frame, then restart.
        collect(&mut counter, 7000);
        counter.write(0x00);

        // The restart is deferred a few cycles, then the full period must elapse again — so no
        // quarter frame at the 457 cycles that were left before the write.
        let events = collect(&mut counter, 1000);
        assert!(events.is_empty(), "sequence should have restarted, got {events:?}");
    }
}
