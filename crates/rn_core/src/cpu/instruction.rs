use std::collections::HashMap;

use parse_display::{Display, FromStr};
use thiserror::Error;

use super::{AddressingMode, Cpu, CpuFlag};
use crate::errors::NesError;

/// Error type for memory-related operations
#[derive(Debug, Error)]
pub enum InstructionDecoderError {
    /// Error for invalid or unimplemented opcodes
    #[error("Invalid opcode: {0:#04X}")]
    InvalidOpcode(u8),

    /// Error for addressing mode not implemented for a specific instruction
    #[error("Unimplemented addressing mode for instruction: {0} {1}")]
    UnimplementedAddressingMode(Instruction, AddressingMode),
}

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
    JSR, // Jump to Subroutine
    RTS, // Return from Subroutine
    BRK, // Break/interrupt
    NOP, // No Operation
    BIT, // Bit Test with memory
    BPL, // Branch on Plus (N flag = 0)
}

impl Instruction {
    pub fn is_branch(&self) -> bool {
        matches!(self, Instruction::BPL)
    }

    pub fn has_implied_addressing(&self) -> bool {
        matches!(self, Instruction::BRK | Instruction::RTS | Instruction::NOP)
    }
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
        self.add_instruction(0xA5, Instruction::LDA, AddressingMode::ZeroPage, 2, 3); // LDA Zero Page
        self.add_instruction(0xAD, Instruction::LDA, AddressingMode::Absolute, 3, 4); // LDA Absolute

        // LDX - Load X Register
        self.add_instruction(0xA2, Instruction::LDX, AddressingMode::Immediate, 2, 2); // LDX Immediate
        self.add_instruction(0xA6, Instruction::LDX, AddressingMode::ZeroPage, 2, 3); // LDX Zero Page
        self.add_instruction(0xAE, Instruction::LDX, AddressingMode::Absolute, 3, 4); // LDX Absolute

        // LDY - Load Y Register
        self.add_instruction(0xA0, Instruction::LDY, AddressingMode::Immediate, 2, 2); // LDY Immediate
        self.add_instruction(0xA4, Instruction::LDY, AddressingMode::ZeroPage, 2, 3); // LDY Zero Page
        self.add_instruction(0xAC, Instruction::LDY, AddressingMode::Absolute, 3, 4); // LDY Absolute

        // STA - Store Accumulator
        self.add_instruction(0x85, Instruction::STA, AddressingMode::ZeroPage, 2, 3); // STA Zero Page
        self.add_instruction(0x8D, Instruction::STA, AddressingMode::Absolute, 3, 4); // STA Absolute

        // STX - Store X Register
        self.add_instruction(0x86, Instruction::STX, AddressingMode::ZeroPage, 2, 3); // STX Zero Page
        self.add_instruction(0x8E, Instruction::STX, AddressingMode::Absolute, 3, 4); // STX Absolute

        // STY - Store Y Register
        self.add_instruction(0x84, Instruction::STY, AddressingMode::ZeroPage, 2, 3); // STY Zero Page
        self.add_instruction(0x8C, Instruction::STY, AddressingMode::Absolute, 3, 4); // STY Absolute

        // JMP - Jump to new location
        self.add_instruction(0x4C, Instruction::JMP, AddressingMode::Absolute, 3, 3); // JMP Absolute
        self.add_instruction(0x6C, Instruction::JMP, AddressingMode::Indirect, 3, 5); // JMP Indirect

        // JSR - Jump to Subroutine
        self.add_instruction(0x20, Instruction::JSR, AddressingMode::Absolute, 3, 6); // JSR Absolute

        // RTS - Return from Subroutine
        self.add_instruction(0x60, Instruction::RTS, AddressingMode::Implied, 1, 6); // RTS Implied

        // BRK - Break/interrupt
        self.add_instruction(0x00, Instruction::BRK, AddressingMode::Implied, 1, 7);
        // BRK Implied

        // NOP - No Operation
        self.add_instruction(0xEA, Instruction::NOP, AddressingMode::Implied, 1, 2);
        // NOP Implied

        // BIT - Bit Test
        self.add_instruction(0x24, Instruction::BIT, AddressingMode::ZeroPage, 2, 3); // BIT Zero Page
        self.add_instruction(0x2C, Instruction::BIT, AddressingMode::Absolute, 3, 4); // BIT Absolute

        // BPL - Branch on Plus (N flag = 0)
        self.add_instruction(0x10, Instruction::BPL, AddressingMode::Relative, 2, 2);
        // BPL Relative
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
        self.instruction_map.insert((instruction, addressing_mode), opcode);
    }

    /// Decode an opcode into instruction metadata
    pub fn decode(&self, opcode: u8) -> Result<InstructionMetadata, InstructionDecoderError> {
        self.instruction_table[opcode as usize].ok_or(InstructionDecoderError::InvalidOpcode(opcode))
    }

    /// Look up instruction metadata by instruction type and addressing mode
    pub fn lookup(
        &self,
        instruction: Instruction,
        addressing_mode: AddressingMode,
    ) -> Result<InstructionMetadata, InstructionDecoderError> {
        self.instruction_map
            .get(&(instruction, addressing_mode))
            .and_then(|&opcode| self.instruction_table[opcode as usize])
            .ok_or(InstructionDecoderError::UnimplementedAddressingMode(
                instruction,
                addressing_mode,
            ))
    }
}

