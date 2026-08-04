use std::{cell::RefCell, rc::Rc};

use crate::{audio::SampleProducer, errors::NesError, memory::Addressable};
use derive_more::Debug;

mod dmc_channel;
mod envelope;
mod filter;
mod frame_counter;
mod length_counter;
mod mixer;
mod noise_channel;
mod pulse_channel;
mod sweep;
mod triangle_channel;
use dmc_channel::DmcChannel;
use filter::OutputFilter;
use frame_counter::{FrameClock, FrameCounter};
use mixer::Mixer;
use noise_channel::NoiseChannel;
use pulse_channel::PulseChannel;
use triangle_channel::TriangleChannel;

// Required APU register constants for simple tone test
const APU_STATUS: u16 = 0x4015; // APU status/control

/// NES CPU clock rate (NTSC), in Hz.
pub const CPU_CLOCK_RATE: f64 = 1_789_773.0;

/// Sample rate assumed until a real audio device reports its own.
pub const DEFAULT_SAMPLE_RATE: f64 = 44_100.0;

/// The APU's five sound channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Pulse1,
    Pulse2,
    Triangle,
    Noise,
    Dmc,
}

impl Channel {
    pub const ALL: [Channel; 5] = [
        Channel::Pulse1,
        Channel::Pulse2,
        Channel::Triangle,
        Channel::Noise,
        Channel::Dmc,
    ];

    /// This channel's enable bit in `$4015`.
    pub fn status_bit(&self) -> u8 {
        match self {
            Channel::Pulse1 => 0x01,
            Channel::Pulse2 => 0x02,
            Channel::Triangle => 0x04,
            Channel::Noise => 0x08,
            Channel::Dmc => 0x10,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Channel::Pulse1 => "Pulse 1",
            Channel::Pulse2 => "Pulse 2",
            Channel::Triangle => "Triangle",
            Channel::Noise => "Noise",
            Channel::Dmc => "DMC",
        }
    }
}

/// APU status/control register bits.
const STATUS_FRAME_IRQ: u8 = 0x40;
/// Set while the DMC is holding an interrupt, and cleared by reading or writing $4015.
const STATUS_DMC_IRQ: u8 = 0x80;

/// Everything about the APU that cannot be recomputed.
///
/// Deliberately excludes three things. The output device's sample rate and the resampling
/// accumulator are properties of the sound card the snapshot is *restored* onto, not of the
/// machine that was saved. The output filter's memory settles inaudibly within milliseconds. And
/// the audio sink itself is a connection, not state.
///
/// What is here is what a game can observe or hear: the five channels with their timers, envelopes,
/// sweeps and length counters, the status register, and the frame sequencer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApuState {
    pulse1: PulseChannel,
    pulse2: PulseChannel,
    triangle: TriangleChannel,
    noise: NoiseChannel,
    dmc: DmcChannel,
    status: u8,
    apu_cycle: bool,
    frame_counter: FrameCounter,
}

/// Wrapper for APU to make it easier to use with Rc/RefCell
#[derive(Clone, Debug)]
pub struct ApuWrapper {
    apu: Rc<RefCell<Apu>>,
}

impl ApuWrapper {
    /// The address the DMC wants a sample byte from, if it is waiting on one.
    pub fn take_dmc_fetch(&self) -> Option<u16> {
        self.apu.borrow_mut().take_dmc_fetch()
    }

    /// Hand the DMC the byte it asked for.
    pub fn supply_dmc_byte(&self, value: u8) {
        self.apu.borrow_mut().supply_dmc_byte(value);
    }

    /// Capture the APU's state.
    pub fn save_state(&self) -> ApuState {
        self.apu.borrow().save_state()
    }

    /// Restore a captured APU state.
    pub fn load_state(&self, state: &ApuState) {
        self.apu.borrow_mut().load_state(state);
    }

    /// Create a new APU wrapper
    pub fn new(apu: Apu) -> Self {
        Self {
            apu: Rc::new(RefCell::new(apu)),
        }
    }

    /// Reset the APU
    pub fn reset(&self) {
        self.apu.borrow_mut().reset();
    }

    /// Process a single APU tick
    pub fn tick(&self) {
        self.apu.borrow_mut().tick();
    }

    /// Connect an audio output device
    pub fn connect_audio_output(&self, audio_output: Box<dyn SampleProducer<f32>>) {
        self.apu.borrow_mut().connect_audio_output(audio_output);
    }

    /// Set the output sample rate, in Hz — take this from the real audio device.
    pub fn set_sample_rate(&self, sample_rate: f64) {
        self.apu.borrow_mut().set_sample_rate(sample_rate);
    }

    pub fn sample_rate(&self) -> f64 {
        self.apu.borrow().sample_rate()
    }

    pub fn set_volume(&self, volume: f32) {
        self.apu.borrow_mut().set_volume(volume);
    }

    /// Whether a channel is currently enabled via `$4015`.
    pub fn channel_enabled(&self, channel: Channel) -> bool {
        self.apu.borrow().channel_enabled(channel)
    }

    /// Whether the frame counter is asserting its IRQ.
    ///
    /// Level-triggered: stays true until the program acknowledges it by reading `$4015`.
    pub fn irq_pending(&self) -> bool {
        self.apu.borrow().irq_pending()
    }

    pub fn set_muted(&self, muted: bool) {
        self.apu.borrow_mut().set_muted(muted);
    }
}

