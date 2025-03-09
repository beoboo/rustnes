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
    STA, // Store Accumulator
    STX, // Store X Register
    STY, // Store Y Register
    JMP, // Jump to new location
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
        
        // STA - Store Accumulator
        self.add_instruction(0x85, Instruction::STA, AddressingMode::ZeroPage, 2, 3);  // STA Zero Page
        self.add_instruction(0x8D, Instruction::STA, AddressingMode::Absolute, 3, 4);  // STA Absolute
        
        // STX - Store X Register
        self.add_instruction(0x86, Instruction::STX, AddressingMode::ZeroPage, 2, 3);  // STX Zero Page
        self.add_instruction(0x8E, Instruction::STX, AddressingMode::Absolute, 3, 4);  // STX Absolute
        
        // STY - Store Y Register
        self.add_instruction(0x84, Instruction::STY, AddressingMode::ZeroPage, 2, 3);  // STY Zero Page
        self.add_instruction(0x8C, Instruction::STY, AddressingMode::Absolute, 3, 4);  // STY Absolute
        
        // JMP - Jump to new location
        self.add_instruction(0x4C, Instruction::JMP, AddressingMode::Absolute, 3, 3);  // JMP Absolute
        self.add_instruction(0x6C, Instruction::JMP, AddressingMode::Indirect, 3, 5);  // JMP Indirect
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
            Instruction::STA => self.sta(instruction_metadata.addressing_mode),
            Instruction::STX => self.stx(instruction_metadata.addressing_mode),
            Instruction::STY => self.sty(instruction_metadata.addressing_mode),
            Instruction::JMP => self.jmp(instruction_metadata.addressing_mode),
        }

        // Increment PC for non-jump instructions (already incremented by 1 in fetch)
        if instruction_metadata.instruction != Instruction::JMP {
            self.pc = self.pc.wrapping_add((instruction_metadata.bytes - 1) as u16);
        }

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
    
    /// Helper method for store instructions (STA, STX, STY)
    fn store_register(&mut self, addressing_mode: AddressingMode, value: u8) {
        // Use the addressing mode to get the target address
        let addr = addressing_mode.get_operand_address(self);
        
        // Store the value to memory
        self.write_byte(addr, value);
        
        // Note: Store instructions do not affect any flags
    }
    
    /// STA - Store Accumulator with support for all addressing modes
    pub fn sta(&mut self, addressing_mode: AddressingMode) {
        self.store_register(addressing_mode, self.a);
    }
    
    /// STX - Store X Register with support for all addressing modes
    pub fn stx(&mut self, addressing_mode: AddressingMode) {
        self.store_register(addressing_mode, self.x);
    }
    
    /// STY - Store Y Register with support for all addressing modes
    pub fn sty(&mut self, addressing_mode: AddressingMode) {
        self.store_register(addressing_mode, self.y);
    }
    
    /// JMP - Jump to new location (Absolute or Indirect)
    pub fn jmp(&mut self, addressing_mode: AddressingMode) {
        // Get the target address from the addressing mode
        let target_address = addressing_mode.get_operand_address(self);
        
        // Set the program counter to the target address
        self.pc = target_address;
        
        // Note: JMP does not affect any processor flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Ram;
    use anyhow::Result;
    use crate::cpu::parser::InstructionParser;

    // Comprehensive tests for LDA to verify the load_register helper
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
    fn test_load_register_flags() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // Test zero flag
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0x00);
        cpu.lda(AddressingMode::Immediate);
        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_flag(CpuFlag::Zero));
        assert!(!cpu.get_flag(CpuFlag::Negative));
        
        // Test negative flag
        cpu.pc = 0x0200;
        cpu.write_byte(0x0200, 0x80);  // Negative value (bit 7 set)
        cpu.lda(AddressingMode::Immediate);
        assert_eq!(cpu.a, 0x80);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(cpu.get_flag(CpuFlag::Negative));
        
        Ok(())
    }
    
    // Basic tests for LDX (uses the shared load_register helper)
    #[test]
    fn test_ldx_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // Test immediate mode
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0x42);
        cpu.ldx(AddressingMode::Immediate);
        assert_eq!(cpu.x, 0x42);
        
        // Test zero page mode
        cpu.pc = 0x0200;
        cpu.write_byte(0x0200, 0x50);
        cpu.write_byte(0x0050, 0x37);
        cpu.ldx(AddressingMode::ZeroPage);
        assert_eq!(cpu.x, 0x37);
        
        // Test absolute mode
        cpu.pc = 0x0300;
        cpu.write_word(0x0300, 0x1234);
        cpu.write_byte(0x1234, 0x29);
        cpu.ldx(AddressingMode::Absolute);
        assert_eq!(cpu.x, 0x29);
        
        Ok(())
    }
    
    // Basic tests for LDY (uses the shared load_register helper)
    #[test]
    fn test_ldy_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // Test immediate mode
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0x42);
        cpu.ldy(AddressingMode::Immediate);
        assert_eq!(cpu.y, 0x42);
        
        // Test zero page mode
        cpu.pc = 0x0200;
        cpu.write_byte(0x0200, 0x50);
        cpu.write_byte(0x0050, 0x37);
        cpu.ldy(AddressingMode::ZeroPage);
        assert_eq!(cpu.y, 0x37);
        
        // Test absolute mode
        cpu.pc = 0x0300;
        cpu.write_word(0x0300, 0x1234);
        cpu.write_byte(0x1234, 0x29);
        cpu.ldy(AddressingMode::Absolute);
        assert_eq!(cpu.y, 0x29);
        
        Ok(())
    }
    
    // Tests for STA - only checking the actual memory writes
    #[test]
    fn test_sta_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // Test zero page mode
        cpu.pc = 0x0100;
        cpu.a = 0x42;
        cpu.write_byte(0x0100, 0x50);  // Zero page address
        cpu.sta(AddressingMode::ZeroPage);
        let stored_value = cpu.read_byte(0x0050);
        assert_eq!(stored_value, 0x42);
        
        // Test absolute mode
        cpu.pc = 0x0200;
        cpu.a = 0x37;
        cpu.write_word(0x0200, 0x1234);  // Absolute address
        cpu.sta(AddressingMode::Absolute);
        let stored_value = cpu.read_byte(0x1234);
        assert_eq!(stored_value, 0x37);
        
        Ok(())
    }
    
    // Tests for STX and STY - checking store behavior
    #[test]
    fn test_stx_sty_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // Test STX zero page
        cpu.pc = 0x0100;
        cpu.x = 0x42;
        cpu.write_byte(0x0100, 0x50);  // Zero page address
        cpu.stx(AddressingMode::ZeroPage);
        let stored_value = cpu.read_byte(0x0050);
        assert_eq!(stored_value, 0x42);
        
        // Test STX absolute
        cpu.pc = 0x0200;
        cpu.x = 0x37;
        cpu.write_word(0x0200, 0x1234);  // Absolute address
        cpu.stx(AddressingMode::Absolute);
        let stored_value = cpu.read_byte(0x1234);
        assert_eq!(stored_value, 0x37);
        
        // Test STY zero page
        cpu.pc = 0x0300;
        cpu.y = 0x55;
        cpu.write_byte(0x0300, 0x60);  // Zero page address
        cpu.sty(AddressingMode::ZeroPage);
        let stored_value = cpu.read_byte(0x0060);
        assert_eq!(stored_value, 0x55);
        
        // Test STY absolute
        cpu.pc = 0x0400;
        cpu.y = 0x66;
        cpu.write_word(0x0400, 0x5678);  // Absolute address
        cpu.sty(AddressingMode::Absolute);
        let stored_value = cpu.read_byte(0x5678);
        assert_eq!(stored_value, 0x66);
        
        Ok(())
    }
    
    // Test for JMP instruction
    #[test]
    fn test_jmp_behavior() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // Test JMP Absolute
        cpu.pc = 0x0100;
        cpu.write_word(0x0100, 0x1234);  // Target address
        cpu.jmp(AddressingMode::Absolute);
        assert_eq!(cpu.pc, 0x1234);
        
        // Test JMP Indirect
        cpu.pc = 0x0200;
        cpu.write_word(0x0200, 0x3456);  // Pointer to target address
        cpu.write_word(0x3456, 0x5678);  // Target address stored at pointer
        cpu.jmp(AddressingMode::Indirect);
        assert_eq!(cpu.pc, 0x5678);
        
        Ok(())
    }
    
    // Integration test for JMP
    #[test]
    fn test_integration_jmp() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        let parser = InstructionParser::new();
        
        // Program:
        // 0x0100: LDA #$42  ; Load 0x42 into A
        // 0x0102: JMP $0108 ; Jump to 0x0108
        // 0x0105: LDA #$24  ; (skipped)
        // 0x0107: BRK       ; (skipped)
        // 0x0108: LDX #$37  ; Load 0x37 into X
        
        // Parse and write instructions
        let instr1 = parser.parse_bytes("LDA #$42")?;
        let instr2 = parser.parse_bytes("JMP $0108")?;
        let instr3 = parser.parse_bytes("LDA #$24")?; // This should be skipped
        let instr4 = parser.parse_bytes("LDX #$37")?;
        
        // Starting position
        cpu.pc = 0x0100;
        
        // Write instructions to memory
        let mut addr = 0x0100;
        for &byte in instr1.iter() {
            cpu.write_byte(addr, byte);
            addr += 1;
        }
        
        for &byte in instr2.iter() {
            cpu.write_byte(addr, byte);
            addr += 1;
        }
        
        // Write a different instruction at 0x0105 (this should be skipped)
        for &byte in instr3.iter() {
            cpu.write_byte(addr, byte);
            addr += 1;
        }
        
        // Write the final instruction at 0x0108
        addr = 0x0108;
        for &byte in instr4.iter() {
            cpu.write_byte(addr, byte);
            addr += 1;
        }
        
        // Execute LDA #$42
        cpu.step()?;
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.pc, 0x0102);
        
        // Execute JMP $0108
        cpu.step()?;
        assert_eq!(cpu.pc, 0x0108);
        
        // Execute LDX #$37 (after the jump)
        cpu.step()?;
        assert_eq!(cpu.x, 0x37);
        
        Ok(())
    }
    
    // Integration tests for various instructions
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
    fn test_integration_step_store_and_load() -> Result<()> {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        let parser = InstructionParser::new();
        
        // Set up test with parser
        cpu.pc = 0x0200;
        
        // Parse and write instructions to memory
        let instr1 = parser.parse_bytes("LDA #$42")?; // Load accumulator with 0x42
        let instr2 = parser.parse_bytes("STA $1234")?; // Store accumulator to 0x1234
        let instr3 = parser.parse_bytes("LDX #$37")?; // Load X with 0x37
        let instr4 = parser.parse_bytes("STX $5678")?; // Store X to 0x5678
        let instr5 = parser.parse_bytes("LDY #$55")?; // Load Y with 0x55
        let instr6 = parser.parse_bytes("STY $90AB")?; // Store Y to 0x90AB
        
        // Write instructions to memory
        let mut addr = 0x0200;
        for &byte in instr1.iter() {
            cpu.write_byte(addr, byte);
            addr += 1;
        }
        for &byte in instr2.iter() {
            cpu.write_byte(addr, byte);
            addr += 1;
        }
        for &byte in instr3.iter() {
            cpu.write_byte(addr, byte);
            addr += 1;
        }
        for &byte in instr4.iter() {
            cpu.write_byte(addr, byte);
            addr += 1;
        }
        for &byte in instr5.iter() {
            cpu.write_byte(addr, byte);
            addr += 1;
        }
        for &byte in instr6.iter() {
            cpu.write_byte(addr, byte);
            addr += 1;
        }
        
        // Execute instructions and verify results
        
        // LDA #$42
        cpu.step()?;
        assert_eq!(cpu.a, 0x42);
        
        // STA $1234
        cpu.step()?;
        assert_eq!(cpu.read_byte(0x1234), 0x42);
        
        // LDX #$37
        cpu.step()?;
        assert_eq!(cpu.x, 0x37);
        
        // STX $5678
        cpu.step()?;
        assert_eq!(cpu.read_byte(0x5678), 0x37);
        
        // LDY #$55
        cpu.step()?;
        assert_eq!(cpu.y, 0x55);
        
        // STY $90AB
        cpu.step()?;
        assert_eq!(cpu.read_byte(0x90AB), 0x55);
        
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