impl Cpu {
    /// Fetch the next instruction
    pub fn fetch(&mut self) -> Result<u8, NesError> {
        let opcode = self.read_byte(self.registers.pc)?;
        self.registers.pc = self.registers.pc.wrapping_add(1);
        Ok(opcode)
    }

    /// Execute a single instruction
    pub fn execute(&mut self, instruction_metadata: InstructionMetadata) -> Result<u8, NesError> {
        let mut cycles = instruction_metadata.cycles;

        // Execute the instruction based on addressing mode
        match instruction_metadata.instruction {
            Instruction::LDA => self.lda(instruction_metadata.addressing_mode)?,
            Instruction::LDX => self.ldx(instruction_metadata.addressing_mode)?,
            Instruction::LDY => self.ldy(instruction_metadata.addressing_mode)?,
            Instruction::STA => self.sta(instruction_metadata.addressing_mode)?,
            Instruction::STX => self.stx(instruction_metadata.addressing_mode)?,
            Instruction::STY => self.sty(instruction_metadata.addressing_mode)?,
            Instruction::JMP => self.jmp(instruction_metadata.addressing_mode)?,
            Instruction::JSR => self.jsr(instruction_metadata.addressing_mode)?,
            Instruction::RTS => self.rts()?,
            Instruction::BRK => self.brk()?,
            Instruction::NOP => self.nop(),
            Instruction::BIT => self.bit(instruction_metadata.addressing_mode)?,
            Instruction::BPL => {
                let additional_cycles = self.bpl()?;
                cycles = cycles.wrapping_add(additional_cycles);
            },
        }

        // Increment PC for non-jump/call/branch instructions (already incremented by 1 in fetch)
        if !matches!(
            instruction_metadata.instruction,
            Instruction::JMP | Instruction::JSR | Instruction::RTS | Instruction::BRK | Instruction::BPL
        ) {
            self.registers.pc = self.registers.pc.wrapping_add((instruction_metadata.bytes - 1) as u16);
        }

        Ok(cycles)
    }

    /// Helper method for load instructions (LDA, LDX, LDY)
    fn load_register(&mut self, addressing_mode: AddressingMode) -> Result<u8, NesError> {
        // Use the addressing mode to get the operand address
        let addr = addressing_mode.get_operand_address(self)?;

        // Get the value from the address
        let value = self.read_byte(addr)?;

        // Set flags
        self.set_flag(CpuFlag::Zero, value == 0);
        self.set_flag(CpuFlag::Negative, (value & 0x80) != 0);

        Ok(value)
    }

    /// LDA - Load Accumulator with support for all addressing modes
    pub fn lda(&mut self, addressing_mode: AddressingMode) -> Result<(), NesError> {
        self.registers.a = self.load_register(addressing_mode)?;
        Ok(())
    }

    /// LDX - Load X Register with support for all addressing modes
    pub fn ldx(&mut self, addressing_mode: AddressingMode) -> Result<(), NesError> {
        self.registers.x = self.load_register(addressing_mode)?;
        Ok(())
    }

    /// LDY - Load Y Register with support for all addressing modes
    pub fn ldy(&mut self, addressing_mode: AddressingMode) -> Result<(), NesError> {
        self.registers.y = self.load_register(addressing_mode)?;
        Ok(())
    }

    /// Helper method for store instructions (STA, STX, STY)
    fn store_register(&mut self, addressing_mode: AddressingMode, value: u8) -> Result<(), NesError> {
        // Use the addressing mode to get the target address
        let addr = addressing_mode.get_operand_address(self)?;

        // Store the value to memory
        self.write_byte(addr, value)?;

        // Note: Store instructions do not affect any flags
        Ok(())
    }

    /// STA - Store Accumulator with support for all addressing modes
    pub fn sta(&mut self, addressing_mode: AddressingMode) -> Result<(), NesError> {
        self.store_register(addressing_mode, self.registers.a)
    }

    /// STX - Store X Register with support for all addressing modes
    pub fn stx(&mut self, addressing_mode: AddressingMode) -> Result<(), NesError> {
        self.store_register(addressing_mode, self.registers.x)
    }

    /// STY - Store Y Register with support for all addressing modes
    pub fn sty(&mut self, addressing_mode: AddressingMode) -> Result<(), NesError> {
        self.store_register(addressing_mode, self.registers.y)
    }

    /// JMP - Jump to new location (Absolute or Indirect)
    pub fn jmp(&mut self, addressing_mode: AddressingMode) -> Result<(), NesError> {
        // Get the target address from the addressing mode
        let target_address = addressing_mode.get_operand_address(self)?;

        // Set the program counter to the target address
        self.registers.pc = target_address;

        // Note: JMP does not affect any processor flags
        Ok(())
    }

    /// JSR - Jump to Subroutine
    pub fn jsr(&mut self, addressing_mode: AddressingMode) -> Result<(), NesError> {
        // Get the target address from the addressing mode
        let target_address = addressing_mode.get_operand_address(self)?;

        // Push the return address (PC+2-1) onto the stack
        // PC currently points to the first byte of the operand, which is PC+1
        // So we need PC+2 for the next instruction after JSR
        let return_address = self.registers.pc + 1;
        self.push_word(return_address)?;

        // Set the program counter to the target address
        self.registers.pc = target_address;

        // Note: JSR does not affect any processor flags
        Ok(())
    }

