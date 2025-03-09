use super::{AddressingMode, Cpu, CpuError, CpuFlag};
use parse_display::{Display, FromStr};
use std::collections::HashMap;

/// 6502 CPU instruction opcodes
/// Starting with just LDA immediate as a first step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, FromStr)]
#[display(style = "UPPERCASE")]
pub enum Instruction {
    LDA, // Load Accumulator
    LDX, // Load X Register
    LDY, // Load Y Register
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
    /// Lookup table for CPU instructions by opcode
    instruction_table: [Option<InstructionMetadata>; 256],
    /// Lookup table for CPU instructions by (instruction, addressing_mode)
    instruction_map: HashMap<(Instruction, AddressingMode), u8>,
}

impl InstructionDecoder {
    /// Create a new instruction decoder with a populated lookup table
    pub fn new() -> Self {
        let mut decoder = Self {
            instruction_table: [None; 256],
            instruction_map: HashMap::new(),
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
        
        // LDX - Load X Register
        self.add_instruction(0xA2, Instruction::LDX, AddressingMode::Immediate, 2, 2); // LDX Immediate
        self.add_instruction(0xA6, Instruction::LDX, AddressingMode::ZeroPage, 2, 3);  // LDX Zero Page
        self.add_instruction(0xAE, Instruction::LDX, AddressingMode::Absolute, 3, 4);  // LDX Absolute
        
        // LDY - Load Y Register
        self.add_instruction(0xA0, Instruction::LDY, AddressingMode::Immediate, 2, 2); // LDY Immediate
        self.add_instruction(0xA4, Instruction::LDY, AddressingMode::ZeroPage, 2, 3);  // LDY Zero Page
        self.add_instruction(0xAC, Instruction::LDY, AddressingMode::Absolute, 3, 4);  // LDY Absolute
    }

    /// Add an instruction to the lookup tables
    fn add_instruction(
        &mut self,
        opcode: u8,
        instruction: Instruction,
        addressing_mode: AddressingMode,
        bytes: u8,
        cycles: u8,
    ) {
        let metadata = InstructionMetadata {
            opcode,
            instruction,
            addressing_mode,
            bytes,
            cycles,
        };
        self.instruction_table[opcode as usize] = Some(metadata);
        self.instruction_map
            .insert((instruction, addressing_mode), opcode);
    }

    /// Decode an opcode into instruction metadata
    pub fn decode(&self, opcode: u8) -> Result<InstructionMetadata, CpuError> {
        self.instruction_table[opcode as usize].ok_or(CpuError::InvalidOpcode(opcode))
    }

    /// Look up instruction metadata by instruction type and addressing mode
    pub fn lookup(
        &self,
        instruction: Instruction,
        addressing_mode: AddressingMode,
    ) -> Result<InstructionMetadata, CpuError> {
        self.instruction_map
            .get(&(instruction, addressing_mode))
            .map(|&opcode| self.instruction_table[opcode as usize].unwrap())
            .ok_or(CpuError::UnimplementedAddressingMode)
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
            Instruction::LDX => self.ldx(instruction_metadata.addressing_mode),
            Instruction::LDY => self.ldy(instruction_metadata.addressing_mode),
        }

        // Increment PC (already incremented by 1 in fetch)
        self.pc = self.pc.wrapping_add((instruction_metadata.bytes - 1) as u16);

        cycles
    }

    /// Helper method for load instructions (LDA, LDX, LDY)
    fn load_register(&mut self, addressing_mode: AddressingMode) -> u8 {
        // Use the addressing mode to get the operand address
        let addr = addressing_mode.get_operand_address(self);
        
        // Get the value from the address
        let value = self.read_byte(addr);
        
        // Set flags
        self.set_flag(CpuFlag::Zero, value == 0);
        self.set_flag(CpuFlag::Negative, (value & 0x80) != 0);
        
        value
    }

    /// LDA - Load Accumulator with support for all addressing modes
    pub fn lda(&mut self, addressing_mode: AddressingMode) {
        self.a = self.load_register(addressing_mode);
    }
    
    /// LDX - Load X Register with support for all addressing modes
    pub fn ldx(&mut self, addressing_mode: AddressingMode) {
        self.x = self.load_register(addressing_mode);
    }
    
    /// LDY - Load Y Register with support for all addressing modes
    pub fn ldy(&mut self, addressing_mode: AddressingMode) {
        self.y = self.load_register(addressing_mode);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Ram;
    use anyhow::Result;
    use crate::cpu::parser::InstructionParser;

    // Instruction behavior tests for LDA
    #[test]
    fn test_lda_immediate_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // For immediate mode, the operand is at PC
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0x42); // Value to load
        
        // Direct call to the instruction with immediate addressing mode
        cpu.lda(AddressingMode::Immediate);
        
        // Verify results
        assert_eq!(cpu.a, 0x42);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(!cpu.get_flag(CpuFlag::Negative));
        
