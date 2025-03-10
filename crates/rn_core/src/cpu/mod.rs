use crate::memory::Addressable;

mod addressing_mode;
pub use addressing_mode::AddressingMode;

mod instruction;
pub use instruction::{Instruction, InstructionDecoder, InstructionMetadata};

mod error;
pub use error::CpuError;

mod assembler;
pub use assembler::{Assembler, AssembleError, ParseResult};

mod disassembler;
pub use disassembler::{Disassembler, DisassembleError};

/// CPU status flags
#[derive(Debug, Clone, Copy)]
#[rustfmt::skip]
pub enum CpuFlag {
    Carry            = 0b00000001,
    Zero             = 0b00000010,
    InterruptDisable = 0b00000100,
    DecimalMode      = 0b00001000, // Not used in NES, but part of the 6502 spec
    Break            = 0b00010000, // Not a real flag, used during CPU stack operations
    Unused           = 0b00100000, // Bit 5 is unused, always set to 1
    Overflow         = 0b01000000,
    Negative         = 0b10000000,
}

/// MOS 6502 CPU implementation
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

    // Memory connection
    memory: Box<dyn Addressable>,
    
    // Instruction decoder
    decoder: InstructionDecoder,
}

impl Cpu {
    /// Create a new CPU instance initialized to power-up state with the provided memory
    pub fn new(memory: Box<dyn Addressable>) -> Self {
        // Initial state according to NES specs
        // See: https://www.nesdev.org/wiki/CPU_power_up_state
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,     // Initial stack pointer
            pc: 0,        // Will be set to the reset vector
            status: 0x34, // 0b00110100 - Unused bit and Interrupt disable set
            cycles: 0,
            memory,
            decoder: InstructionDecoder::new(),
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

    /// Read a byte from memory
    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory.read_byte(address)
    }