    /// RTS - Return from Subroutine
    pub fn rts(&mut self) -> Result<(), NesError> {
        // Pull the return address from the stack
        let return_address = self.pop_word()?;

        // Return address points to the last byte of JSR, so add 1 to get to the next instruction
        self.registers.pc = return_address.wrapping_add(1);

        // Note: RTS does not affect any processor flags
        Ok(())
    }

    /// BRK - Break/interrupt
    pub fn brk(&mut self) -> Result<(), NesError> {
        // BRK pushes PC+2 to the stack (PC+1 for the opcode fetch, +1 for the padding byte)
        // The 6502 BRK instruction is 2 bytes long (opcode + padding)
        let pc_to_push = self.registers.pc.wrapping_add(1);

        self.push_word(pc_to_push)?;

        // Push status register with Break flag set
        // The B flag (bit 4) is set in the status byte pushed to the stack
        let status_with_break = self.registers.status | CpuFlag::Break as u8 | CpuFlag::Unused as u8;

        self.push_byte(status_with_break)?;

        // Set the interrupt disable flag
        self.set_flag(CpuFlag::InterruptDisable, true);

        // Load the IRQ/BRK vector (0xFFFE-0xFFFF) into PC
        self.registers.pc = self.read_word(0xFFFE)?;

        Ok(())
    }

    /// NOP - No Operation
    pub fn nop(&mut self) {
        // NOP does not affect any processor state
    }

    /// BIT - Bit Test with memory
    pub fn bit(&mut self, addressing_mode: AddressingMode) -> Result<(), NesError> {
        // Get the value from the memory address
        let addr = addressing_mode.get_operand_address(self)?;
        let value = self.read_byte(addr)?;

        // Perform AND with accumulator but don't store the result
        let result = self.registers.a & value;

        // Set the Zero flag based on the result of A & M
        self.set_flag(CpuFlag::Zero, result == 0);

        // Copy bit 7 of memory to Negative flag
        self.set_flag(CpuFlag::Negative, (value & 0x80) != 0);

        // Copy bit 6 of memory to Overflow flag
        self.set_flag(CpuFlag::Overflow, (value & 0x40) != 0);

        Ok(())
    }

