use std::{cell::RefCell, rc::Rc};

use crate::{audio::AudioOutput, errors::NesError, memory::Addressable};

mod envelope;
mod length_counter;
mod pulse_channel;
mod sweep;
mod triangle_channel;
use pulse_channel::PulseChannel;
use triangle_channel::TriangleChannel;

// Required APU register constants for simple tone test
const APU_STATUS: u16 = 0x4015; // APU status/control

// Constants for audio generation
const CPU_CLOCK_RATE: f64 = 1789773.0; // NES CPU clock rate (NTSC)
const DEFAULT_SAMPLE_RATE: u32 = 44100; // Default audio sample rate

// Frame counter constants
const QUARTER_FRAME_PERIOD: u64 = 7457; // CPU cycles between quarter frame ticks (NTSC)

/// Wrapper for APU to make it easier to use with Rc/RefCell
#[derive(Clone, Debug)]
pub struct ApuWrapper {
    apu: Rc<RefCell<Apu>>,
}

impl ApuWrapper {
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
    pub fn connect_audio_output(&self, audio_output: Box<dyn AudioOutput>) {
        self.apu.borrow_mut().connect_audio_output(audio_output);
    }

    pub fn set_volume(&self, volume: f32) {
        self.apu.borrow_mut().set_volume(volume);
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
            0x4015 => true,   // APU status/control
            _ => false,
        }
    }

    fn read_byte(&self, address: u16) -> Result<u8, NesError> {
        self.apu.borrow().read_byte(address)
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

    // Status register ($4015)
    status: u8,

    // Audio output device
    audio_output: Option<Box<dyn AudioOutput>>,

    // Sample generation state
    cycle_counter: u64,
    sample_counter: f64,
    samples_per_cycle: f64,

    // Frame counter state
    frame_counter: u64,
    frame_mode: u8, // 0 = 4-step, 1 = 5-step (not fully implemented yet)
}

impl Apu {
    /// Create a new APU instance
    pub fn new() -> Self {
        Self {
            // Initialize pulse channels
            pulse1: PulseChannel::new(true),  // Pulse 1
            pulse2: PulseChannel::new(false), // Pulse 2

            // Initialize triangle channel
            triangle: TriangleChannel::new(),

            // Initialize status register
            status: 0,

            // No audio output initially
            audio_output: None,

            // Sample generation state
            cycle_counter: 0,
            sample_counter: 0.0,
            samples_per_cycle: DEFAULT_SAMPLE_RATE as f64 / CPU_CLOCK_RATE,

            // Frame counter state
            frame_counter: 0,
            frame_mode: 0, // 4-step mode by default
        }
    }

    /// Reset the APU to initial state
    pub fn reset(&mut self) {
        // Reset pulse channels
        self.pulse1.reset();
        self.pulse2.reset();

        // Reset triangle channel
        self.triangle.reset();

        // Reset status register
        self.status = 0;

        // Reset sample generation state
        self.cycle_counter = 0;
        self.sample_counter = 0.0;

        // Reset frame counter
        self.frame_counter = 0;

        // Clear any pending audio output
        if let Some(audio_output) = &mut self.audio_output {
            audio_output.clear();
        }
    }

    /// Process a single APU cycle
    pub fn tick(&mut self) {
        // Track cycles for sample generation
        self.cycle_counter += 1;

        // Process frame counter for envelopes and other clocked components
        self.frame_counter += 1;

        // Check if we need to process quarter frame events (envelope)
        if self.frame_counter % QUARTER_FRAME_PERIOD == 0 {
            self.tick_quarter_frame();
        }

        // Check if we need to process half frame events (sweep and length counter)
        if self.frame_counter % (QUARTER_FRAME_PERIOD * 2) == 0 {
            self.tick_half_frame();
        }

        // Process pulse channels
        self.pulse1.tick();
        self.pulse2.tick();

        // Process triangle channel
        self.triangle.tick();

        // Generate audio samples when needed
        // NES APU generates samples at a rate determined by the CPU clock rate and sample rate
        self.sample_counter += self.samples_per_cycle;
        while self.sample_counter >= 1.0 {
            self.sample_counter -= 1.0;
            self.generate_sample();
        }
    }

    /// Process quarter frame events (240Hz) - envelope and triangle linear counter
    fn tick_quarter_frame(&mut self) {
        // Process envelope
        self.pulse1.tick_envelope();
        self.pulse2.tick_envelope();

        // Process triangle linear counter
        self.triangle.tick_linear_counter();
    }

    /// Process half frame events (120Hz) - sweep and length counter
    fn tick_half_frame(&mut self) {
        // Process sweep
        self.pulse1.tick_sweep();
        self.pulse2.tick_sweep();

        // Process length counter
        self.pulse1.tick_length_counter();
        self.pulse2.tick_length_counter();
        self.triangle.tick_length_counter();
    }

