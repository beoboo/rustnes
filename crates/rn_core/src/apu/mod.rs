use std::{
    cell::RefCell,
    fmt::Debug,
    rc::Rc,
};

use crate::{errors::NesError, memory::Addressable};

// Required APU register constants for simple tone test
const PULSE1_CONTROL: u16 = 0x4000;    // Volume/Duty/Envelope control
const PULSE1_SWEEP: u16 = 0x4001;      // Sweep control
const PULSE1_TIMER_LO: u16 = 0x4002;   // Timer low byte
const PULSE1_TIMER_HI: u16 = 0x4003;   // Timer high byte
const APU_STATUS: u16 = 0x4015;        // APU status/control

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
    // Pulse channel 1 registers
    pulse1_control: u8,
    pulse1_sweep: u8,
    pulse1_timer_lo: u8,
    pulse1_timer_hi: u8,
    
    // Status register ($4015)
    status: u8,
    
    // Internal state
    pulse1_enabled: bool,
}

impl Apu {
    /// Create a new APU instance
    pub fn new() -> Self {
        Self {
            // Initialize all registers to 0
            pulse1_control: 0,
            pulse1_sweep: 0,
            pulse1_timer_lo: 0,
            pulse1_timer_hi: 0,
            status: 0,
            
            // Channel initially disabled
            pulse1_enabled: false,
        }
    }
    
    /// Reset the APU to initial state
    pub fn reset(&mut self) {
        // Reset all registers to 0
        self.pulse1_control = 0;
        self.pulse1_sweep = 0;
        self.pulse1_timer_lo = 0;
        self.pulse1_timer_hi = 0;
        self.status = 0;
        
        // Disable channel
        self.pulse1_enabled = false;
    }
    
    /// Process a single APU cycle
    pub fn tick(&mut self) {
        // For minimal implementation, just track that we got called
        // No actual audio generation yet
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
                let value = if self.pulse1_enabled { 0x01 } else { 0x00 };
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
                self.pulse1_control = value;
                Ok(())
            },
            PULSE1_SWEEP => {
                self.pulse1_sweep = value;
                Ok(())
            },
            PULSE1_TIMER_LO => {
                self.pulse1_timer_lo = value;
                Ok(())
            },
            PULSE1_TIMER_HI => {
                self.pulse1_timer_hi = value;
                Ok(())
            },
            
            // APU status register ($4015)
            APU_STATUS => {
                // Update channel enable flag (only bit 0 for pulse channel 1)
                self.pulse1_enabled = (value & 0x01) != 0;
                self.status = value;
                Ok(())
            },
            
            // Ignore other registers for minimal implementation
            _ => Ok(()),
        }
    }
    
    fn reset(&mut self) {
        // Use our own implementation instead of calling self.reset() recursively
        // This was causing stack overflow in tests
        // Reset all registers to 0
        self.pulse1_control = 0;
        self.pulse1_sweep = 0;
        self.pulse1_timer_lo = 0;
        self.pulse1_timer_hi = 0;
        self.status = 0;
        
        // Disable channel
        self.pulse1_enabled = false;
    }
} 