    /// BPL - Branch on Plus (N flag = 0)
    pub fn bpl(&mut self) -> Result<u8, NesError> {
        // Initially no additional cycles
        let mut additional_cycles = 0;

        // Only branch if the Negative flag is clear (positive result)
        if !self.get_flag(CpuFlag::Negative) {
            // For branch instructions, we need to read the offset directly
            // The offset is stored at PC (PC points to the offset byte after fetch)
            let offset = self.read_byte(self.registers.pc)? as i8; // Read as signed byte

            // Add 1 cycle for taking the branch
            additional_cycles += 1;

            // Save the old PC for page boundary check
            let old_pc = self.registers.pc;

            // Calculate the target address
            // PC+1 is the address of the next instruction after BPL
            // We add the offset to that
            let target = ((self.registers.pc as i32) + 1 + (offset as i32)) as u16;

            // Set the PC to the target address
            self.registers.pc = target;

            // Add 1 more cycle if the branch crosses a page boundary
            if (old_pc & 0xFF00) != (target & 0xFF00) {
                additional_cycles += 1;
            }

            // No need to subtract from PC since we're setting it directly to the target
            // and execute() won't increment it for branch instructions
        } else {
            // When branch is not taken, we need to increment the PC to skip the offset byte
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }

        Ok(additional_cycles)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use anyhow::Result;

    use super::*;
    use crate::{cpu::assembler::Assembler, memory::Ram};

    /// Helper function to set up a CPU with memory for testing
    fn setup_cpu() -> Cpu {
        setup_cpu_with_memory(Ram::default())
    }

    fn setup_cpu_with_memory(memory: Ram) -> Cpu {
        let mut cpu = Cpu::new();
        cpu.connect_memory(Rc::new(RefCell::new(memory)));
        cpu
    }

    // Comprehensive tests for LDA to verify the load_register helper
    #[test]
    fn test_lda_immediate_behavior() -> Result<()> {
        let mut cpu = setup_cpu();

        // For immediate mode, the operand is at PC
        cpu.registers.pc = 0x0100;
        cpu.write_byte(0x0100, 0x42)?; // Value to load

        // Direct call to the instruction with immediate addressing mode
        cpu.lda(AddressingMode::Immediate)?;

        // Verify results
        assert_eq!(cpu.registers.a, 0x42);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(!cpu.get_flag(CpuFlag::Negative));

        Ok(())
    }

    #[test]
    fn test_lda_zero_page_behavior() -> Result<()> {
        let mut cpu = setup_cpu();

        // For zero page mode:
        // 1. The zero page address is read from PC
        // 2. The value is loaded from that zero page address
        cpu.registers.pc = 0x0100;
        cpu.write_byte(0x0100, 0x42)?; // Zero page address to use
        cpu.write_byte(0x0042, 0x37)?; // Value at zero page address

        // Direct call to the instruction with zero page addressing mode
        cpu.lda(AddressingMode::ZeroPage)?;

        // Verify results
        assert_eq!(cpu.registers.a, 0x37);

        Ok(())
    }

    #[test]
    fn test_lda_absolute_behavior() -> Result<()> {
        let mut cpu = setup_cpu();

        // For absolute mode:
        // 1. The 16-bit address is read from PC and PC+1
        // 2. The value is loaded from that absolute address
        cpu.registers.pc = 0x0100;
        cpu.write_word(0x0100, 0x1234)?; // Absolute address to use
        cpu.write_byte(0x1234, 0x80)?; // Value at absolute address (0x80 has bit 7 set)

        // Direct call to the instruction with absolute addressing mode
        cpu.lda(AddressingMode::Absolute)?;

        // Verify results
        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(cpu.get_flag(CpuFlag::Negative)); // 0x80 has bit 7 set

        Ok(())
    }

    #[test]
    fn test_load_register_flags() -> Result<()> {
        let mut cpu = setup_cpu();

        // Test zero flag
        cpu.registers.pc = 0x0100;
        cpu.write_byte(0x0100, 0x00)?;
        cpu.lda(AddressingMode::Immediate)?;
        assert_eq!(cpu.registers.a, 0x00);
        assert!(cpu.get_flag(CpuFlag::Zero));
        assert!(!cpu.get_flag(CpuFlag::Negative));

        // Test negative flag
        cpu.registers.pc = 0x0200;
        cpu.write_byte(0x0200, 0x80)?; // Negative value (bit 7 set)
        cpu.lda(AddressingMode::Immediate)?;
        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(cpu.get_flag(CpuFlag::Negative));

        Ok(())
    }

    // Basic tests for LDX (uses the shared load_register helper)
    #[test]
    fn test_ldx_behavior() -> Result<()> {
        let mut cpu = setup_cpu();

        // Test immediate mode
        cpu.registers.pc = 0x0100;
        cpu.write_byte(0x0100, 0x42)?;
        cpu.ldx(AddressingMode::Immediate)?;
        assert_eq!(cpu.registers.x, 0x42);

        // Test zero page mode
        cpu.registers.pc = 0x0200;
        cpu.write_byte(0x0200, 0x50)?;
        cpu.write_byte(0x0050, 0x37)?;
        cpu.ldx(AddressingMode::ZeroPage)?;
        assert_eq!(cpu.registers.x, 0x37);

        // Test absolute mode
        cpu.registers.pc = 0x0300;
        cpu.write_word(0x0300, 0x1234)?;
        cpu.write_byte(0x1234, 0x29)?;
        cpu.ldx(AddressingMode::Absolute)?;
        assert_eq!(cpu.registers.x, 0x29);

        Ok(())
    }

    // Basic tests for LDY (uses the shared load_register helper)
    #[test]
    fn test_ldy_behavior() -> Result<()> {
        let mut cpu = setup_cpu();

        // Test immediate mode
        cpu.registers.pc = 0x0100;
        cpu.write_byte(0x0100, 0x42)?;
        cpu.ldy(AddressingMode::Immediate)?;
        assert_eq!(cpu.registers.y, 0x42);

        // Test zero page mode
        cpu.registers.pc = 0x0200;
        cpu.write_byte(0x0200, 0x50)?;
        cpu.write_byte(0x0050, 0x37)?;
        cpu.ldy(AddressingMode::ZeroPage)?;
        assert_eq!(cpu.registers.y, 0x37);

        // Test absolute mode
        cpu.registers.pc = 0x0300;
        cpu.write_word(0x0300, 0x1234)?;
        cpu.write_byte(0x1234, 0x29)?;
        cpu.ldy(AddressingMode::Absolute)?;
        assert_eq!(cpu.registers.y, 0x29);

        Ok(())
    }

    // Tests for STA - only checking the actual memory writes
    #[test]
    fn test_sta_behavior() -> Result<()> {
        let mut cpu = setup_cpu();

        // Test zero page mode
        cpu.registers.pc = 0x0100;
        cpu.registers.a = 0x42;
        cpu.write_byte(0x0100, 0x50)?; // Zero page address
        cpu.sta(AddressingMode::ZeroPage)?;
        let stored_value = cpu.read_byte(0x0050)?;
        assert_eq!(stored_value, 0x42);

        // Test absolute mode
        cpu.registers.pc = 0x0200;
        cpu.registers.a = 0x37;
        cpu.write_word(0x0200, 0x1234)?; // Absolute address
        cpu.sta(AddressingMode::Absolute)?;
        let stored_value = cpu.read_byte(0x1234)?;
        assert_eq!(stored_value, 0x37);

        Ok(())
    }

    // Tests for STX and STY - checking store behavior
    #[test]
    fn test_stx_sty_behavior() -> Result<()> {
        let mut cpu = setup_cpu();

        // Test STX zero page
        cpu.registers.pc = 0x0100;
        cpu.registers.x = 0x42;
        cpu.write_byte(0x0100, 0x50)?; // Zero page address
        cpu.stx(AddressingMode::ZeroPage)?;
        let stored_value = cpu.read_byte(0x0050)?;
        assert_eq!(stored_value, 0x42);

        // Test STX absolute
        cpu.registers.pc = 0x0200;
        cpu.registers.x = 0x37;
        cpu.write_word(0x0200, 0x1234)?; // Absolute address
        cpu.stx(AddressingMode::Absolute)?;
        let stored_value = cpu.read_byte(0x1234)?;
        assert_eq!(stored_value, 0x37);

        // Test STY zero page
        cpu.registers.pc = 0x0300;
        cpu.registers.y = 0x55;
        cpu.write_byte(0x0300, 0x60)?; // Zero page address
        cpu.sty(AddressingMode::ZeroPage)?;
        let stored_value = cpu.read_byte(0x0060)?;
        assert_eq!(stored_value, 0x55);

        // Test STY absolute
        cpu.registers.pc = 0x0400;
        cpu.registers.y = 0x66;
        cpu.write_word(0x0400, 0x5678)?; // Absolute address
        cpu.sty(AddressingMode::Absolute)?;
        let stored_value = cpu.read_byte(0x5678)?;
        assert_eq!(stored_value, 0x66);

        Ok(())
    }

    // Test for JMP instruction
    #[test]
    fn test_jmp_behavior() -> Result<()> {
        let mut cpu = setup_cpu();

        // Test JMP Absolute
        cpu.registers.pc = 0x0100;
        cpu.write_word(0x0100, 0x1234)?; // Target address
        cpu.jmp(AddressingMode::Absolute)?;
        assert_eq!(cpu.registers.pc, 0x1234);

        // Test JMP Indirect
        cpu.registers.pc = 0x0200;
        cpu.write_word(0x0200, 0x3456)?; // Pointer to target address
        cpu.write_word(0x3456, 0x5678)?; // Target address stored at pointer
        cpu.jmp(AddressingMode::Indirect)?;
        assert_eq!(cpu.registers.pc, 0x5678);

        Ok(())
    }

    // Integration test for JMP
    #[test]
    fn test_integration_jmp() -> Result<()> {
        let mut cpu = setup_cpu();
        let mut assembler = Assembler::new(0);

        // Program:
        // 0x0100: LDA #$42  ; Load 0x42 into A
        // 0x0102: JMP $0108 ; Jump to 0x0108
        // 0x0105: LDA #$24  ; (skipped)
        // 0x0107: BRK       ; (skipped)
        // 0x0108: LDX #$37  ; Load 0x37 into X

        // Parse and write instructions
        let labels = HashMap::new();
        let instr1 = assembler.assemble_instruction("LDA #$42", &labels)?;
        let instr2 = assembler.assemble_instruction("JMP $0108", &labels)?;
        let instr3 = assembler.assemble_instruction("LDA #$24", &labels)?; // This should be skipped
        let instr4 = assembler.assemble_instruction("LDX #$37", &labels)?;

        // Starting position
        cpu.registers.pc = 0x0100;

        // Write instructions to memory
        let mut addr = 0x0100;
        for &byte in instr1.iter() {
            cpu.write_byte(addr, byte)?;
            addr += 1;
        }

        for &byte in instr2.iter() {
            cpu.write_byte(addr, byte)?;
            addr += 1;
        }

        // Write a different instruction at 0x0105 (this should be skipped)
        for &byte in instr3.iter() {
            cpu.write_byte(addr, byte)?;
            addr += 1;
        }

        // Write the final instruction at 0x0108
        addr = 0x0108;
        for &byte in instr4.iter() {
            cpu.write_byte(addr, byte)?;
            addr += 1;
        }

        // Execute LDA #$42
        cpu.step()?;
        assert_eq!(cpu.registers.a, 0x42);
        assert_eq!(cpu.registers.pc, 0x0102);

        // Execute JMP $0108
        cpu.step()?;
        assert_eq!(cpu.registers.pc, 0x0108);

        // Execute LDX #$37 (after the jump)
        cpu.step()?;
        assert_eq!(cpu.registers.x, 0x37);

        Ok(())
    }

    // Integration tests for various instructions
    #[test]
    fn test_integration_step_lda() -> Result<()> {
        let mut cpu = setup_cpu();
        let mut assembler = Assembler::new(0);

        // Set up test with parser
        cpu.registers.pc = 0x0100;

        // Parse an LDA instruction with immediate addressing mode
        let bytes = assembler.assemble_instruction("LDA #$42", &HashMap::new())?;

        // Write bytes to memory
        for (i, &byte) in bytes.iter().enumerate() {
            cpu.write_byte(0x0100 + i as u16, byte)?;
        }

        // Execute a full CPU step (fetch-decode-execute)
        let cycles = cpu.step()?;

        // Verify results
        assert_eq!(cpu.registers.a, 0x42);
        assert_eq!(cpu.registers.pc, 0x0102);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.cycles, 2);

        Ok(())
    }

