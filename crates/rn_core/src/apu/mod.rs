use std::{
    cell::RefCell,
    rc::Rc,
};

use crate::{errors::NesError, memory::Addressable, audio::AudioOutput};

mod pulse_channel;
use pulse_channel::PulseChannel;

// Required APU register constants for simple tone test
const PULSE1_CONTROL: u16 = 0x4000;    // Volume/Duty/Envelope control
const PULSE1_SWEEP: u16 = 0x4001;      // Sweep control
const PULSE1_TIMER_LO: u16 = 0x4002;   // Timer low byte
const PULSE1_TIMER_HI: u16 = 0x4003;   // Timer high byte
const APU_STATUS: u16 = 0x4015;        // APU status/control

// Constants for audio generation
const CPU_CLOCK_RATE: f64 = 1789773.0; // NES CPU clock rate (NTSC)
const DEFAULT_SAMPLE_RATE: u32 = 44100; // Default audio sample rate

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
        
        // Clear any pending audio output
        if let Some(audio_output) = &mut self.audio_output {
            audio_output.clear();
        }
    }
    
    /// Process a single APU cycle
    pub fn tick(&mut self) {
        // Track cycles for sample generation
        self.cycle_counter += 1;
        
        // Process pulse channel
        self.pulse1.tick();
        
        // Generate audio samples when needed
        self.sample_counter += self.samples_per_cycle;
        while self.sample_counter >= 1.0 {
            self.sample_counter -= 1.0;
            self.generate_sample();
        }
    }
    
    /// Connect an audio output device
    pub fn connect_audio_output(&mut self, mut audio_output: Box<dyn AudioOutput>) {
        audio_output.set_sample_rate(DEFAULT_SAMPLE_RATE as f32);
        
        // Store the audio output
        self.audio_output = Some(audio_output);
    }
    
    /// Generate a single audio sample and send it to the audio output
    fn generate_sample(&mut self) {
        if let Some(output) = &mut self.audio_output {
            if output.is_ready() {
                // Calculate the current sample for pulse channel 1
                let pulse1_sample = self.pulse1.generate_sample();
                
                // For now, we only have one channel, so the sample is just the pulse1 sample
                output.queue_sample(pulse1_sample);
            }
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