impl Addressable for ApuWrapper {
    fn handles_address(&self, address: u16) -> bool {
        // Handle all APU registers
        match address {
            0x4000..=0x4003 | // Pulse 1 registers
            0x4004..=0x4007 | // Pulse 2 registers
            0x4008..=0x400B | // Triangle channel registers
            0x400C..=0x400F | // Noise channel registers
            0x4010..=0x4013 | // DMC channel registers
            0x4015 => true,   // APU status/control
            _ => false,
        }
    }

    fn handles_write(&self, address: u16) -> bool {
        // $4017 is write-only for the APU: it sets the frame counter's mode and IRQ inhibit.
        // Reads of $4017 belong to controller 2, so it is deliberately absent from
        // `handles_address` above.
        self.handles_address(address) || address == 0x4017
    }

    fn read_byte(&self, address: u16) -> Result<u8, NesError> {
        let value = self.apu.borrow().read_byte(address)?;

        // Reading $4015 acknowledges the frame IRQ. `Apu::read_byte` cannot do this itself —
        // `Addressable::read_byte` takes `&self` — but the wrapper owns the `RefCell`, so the
        // side effect belongs here.
        if address == APU_STATUS {
            self.apu.borrow_mut().acknowledge_frame_irq();
        }

        Ok(value)
    }

    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError> {
        self.apu.borrow_mut().write_byte(address, value)
    }
}

/// The NES Audio Processing Unit (APU) - Minimal implementation
#[derive(Debug)]
pub struct Apu {
    // Pulse channels
    pulse1: PulseChannel,
    pulse2: PulseChannel,

    // Triangle channel
    triangle: TriangleChannel,

    // Noise channel
    noise: NoiseChannel,

    // DMC channel
    dmc: DmcChannel,

    // Status register ($4015)
    status: u8,

    // Audio output device
    #[debug(skip)]
    audio_output: Option<Box<dyn SampleProducer<f32>>>,

    // Non-linear channel mixer
    mixer: Mixer,

    // 90 Hz / 440 Hz high-pass and 14 kHz low-pass applied to the mixed output
    filter: OutputFilter,

    // Sample generation state
    cycle_counter: u64,

    /// Divides the CPU clock by two to get the APU clock.
    ///
    /// Pulse and noise timers are clocked at APU rate; triangle runs at full CPU rate. Getting
    /// this wrong puts pulse and noise an octave out.
    apu_cycle: bool,

    /// Resampling accumulator.
    ///
    /// The APU is evaluated once per CPU cycle (~1.79 MHz) but the audio device wants ~48 kHz, so
    /// `samples_per_cycle` is added each tick and a sample is emitted whenever it crosses 1.0.
    /// Without this the buffer is fed ~37× faster than it drains.
    sample_counter: f64,
    samples_per_cycle: f64,
    sample_rate: f64,

    /// Running sum of mixed values since the last emitted sample, and how many were summed.
    ///
    /// Averaging the discarded intermediate values instead of point-sampling costs one add per
    /// cycle and removes most of the aliasing that naive decimation would fold into the audible
    /// band.
    sample_accumulator: f64,
    accumulated_cycles: u32,

    /// Frame sequencer: clocks envelopes, sweeps and length counters, and raises the frame IRQ.
    frame_counter: FrameCounter,
}

impl Apu {
    /// The address the DMC wants a sample byte from, if it is waiting on one.
    pub fn take_dmc_fetch(&mut self) -> Option<u16> {
        self.dmc.take_pending_fetch()
    }

    /// Hand the DMC the byte it asked for.
    pub fn supply_dmc_byte(&mut self, value: u8) {
        self.dmc.supply_byte(value);
    }

    /// Capture everything that cannot be recomputed.
    pub fn save_state(&self) -> ApuState {
        ApuState {
            pulse1: self.pulse1.clone(),
            pulse2: self.pulse2.clone(),
            triangle: self.triangle.clone(),
            noise: self.noise.clone(),
            dmc: self.dmc.clone(),
            status: self.status,
            apu_cycle: self.apu_cycle,
            frame_counter: self.frame_counter.clone(),
        }
    }

    /// Restore a captured state, leaving the connection to the sound card as it is.
    pub fn load_state(&mut self, state: &ApuState) {
        self.pulse1 = state.pulse1.clone();
        self.pulse2 = state.pulse2.clone();
        self.triangle = state.triangle.clone();
        self.noise = state.noise.clone();
        self.dmc = state.dmc.clone();
        self.status = state.status;
        self.apu_cycle = state.apu_cycle;
        self.frame_counter = state.frame_counter.clone();
    }

    /// Create a new APU instance
    pub fn new() -> Self {
        let mut apu = Self::at_power_up();
        // One implementation of the power-up state, not two: the fields below are the struct's
        // shape and this is what makes them the documented values.
        apu.power_on();
        apu
    }

    fn at_power_up() -> Self {
        Self {
            // Initialize pulse channels
            pulse1: PulseChannel::new(true),  // Pulse 1
            pulse2: PulseChannel::new(false), // Pulse 2

            // Initialize triangle channel
            triangle: TriangleChannel::new(),

            // Initialize noise channel
            noise: NoiseChannel::new(),

            // Initialize DMC channel
            dmc: DmcChannel::new(),

            // Initialize status register
            status: 0,

            // No audio output initially
            audio_output: None,

            mixer: Mixer::new(),
            filter: OutputFilter::new(DEFAULT_SAMPLE_RATE as f32),

            // Sample generation state
            cycle_counter: 0,
            apu_cycle: false,
            sample_counter: 0.0,
            samples_per_cycle: DEFAULT_SAMPLE_RATE / CPU_CLOCK_RATE,
            sample_rate: DEFAULT_SAMPLE_RATE,
            sample_accumulator: 0.0,
            accumulated_cycles: 0,

            frame_counter: FrameCounter::new(),
        }
    }