    #[test]
    fn test_integration_step_store_and_load() -> Result<()> {
        let mut cpu = setup_cpu();
        let mut assembler = Assembler::new(0);

        // Set up test with parser
        cpu.registers.pc = 0x0200;

        // Parse and write instructions to memory
        let labels = HashMap::new();
        let instr1 = assembler.assemble_instruction("LDA #$42", &labels)?; // Load accumulator with 0x42
        let instr2 = assembler.assemble_instruction("STA $1234", &labels)?; // Store accumulator to 0x1234
        let instr3 = assembler.assemble_instruction("LDX #$37", &labels)?; // Load X with 0x37
        let instr4 = assembler.assemble_instruction("STX $5678", &labels)?; // Store X to 0x5678
        let instr5 = assembler.assemble_instruction("LDY #$55", &labels)?; // Load Y with 0x55
        let instr6 = assembler.assemble_instruction("STY $90AB", &labels)?; // Store Y to 0x90AB

        // Write instructions to memory
        let mut addr = 0x0200;
        for &byte in instr1.iter() {
            cpu.write_byte(addr, byte)?;
            addr += 1;
        }
        for &byte in instr2.iter() {
            cpu.write_byte(addr, byte)?;
            addr += 1;
        }
        for &byte in instr3.iter() {
            cpu.write_byte(addr, byte)?;
            addr += 1;
        }
        for &byte in instr4.iter() {
            cpu.write_byte(addr, byte)?;
            addr += 1;
        }
        for &byte in instr5.iter() {
            cpu.write_byte(addr, byte)?;
            addr += 1;
        }
        for &byte in instr6.iter() {
            cpu.write_byte(addr, byte)?;
            addr += 1;
        }

        // Execute instructions and verify results

        // LDA #$42
        cpu.step()?;
        assert_eq!(cpu.registers.a, 0x42);

        // STA $1234
        cpu.step()?;
        assert_eq!(cpu.read_byte(0x1234)?, 0x42);

        // LDX #$37
        cpu.step()?;
        assert_eq!(cpu.registers.x, 0x37);

        // STX $5678
        cpu.step()?;
        assert_eq!(cpu.read_byte(0x5678)?, 0x37);

        // LDY #$55
        cpu.step()?;
        assert_eq!(cpu.registers.y, 0x55);

        // STY $90AB
        cpu.step()?;
        assert_eq!(cpu.read_byte(0x90AB)?, 0x55);

        Ok(())
    }

