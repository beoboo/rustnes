use std::{cell::RefCell, rc::Rc};

use crate::{audio::AudioOutput, errors::NesError, memory::Addressable};

mod dmc_channel;
mod envelope;
mod length_counter;
mod noise_channel;
mod pulse_channel;
mod sweep;
mod triangle_channel;
use dmc_channel::DmcChannel;
use noise_channel::NoiseChannel;
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
            0x400C..=0x400F | // Noise channel registers
            0x4010..=0x4013 | // DMC channel registers
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

    // Noise channel
    noise: NoiseChannel,

    // DMC channel
    dmc: DmcChannel,

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

            // Initialize noise channel
            noise: NoiseChannel::new(),

            // Initialize DMC channel
            dmc: DmcChannel::new(),

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

        // Reset noise channel
        self.noise.reset();

        // Reset DMC channel
        self.dmc.reset();

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
        // Increment cycle counter
        self.cycle_counter += 1;

        // Process frame counter
        if self.cycle_counter % QUARTER_FRAME_PERIOD == 0 {
            // Process quarter frame
            self.pulse1.tick_envelope();
            self.pulse2.tick_envelope();
            self.triangle.tick_linear_counter();
            self.noise.tick_envelope();
        }

        if self.cycle_counter % (QUARTER_FRAME_PERIOD * 2) == 0 {
            // Process half frame
            self.pulse1.tick_sweep();
            self.pulse2.tick_sweep();
            self.pulse1.tick_length_counter();
            self.pulse2.tick_length_counter();
            self.triangle.tick_length_counter();
            self.noise.tick_length_counter();
        }

        // Process channel cycles
        self.pulse1.tick();
        self.pulse2.tick();
        self.triangle.tick();
        self.noise.tick();
        self.dmc.tick();

        // Generate samples
        self.generate_sample();
    }

    fn generate_sample(&mut self) {
        // Only generate samples if we have an audio output device
        if let Some(audio_output) = &mut self.audio_output {
            // Get samples from each channel
            let pulse1_sample = self.pulse1.generate_sample();
            let pulse2_sample = self.pulse2.generate_sample();
            let triangle_sample = self.triangle.generate_sample();
            let noise_sample = self.noise.generate_sample();
            let dmc_sample = self.dmc.generate_sample();

            // Mix pulse channels (with 95.88/15 scaling)
            let pulse_out = (pulse1_sample + pulse2_sample) * 95.88f32 / 15.0f32;

            // Mix TND channels (with respective scalings)
            let tnd_out = (triangle_sample * 159.79 + noise_sample * 159.79f32 + dmc_sample * 127.0f32) / 15.0f32;

            // Combine outputs and normalize to [-1.0, 1.0]
            let mixed_sample = (pulse_out + tnd_out) / 400.0f32;

            // Output the sample
            audio_output.queue_sample(mixed_sample);
        }
    }

    /// Connect an audio output device
    pub fn connect_audio_output(&mut self, audio_output: Box<dyn AudioOutput>) {
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
            0x4000..=0x400F | 0x4015 | 0x4017 => true,
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
                if self.noise.is_length_counter_active() {
                    status |= 0x08;
                }
                if self.dmc.is_length_counter_active() {
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

    #[test]
    fn test_all_channels_mixing() -> Result<()> {
        let mut apu = Apu::new();
        let test_output = Rc::new(RefCell::new(TestAudioOutput::new()));

        // Test 1: Individual channel contributions
        // Configure pulse channel 1 only
        apu.write_byte(0x4000, 0b01111111)?; // 25% duty, constant volume (15)
        apu.write_byte(0x4001, 0x08)?; // No sweep
        apu.write_byte(0x4002, 0x0F)?; // Timer low
        apu.write_byte(0x4015, 0x01)?; // Enable only pulse 1
        apu.write_byte(0x4003, 0x08)?; // Timer high and length counter (non-zero length)

        // Set duty position to 0 for 25% duty cycle
        apu.pulse1.set_duty_pos(0);

        // Connect test output
        apu.connect_audio_output(Box::new(TestOutputWrapper(test_output.clone())));

        // Generate samples
        for _ in 0..100 {
            apu.tick();
        }

        // Verify pulse 1 output (should be scaled by 95.88/15)
        let pulse1_samples: Vec<f32> = test_output.borrow().samples.iter().map(|&s| s * 400.0f32).collect();
        let pulse1_max = pulse1_samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(
            (pulse1_max - 95.88f32 / 15.0f32).abs() < 0.01f32,
            "Pulse 1 scaling incorrect"
        );

        // Clear samples
        test_output.borrow_mut().clear();

        // Configure pulse channel 2 only
        apu.write_byte(0x4004, 0b01111111)?; // 25% duty, constant volume (15)
        apu.write_byte(0x4005, 0x08)?; // No sweep
        apu.write_byte(0x4006, 0x0F)?; // Timer low
        apu.write_byte(0x4007, 0x08)?; // Timer high and length counter (non-zero length)

        // Set duty position to 0 for 25% duty cycle
        apu.pulse2.set_duty_pos(0);

        // Generate samples
        for _ in 0..100 {
            apu.tick();
        }

        // Verify pulse 2 output (should be scaled by 95.88/15)
        let pulse2_samples: Vec<f32> = test_output.borrow().samples.iter().map(|&s| s * 400.0f32).collect();
        let pulse2_max = pulse2_samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(
            (pulse2_max - 95.88f32 / 15.0f32).abs() < 0.01f32,
            "Pulse 2 scaling incorrect"
        );

        // Clear samples
        test_output.borrow_mut().clear();

        // Configure triangle channel only
        apu.write_byte(0x4015, 0x04)?; // Enable only triangle
        apu.write_byte(0x4008, 0b10001111)?; // Linear counter control (reload value 15, halt flag set)
        apu.write_byte(0x400A, 0x01)?; // Timer low
        apu.write_byte(0x400B, 0x00)?; // Timer high and length counter

        // Generate samples
        for _ in 0..100 {
            apu.tick();
        }

        // Verify triangle output (should be scaled by 159.79/15)
        let triangle_samples: Vec<f32> = test_output.borrow().samples.iter().map(|&s| s * 400.0f32).collect();
        let triangle_max = triangle_samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(
            (triangle_max - 159.79f32 / 15.0f32).abs() < 0.01f32,
            "Triangle scaling incorrect"
        );

        // Clear samples
        test_output.borrow_mut().clear();

        // Configure noise channel only
        apu.write_byte(0x4015, 0x08)?; // Enable only noise
        apu.write_byte(0x400C, 0b00011111)?; // Volume control
        apu.write_byte(0x400E, 0x00)?; // Mode and period
        apu.write_byte(0x400F, 0x08)?; // Length counter load

        // Generate samples
        for _ in 0..100 {
            apu.tick();
        }

        // Verify noise output (should be scaled by 159.79/15)
        let noise_samples: Vec<f32> = test_output.borrow().samples.iter().map(|&s| s * 400.0f32).collect();
        let noise_max = noise_samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(
            (noise_max - 159.79f32 / 15.0f32).abs() < 0.01f32,
            "Noise scaling incorrect"
        );

        // Clear samples
        test_output.borrow_mut().clear();

        // Configure DMC channel only
        apu.write_byte(0x4010, 0x00)?; // Sample rate and loop
        apu.write_byte(0x4011, 0x7F)?; // Direct load (maximum)
        apu.write_byte(0x4012, 0x00)?; // Sample address
        apu.write_byte(0x4013, 0x01)?; // Sample length
        apu.write_byte(0x4015, 0x10)?; // Enable only DMC

        // Generate samples
        for _ in 0..100 {
            apu.tick();
        }

        // Verify DMC output (should be scaled by 127.0/15)
        let dmc_samples: Vec<f32> = test_output.borrow().samples.iter().map(|&s| s * 400.0f32).collect();
        let dmc_max = dmc_samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!((dmc_max - 127.0f32 / 15.0f32).abs() < 0.01f32, "DMC scaling incorrect");

        // Test 2: Combined channel mixing

        // Clear samples
        test_output.borrow_mut().clear();

        // Enable all channels first
        apu.write_byte(0x4015, 0x1F)?;

        // Configure all channels with maximum values
        // Pulse 1
        apu.write_byte(0x4000, 0b01111111)?; // 25% duty, constant volume (15)
        apu.write_byte(0x4001, 0x08)?; // No sweep
        apu.write_byte(0x4002, 0x0F)?; // Timer low
        apu.write_byte(0x4003, 0x08)?; // Timer high and length counter
        apu.pulse1.set_duty_pos(0);

        // Pulse 2
        apu.write_byte(0x4004, 0b01111111)?; // 25% duty, constant volume (15)
        apu.write_byte(0x4005, 0x08)?; // No sweep
        apu.write_byte(0x4006, 0x0F)?; // Timer low
        apu.write_byte(0x4007, 0x08)?; // Timer high and length counter
        apu.pulse2.set_duty_pos(0);

        // Triangle
        apu.write_byte(0x4008, 0b10001111)?; // Linear counter control (reload value 15, halt flag set)
        apu.write_byte(0x400A, 0x01)?; // Timer low
        apu.write_byte(0x400B, 0x08)?; // Timer high and length counter

        // Noise
        apu.write_byte(0x400C, 0b00011111)?; // Volume control
        apu.write_byte(0x400E, 0x00)?; // Mode and period
        apu.write_byte(0x400F, 0x08)?; // Length counter load

        // DMC
        apu.write_byte(0x4010, 0x00)?; // Sample rate and loop
        apu.write_byte(0x4011, 0x7F)?; // Direct load (maximum)
        apu.write_byte(0x4012, 0x00)?; // Sample address
        apu.write_byte(0x4013, 0x01)?; // Sample length

        // Generate samples
        for _ in 0..100 {
            apu.tick();
        }

        // Verify combined output
        let mixed_samples: Vec<f32> = test_output.borrow().samples.iter().map(|&s| s * 400.0f32).collect();
        let mixed_max = mixed_samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));

        // We're using the empirically measured maximum value for assertion
        // This is based on all channels at their peak levels
        let expected_max = 33.323288f32;

        // Since we're measuring the maximum across all samples, we just need to verify
        // that the maximum is close to the expected maximum
        assert!(
            (mixed_max - expected_max).abs() < 0.5f32,
            "Combined output scaling incorrect"
        );

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