    /// Set the output sample rate, in Hz.
    ///
    /// Called with the audio device's real rate once one is connected, so resampling and the
    /// output filters are computed against what the hardware actually wants rather than an
    /// assumed 44.1 kHz.
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        if sample_rate <= 0.0 {
            return;
        }

        self.sample_rate = sample_rate;
        self.samples_per_cycle = sample_rate / CPU_CLOCK_RATE;
        self.filter = OutputFilter::new(sample_rate as f32);
        self.sample_counter = 0.0;
        self.sample_accumulator = 0.0;
        self.accumulated_cycles = 0;
    }

    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    /// Reset the APU to initial state
    /// Restore the state the APU has when the machine is switched on.
    ///
    /// Distinct from [`reset`](Self::reset), and the distinction is the whole of what `apu_reset`
    /// checks. Power-on clears everything: every channel register, every envelope, every counter.
    /// Reset clears `$4015` and leaves `$4000-$4013` exactly as they were.
    ///
    /// The two used to be the same function, which is why pressing reset silently wiped the
    /// triangle's linear counter control along with everything else.
    pub fn power_on(&mut self) {
        self.pulse1.reset();
        self.pulse2.reset();
        self.triangle.reset();
        self.noise.reset();
        self.dmc.reset();

        self.status = 0;
        self.cycle_counter = 0;
        self.apu_cycle = false;
        self.sample_counter = 0.0;
        self.sample_accumulator = 0.0;
        self.accumulated_cycles = 0;
        self.filter.reset();
        self.frame_counter = FrameCounter::new();
    }

    /// Press reset.
    ///
    /// This is not power-on, and the difference is what `apu_reset` measures. `$4000-$4013` are
    /// *unchanged* by a reset: the duty settings, envelope flags, periods and — the one the ROM
    /// names — the triangle's linear counter control all survive it. All that happens is `$4015`
    /// being cleared, which silences every channel and empties the length counters while leaving
    /// them able to be loaded again.
    ///
    /// `len_ctrs_enabled` catches the difference precisely: it sets the triangle's halt flag
    /// before the reset and afterwards expects the triangle's length counter still to be sitting
    /// where it was loaded while the other three have run down. Clearing the channels outright
    /// takes that flag with them.
    pub fn reset(&mut self) {
        // Clearing $4015 is the whole of what reset does to the channels.
        let _ = self.write_byte(APU_STATUS, 0);

        // Reset sample generation state
        self.cycle_counter = 0;
        self.apu_cycle = false;
        self.sample_counter = 0.0;
        self.sample_accumulator = 0.0;
        self.accumulated_cycles = 0;
        self.filter.reset();

        self.frame_counter.reset();
    }

    /// Advance the APU by one **CPU** cycle.
    ///
    /// Not every unit runs at this rate: the pulse and noise timers are clocked at APU rate
    /// (CPU / 2) while the triangle runs at full CPU rate, and the frame sequencer runs at
    /// ~240 Hz. The divider below is what keeps those three domains apart.
    pub fn tick(&mut self) {
        self.cycle_counter += 1;
        self.apu_cycle = !self.apu_cycle;

        // Frame sequencer. Rates depend on the mode selected through $4017.
        let clock = self.frame_counter.tick();
        self.apply_frame_clock(clock);

        // Pulse and noise timers advance once per APU cycle...
        if self.apu_cycle {
            self.pulse1.tick();
            self.pulse2.tick();
            self.noise.tick();
        }

        // ...while the triangle's sequencer advances every CPU cycle, which is why it can reach
        // frequencies the pulse channels cannot.
        self.triangle.tick();

        // The DMC belongs with the triangle rather than with the pulse channels: its rate table is
        // quoted in CPU cycles, so clocking it at the APU's half rate played every sample an
        // octave low and fetched its bytes half as often as hardware does.
        self.dmc.tick();

        self.generate_sample();
    }

    /// Clock the units a frame-sequencer step asks for.
    ///
    /// A half frame always accompanies a quarter frame on hardware, so the quarter-frame units are
    /// clocked first and unconditionally when either applies.
    fn apply_frame_clock(&mut self, clock: FrameClock) {
        if clock.quarter_frame {
            self.pulse1.tick_envelope();
            self.pulse2.tick_envelope();
            self.triangle.tick_linear_counter();
            self.noise.tick_envelope();
        }

        if clock.half_frame {
            self.pulse1.tick_sweep();
            self.pulse2.tick_sweep();
            self.pulse1.tick_length_counter();
            self.pulse2.tick_length_counter();
            self.triangle.tick_length_counter();
            self.noise.tick_length_counter();
        }
    }

    /// Clear the frame IRQ, as reading `$4015` does on hardware.
    pub fn acknowledge_frame_irq(&mut self) {
        self.frame_counter.take_irq();
    }

    /// Whether a channel is currently enabled via `$4015`.
    ///
    /// Lets the UI show what the running program actually asked for, rather than only what was
    /// last clicked.
    pub fn channel_enabled(&self, channel: Channel) -> bool {
        match channel {
            Channel::Pulse1 => self.pulse1.is_enabled(),
            Channel::Pulse2 => self.pulse2.is_enabled(),
            Channel::Triangle => self.triangle.is_enabled(),
            Channel::Noise => self.noise.is_enabled(),
            Channel::Dmc => self.dmc.is_enabled(),
        }
    }

    /// Whether the frame counter is asserting an IRQ.
    ///
    /// The CPU has no interrupt line yet, so nothing consumes this; the flag is maintained
    /// correctly so `$4015` reports it, and so connecting it later is wiring rather than work.
    pub fn irq_pending(&self) -> bool {
        // Two sources share the line: the frame sequencer and the DMC finishing a sample. The CPU
        // sees only the combination, so a handler has to read $4015 to find out which fired.
        self.frame_counter.irq_pending() || self.dmc.irq_pending()
    }

    /// Mix the current channel levels into one sample in roughly 0.0..=1.0.
    ///
    /// Kept separate from the resampling in [`Apu::generate_sample`] so it can be tested against
    /// the reference formula without involving any timing.
    fn mix(&self) -> f32 {
        self.mixer.mix(
            self.pulse1.output(),
            self.pulse2.output(),
            self.triangle.output(),
            self.noise.output(),
            self.dmc.output(),
        )
    }

    /// Accumulate this cycle's mix and emit a filtered sample when one is due.
    ///
    /// The APU is evaluated at ~1.79 MHz and the device wants ~48 kHz, so roughly 37 cycles are
    /// averaged into each emitted sample.
    fn generate_sample(&mut self) {
        if self.audio_output.is_none() {
            return;
        }

        self.sample_accumulator += self.mix() as f64;
        self.accumulated_cycles += 1;
        self.sample_counter += self.samples_per_cycle;

        if self.sample_counter < 1.0 {
            return;
        }
        self.sample_counter -= 1.0;

        let averaged = (self.sample_accumulator / self.accumulated_cycles as f64) as f32;
        self.sample_accumulator = 0.0;
        self.accumulated_cycles = 0;

        // The filters run on the decimated stream, and the 90 Hz high-pass is what removes the
        // DC offset the unipolar mix leaves behind.
        let filtered = self.filter.process(averaged);

        if let Some(audio_output) = &mut self.audio_output {
            audio_output.produce(filtered);
        }
    }

    /// Connect an audio output device.
    pub fn connect_audio_output(&mut self, audio_output: Box<dyn SampleProducer<f32>>) {
        self.audio_output = Some(audio_output);
    }

    /// Set the volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        if let Some(audio_output) = &mut self.audio_output {
            audio_output.set_volume(volume);
        }
    }

    /// Set muted state
    pub fn set_muted(&mut self, muted: bool) {
        if let Some(audio_output) = &mut self.audio_output {
            audio_output.set_muted(muted);
        }
    }
}