    #[test]
    fn test_invalid_opcode() -> Result<()> {
        let decoder = InstructionDecoder::new();

        // Test an invalid opcode (0xFF)
        let result = decoder.decode(0xFF);

        // Should return an InvalidOpcode error
        assert!(result.is_err(), "Expected an error for invalid opcode");

        if let Err(InstructionDecoderError::InvalidOpcode(opcode)) = result {
            assert_eq!(opcode, 0xFF);
        } else {
            anyhow::bail!("Expected InvalidOpcode error, got: {:?}", result);
        }

        Ok(())
    }

    #[test]
    fn test_instruction_decoder() {
        let decoder = InstructionDecoder::new();

        // Test LDA Immediate
        let metadata = decoder.decode(0xA9).unwrap();
        assert_eq!(metadata.instruction, Instruction::LDA);
        assert_eq!(metadata.addressing_mode, AddressingMode::Immediate);
        assert_eq!(metadata.bytes, 2);
        assert_eq!(metadata.cycles, 2);

        // Test LDA Zero Page
        let metadata = decoder.decode(0xA5).unwrap();
        assert_eq!(metadata.instruction, Instruction::LDA);
        assert_eq!(metadata.addressing_mode, AddressingMode::ZeroPage);
        assert_eq!(metadata.bytes, 2);
        assert_eq!(metadata.cycles, 3);

        // Test LDA Absolute
        let metadata = decoder.decode(0xAD).unwrap();
        assert_eq!(metadata.instruction, Instruction::LDA);
        assert_eq!(metadata.addressing_mode, AddressingMode::Absolute);
        assert_eq!(metadata.bytes, 3);
        assert_eq!(metadata.cycles, 4);

        // Test JMP Absolute
        let metadata = decoder.decode(0x4C).unwrap();
        assert_eq!(metadata.instruction, Instruction::JMP);
        assert_eq!(metadata.addressing_mode, AddressingMode::Absolute);
        assert_eq!(metadata.bytes, 3);
        assert_eq!(metadata.cycles, 3);

        // Test JMP Indirect
        let metadata = decoder.decode(0x6C).unwrap();
        assert_eq!(metadata.instruction, Instruction::JMP);
        assert_eq!(metadata.addressing_mode, AddressingMode::Indirect);
        assert_eq!(metadata.bytes, 3);
        assert_eq!(metadata.cycles, 5);

        // Test JSR Absolute
        let metadata = decoder.decode(0x20).unwrap();
        assert_eq!(metadata.instruction, Instruction::JSR);
        assert_eq!(metadata.addressing_mode, AddressingMode::Absolute);
        assert_eq!(metadata.bytes, 3);
        assert_eq!(metadata.cycles, 6);

        // Test RTS Implied
        let metadata = decoder.decode(0x60).unwrap();
        assert_eq!(metadata.instruction, Instruction::RTS);
        assert_eq!(metadata.addressing_mode, AddressingMode::Implied);
        assert_eq!(metadata.bytes, 1);
        assert_eq!(metadata.cycles, 6);
    }

    #[test]
    fn test_jsr_rts() -> Result<()> {
        let ram = Ram::with_range(0x0000, 0xFFFF);
        let mut cpu = setup_cpu_with_memory(ram);

        // Set up a simple program:
        // 0x0200: JSR $0210 (20 10 02)
        // 0x0203: LDA #$42  (A9 42)
        // ...
        // 0x0210: LDX #$24  (A2 24)
        // 0x0212: RTS       (60)

        // JSR instruction
        cpu.write_byte(0x0200, 0x20)?;
        cpu.write_byte(0x0201, 0x10)?;
        cpu.write_byte(0x0202, 0x02)?;

        // LDA #$42
        cpu.write_byte(0x0203, 0xA9)?;
        cpu.write_byte(0x0204, 0x42)?;

        // LDX #$24
        cpu.write_byte(0x0210, 0xA2)?;
        cpu.write_byte(0x0211, 0x24)?;

        // RTS
        cpu.write_byte(0x0212, 0x60)?;

        // Set initial PC and execute
        cpu.registers.pc = 0x0200;

        // Execute JSR
        let opcode = cpu.fetch()?;
        let metadata = cpu.decoder.decode(opcode)?;
        cpu.execute(metadata)?;

        // Check PC jumped to subroutine
        assert_eq!(cpu.registers.pc, 0x0210);

        // Check return address pushed to stack
        let stack_addr = 0x0100 | (cpu.registers.sp.wrapping_add(1) as u16);
        let pushed_pc = cpu.read_word(stack_addr)?;
        assert_eq!(pushed_pc, 0x0202, "Return address should be pushed to stack");

        // Execute LDX at the subroutine
        let opcode = cpu.fetch()?;
        let metadata = cpu.decoder.decode(opcode)?;
        cpu.execute(metadata)?;

        // Check X register loaded
        assert_eq!(cpu.registers.x, 0x24);

        // Execute RTS
        let opcode = cpu.fetch()?;
        let metadata = cpu.decoder.decode(opcode)?;
        cpu.execute(metadata)?;

        // Check PC returned to instruction after JSR
        assert_eq!(cpu.registers.pc, 0x0203);

        // Execute LDA after return
        let opcode = cpu.fetch()?;
        let metadata = cpu.decoder.decode(opcode)?;
        cpu.execute(metadata)?;

        // Check A register loaded
        assert_eq!(cpu.registers.a, 0x42);

        Ok(())
    }

