use std::{cell::RefCell, rc::Rc};

use crate::{audio::AudioOutput, errors::NesError, memory::Addressable};

mod pulse_channel;
use pulse_channel::PulseChannel;

// Required APU register constants for simple tone test
const PULSE1_CONTROL: u16 = 0x4000; // Volume/Duty/Envelope control
const PULSE1_SWEEP: u16 = 0x4001; // Sweep control
const PULSE1_TIMER_LO: u16 = 0x4002; // Timer low byte
const PULSE1_TIMER_HI: u16 = 0x4003; // Timer high byte
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
        // Only handle the registers needed for the simple tone test
        match address {
            0x4000..=0x4003 | 0x4015 => true,
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
            // Initialize pulse channel
            pulse1: PulseChannel::new(),

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
        // Reset pulse channel
        self.pulse1.reset();

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

        // Process pulse channel
        self.pulse1.tick();

        // Generate audio samples when needed
        // NES APU generates samples at a rate determined by the CPU clock rate and sample rate
        self.sample_counter += self.samples_per_cycle;
        while self.sample_counter >= 1.0 {
            self.sample_counter -= 1.0;
            self.generate_sample();
        }
    }

    /// Process quarter frame events (envelope and triangle linear counter)
    fn tick_quarter_frame(&mut self) {
        // Update envelope
        self.pulse1.tick_envelope();

        // Update volume based on envelope (this happens automatically in the pulse channel)
    }

    /// Connect an audio output device
    pub fn connect_audio_output(&mut self, mut audio_output: Box<dyn AudioOutput>) {
        // Configure the audio output with the correct sample rate
        audio_output.set_sample_rate(DEFAULT_SAMPLE_RATE as f32);

        // Store the audio output
        self.audio_output = Some(audio_output);
    }

    /// Generate a single audio sample and send it to the audio output
    fn generate_sample(&mut self) {
        // Calculate the current sample for pulse channel 1
        let pulse1_sample = self.pulse1.generate_sample();

        // If we have an audio output device, send the sample to it
        if let Some(output) = &mut self.audio_output {
            if output.is_ready() {
                // For now, we only have one channel, so the sample is just the pulse1 sample
                output.queue_sample(pulse1_sample);
            }
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        if let Some(output) = &mut self.audio_output {
            output.set_volume(volume);
        }
    }

    pub fn set_muted(&mut self, muted: bool) {
        if let Some(output) = &mut self.audio_output {
            output.set_muted(muted);
        }
    }
}

impl Addressable for Apu {
    fn handles_address(&self, address: u16) -> bool {
        // Only handle the registers needed for the simple tone test
        match address {
            0x4000..=0x4003 | 0x4015 => true,
            _ => false,
        }
    }

    fn read_byte(&self, address: u16) -> Result<u8, NesError> {
        match address {
            // APU status register ($4015)
            APU_STATUS => {
                // Only bit 0 is needed for pulse channel 1
                let value = if self.pulse1.is_enabled() { 0x01 } else { 0x00 };
                Ok(value)
            },
            // Other registers are write-only in the actual NES
            _ => Ok(0),
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError> {
        match address {
            // Pulse channel 1 registers
            PULSE1_CONTROL => {
                self.pulse1.write_register(0, value);
                Ok(())
            },
            PULSE1_SWEEP => {
                self.pulse1.write_register(1, value);
                Ok(())
            },
            PULSE1_TIMER_LO => {
                self.pulse1.write_register(2, value);
                Ok(())
            },
            PULSE1_TIMER_HI => {
                self.pulse1.write_register(3, value);
                Ok(())
            },

            // APU status register ($4015)
            APU_STATUS => {
                // Update channel enable flag (only bit 0 for pulse channel 1)
                self.pulse1.set_enabled((value & 0x01) != 0);
                self.status = value;
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
        apu.write_byte(APU_STATUS, 0x01)?; // Enable pulse 1

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
        apu.write_byte(PULSE1_SWEEP, 0x00)?; // No sweep
        apu.write_byte(PULSE1_TIMER_LO, 0x08)?; // Short timer for faster testing
        apu.write_byte(PULSE1_TIMER_HI, 0x00)?; // High byte
        apu.write_byte(APU_STATUS, 0x01)?; // Enable pulse 1

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