    /// Write a byte to memory
    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.memory.write_byte(address, value);
    }

    /// Read a word (16-bits) from memory
    pub fn read_word(&self, address: u16) -> u16 {
        self.memory.read_word(address)
    }

    /// Write a word (16-bits) to memory
    pub fn write_word(&mut self, address: u16, value: u16) {
        self.memory.write_word(address, value);
    }

    /// Push a byte onto the stack
    pub fn push_byte(&mut self, value: u8) {
        let stack_addr = 0x0100 | (self.sp as u16);
        self.write_byte(stack_addr, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    /// Pop a byte from the stack
    pub fn pop_byte(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let stack_addr = 0x0100 | (self.sp as u16);
        self.read_byte(stack_addr)
    }

    /// Push a word onto the stack (high byte first, then low byte)
    pub fn push_word(&mut self, value: u16) {
        let high = (value >> 8) as u8;
        let low = (value & 0xFF) as u8;
        self.push_byte(high);
        self.push_byte(low);
    }

    /// Pop a word from the stack (low byte first, then high byte)
    pub fn pop_word(&mut self) -> u16 {
        let low = self.pop_byte() as u16;
        let high = self.pop_byte() as u16;
        (high << 8) | low
    }

    /// Reset the CPU
    pub fn reset(&mut self) {
        // Set registers to their initial values
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.status = 0x34;

        // Read the reset vector from 0xFFFC-0xFFFD
        self.pc = self.read_word(0xFFFC);

        // Reset takes 7 cycles
        self.cycles = 7;
    }
    
    /// Load a program into memory and set up the reset vector
    pub fn load_program(&mut self, program: &[u8], load_address: u16) {
        // Load the program into memory
        for (i, &byte) in program.iter().enumerate() {
            self.write_byte(load_address.wrapping_add(i as u16), byte);
        }
        
        // Set the reset vector to point to our program
        self.write_word(0xFFFC, load_address);
        
        // Reset the CPU to prepare it for execution
        self.reset();
    }
    
    /// Read a byte using the specified addressing mode - simplified for tests
    pub fn read_byte_using_mode(&self, mode: AddressingMode) -> u8 {
        let addr = mode.get_operand_address(self);
        self.read_byte(addr)
    }

    /// Execute a single CPU instruction and return the number of cycles used
    pub fn step(&mut self) -> Result<u8, CpuError> {
        // Fetch opcode
        let opcode = self.fetch();
        
        // Decode instruction
        let metadata = self.decoder.decode(opcode)?;
        
        // Execute instruction and update cycle count
        let cycles = self.execute(metadata);
        self.cycles += cycles as u64;
        
        Ok(cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Ram;
    use anyhow::Result;

    /// Helper function to set up a CPU with memory for testing
    fn setup_cpu() -> Cpu {
        Cpu::new(Box::new(Ram::default()))
    }

    #[test]
    fn test_cpu_flags() {
        let mut cpu = setup_cpu();

        // Test flag is initially not set
        assert!(!cpu.get_flag(CpuFlag::Zero));

        // Test setting a flag
        cpu.set_flag(CpuFlag::Zero, true);
        assert!(cpu.get_flag(CpuFlag::Zero));

        // Test clearing a flag
        cpu.set_flag(CpuFlag::Zero, false);
        assert!(!cpu.get_flag(CpuFlag::Zero));
    }

    #[test]
    fn test_cpu_memory_interaction() {
        let mut cpu = setup_cpu();

        // Test writing and reading bytes
        cpu.write_byte(0x1000, 0x42);
        assert_eq!(cpu.read_byte(0x1000), 0x42);

        // Test writing and reading words
        cpu.write_word(0x2000, 0x1234);
        assert_eq!(cpu.read_word(0x2000), 0x1234);
    }

    #[test]
    fn test_stack_operations() {
        let mut cpu = setup_cpu();

        // Test push and pop byte
        cpu.push_byte(0x42);
        assert_eq!(cpu.sp, 0xFC);
        assert_eq!(cpu.pop_byte(), 0x42);
        assert_eq!(cpu.sp, 0xFD);

        // Test push and pop word
        cpu.push_word(0x1234);
        assert_eq!(cpu.sp, 0xFB);
        assert_eq!(cpu.pop_word(), 0x1234);
        assert_eq!(cpu.sp, 0xFD);
    }

    #[test]
    fn test_reset() {
        // Use RAM with full address space (0x0000-0xFFFF) for testing
        let mut ram = Ram::default();

        // Set reset vector
        ram.write_byte(0xFFFC, 0x34);
        ram.write_byte(0xFFFD, 0x12);

        let mut cpu = Cpu::new(Box::new(ram));
        cpu.reset();

        // Check if PC was set to the reset vector
        assert_eq!(cpu.pc, 0x1234);
        // Check if SP was set to 0xFD
        assert_eq!(cpu.sp, 0xFD);
        // Check if cycles were set to 7
        assert_eq!(cpu.cycles, 7);
    }
    
    #[test]
    fn test_step_lda_immediate() -> Result<()> {
        let mut ram = Ram::default();
        
        // Set up a simple program: LDA #$42
        ram.write_byte(0x0000, 0xA9); // LDA immediate
        ram.write_byte(0x0001, 0x42); // Value to load
        
        let mut cpu = Cpu::new(Box::new(ram));
        cpu.pc = 0x0000; // Set PC to our program
        
        // Execute one instruction
        let cycles = cpu.step()?;
        
        // Verify results
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.pc, 0x0002);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.cycles, 2);
        
        Ok(())
    }
    
    #[test]
    fn test_unknown_opcode() -> Result<()> {
        let mut ram = Ram::default();
        
        // Set up an unknown opcode (0xFF is not used in 6502)
        ram.write_byte(0x0000, 0xFF);
        
        let mut cpu = Cpu::new(Box::new(ram));
        cpu.pc = 0x0000;
        
        // Execute one instruction - this should now return an error for the invalid opcode
        let result = cpu.step();
        
        // Verify the error is the expected one
        assert!(result.is_err(), "Expected an error for invalid opcode");
        if let Err(CpuError::InvalidOpcode(op)) = result {
            assert_eq!(op, 0xFF);
        } else {
            anyhow::bail!("Expected InvalidOpcode error, got: {:?}", result);
        }
        
        // PC should still be incremented because fetch still happened
        assert_eq!(cpu.pc, 0x0001);
        
        Ok(())
    }
    
    #[test]
    fn test_load_program() -> Result<()> {
        let mut cpu = setup_cpu();
        
        // Simple program: LDA #$42, STA $0200, BRK
        let program = [0xA9, 0x42, 0x8D, 0x00, 0x02, 0x00];
        let load_address = 0x8000;
        
        // Load the program
        cpu.load_program(&program, load_address);
        
        // Verify the program was loaded correctly
        for (i, &byte) in program.iter().enumerate() {
            assert_eq!(cpu.read_byte(load_address + i as u16), byte);
        }
        
        // Verify the reset vector was set correctly
        assert_eq!(cpu.read_word(0xFFFC), load_address);
        
        // Verify the CPU was reset and PC points to the program
        assert_eq!(cpu.pc, load_address);
        
        // Execute the first instruction (LDA #$42)
        cpu.step()?;
        assert_eq!(cpu.a, 0x42);
        
        // Execute the second instruction (STA $0200)
        cpu.step()?;
        assert_eq!(cpu.read_byte(0x0200), 0x42);
        
        Ok(())
    }
}