    #[test]
    fn test_brk_instruction() -> Result<()> {
        let mut cpu = setup_cpu();

        // Set initial state
        cpu.registers.pc = 0x8000;
        cpu.registers.sp = 0xFD;
        cpu.registers.status = 0x20; // Just the unused bit set

        // Set up the IRQ/BRK vector
        cpu.write_word(0xFFFE, 0x1234)?;

        // Set up a BRK instruction at 0x8000
        cpu.write_byte(0x8000, 0x00)?; // BRK opcode

        // Execute one instruction
        let cycles = cpu.step()?;

        // Verify the CPU state after BRK
        assert_eq!(cpu.registers.pc, 0x1234, "PC should be set to IRQ/BRK vector");
        assert_eq!(
            cpu.registers.sp, 0xFA,
            "SP should be decreased by 3 (2 for PC, 1 for status)"
        );
        assert!(
            cpu.get_flag(CpuFlag::InterruptDisable),
            "Interrupt disable should be set"
        );

        // After BRK, the stack should contain:
        // 0x01FB: status byte (last pushed)
        // 0x01FC: PC low byte (second pushed)
        // 0x01FD: PC high byte (first pushed)
        // And the SP will be 0xFA (pointing to the next available slot)

        // Verify stack contents
        // The status byte is at 0x01FB (last pushed value)
        let stack_status_addr = 0x01FB;
        let stack_status = cpu.read_byte(stack_status_addr)?;

        // The expected status value is the original (0x20) + Break (0x10) + Unused (0x20)
        let expected_status = 0x20 | CpuFlag::Break as u8 | CpuFlag::Unused as u8;

        // Direct assertion on the expected full status value
        assert_eq!(stack_status, expected_status, "Status on stack should be 0x30");

        let pushed_pc = cpu.read_word(0x01FC)?;

        assert_eq!(pushed_pc, 0x8002, "PC+1 should be pushed to stack");

        // Verify the cycle count
        assert_eq!(cycles, 7, "BRK should take 7 cycles");

        Ok(())
    }

    #[test]
    fn test_nop_instruction() -> Result<()> {
        let mut cpu = setup_cpu();
        let mut assembler = Assembler::new(0);

        // Set up test with parser
        cpu.registers.pc = 0x0100;

        // Parse a NOP instruction
        let bytes = assembler.assemble_instruction("NOP", &HashMap::new())?;
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 0xEA);

        // Write bytes to memory
        cpu.write_byte(0x0100, bytes[0])?;

        // Execute a full CPU step (fetch-decode-execute)
        let cycles = cpu.step()?;

