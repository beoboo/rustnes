
/// CPU status flags
#[derive(Debug, Clone, Copy)]
pub enum CpuFlag {
    Carry      = 0b00000001,
    Zero       = 0b00000010,
    InterruptDisable = 0b00000100,
    DecimalMode = 0b00001000, // Not used in NES, but part of the 6502 spec
    Break      = 0b00010000, // Not a real flag, used during CPU stack operations
    Unused     = 0b00100000, // Bit 5 is unused, always set to 1
    Overflow   = 0b01000000,
    Negative   = 0b10000000,
}

/// The 6502 CPU implementation for the NES (Ricoh 2A03)
pub struct Cpu {
    // Registers
    pub a: u8,      // Accumulator
    pub x: u8,      // X index register
    pub y: u8,      // Y index register
    pub sp: u8,     // Stack pointer (0x00-0xFF, 0x100-0x1FF in memory)
    pub pc: u16,    // Program counter
    pub status: u8, // Status register (flags)
    
    // CPU cycle count
    pub cycles: u64,
}

impl Cpu {
    /// Create a new CPU instance initialized to power-up state
    pub fn new() -> Self {
        // Initial state according to NES specs
        // See: https://www.nesdev.org/wiki/CPU_power_up_state
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD, // Initial stack pointer
            pc: 0,    // Will be set to the reset vector
            status: 0x34, // 0b00110100 - Unused bit and Interrupt disable set
            cycles: 0,
        }
    }
    
    /// Get the value of a specific CPU flag
    pub fn get_flag(&self, flag: CpuFlag) -> bool {
        (self.status & flag as u8) != 0
    }
    
    /// Set a specific CPU flag to the given value
    pub fn set_flag(&mut self, flag: CpuFlag, value: bool) {
        if value {
            self.status |= flag as u8;
        } else {
            self.status &= !(flag as u8);
        }
    }
    
    // We'll add more methods for CPU operations here
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_flags() {
        let mut cpu = Cpu::new();
        
        // Test flag is initially not set
        assert!(!cpu.get_flag(CpuFlag::Zero));
        
        // Test setting a flag
        cpu.set_flag(CpuFlag::Zero, true);
        assert!(cpu.get_flag(CpuFlag::Zero));
        
        // Test clearing a flag
        cpu.set_flag(CpuFlag::Zero, false);
        assert!(!cpu.get_flag(CpuFlag::Zero));
    }
}