        Ok(())
    }
    
    #[test]
    fn test_lda_zero_page_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // For zero page mode:
        // 1. The zero page address is read from PC
        // 2. The value is loaded from that zero page address
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0x42);  // Zero page address to use
        cpu.write_byte(0x0042, 0x37);  // Value at zero page address
        
        // Direct call to the instruction with zero page addressing mode
        cpu.lda(AddressingMode::ZeroPage);
        
        // Verify results
        assert_eq!(cpu.a, 0x37);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(!cpu.get_flag(CpuFlag::Negative));
        
        Ok(())
    }
    
    #[test]
    fn test_lda_absolute_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // For absolute mode:
        // 1. The 16-bit address is read from PC and PC+1
        // 2. The value is loaded from that absolute address
        cpu.pc = 0x0100;
        cpu.write_word(0x0100, 0x1234);  // Absolute address to use
        cpu.write_byte(0x1234, 0x80);    // Value at absolute address (0x80 has bit 7 set)
        
        // Direct call to the instruction with absolute addressing mode
        cpu.lda(AddressingMode::Absolute);
        
        // Verify results
        assert_eq!(cpu.a, 0x80);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(cpu.get_flag(CpuFlag::Negative)); // 0x80 has bit 7 set
        
        Ok(())
    }
    
    #[test]
    fn test_lda_zero_flag_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // Set up for immediate mode with value 0
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0x00);
        
        // Direct call to instruction
        cpu.lda(AddressingMode::Immediate);
        
        // Verify zero flag is set
        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_flag(CpuFlag::Zero));
        assert!(!cpu.get_flag(CpuFlag::Negative));
        
        Ok(())
    }
    
    // Instruction behavior tests for LDX
    #[test]
    fn test_ldx_immediate_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // For immediate mode, the operand is at PC
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0x42); // Value to load
        
        // Direct call to the instruction with immediate addressing mode
        cpu.ldx(AddressingMode::Immediate);
        
        // Verify results
        assert_eq!(cpu.x, 0x42);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(!cpu.get_flag(CpuFlag::Negative));
        
        Ok(())
    }
    
    #[test]
    fn test_ldx_zero_page_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // For zero page mode:
        // 1. The zero page address is read from PC
        // 2. The value is loaded from that zero page address
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0x42);  // Zero page address to use
        cpu.write_byte(0x0042, 0x37);  // Value at zero page address
        
        // Direct call to the instruction with zero page addressing mode
        cpu.ldx(AddressingMode::ZeroPage);
        
        // Verify results
        assert_eq!(cpu.x, 0x37);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(!cpu.get_flag(CpuFlag::Negative));
        
        Ok(())
    }
    
    #[test]
    fn test_ldx_absolute_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // For absolute mode:
        // 1. The 16-bit address is read from PC and PC+1
        // 2. The value is loaded from that absolute address
        cpu.pc = 0x0100;
        cpu.write_word(0x0100, 0x1234);  // Absolute address to use
        cpu.write_byte(0x1234, 0x80);    // Value at absolute address (0x80 has bit 7 set)
        
        // Direct call to the instruction with absolute addressing mode
        cpu.ldx(AddressingMode::Absolute);
        
        // Verify results
        assert_eq!(cpu.x, 0x80);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(cpu.get_flag(CpuFlag::Negative)); // 0x80 has bit 7 set
        
        Ok(())
    }
    
    #[test]
    fn test_ldx_zero_flag_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // Set up for immediate mode with value 0
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0x00);
        
        // Direct call to instruction
        cpu.ldx(AddressingMode::Immediate);
        
        // Verify zero flag is set
        assert_eq!(cpu.x, 0x00);
        assert!(cpu.get_flag(CpuFlag::Zero));
        assert!(!cpu.get_flag(CpuFlag::Negative));
        
        Ok(())
    }
    
    // Instruction behavior tests for LDY
    #[test]
    fn test_ldy_immediate_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // For immediate mode, the operand is at PC
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0x42); // Value to load
        
        // Direct call to the instruction with immediate addressing mode
        cpu.ldy(AddressingMode::Immediate);
        
        // Verify results
        assert_eq!(cpu.y, 0x42);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(!cpu.get_flag(CpuFlag::Negative));
        
        Ok(())
    }
    
    #[test]
    fn test_ldy_zero_page_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // For zero page mode:
        // 1. The zero page address is read from PC
        // 2. The value is loaded from that zero page address
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0x42);  // Zero page address to use
        cpu.write_byte(0x0042, 0x37);  // Value at zero page address
        
        // Direct call to the instruction with zero page addressing mode
        cpu.ldy(AddressingMode::ZeroPage);
        
        // Verify results
        assert_eq!(cpu.y, 0x37);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(!cpu.get_flag(CpuFlag::Negative));
        
        Ok(())
    }
    
    #[test]
    fn test_ldy_absolute_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // For absolute mode:
        // 1. The 16-bit address is read from PC and PC+1
        // 2. The value is loaded from that absolute address
        cpu.pc = 0x0100;
        cpu.write_word(0x0100, 0x1234);  // Absolute address to use
        cpu.write_byte(0x1234, 0x80);    // Value at absolute address (0x80 has bit 7 set)
        
        // Direct call to the instruction with absolute addressing mode
        cpu.ldy(AddressingMode::Absolute);
        
        // Verify results
        assert_eq!(cpu.y, 0x80);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(cpu.get_flag(CpuFlag::Negative)); // 0x80 has bit 7 set
        
        Ok(())
    }
    
    #[test]
    fn test_ldy_zero_flag_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // Set up for immediate mode with value 0
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0x00);
        
        // Direct call to instruction
        cpu.ldy(AddressingMode::Immediate);
        
        // Verify zero flag is set
        assert_eq!(cpu.y, 0x00);
        assert!(cpu.get_flag(CpuFlag::Zero));
        assert!(!cpu.get_flag(CpuFlag::Negative));
        
        Ok(())
    }
    
    // CPU integration tests
    #[test]
    fn test_integration_step_lda() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        let parser = InstructionParser::new();
        
        // Set up test with parser
        cpu.pc = 0x0100;
        
        // Parse an LDA instruction with immediate addressing mode
        let bytes = parser.parse_bytes("LDA #$42")?;
        
        // Write bytes to memory
        for (i, &byte) in bytes.iter().enumerate() {
            cpu.write_byte(0x0100 + i as u16, byte);
        }
        
        // Execute a full CPU step (fetch-decode-execute)
        let cycles = cpu.step()?;
        
        // Verify results
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.pc, 0x0102);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.cycles, 2);
        
        Ok(())
    }
    
    #[test]
    fn test_integration_step_ldx() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        let parser = InstructionParser::new();
        
        // Set up test with parser
        cpu.pc = 0x0100;
        
        // Parse an LDX instruction with immediate addressing mode
        let bytes = parser.parse_bytes("LDX #$37")?;
        
        // Write bytes to memory
        for (i, &byte) in bytes.iter().enumerate() {
            cpu.write_byte(0x0100 + i as u16, byte);
        }
        
        // Execute a full CPU step (fetch-decode-execute)
        let cycles = cpu.step()?;
        
        // Verify results
        assert_eq!(cpu.x, 0x37);
        assert_eq!(cpu.pc, 0x0102);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.cycles, 2);
        
        Ok(())
    }
    
    #[test]
    fn test_integration_step_ldy() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        let parser = InstructionParser::new();
        
        // Set up test with parser
        cpu.pc = 0x0100;
        
        // Parse an LDY instruction with immediate addressing mode
        let bytes = parser.parse_bytes("LDY #$55")?;
        
        // Write bytes to memory
        for (i, &byte) in bytes.iter().enumerate() {
            cpu.write_byte(0x0100 + i as u16, byte);
        }
        
        // Execute a full CPU step (fetch-decode-execute)
        let cycles = cpu.step()?;
        
        // Verify results
        assert_eq!(cpu.y, 0x55);
        assert_eq!(cpu.pc, 0x0102);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.cycles, 2);
        
        Ok(())
    }
    
    #[test]
    fn test_integration_multiple_instructions() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        let parser = InstructionParser::new();
        
        // Set up several instructions at different addresses
        cpu.pc = 0x0200;
        
        // Parse and write instructions to memory
        let instr1 = parser.parse_bytes("LDA #$42")?;
        let instr2 = parser.parse_bytes("LDX #$37")?;
        let instr3 = parser.parse_bytes("LDY #$55")?;
        
        // Write first instruction
        for (i, &byte) in instr1.iter().enumerate() {
            cpu.write_byte(0x0200 + i as u16, byte);
        }
        
        // Write second instruction (after the first one)
        for (i, &byte) in instr2.iter().enumerate() {
            cpu.write_byte(0x0200 + instr1.len() as u16 + i as u16, byte);
        }
        
        // Write third instruction (after the second one)
        for (i, &byte) in instr3.iter().enumerate() {
            cpu.write_byte(0x0200 + instr1.len() as u16 + instr2.len() as u16 + i as u16, byte);
        }
        
        // First step (LDA #$42)
        let cycles1 = cpu.step()?;
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cycles1, 2);
        
        // Second step (LDX #$37)
        let cycles2 = cpu.step()?;
        assert_eq!(cpu.x, 0x37);
        assert_eq!(cycles2, 2);
        
        // Third step (LDY #$55)
        let cycles3 = cpu.step()?;
        assert_eq!(cpu.y, 0x55);
        assert_eq!(cycles3, 2);
        
        // Verify total cycles
        assert_eq!(cpu.cycles, 6);
        
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