        // Verify results - NOP doesn't change any registers but consumes 2 cycles
        assert_eq!(cpu.registers.pc, 0x0101);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.cycles, 2);

        Ok(())
    }

    /// Test the BIT instruction
    #[test]
    fn test_bit_instruction() -> Result<()> {
        let mut cpu = setup_cpu();
        let mut assembler = Assembler::new(0);

        // Setup test values in memory
        cpu.write_byte(0x0080, 0xC0)?; // Value with bits 7 and 6 set (11000000)
        cpu.write_byte(0x0081, 0x00)?; // Zero value

        // Test 1: BIT with memory value 0xC0 (bits 7 and 6 set)
        cpu.registers.a = 0xFF; // Set accumulator to all 1s

        // Clear all flags to ensure a clean state
        cpu.registers.status = 0;

        // Set up test with parser
        cpu.registers.pc = 0x0100;

        let labels = HashMap::new();

        // Parse a BIT instruction with zero page addressing mode
        let bytes = assembler.assemble_instruction("BIT $80", &labels)?;

        // Write bytes to memory
        for (i, &byte) in bytes.iter().enumerate() {
            cpu.write_byte(0x0100 + i as u16, byte)?;
        }

        // Execute a full CPU step (fetch-decode-execute)
        let cycles = cpu.step()?;

        // Result of AND is 0xFF & 0xC0 = 0xC0 (non-zero)
        // Negative set (bit 7 is set in 0xC0)
        // Overflow set (bit 6 is set in 0xC0)
        assert!(!cpu.get_flag(CpuFlag::Zero), "Zero flag should be clear");
        assert!(cpu.get_flag(CpuFlag::Negative), "Negative flag should be set");
        assert!(cpu.get_flag(CpuFlag::Overflow), "Overflow flag should be set");
        assert_eq!(cycles, 3, "BIT Zero Page should take 3 cycles");

        // Reset status register for next test
        cpu.registers.status = 0;

        // Test 2: BIT with zero value (0x00)
        cpu.registers.a = 0xFF; // Keep accumulator at all 1s

        // Set up test with parser
        cpu.registers.pc = 0x0200;

        // Parse a BIT instruction with zero page addressing mode
        let bytes = assembler.assemble_instruction("BIT $81", &labels)?;

        // Write bytes to memory
        for (i, &byte) in bytes.iter().enumerate() {
            cpu.write_byte(0x0200 + i as u16, byte)?;
        }

        // Execute a full CPU step (fetch-decode-execute)
        let cycles = cpu.step()?;

        // Result of AND is 0xFF & 0x00 = 0x00 (zero)
        // Negative clear (bit 7 is clear in 0x00)
        // Overflow clear (bit 6 is clear in 0x00)
        assert!(cpu.get_flag(CpuFlag::Zero), "Zero flag should be set");
        assert!(!cpu.get_flag(CpuFlag::Negative), "Negative flag should be clear");
        assert!(!cpu.get_flag(CpuFlag::Overflow), "Overflow flag should be clear");
        assert_eq!(cycles, 3, "BIT Zero Page should take 3 cycles");

        // Test 3: Check that accumulator is not changed
        assert_eq!(
            cpu.registers.a, 0xFF,
            "Accumulator should not be changed by BIT instruction"
        );

        // Test 4: BIT Absolute addressing mode
        cpu.registers.a = 0xFF;
        cpu.registers.status = 0;
        cpu.registers.pc = 0x0300;

        // Write test value to memory
        cpu.write_byte(0x1234, 0xC0)?;

        // Parse a BIT instruction with absolute addressing mode
        let bytes = assembler.assemble_instruction("BIT $1234", &labels)?;

        // Write bytes to memory
        for (i, &byte) in bytes.iter().enumerate() {
            cpu.write_byte(0x0300 + i as u16, byte)?;
        }

        // Execute a full CPU step
        let cycles = cpu.step()?;

        // Verify flags
        assert!(!cpu.get_flag(CpuFlag::Zero), "Zero flag should be clear");
        assert!(cpu.get_flag(CpuFlag::Negative), "Negative flag should be set");
        assert!(cpu.get_flag(CpuFlag::Overflow), "Overflow flag should be set");
        assert_eq!(cycles, 4, "BIT Absolute should take 4 cycles");

        Ok(())
    }

    /// Test the BPL instruction (Branch on Plus)
    #[test]
    fn test_bpl_instruction() -> Result<()> {
        let mut cpu = setup_cpu();

        // Test 1: BPL when N flag is clear (should branch)
        cpu.registers.status = 0; // Clear all flags
        cpu.registers.pc = 0x0100;

        // Write a target byte to memory (just to verify we reached it)
        cpu.write_byte(0x0112, 0x42)?;

        // Write the BPL instruction to branch 16 bytes forward
        // Manually write the BPL instruction with offset 16
        cpu.write_byte(0x0100, 0x10)?; // BPL opcode
        cpu.write_byte(0x0101, 0x10)?; // Offset 16 (decimal)

        // Execute the BPL instruction
        let cycles = cpu.step()?;

        // PC should now be at the branch target (0x0100 + 2 + 16 = 0x0112)
        assert_eq!(cpu.registers.pc, 0x0112, "PC should be at branch target when N=0");
        // Branch taken: base cycles (2) + branch taken (1) = 3
        assert_eq!(cycles, 3, "Branch taken should take 3 cycles");

        // Test 2: BPL when N flag is set (should not branch)
        cpu.registers.status = 0; // Clear all flags
        cpu.set_flag(CpuFlag::Negative, true); // Set N flag
        cpu.registers.pc = 0x0200;

        // Write the BPL instruction to branch 16 bytes forward
        // Manually write the BPL instruction with offset 16
        cpu.write_byte(0x0200, 0x10)?; // BPL opcode
        cpu.write_byte(0x0201, 0x10)?; // Offset 16 (decimal)

        // Execute the BPL instruction
        let cycles = cpu.step()?;

        // PC should now be right after the BPL instruction (0x0202)
        assert_eq!(cpu.registers.pc, 0x0202, "PC should not branch when N=1");
        // Branch not taken: base cycles (2)
        assert_eq!(cycles, 2, "Branch not taken should take 2 cycles");

        // Test 3: BPL with page crossing (should add extra cycle)
        cpu.registers.status = 0; // Clear all flags
        cpu.registers.pc = 0x02F0;

        // Write the BPL instruction to branch 32 bytes forward (crosses page)
        // Manually write the BPL instruction with offset 32
        cpu.write_byte(0x02F0, 0x10)?; // BPL opcode
        cpu.write_byte(0x02F1, 0x20)?; // Offset 32 (decimal)

        // Execute the BPL instruction
        let cycles = cpu.step()?;

        // PC should now be at the branch target (0x02F0 + 2 + 32 = 0x0312)
        assert_eq!(
            cpu.registers.pc, 0x0312,
            "PC should be at branch target when crossing page"
        );
        // Branch taken with page cross: base cycles (2) + branch taken (1) + page cross (1) = 4
        assert_eq!(cycles, 4, "Branch taken with page cross should take 4 cycles");

        Ok(())
    }
}