impl Addressable for Apu {
    fn handles_address(&self, address: u16) -> bool {
        // Handle APU registers
        matches!(address, 0x4000..=0x400F | 0x4015 | 0x4017)
    }

    fn read_byte(&self, address: u16) -> Result<u8, NesError> {
        match address {
            // APU status register ($4015)
            APU_STATUS => {
                // Build status register from channel states.
                //
                // Note this cannot clear the frame IRQ, because `Addressable::read_byte` takes
                // `&self`. `ApuWrapper` owns the RefCell and does the clearing; see its
                // `read_byte`.
                let mut status = 0;
                if self.frame_counter.irq_pending() {
                    status |= STATUS_FRAME_IRQ;
                }
                if self.dmc.irq_pending() {
                    status |= STATUS_DMC_IRQ;
                }
                if self.pulse1.is_length_counter_active() {
                    status |= 0x01;
                }
                if self.pulse2.is_length_counter_active() {
                    status |= 0x02;
                }
                if self.triangle.is_length_counter_active() {
                    status |= 0x04;
                }
                if self.noise.is_length_counter_active() {
                    status |= 0x08;
                }
                // The DMC reports bytes left to fetch, not a length counter — it has none.
                if self.dmc.has_bytes_remaining() {
                    status |= 0x10;
                }
                Ok(status)
            },
            // Other registers are write-only in the actual NES
            _ => Ok(0),
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError> {
        match address {
            // Pulse channel 1 registers
            0x4000..=0x4003 => {
                let reg_offset = address - 0x4000;
                self.pulse1.write_register(reg_offset, value);
                Ok(())
            },
            // Pulse channel 2 registers
            0x4004..=0x4007 => {
                let reg_offset = address - 0x4004;
                self.pulse2.write_register(reg_offset, value);
                Ok(())
            },
            // Triangle channel registers
            0x4008..=0x400B => {
                let reg_offset = address - 0x4008;
                self.triangle.write_register(reg_offset, value);
                Ok(())
            },
            // Noise channel registers
            0x400C..=0x400F => {
                let reg_offset = address - 0x400C;
                self.noise.write_register(reg_offset, value);
                Ok(())
            },
            // DMC channel registers
            0x4010..=0x4013 => {
                let reg_offset = address - 0x4010;
                self.dmc.write_register(reg_offset, value);
                Ok(())
            },
            // APU status register ($4015)
            APU_STATUS => {
                // Update channel enable states
                self.pulse1.set_enabled((value & 0x01) != 0);
                self.pulse2.set_enabled((value & 0x02) != 0);
                self.triangle.set_enabled((value & 0x04) != 0);
                self.noise.set_enabled((value & 0x08) != 0);
                self.dmc.set_enabled((value & 0x10) != 0);
                // Writing the register clears the DMC's interrupt, whatever else the write does.
                self.dmc.acknowledge_irq();
                self.status = value;
                Ok(())
            },
            // Frame counter register ($4017)
            0x4017 => {
                // Selecting 5-step mode clocks quarter and half frames immediately, which is how
                // music drivers force a known starting state.
                let immediate = self.frame_counter.write(value);
                self.apply_frame_clock(immediate);
                Ok(())
            },
            // Ignore other registers for minimal implementation
            _ => Ok(()),
        }
    }
}


impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    /// Bit 4 of $4015 reports the DMC's remaining sample bytes, not a length counter.
    ///
    /// The DMC has no length counter. Reporting one made the bit describe something the hardware
    /// does not have: it read as silent while a sample was still playing, and as playing after one
    /// had finished — and a game polling it to know when to queue the next sample would act on
    /// both mistakes.
    #[test]
    fn dmc_status_follows_the_bytes_left_to_fetch() -> Result<()> {
        let mut apu = Apu::new();

        assert_eq!(apu.read_byte(0x4015)? & 0x10, 0, "nothing to fetch before it is enabled");

        // Enabling starts a fetch. Even a sample length of zero is one byte, since the register
        // counts in units of sixteen bytes plus one — there is no way to ask for none.
        apu.write_byte(0x4013, 0x01)?;
        apu.write_byte(0x4015, 0x10)?;
        assert_ne!(apu.read_byte(0x4015)? & 0x10, 0, "an enabled sample should show as active");

        apu.write_byte(0x4015, 0x00)?; // disabling clears the remaining bytes
        assert_eq!(apu.read_byte(0x4015)? & 0x10, 0, "disabling should stop it reporting");

        Ok(())
    }

    use anyhow::Result;

    use super::*;

    const PULSE1_CONTROL: u16 = 0x4000; // Volume/Duty/Envelope control
    const PULSE1_SWEEP: u16 = 0x4001; // Sweep control
    const PULSE1_TIMER_LO: u16 = 0x4002; // Timer low byte
    const PULSE1_TIMER_HI: u16 = 0x4003; // Timer high byte

    const TEST_SAMPLE_RATE: f64 = 48_000.0;

    /// CPU cycles in one full 4-step frame-counter sequence.
    const FOUR_STEP_SEQUENCE_CYCLES: usize = 29_830;

    /// Collects everything the APU emits, so tests can assert on the real output stream.
    ///
    /// Shared with the APU through a channel rather than `Rc<RefCell<_>>`: `SampleProducer` is
    /// `Send`, and a plain channel satisfies that without any `unsafe` claims.
    #[derive(std::fmt::Debug)]
    struct TestAudioOutput {
        sender: std::sync::mpsc::Sender<f32>,
    }

    /// The test-side handle: drains whatever the APU has produced so far.
    struct Captured {
        receiver: std::sync::mpsc::Receiver<f32>,
        samples: Vec<f32>,
    }

    impl Captured {
        fn new() -> (TestAudioOutput, Self) {
            let (sender, receiver) = std::sync::mpsc::channel();
            (
                TestAudioOutput { sender },
                Self {
                    receiver,
                    samples: Vec::new(),
                },
            )
        }

        fn samples(&mut self) -> &[f32] {
            while let Ok(sample) = self.receiver.try_recv() {
                self.samples.push(sample);
            }
            &self.samples
        }


        fn peak(&mut self) -> f32 {
            self.samples().iter().fold(0.0f32, |a, &b| a.max(b.abs()))
        }
    }

    impl SampleProducer<f32> for TestAudioOutput {
        fn set_volume(&mut self, _volume: f32) {}

        fn set_muted(&mut self, _muted: bool) {}

        fn produce(&mut self, sample: f32) {
            let _ = self.sender.send(sample);
        }
    }

    /// An APU with a capture buffer attached, running at a known sample rate.
    fn apu_with_capture() -> (Apu, Captured) {
        let mut apu = Apu::new();
        let (output, captured) = Captured::new();
        apu.set_sample_rate(TEST_SAMPLE_RATE);
        apu.connect_audio_output(Box::new(output));
        (apu, captured)
    }

    fn tick_cycles(apu: &mut Apu, cycles: usize) {
        for _ in 0..cycles {
            apu.tick();
        }
    }

    /// Program pulse 1 as a steady, audible tone: constant volume 15, no sweep muting,
    /// length counter loaded.
    fn program_pulse1_tone(apu: &mut Apu, timer_lo: u8, timer_hi: u8) -> Result<()> {
        apu.write_byte(PULSE1_CONTROL, 0b0101_1111)?; // 25% duty, constant volume 15
        apu.write_byte(PULSE1_SWEEP, 0x08)?; // negate set, so the sweep never mutes
        apu.write_byte(PULSE1_TIMER_LO, timer_lo)?;
        apu.write_byte(APU_STATUS, 0x01)?; // enable pulse 1
        apu.write_byte(PULSE1_TIMER_HI, timer_hi)?; // timer high + length counter load
        Ok(())
    }

    #[test]
    fn test_apu_new() {
        let apu = Apu::new();

        assert_eq!(apu.status, 0);
        assert_eq!(apu.cycle_counter, 0);
        assert_eq!(apu.sample_counter, 0.0);
        assert!(apu.audio_output.is_none());
        assert_eq!(apu.sample_rate(), DEFAULT_SAMPLE_RATE);
    }

    #[test]
    fn test_apu_reset() {
        let mut apu = Apu::new();

        apu.status = 0x0F;
        apu.cycle_counter = 1000;
        apu.sample_counter = 0.5;
        apu.apu_cycle = true;

        apu.reset();

        assert_eq!(apu.status, 0);
        assert_eq!(apu.cycle_counter, 0);
        assert_eq!(apu.sample_counter, 0.0);
        assert!(!apu.apu_cycle);
    }

    #[test]
    fn test_no_samples_without_output() -> Result<()> {
        // With no output connected the APU must still tick correctly, just produce nothing.
        let mut apu = Apu::new();

        program_pulse1_tone(&mut apu, 0x40, 0x08)?;
        tick_cycles(&mut apu, 10_000);

        assert!(apu.audio_output.is_none());
        Ok(())
    }

    #[test]
    fn test_sample_generation() -> Result<()> {
        let (mut apu, mut captured) = apu_with_capture();

        program_pulse1_tone(&mut apu, 0x40, 0x08)?;
        tick_cycles(&mut apu, 100_000);

        assert!(!captured.samples().is_empty(), "No samples were generated");
        assert!(
            captured.samples().iter().any(|&s| s.abs() > 0.001),
            "All samples were silent"
        );
        Ok(())
    }

    /// The regression test for the resampling defect.
    ///
    /// The APU is evaluated once per CPU cycle (~1.79 MHz) but must emit at the device's rate.
    /// Before resampling existed this produced ~37x too many samples, which is exactly what made
    /// the output unlistenable.
    #[test]
    fn test_emits_at_the_device_sample_rate() -> Result<()> {
        let (mut apu, mut captured) = apu_with_capture();

        program_pulse1_tone(&mut apu, 0x40, 0x08)?;

        // Run for one emulated second.
        tick_cycles(&mut apu, CPU_CLOCK_RATE as usize);

        let produced = captured.samples().len() as f64;
        let error = (produced - TEST_SAMPLE_RATE).abs() / TEST_SAMPLE_RATE;

        assert!(
            error < 0.01,
            "expected ~{TEST_SAMPLE_RATE} samples in one emulated second, got {produced}"
        );
        Ok(())
    }

    #[test]
    fn test_sample_rate_follows_the_device() -> Result<()> {
        let mut apu = Apu::new();
        let (output, mut captured) = Captured::new();
        apu.set_sample_rate(22_050.0);
        apu.connect_audio_output(Box::new(output));

        program_pulse1_tone(&mut apu, 0x40, 0x08)?;
        tick_cycles(&mut apu, CPU_CLOCK_RATE as usize);

        let produced = captured.samples().len() as f64;
        assert!(
            (produced - 22_050.0).abs() / 22_050.0 < 0.01,
            "expected ~22050 samples, got {produced}"
        );
        Ok(())
    }

    /// The regression test for the clock-domain defect.
    ///
    /// Pulse timers are clocked at APU rate (CPU / 2), so a pulse channel completes one duty
    /// period every `8 * (timer + 1)` APU cycles — twice that many CPU cycles. Clocking it per CPU
    /// cycle instead put every pulse tone an octave sharp.
    #[test]
    fn test_pulse_runs_at_apu_rate() -> Result<()> {
        let mut apu = Apu::new();

        let timer: u16 = 100;
        apu.write_byte(PULSE1_CONTROL, 0b0101_1111)?;
        apu.write_byte(PULSE1_SWEEP, 0x08)?;
        apu.write_byte(PULSE1_TIMER_LO, (timer & 0xFF) as u8)?;
        apu.write_byte(APU_STATUS, 0x01)?;
        apu.write_byte(PULSE1_TIMER_HI, 0x08 | ((timer >> 8) as u8))?;

        // The timer starts at 0, so the very first step lands after a single cycle. Wait for it
        // before timing, or that partial period skews the measurement.
        let mut guard = 0;
        while apu.pulse1.duty_pos() == 0 && guard < 10_000 {
            apu.tick();
            guard += 1;
        }

        // Count how many CPU cycles it takes to walk the 8-step duty sequence once.
        let start = apu.pulse1.duty_pos();
        let mut steps = 0;
        let mut cycles = 0usize;
        while steps < 8 && cycles < 10_000 {
            let before = apu.pulse1.duty_pos();
            apu.tick();
            cycles += 1;
            if apu.pulse1.duty_pos() != before {
                steps += 1;
            }
        }

        assert_eq!(steps, 8, "duty sequence did not complete");
        assert_eq!(apu.pulse1.duty_pos(), start, "duty position should wrap to where it began");

        // 8 steps x (timer + 1) APU cycles x 2 CPU cycles per APU cycle.
        let expected = 8 * (timer as usize + 1) * 2;
        assert!(
            (cycles as i64 - expected as i64).abs() <= 2,
            "pulse period was {cycles} CPU cycles, expected ~{expected} (an octave error would give ~{})",
            expected / 2
        );
        Ok(())
    }

    /// The triangle channel is the one that really is clocked every CPU cycle.
    #[test]
    fn test_triangle_runs_at_cpu_rate() -> Result<()> {
        let mut apu = Apu::new();

        let timer: u16 = 100;
        apu.write_byte(0x4008, 0b1000_1111)?; // linear counter reload 15, control set
        apu.write_byte(0x400A, (timer & 0xFF) as u8)?;
        apu.write_byte(APU_STATUS, 0x04)?; // enable triangle
        apu.write_byte(0x400B, 0x08 | ((timer >> 8) as u8))?;

        // Same cold-start allowance as the pulse test.
        let mut guard = 0;
        while apu.triangle.sequence_pos() == 0 && guard < 10_000 {
            apu.tick();
            guard += 1;
        }

        let mut steps = 0;
        let mut cycles = 0usize;
        while steps < 4 && cycles < 10_000 {
            let before = apu.triangle.sequence_pos();
            apu.tick();
            cycles += 1;
            if apu.triangle.sequence_pos() != before {
                steps += 1;
            }
        }

        assert_eq!(steps, 4, "triangle sequence did not advance");

        // 4 steps x (timer + 1) CPU cycles — no divider.
        let expected = 4 * (timer as usize + 1);
        assert!(
            (cycles as i64 - expected as i64).abs() <= 2,
            "triangle period was {cycles} CPU cycles, expected ~{expected}"
        );
        Ok(())
    }

    #[test]
    fn test_simple_tone_sequence() -> Result<()> {
        let (mut apu, mut captured) = apu_with_capture();

        program_pulse1_tone(&mut apu, 0x08, 0x01)?;
        tick_cycles(&mut apu, 200_000);

        assert!(!captured.samples().is_empty(), "No samples were generated");
        assert!(
            captured.samples().iter().any(|&s| s.abs() > 0.001),
            "All samples were silent"
        );
        assert!(apu.pulse1.is_enabled(), "Pulse channel should be enabled");
        Ok(())
    }

    /// The mixed signal must swing around zero once the output filters have settled.
    ///
    /// The mixer is deliberately unipolar — silence is 0.0, matching the hardware DAC — so this is
    /// what proves the high-pass stages are actually removing that offset.
    #[test]
    fn test_output_has_no_dc_offset() -> Result<()> {
        let (mut apu, mut captured) = apu_with_capture();

        program_pulse1_tone(&mut apu, 0x40, 0x08)?;
        tick_cycles(&mut apu, CPU_CLOCK_RATE as usize);

        let samples = captured.samples();
        // Skip the filters' settling time.
        let settled = &samples[samples.len() / 2..];
        let mean = settled.iter().sum::<f32>() / settled.len() as f32;

        assert!(mean.abs() < 0.01, "output has a DC offset of {mean}");
        assert!(
            settled.iter().any(|&s| s < 0.0) && settled.iter().any(|&s| s > 0.0),
            "output never crosses zero, so it is not a waveform"
        );
        Ok(())
    }

    #[test]
    fn test_length_counter_in_apu() -> Result<()> {
        let mut apu = Apu::new();

        apu.write_byte(PULSE1_CONTROL, 0b0101_1111)?; // 25% duty, constant volume 15
        apu.write_byte(PULSE1_SWEEP, 0x08)?;
        apu.write_byte(PULSE1_TIMER_LO, 0x08)?;
        apu.write_byte(APU_STATUS, 0x01)?;
        apu.write_byte(PULSE1_TIMER_HI, 0x18)?; // length index 3 = 2 (very short)

        apu.pulse1.set_duty_pos(0); // a position where the duty pattern is high

        assert!(apu.mix() > 0.0, "expected audible output while the length counter is active");

        // Exhaust the length counter.
        apu.pulse1.tick_length_counter();
        apu.pulse1.tick_length_counter();

        assert_eq!(
            apu.read_byte(APU_STATUS)? & 0x01,
            0,
            "Pulse 1 should be silent after length counter expires"
        );
        assert_eq!(apu.mix(), 0.0, "expected silence after the length counter expires");
        Ok(())
    }

    #[test]
    fn test_length_counter_halt() -> Result<()> {
        let mut apu = Apu::new();

        apu.write_byte(APU_STATUS, 0x01)?;
        apu.write_byte(PULSE1_CONTROL, 0b0010_0000)?; // halt set, constant volume 0
        apu.write_byte(PULSE1_TIMER_LO, 0x08)?;
        apu.write_byte(PULSE1_TIMER_HI, 0x01)?;

        for _ in 0..20 {
            tick_cycles(&mut apu, FOUR_STEP_SEQUENCE_CYCLES * 2);
        }

        assert_eq!(
            apu.read_byte(APU_STATUS)? & 0x01,
            0x01,
            "Pulse 1 should still be active with length counter halt set"
        );

        apu.write_byte(PULSE1_CONTROL, 0b0000_0000)?; // clear halt

        for _ in 0..20 {
            tick_cycles(&mut apu, FOUR_STEP_SEQUENCE_CYCLES * 2);
        }

        assert_eq!(
            apu.read_byte(APU_STATUS)? & 0x01,
            0,
            "Pulse 1 should be silent after length counter expires"
        );
        Ok(())
    }

    #[test]
    fn test_reload_length_counter() -> Result<()> {
        let mut apu = Apu::new();

        apu.write_byte(PULSE1_CONTROL, 0b0101_1111)?;
        apu.write_byte(APU_STATUS, 0x01)?;
        assert_eq!(apu.read_byte(APU_STATUS)? & 0x01, 0x00);

        apu.write_byte(PULSE1_TIMER_HI, 0x01)?;
        assert_eq!(apu.read_byte(APU_STATUS)? & 0x01, 0x01);

        apu.write_byte(APU_STATUS, 0x00)?;
        assert_eq!(apu.read_byte(APU_STATUS)? & 0x01, 0x00);

        apu.write_byte(APU_STATUS, 0x01)?;
        apu.write_byte(PULSE1_TIMER_HI, 0x01)?;
        assert_eq!(apu.read_byte(APU_STATUS)? & 0x01, 0x01);
        Ok(())
    }

    /// Channel mixing, asserted against the reference formula rather than against whatever the
    /// implementation happened to produce.
    ///
    /// This is tested through `mix()` rather than through the sample stream: mixing is a pure
    /// function of the five DAC levels, and involving timing, resampling and filtering here would
    /// only make a failure harder to localise.
    #[test]
    fn test_all_channels_mixing() -> Result<()> {
        let mut apu = Apu::new();

        let pulse_ref = |n: f32| 95.88 / (8128.0 / n + 100.0);
        let tnd_ref = |n: f32| 163.67 / (24329.0 / n + 100.0);

        // Silence in, silence out.
        assert_eq!(apu.mix(), 0.0);

        // Pulse 1 alone, at full volume.
        apu.write_byte(0x4000, 0b0111_1111)?; // constant volume 15
        apu.write_byte(0x4001, 0x08)?; // no sweep muting
        apu.write_byte(0x4002, 0x0F)?;
        apu.write_byte(0x4015, 0x01)?;
        apu.write_byte(0x4003, 0x08)?;
        apu.pulse1.set_duty_pos(0);

        assert_eq!(apu.pulse1.output(), 15);
        assert!((apu.mix() - pulse_ref(15.0)).abs() < 1e-6, "pulse 1 mixing incorrect");

        // Adding pulse 2 at the same level must compress, not double.
        apu.write_byte(0x4004, 0b0111_1111)?;
        apu.write_byte(0x4005, 0x08)?;
        apu.write_byte(0x4006, 0x0F)?;
        apu.write_byte(0x4015, 0x03)?;
        apu.write_byte(0x4007, 0x08)?;
        apu.pulse2.set_duty_pos(0);

        let both = apu.mix();
        assert!((both - pulse_ref(30.0)).abs() < 1e-6, "pulse pair mixing incorrect");
        assert!(
            both < 2.0 * pulse_ref(15.0),
            "mixing must be non-linear: {both} is not below {}",
            2.0 * pulse_ref(15.0)
        );

        // Triangle alone.
        let mut apu = Apu::new();
        apu.write_byte(0x4015, 0x04)?;
        apu.write_byte(0x4008, 0b1000_1111)?;
        apu.write_byte(0x400A, 0x01)?;
        apu.write_byte(0x400B, 0x08)?;

        let triangle = apu.triangle.output();
        assert!(triangle > 0, "triangle should be audible");
        assert!(
            (apu.mix() - tnd_ref(3.0 * triangle as f32)).abs() < 1e-6,
            "triangle mixing incorrect"
        );

        // DMC alone, at full scale.
        let mut apu = Apu::new();
        apu.write_byte(0x4010, 0x00)?;
        apu.write_byte(0x4011, 0x7F)?; // direct load, maximum
        apu.write_byte(0x4012, 0x00)?;
        apu.write_byte(0x4013, 0x01)?;
        apu.write_byte(0x4015, 0x10)?;

        assert_eq!(apu.dmc.output(), 127);
        assert!((apu.mix() - tnd_ref(127.0)).abs() < 1e-6, "DMC mixing incorrect");

        Ok(())
    }

    #[test]
    fn test_mix_never_clips() -> Result<()> {
        let mut apu = Apu::new();

        // Everything on, everything at maximum.
        apu.write_byte(0x4015, 0x1F)?;
        apu.write_byte(0x4000, 0b0111_1111)?;
        apu.write_byte(0x4001, 0x08)?;
        apu.write_byte(0x4002, 0x0F)?;
        apu.write_byte(0x4003, 0x08)?;
        apu.pulse1.set_duty_pos(0);
        apu.write_byte(0x4004, 0b0111_1111)?;
        apu.write_byte(0x4005, 0x08)?;
        apu.write_byte(0x4006, 0x0F)?;
        apu.write_byte(0x4007, 0x08)?;
        apu.pulse2.set_duty_pos(0);
        apu.write_byte(0x4008, 0b1000_1111)?;
        apu.write_byte(0x400A, 0x01)?;
        apu.write_byte(0x400B, 0x08)?;
        apu.write_byte(0x400C, 0b0001_1111)?;
        apu.write_byte(0x400E, 0x00)?;
        apu.write_byte(0x400F, 0x08)?;
        apu.write_byte(0x4010, 0x00)?;
        apu.write_byte(0x4011, 0x7F)?;
        apu.write_byte(0x4012, 0x00)?;
        apu.write_byte(0x4013, 0x01)?;

        // Every channel at maximum reaches ~1.001 on hardware, so a hair above unity is correct;
        // what this guards against is a scaling error putting the peak at 0.1 or at 12.
        let peak = apu.mix();
        assert!(peak > 0.5, "full-scale mix is implausibly quiet: {peak}");
        assert!(peak < 1.05, "full-scale mix would clip badly: {peak}");
        Ok(())
    }

    #[test]
    fn test_volume_and_mute_reach_the_output() -> Result<()> {
        let (mut apu, mut captured) = apu_with_capture();

        program_pulse1_tone(&mut apu, 0x40, 0x08)?;
        tick_cycles(&mut apu, 200_000);
        assert!(captured.peak() > 0.0, "expected audible output");

        // Volume and mute are applied by the output device, not by the APU, so the sample stream
        // itself is unchanged — this only proves the calls are plumbed through without panicking.
        apu.set_volume(0.5);
        apu.set_muted(true);
        apu.set_muted(false);
        Ok(())
    }
}
