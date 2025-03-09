use super::{AddressingMode, Cpu, CpuFlag, CpuError};

/// 6502 CPU instruction opcodes
/// Starting with just LDA immediate as a first step
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    LDA, // Load Accumulator
}

/// Instruction metadata containing the opcode, instruction type, addressing mode,
/// and cycle count information
#[derive(Debug, Clone, Copy)]
pub struct InstructionMetadata {
    pub opcode: u8,
    pub instruction: Instruction,
    pub addressing_mode: AddressingMode,
    pub bytes: u8,
    pub cycles: u8,
}

/// Instruction decoder for the 6502 CPU
pub struct InstructionDecoder {
    /// Lookup table for CPU instructions
    instruction_table: [Option<InstructionMetadata>; 256],
}

impl InstructionDecoder {
    /// Create a new instruction decoder with a populated lookup table
    pub fn new() -> Self {
        let mut decoder = Self {
            instruction_table: [None; 256],
        };
        decoder.populate_instruction_table();
        decoder
    }

    /// Populate the instruction lookup table with LDA addressing modes for T1
    fn populate_instruction_table(&mut self) {
        // LDA - Load Accumulator
        self.add_instruction(0xA9, Instruction::LDA, AddressingMode::Immediate, 2, 2); // LDA Immediate
        self.add_instruction(0xA5, Instruction::LDA, AddressingMode::ZeroPage, 2, 3);  // LDA Zero Page
        self.add_instruction(0xAD, Instruction::LDA, AddressingMode::Absolute, 3, 4);  // LDA Absolute
    }

    /// Add an instruction to the lookup table
    fn add_instruction(
        &mut self,
        opcode: u8,
        instruction: Instruction,
        addressing_mode: AddressingMode,
        bytes: u8,
        cycles: u8,
    ) {
        self.instruction_table[opcode as usize] = Some(InstructionMetadata {
            opcode,
            instruction,
            addressing_mode,
            bytes,
            cycles,
        });
    }

    /// Decode an opcode into instruction metadata
    pub fn decode(&self, opcode: u8) -> Result<InstructionMetadata, CpuError> {
        self.instruction_table[opcode as usize]
            .ok_or(CpuError::InvalidOpcode(opcode))
    }
}

impl Cpu {
    /// Fetch the next instruction
    pub fn fetch(&mut self) -> u8 {
        let opcode = self.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        opcode
    }

    /// Execute a single instruction
    pub fn execute(&mut self, instruction_metadata: InstructionMetadata) -> u8 {
        let cycles = instruction_metadata.cycles;
        
        // Execute the instruction based on addressing mode
        match instruction_metadata.instruction {
            Instruction::LDA => self.lda(instruction_metadata.addressing_mode),
        }

        // Increment PC (already incremented by 1 in fetch)
        self.pc = self.pc.wrapping_add((instruction_metadata.bytes - 1) as u16);

        cycles
    }

    /// LDA - Load Accumulator with support for all addressing modes
    fn lda(&mut self, addressing_mode: AddressingMode) {
        // Use the addressing mode to get the operand address
        let addr = addressing_mode.get_operand_address(self);
        
        // Get the value from the address (immediate mode automatically returns the correct address)
        let value = self.read_byte(addr);
        
        // Set the accumulator
        self.a = value;
        
        // Set flags
        self.set_flag(CpuFlag::Zero, value == 0);
        self.set_flag(CpuFlag::Negative, (value & 0x80) != 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Ram;
    use anyhow::Result;

    #[test]
    fn test_lda_immediate() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // Set up test
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0xA9); // LDA immediate opcode
        cpu.write_byte(0x0101, 0x42); // Value to load
        
        // Execute
        let opcode = cpu.fetch();
        let decoder = InstructionDecoder::new();
        let metadata = decoder.decode(opcode)?;
        let cycles = cpu.execute(metadata);
        
        // Verify results
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.pc, 0x0102);
        assert_eq!(cycles, 2);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(!cpu.get_flag(CpuFlag::Negative));
        
        Ok(())
    }

    #[test]
    fn test_lda_zero_page() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // Set up test
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0xA5); // LDA zero page opcode
        cpu.write_byte(0x0101, 0x42); // Zero page address
        cpu.write_byte(0x0042, 0x37); // Value to load from zero page
        
        // Execute
        let opcode = cpu.fetch();
        let decoder = InstructionDecoder::new();
        let metadata = decoder.decode(opcode)?;
        let cycles = cpu.execute(metadata);
        
        // Verify results
        assert_eq!(cpu.a, 0x37);
        assert_eq!(cpu.pc, 0x0102);
        assert_eq!(cycles, 3);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(!cpu.get_flag(CpuFlag::Negative));
        
        Ok(())
    }

    #[test]
    fn test_lda_absolute() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // Set up test
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0xAD); // LDA absolute opcode
        cpu.write_word(0x0101, 0x1234); // Address to load from
        cpu.write_byte(0x1234, 0x80); // Value to load from absolute address (0x80 has bit 7 set)
        
        // Execute
        let opcode = cpu.fetch();
        let decoder = InstructionDecoder::new();
        let metadata = decoder.decode(opcode)?;
        let cycles = cpu.execute(metadata);
        
        // Verify results
        assert_eq!(cpu.a, 0x80);
        assert_eq!(cpu.pc, 0x0103);
        assert_eq!(cycles, 4);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(cpu.get_flag(CpuFlag::Negative)); // 0x80 has bit 7 set
        
        Ok(())
    }
    
    #[test]
    fn test_invalid_opcode() -> Result<()> {
        let decoder = InstructionDecoder::new();
        
        // Test an invalid opcode (0xFF)
        let result = decoder.decode(0xFF);
        
        // Should return an InvalidOpcode error
        assert!(result.is_err(), "Expected an error for invalid opcode");
        if let Err(CpuError::InvalidOpcode(opcode)) = result {
            assert_eq!(opcode, 0xFF);
        } else {
            anyhow::bail!("Expected InvalidOpcode error, got: {:?}", result);
        }
        
        Ok(())
    }
} 