    /// Generate and output a single audio sample
    fn generate_sample(&mut self) {
        // Only generate samples if we have an audio output device
        if let Some(audio_output) = &mut self.audio_output {
            // Mix pulse channels and triangle channel
            let pulse1_sample = self.pulse1.generate_sample();
            let pulse2_sample = self.pulse2.generate_sample();
            let triangle_sample = self.triangle.generate_sample();

            // Mix the samples (simple average for now)
            let mixed_sample = (pulse1_sample + pulse2_sample + triangle_sample) / 3.0;

            // Output the sample
            audio_output.queue_sample(mixed_sample);
        }
    }

    /// Connect an audio output device
    pub fn connect_audio_output(&mut self, mut audio_output: Box<dyn AudioOutput>) {
        // Configure the audio output with the correct sample rate
        audio_output.set_sample_rate(DEFAULT_SAMPLE_RATE as f32);

        // Store the audio output
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
        match address {
            0x4000..=0x4007 | 0x4015 | 0x4017 => true,
            _ => false,
        }
    }

    fn read_byte(&self, address: u16) -> Result<u8, NesError> {
        match address {
            // APU status register ($4015)
            APU_STATUS => {
                // Build status register from channel states
                let mut status = 0;
                if self.pulse1.is_length_counter_active() {
                    status |= 0x01;
                }
                if self.pulse2.is_length_counter_active() {
                    status |= 0x02;
                }
                if self.triangle.is_length_counter_active() {
                    status |= 0x04;
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
            // APU status register ($4015)
            APU_STATUS => {
                // Update channel enable states
                self.pulse1.set_enabled((value & 0x01) != 0);
                self.pulse2.set_enabled((value & 0x02) != 0);
                self.triangle.set_enabled((value & 0x04) != 0);
                self.status = value;
                Ok(())
            },
            // Frame counter register ($4017)
            0x4017 => {
                // Set frame counter mode
                self.frame_mode = value & 0x80;
                // Reset frame counter if bit 7 is set
                if (value & 0x80) != 0 {
                    self.frame_counter = 0;
                }
                Ok(())
            },
            // Ignore other registers for minimal implementation
            _ => Ok(()),
        }
    }

    fn reset(&mut self) {
        // Call our main reset method
        Apu::reset(self);
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;
    const PULSE1_CONTROL: u16 = 0x4000; // Volume/Duty/Envelope control
    const PULSE1_SWEEP: u16 = 0x4001; // Sweep control
    const PULSE1_TIMER_LO: u16 = 0x4002; // Timer low byte
    const PULSE1_TIMER_HI: u16 = 0x4003; // Timer high byte

    // A very simple audio output implementation for testing
    #[derive(Debug)]
    struct TestAudioOutput {
        samples: Vec<f32>,
        ready: bool,
    }

    impl TestAudioOutput {
        fn new() -> Self {
            Self {
                samples: Vec::new(),
                ready: true,
            }
        }

        fn set_ready(&mut self, ready: bool) {
            self.ready = ready;
        }
    }

    impl AudioOutput for TestAudioOutput {
        fn set_volume(&mut self, _volume: f32) {
            // Do nothing for test
        }

        fn set_muted(&mut self, _muted: bool) {
            // Do nothing for test
        }

        fn set_sample_rate(&mut self, _rate: f32) {
            // Do nothing for test
        }

        fn queue_sample(&mut self, sample: f32) {
            if self.ready {
                self.samples.push(sample);
            }
        }

        fn clear(&mut self) {
            self.samples.clear();
        }

        fn is_ready(&self) -> bool {
            self.ready
        }
    }

    #[test]
    fn test_apu_new() {
        let apu = Apu::new();

        // Check default state
        assert_eq!(apu.status, 0);
        assert_eq!(apu.cycle_counter, 0);
        assert_eq!(apu.sample_counter, 0.0);
        assert!(apu.audio_output.is_none());
    }

    #[test]
    fn test_apu_reset() {
        let mut apu = Apu::new();

        // Set some non-default values
        apu.status = 0x0F;
        apu.cycle_counter = 1000;
        apu.sample_counter = 0.5;

        // Reset and check that values are back to defaults
        apu.reset();

        assert_eq!(apu.status, 0);
        assert_eq!(apu.cycle_counter, 0);
        assert_eq!(apu.sample_counter, 0.0);
    }

    #[test]
    fn test_apu_tick_no_sample_when_not_ready() -> Result<()> {
        let mut apu = Apu::new();
        let test_output = Rc::new(RefCell::new(TestAudioOutput::new()));

        // Set output to not ready
        test_output.borrow_mut().set_ready(false);

        // Enable the pulse channel first
        apu.write_byte(PULSE1_CONTROL, 0b01011111)?; // 25% duty, constant volume (15)
        apu.write_byte(APU_STATUS, 0x01)?; // Enable pulse 1

        // Connect the test output
        apu.connect_audio_output(Box::new(TestOutputWrapper(test_output.clone())));

        // Tick several times
        for _ in 0..100 {
            apu.tick();
        }

        // Verify that no samples were queued since output was not ready
        assert_eq!(
            test_output.borrow().samples.len(),
            0,
            "Samples were queued when output was not ready"
        );

        Ok(())
    }

    #[test]
    fn test_apu_tick_sample_generation() -> Result<()> {
        let mut apu = Apu::new();
        let test_output = Rc::new(RefCell::new(TestAudioOutput::new()));

        // Configure pulse channel with a very short timer to cycle through duty positions quickly
        apu.write_byte(PULSE1_CONTROL, 0b01011111)?; // 25% duty, constant volume (15)
        apu.write_byte(PULSE1_TIMER_LO, 0x01)?; // Very short timer to cycle through positions quickly
        apu.write_byte(PULSE1_TIMER_HI, 0x00)?; // Set high timer byte

        // Disable sweep to prevent muting (for test)
        apu.write_byte(PULSE1_SWEEP, 0x08)?; // Negate flag set to prevent muting

        // Enable pulse 1 and load initial length counter value
        apu.write_byte(APU_STATUS, 0x01)?; // Enable pulse 1
        apu.write_byte(PULSE1_TIMER_HI, 0x08)?; // Reload length counter

        // Connect the test output
        apu.connect_audio_output(Box::new(TestOutputWrapper(test_output.clone())));

        // Tick many times to cycle through all duty positions and generate many samples
        for _ in 0..1000 {
            apu.tick();
        }

        // Verify that samples were generated
        assert!(test_output.borrow().samples.len() > 0, "No samples were generated");

        // Verify that some samples are non-zero
        let has_non_zero = test_output.borrow().samples.iter().any(|&s| s.abs() > 0.001);
        assert!(has_non_zero, "All samples were zero");

        Ok(())
    }

    #[test]
    fn test_simple_tone_sequence() -> Result<()> {
        let mut apu = Apu::new();
        let test_output = Rc::new(RefCell::new(TestAudioOutput::new()));

        // Program the APU to play a tone - similar to the basic_tone_test.asm
        apu.write_byte(PULSE1_CONTROL, 0b01011111)?; // 25% duty, constant volume (15)
        apu.write_byte(PULSE1_SWEEP, 0x08)?; // No sweep, negate bit set to prevent muting
        apu.write_byte(PULSE1_TIMER_LO, 0x8)?; // Short timer for faster testing

        // Enable pulse 1
        apu.write_byte(APU_STATUS, 0x01)?; // Enable pulse 1

        // Load timer high and length counter in one operation
        apu.write_byte(PULSE1_TIMER_HI, 0x01)?; // High byte (period over 8 to avoid muting)

        // Connect the test output
        apu.connect_audio_output(Box::new(TestOutputWrapper(test_output.clone())));

        // Run for a significant amount of time to generate many samples
        for _ in 0..2000 {
            apu.tick();
        }

        // Verify that samples were generated
        assert!(test_output.borrow().samples.len() > 0, "No samples were generated");

        // Verify that some samples are non-zero
        let has_non_zero = test_output.borrow().samples.iter().any(|&s| s.abs() > 0.001);
        assert!(has_non_zero, "All samples were zero");

        // Verify the APU configuration is correct for tone generation
        assert_eq!(apu.pulse1.is_enabled(), true, "Pulse channel should be enabled");

        Ok(())
    }

    #[test]
    fn test_length_counter_in_apu() -> Result<()> {
        let mut apu = Apu::new();
        let test_output = Rc::new(RefCell::new(TestAudioOutput::new()));

        // Configure pulse channel with a very short timer for quick testing
        apu.write_byte(PULSE1_CONTROL, 0b01011111)?; // 25% duty, constant volume (15)
        apu.write_byte(PULSE1_TIMER_LO, 0x08)?; // Timer low

        // Enable pulse channel
        apu.write_byte(APU_STATUS, 0x01)?;

        // Load timer high with a length value - use shortest length value
        apu.write_byte(PULSE1_TIMER_HI, 0x18)?; // Index 3 = value 2 (very short)

        // Connect test output
        apu.connect_audio_output(Box::new(TestOutputWrapper(test_output.clone())));

        // We'll directly manipulate the pulse channel for testing
        // This is more reliable than waiting for the timer to advance
        apu.pulse1.set_duty_pos(0); // Set position where output is high

        // Force a sample generation to verify we have sound
        apu.generate_sample();

        // Verify that we did produce non-zero sound
        assert!(test_output.borrow().samples.len() > 0, "No samples were generated");
        assert!(
            test_output.borrow().samples.iter().any(|&s| s > 0.0),
            "Expected non-zero samples but all were zero"
        );

        // Clear samples
        test_output.borrow_mut().clear();

        // Manually tick the length counter twice to exhaust it
        // Since we used a length of 2, this should silence the channel
        apu.pulse1.tick_length_counter();
        apu.pulse1.tick_length_counter();

        // Verify the length counter is now inactive
        assert_eq!(
            apu.read_byte(APU_STATUS)? & 0x01,
            0,
            "Pulse 1 should be silent after length counter expires"
        );

        // Generate another sample
        apu.generate_sample();

        // Verify it produced silence
        assert!(
            test_output.borrow().samples.iter().all(|&s| s == 0.0),
            "All samples should be zero after length counter expires"
        );

        Ok(())
    }

    #[test]
    fn test_length_counter_halt() -> Result<()> {
        let mut apu = Apu::new();

        // Enable pulse channel first
        apu.write_byte(APU_STATUS, 0x01)?;

        // Configure pulse with length counter halt flag set (bit 5 of control register)
        apu.write_byte(PULSE1_CONTROL, 0b00100000)?; // Halt bit set, constant volume (0)
        apu.write_byte(PULSE1_TIMER_LO, 0x08)?;

        // Now load timer high to get length counter
        apu.write_byte(PULSE1_TIMER_HI, 0x01)?; // Timer high with length counter

        // Tick many half-frames - length counter shouldn't decrement because halt is set
        for _ in 0..20 {
            // Simulate many CPU cycles to trigger multiple half-frames
            for _ in 0..QUARTER_FRAME_PERIOD * 4 {
                apu.tick();
            }
        }

        // Status register should still show pulse 1 as active
        assert_eq!(
            apu.read_byte(APU_STATUS)? & 0x01,
            0x01,
            "Pulse 1 should still be active with length counter halt set"
        );

        // Now clear the halt flag
        apu.write_byte(PULSE1_CONTROL, 0b00000000)?; // Halt bit cleared, constant volume (0)

        // Tick many half-frames - length counter should now decrement
        for _ in 0..20 {
            // Simulate many CPU cycles to trigger multiple half-frames
            for _ in 0..QUARTER_FRAME_PERIOD * 4 {
                apu.tick();
            }
        }

        // Status register should now show pulse 1 as inactive
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

        // Enable pulse channel but without loading a length counter
        apu.write_byte(PULSE1_CONTROL, 0b01011111)?; // 25% duty, constant volume (15)
        apu.write_byte(APU_STATUS, 0x01)?;

        // Check that it's not active yet (no length value loaded)
        assert_eq!(apu.read_byte(APU_STATUS)? & 0x01, 0x00);

        // Now load a length counter
        apu.write_byte(PULSE1_TIMER_HI, 0x01)?; // Any value will load a length

        // Check that it's active now
        assert_eq!(apu.read_byte(APU_STATUS)? & 0x01, 0x01);

        // Disable the channel
        apu.write_byte(APU_STATUS, 0x00)?;

        // Should be inactive
        assert_eq!(apu.read_byte(APU_STATUS)? & 0x01, 0x00);

        // Re-enable and load a length value
        apu.write_byte(APU_STATUS, 0x01)?;
        apu.write_byte(PULSE1_TIMER_HI, 0x01)?;

        // Should be active again
        assert_eq!(apu.read_byte(APU_STATUS)? & 0x01, 0x01);

        Ok(())
    }

    // Wrapper to allow sharing TestAudioOutput through Rc<RefCell>
    #[derive(Debug)]
    struct TestOutputWrapper(Rc<RefCell<TestAudioOutput>>);

    impl AudioOutput for TestOutputWrapper {
        fn set_volume(&mut self, _volume: f32) {
            // Do nothing for test
        }

        fn set_muted(&mut self, _muted: bool) {
            // Do nothing for test
        }

        fn set_sample_rate(&mut self, _rate: f32) {
            // Do nothing for test
        }

        fn queue_sample(&mut self, sample: f32) {
            if self.0.borrow().ready {
                self.0.borrow_mut().samples.push(sample);
            }
        }

        fn clear(&mut self) {
            self.0.borrow_mut().samples.clear();
        }

        fn is_ready(&self) -> bool {
            self.0.borrow().ready
        }
    }
}
