use std::{fmt, str::FromStr};

use super::{Cpu, Instruction};
use crate::{errors::NesError, helpers::parse::parse_value};

/// Error type for addressing mode parsing
#[derive(Debug, PartialEq, Eq)]
pub enum AddressingModeError {
    /// The input format was invalid
    InvalidFormat(String),
    /// The addressing mode is ambiguous and cannot be determined from the input alone
    Ambiguous(String),
}

impl fmt::Display for AddressingModeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat(msg) => write!(f, "Invalid addressing mode format: {}", msg),
            Self::Ambiguous(msg) => write!(f, "Ambiguous addressing mode: {}", msg),
        }
    }
}

impl std::error::Error for AddressingModeError {}

/// Addressing modes for the 6502 CPU
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect, // (Indirect,X) - Pre-indexed indirect
    IndirectIndexed, // (Indirect),Y - Post-indexed indirect
    Implied,         // Implied addressing mode (no operand)
    Relative,        // Relative addressing mode (for branch instructions)
    Accumulator,     // Operations on the accumulator
}

impl FromStr for AddressingMode {
    type Err = AddressingModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // Immediate: #$xx
        if s.starts_with('#') {
            return Ok(Self::Immediate);
        }

        // Handle parentheses addressing modes
        if s.starts_with('(') {
            let s_clean = s.trim_start_matches('(').trim_end_matches(')').trim();

            // Indexed Indirect: ($xx,X)
            if s_clean.to_lowercase().ends_with(",x") {
                return Ok(Self::IndexedIndirect);
            }

            // Indirect Indexed: ($xx),Y
            if s.contains(')')
                && (s.to_lowercase().ends_with(",y") || 
                // Handle case with spaces: "($xx), Y"
                s.to_lowercase().replace(" ", "").ends_with("),y"))
            {
                return Ok(Self::IndirectIndexed);
            }

            // Indirect: ($xxxx)
            if !s_clean.contains(',') {
                if s_clean.starts_with('$') && s_clean.len() == 5 {
                    return Ok(Self::Indirect);
                }
                return Err(AddressingModeError::InvalidFormat(format!(
                    "'{}' could be indirect with various address sizes: {}",
                    s,
                    s.len()
                )));
            }
        }

        // Handle addressing modes with commas (indexed addressing)
        if s.contains(',') {
            let parts: Vec<&str> = s.split(',').collect();
            if parts.len() == 2 {
                let addr_part = parts[0].trim();
                let index_part = parts[1].trim().to_lowercase();

                if index_part == "x" {
                    if addr_part.starts_with('$') {
                        if addr_part.len() == 3 {
                            return Ok(Self::ZeroPageX);
                        }
                        if addr_part.len() == 5 {
                            return Ok(Self::AbsoluteX);
                        }
                        return Err(AddressingModeError::InvalidFormat(format!(
                            "'{}' invalid number of bytes: {}",
                            s,
                            addr_part.len()
                        )));
                    }
                } else if index_part == "y" {
                    if addr_part.starts_with('$') {
                        if addr_part.len() == 3 {
                            return Ok(Self::ZeroPageY);
                        }
                        if addr_part.len() == 5 {
                            return Ok(Self::AbsoluteY);
                        }
                        return Err(AddressingModeError::InvalidFormat(format!(
                            "'{}' invalid number of bytes: {}",
                            s,
                            addr_part.len()
                        )));
                    }
                }
            }
        }

        // Absolute: $xxxx (where xxxx is 0000-FFFF)
        if s.starts_with('$') && s.len() == 5 {
            return Ok(Self::Absolute);
        }

        // Zero Page: $xx (where xx is 00-FF)
        if s.starts_with('$') && s.len() == 3 {
            return Ok(Self::ZeroPage);
        }

        // Ambiguous: $ address could be ZeroPage or Absolute based on length
        if s.starts_with('$') {
            return Err(AddressingModeError::InvalidFormat(format!(
                "'{}' invalid number of bytes: {}",
                s,
                s.len()
            )));
        }

        // Implied addressing - empty string or just "A" (accumulator)
        if s.is_empty() || s == "A" || s == "a" {
            return Ok(Self::Implied);
        }

        // Relative addressing can't be determined from format alone
        // In a real assembler, this would be contextual based on the instruction

        Err(AddressingModeError::InvalidFormat(s.to_string()))
    }
}

impl fmt::Display for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            AddressingMode::Immediate => "Immediate mode",
            AddressingMode::ZeroPage => "Zero Page mode",
            AddressingMode::ZeroPageX => "Zero Page,X mode",
            AddressingMode::ZeroPageY => "Zero Page,Y mode",
            AddressingMode::Absolute => "Absolute mode",
            AddressingMode::AbsoluteX => "Absolute,X mode",
            AddressingMode::AbsoluteY => "Absolute,Y mode",
            AddressingMode::Indirect => "Indirect mode",
            AddressingMode::IndexedIndirect => "Indexed Indirect (X) mode",
            AddressingMode::IndirectIndexed => "Indirect Indexed (Y) mode",
            AddressingMode::Implied => "Implied mode",
            AddressingMode::Relative => "Relative mode",
            AddressingMode::Accumulator => "Accumulator mode",
        };
        write!(f, "{}", description)
    }
}

impl AddressingMode {
    /// Returns the size in bytes of an instruction with this addressing mode
    pub fn size(&self) -> u16 {
        match self {
            Self::Implied | Self::Accumulator => 1,
            Self::Immediate
            | Self::ZeroPage
            | Self::ZeroPageX
            | Self::ZeroPageY
            | Self::Relative
            | Self::IndexedIndirect
            | Self::IndirectIndexed => 2,
            Self::Absolute | Self::AbsoluteX | Self::AbsoluteY | Self::Indirect => 3,
        }
    }

    /// Returns the operand address for the given addressing mode
    /// This method assumes PC points to the operand byte (rather than the opcode)
    pub fn get_operand_address(&self, cpu: &Cpu) -> Result<u16, NesError> {
        match self {
            AddressingMode::Immediate => Ok(cpu.registers.pc),
            AddressingMode::ZeroPage => {
                // Zero page addressing uses only a single byte for the address
                // We read that byte and use it as an address in the range $0000-$00FF
                let zero_page_addr = cpu.read_byte(cpu.registers.pc)?;
                Ok(zero_page_addr as u16)
            },
            AddressingMode::ZeroPageX => {
                // Get the zero page address from the current PC
                let zero_page_addr = cpu.read_byte(cpu.registers.pc)?;

                // Add the X register to it (with wrap-around in the zero page)
                let effective_addr = (zero_page_addr.wrapping_add(cpu.registers.x)) as u16;

                // The high byte is always 0 since we stay in the zero page
                Ok(effective_addr)
            },
            AddressingMode::ZeroPageY => {
                // Get the zero page address from the current PC
                let zero_page_addr = cpu.read_byte(cpu.registers.pc)?;

                // Add the Y register to it (with wrap-around in the zero page)
                let effective_addr = (zero_page_addr.wrapping_add(cpu.registers.y)) as u16;

                // The high byte is always 0 since we stay in the zero page
                Ok(effective_addr)
            },
            AddressingMode::Absolute => {
                // Read a full 16-bit address (little-endian)
                cpu.read_word(cpu.registers.pc)
            },
            AddressingMode::AbsoluteX => {
                // Read the base address and add X register
                let base_addr = cpu.read_word(cpu.registers.pc)?;
                Ok(base_addr.wrapping_add(cpu.registers.x as u16))
            },
            AddressingMode::AbsoluteY => {
                // Read the base address and add Y register
                let base_addr = cpu.read_word(cpu.registers.pc)?;
                Ok(base_addr.wrapping_add(cpu.registers.y as u16))
            },
            AddressingMode::Indirect => {
                // Get the pointer address from the current PC
                let ptr_addr = cpu.read_word(cpu.registers.pc)?;

                // Handle the 6502 JMP indirect bug:
                // If the pointer address ends in $xxFF (page boundary),
                // the second byte is fetched from $xx00 instead of $xx+1:00
                if (ptr_addr & 0x00FF) == 0x00FF {
                    // Get the low byte from the given address
                    let low_byte = cpu.read_byte(ptr_addr)? as u16;

                    // Get the high byte from the same page (wrap around)
                    let high_byte = cpu.read_byte(ptr_addr & 0xFF00)? as u16;

                    // Combine into the effective address
                    Ok((high_byte << 8) | low_byte)
                } else {
                    // Normal case - just read the word from the pointer address
                    cpu.read_word(ptr_addr)
                }
            },
            AddressingMode::IndexedIndirect => {
                // 1. Get the zero page pointer base from the current PC
                let base_ptr = cpu.read_byte(cpu.registers.pc)?;

                // 2. Add X register to get the effective pointer (with zero page wrap-around)
                let eff_ptr = base_ptr.wrapping_add(cpu.registers.x);

                // 3. Read the target address from the zero page (with wrap-around for the high byte)
                cpu.read_word(eff_ptr as u16)
            },
            AddressingMode::IndirectIndexed => {
                // 1. Get the zero page pointer from the current PC
                let zp_ptr = cpu.read_byte(cpu.registers.pc)? as u16;

                // 2. Read the base address from zero page (wrapping around for high byte)
                let low_byte = cpu.read_byte(zp_ptr)? as u16;
                let high_byte = cpu.read_byte(zp_ptr.wrapping_add(1) & 0xFF)? as u16;
                let base_addr = (high_byte << 8) | low_byte;

                // 3. Add Y register to get the final effective address
                Ok(base_addr.wrapping_add(cpu.registers.y as u16))
            },
            AddressingMode::Implied => {
                // Implied addressing mode doesn't use an operand address
                // Return the current PC for consistency
                Ok(cpu.registers.pc)
            },
            AddressingMode::Relative => {
                // For relative addressing, we read a signed byte offset from the current PC
                // and add it to the PC+2 (PC+1 for the opcode, PC+1 for the offset byte)
                let offset = cpu.read_byte(cpu.registers.pc)? as i8; // Read as signed byte

                // Calculate the target address by adding the offset to PC+2
                let target = ((cpu.registers.pc as i32) + 2 + (offset as i32)) as u16;

                Ok(target)
            },
            AddressingMode::Accumulator => {
                // Accumulator mode doesn't use an operand address
                // Return the current PC for consistency
                Ok(cpu.registers.pc)
            },
        }
    }

    /// Checks if the addressing mode crosses a page boundary
    pub fn crosses_page_boundary(&self, cpu: &Cpu) -> Result<bool, NesError> {
        let res = match self {
            // Only these modes can cross page boundaries
            AddressingMode::AbsoluteX => {
                let base_addr = cpu.read_word(cpu.registers.pc)?;
                Self::crosses_boundary(base_addr, cpu.registers.x as u16)
            },
            AddressingMode::AbsoluteY => {
                let base_addr = cpu.read_word(cpu.registers.pc)?;
                Self::crosses_boundary(base_addr, cpu.registers.y as u16)
            },
            AddressingMode::IndirectIndexed => {
                let zp_ptr = cpu.read_byte(cpu.registers.pc)? as u16;
                let low_byte = cpu.read_byte(zp_ptr)? as u16;
                let high_byte = cpu.read_byte(zp_ptr.wrapping_add(1) & 0xFF)? as u16;
                let base_addr = (high_byte << 8) | low_byte;
                Self::crosses_boundary(base_addr, cpu.registers.y as u16)
            },
            // All other modes never cross page boundaries
            _ => false,
        };

        Ok(res)
    }

    /// Helper function to check if adding an offset to a base address crosses a page boundary
    fn crosses_boundary(base_addr: u16, offset: u16) -> bool {
        (base_addr & 0xFF00) != ((base_addr.wrapping_add(offset)) & 0xFF00)
    }

    /// Returns the additional cycles required for the addressing mode
    pub fn get_additional_cycles(&self, page_crossed: bool) -> u8 {
        match self {
            // Modes that always have additional cycles
            AddressingMode::ZeroPageX | AddressingMode::ZeroPageY => 1,
            AddressingMode::Indirect => 2,
            AddressingMode::IndexedIndirect => 4,

            // Modes with page crossing penalties
            AddressingMode::AbsoluteX | AddressingMode::AbsoluteY | AddressingMode::IndirectIndexed => {
                if page_crossed {
                    1
                } else {
                    0
                }
            },

            // All other modes have no additional cycles
            _ => 0,
        }
    }

    /// Determines the addressing mode based on the instruction and operand format
    pub fn from_instruction(instruction: Instruction, operand: &str) -> Result<Self, AddressingModeError> {
        let operand = operand.trim();

        // 1. Branch instructions always use Relative addressing
        if instruction.is_branch() {
            return Ok(Self::Relative);
        }

        // 2. Handle empty operand for implied addressing
        if operand.is_empty() {
            return Ok(Self::Implied);
        }
        
        // 2a. Handle "A" operand for accumulator addressing
        if operand.eq_ignore_ascii_case("a") {
            // Check if the instruction supports accumulator addressing
            match instruction {
                Instruction::ASL | Instruction::LSR => {
                    return Ok(Self::Accumulator);
                },
                _ => return Ok(Self::Implied),
            }
        }

        // 3. Handle immediate addressing (#$xx)
        if operand.starts_with('#') {
            // Verify it's a valid immediate format
            let value_part = &operand[1..];
            match parse_value::<u8>(value_part) {
                Ok(_) => return Ok(Self::Immediate),
                Err(_) => return Err(AddressingModeError::InvalidFormat(operand.to_string())),
            }
        }

        // 4. Handle JMP instruction (special case)
        if instruction == Instruction::JMP {
            return Self::parse_jump_addressing(operand);
        }

        // 5. Try to parse the operand format
        match Self::parse_addressing_format(operand) {
            Ok(addressing_mode) => {
                return Ok(addressing_mode);
            },
            Err(_) => {
                // 6. If we reach here and the operand doesn't contain special characters,
                // it might be a label reference
                if !operand.contains('$') && !operand.contains('#') && !operand.contains('(') {
                    return Ok(Self::Absolute); // Most non-branch labels use absolute addressing
                }
            },
        }

        // 7. If all else fails, report an invalid format
        Err(AddressingModeError::InvalidFormat(operand.to_string()))
    }

    // Helper method for parsing JMP addressing modes
    fn parse_jump_addressing(operand: &str) -> Result<Self, AddressingModeError> {
        // Handle indirect JMP format: JMP ($xxxx)
        if operand.starts_with('(') && operand.ends_with(')') {
            let inner = &operand[1..operand.len() - 1];
            // Try to parse the inner value
            match parse_value::<u16>(inner) {
                Ok(_) => return Ok(Self::Indirect),
                Err(_) => return Err(AddressingModeError::InvalidFormat(operand.to_string())),
            }
        }

        // Try to parse as a direct address
        match parse_value::<u16>(operand) {
            Ok(_) => Ok(Self::Absolute),
            Err(_) => {
                // If not starting with $ but looks like a label
                if !operand.contains('$') && !operand.contains('#') && !operand.contains('(') {
                    Ok(Self::Absolute)
                } else {
                    Err(AddressingModeError::InvalidFormat(operand.to_string()))
                }
            },
        }
    }

    // Helper method for parsing different addressing formats
    fn parse_addressing_format(operand: &str) -> Result<Self, AddressingModeError> {
        // Handle indexed indirect: ($xx,X)
        if operand.starts_with('(') && operand.to_lowercase().contains(",x)") {
            let inner = operand[1..operand.len() - 1].split(',').next().unwrap_or("").trim();
            match parse_value::<u16>(inner) {
                Ok(_) => return Ok(Self::IndexedIndirect),
                Err(_) => return Err(AddressingModeError::InvalidFormat(operand.to_string())),
            }
        }

        // Handle indirect indexed: ($xx),Y
        if operand.starts_with('(') && operand.to_lowercase().contains("),y") {
            let inner = operand[1..].split(')').next().unwrap_or("").trim();
            match parse_value::<u16>(inner) {
                Ok(_) => return Ok(Self::IndirectIndexed),
                Err(_) => return Err(AddressingModeError::InvalidFormat(operand.to_string())),
            }
        }

        // Handle zero page and absolute indexed
        if operand.contains(',') {
            let parts: Vec<&str> = operand.split(',').collect();
            if parts.len() == 2 {
                let addr_part = parts[0].trim();
                let index_part = parts[1].trim().to_lowercase();

                // Try to parse the address part
                match parse_value::<u16>(addr_part) {
                    Ok(value) => {
                        if index_part == "x" {
                            if value <= 0xFF {
                                return Ok(Self::ZeroPageX);
                            } else {
                                return Ok(Self::AbsoluteX);
                            }
                        } else if index_part == "y" {
                            if value <= 0xFF {
                                return Ok(Self::ZeroPageY);
                            } else {
                                return Ok(Self::AbsoluteY);
                            }
                        }
                    },
                    Err(_) => {
                        // If it contains invalid characters
                        return Err(AddressingModeError::InvalidFormat(operand.to_string()));
                    },
                }
            }

            // Invalid indexed format
            return Err(AddressingModeError::InvalidFormat(operand.to_string()));
        }

        // Handle simple zero page or absolute
        match parse_value::<u16>(operand) {
            Ok(value) => {
                if value <= 0xFF {
                    return Ok(Self::ZeroPage);
                } else {
                    return Ok(Self::Absolute);
                }
            },
            Err(err) => {
                // Check if the error message contains "ambiguous"
                // This is a bit of a hack, but it should work for now
                let err_str = err.to_string();
                if err_str.contains("ambiguous") {
                    return Err(AddressingModeError::Ambiguous(format!(
                        "'{}' could be ZeroPage or Absolute",
                        operand
                    )));
                }

                return Err(AddressingModeError::InvalidFormat(operand.to_string()));
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use anyhow::Result;

    use super::*;
    use crate::{cpu::Cpu, memory::Ram};

    /// Helper function to set up a CPU with memory for testing
    fn setup_cpu() -> Cpu {
        let mut cpu = Cpu::new();
        cpu.connect_memory(Rc::new(RefCell::new(Ram::default())));
        cpu
    }

    /// Helper function to set up a test case for addressing modes
    fn setup_test_case(
        cpu: &mut Cpu,
        operand_addr: u16,
        operand_data: &[u8],
        target_addr: u16,
        target_value: u8,
    ) -> Result<()> {
        // Write the operand data (can be 1 or 2 bytes)
        for (i, &byte) in operand_data.iter().enumerate() {
            cpu.write_byte(operand_addr + i as u16, byte)?;
        }

        // Write the target value at the target address
        cpu.write_byte(target_addr, target_value)?;

        // Set PC to point to the operand
        cpu.registers.pc = operand_addr;

        Ok(())
    }

    /// Helper function to simplify writing a word to memory
    fn write_word_at(cpu: &mut Cpu, addr: u16, value: u16) -> Result<()> {
        cpu.write_word(addr, value)?;
        Ok(())
    }

    /// Helper function to assert that an addressing mode returns the expected address
    fn assert_address(cpu: &Cpu, mode: AddressingMode, expected_addr: u16) -> Result<()> {
        let actual_addr = mode.get_operand_address(cpu)?;

        assert_eq!(
            actual_addr, expected_addr,
            "{} should return address ${:04X}, got ${:04X}",
            mode, expected_addr, actual_addr
        );

        Ok(())
    }

    /// Helper function to assert that an addressing mode returns the expected value
    fn assert_value(cpu: &Cpu, mode: AddressingMode, expected_value: u8) -> Result<(), NesError> {
        let actual_value = cpu.read_byte_using_mode(mode)?;
        assert_eq!(
            actual_value, expected_value,
            "{} should read value ${:02X}, got ${:02X}",
            mode, expected_value, actual_value
        );

        Ok(())
    }

    #[test]
    fn test_immediate_addressing_mode() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDA #$42 (Immediate)
        setup_test_case(
            &mut cpu,
            0x0200,  // operand address
            &[0x42], // operand data
            0,       // target address (N/A for immediate)
            0,       // target value (N/A for immediate)
        )?;

        // For immediate addressing, the operand is directly at PC
        assert_address(&cpu, AddressingMode::Immediate, 0x0200)?;
        assert_value(&cpu, AddressingMode::Immediate, 0x42)?;

        Ok(())
    }

    #[test]
    fn test_zero_page_addressing_mode() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDA $42 (Zero Page)
        // At $0042: The value $37
        setup_test_case(
            &mut cpu,
            0x0200,  // operand address
            &[0x42], // zero page address
            0x0042,  // target address
            0x37,    // target value
        )?;

        // Test zero page addressing
        assert_address(&cpu, AddressingMode::ZeroPage, 0x0042)?;
        assert_value(&cpu, AddressingMode::ZeroPage, 0x37)?;

        Ok(())
    }

    #[test]
    fn test_zero_page_x_addressing_mode() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDA $40,X with X=$05 (Zero Page,X)
        // Effective address: $40 + $05 = $45
        setup_test_case(
            &mut cpu,
            0x0200,  // operand address
            &[0x40], // zero page base address
            0x0045,  // target address (base + X)
            0x67,    // target value
        )?;

        // Set X register
        cpu.registers.x = 0x05;

        // Test zero page X addressing
        assert_address(&cpu, AddressingMode::ZeroPageX, 0x0045)?;
        assert_value(&cpu, AddressingMode::ZeroPageX, 0x67)?;

        Ok(())
    }

    #[test]
    fn test_zero_page_x_addressing_mode_wrap_around() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDA $F0,X with X=$20 (Zero Page,X with wrap-around)
        // Effective address: $F0 + $20 = $10 (wrap around)
        setup_test_case(
            &mut cpu,
            0x0200,  // operand address
            &[0xF0], // zero page base address
            0x0010,  // target address (wrapped around)
            0x42,    // target value
        )?;

        // Set X register
        cpu.registers.x = 0x20;

        // Test zero page X addressing with wrap-around
        assert_address(&cpu, AddressingMode::ZeroPageX, 0x0010)?;
        assert_value(&cpu, AddressingMode::ZeroPageX, 0x42)?;

        Ok(())
    }

    #[test]
    fn test_zero_page_y_addressing_mode() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDX $40,Y with Y=$07 (Zero Page,Y)
        // Effective address: $40 + $07 = $47
        setup_test_case(
            &mut cpu,
            0x0200,  // operand address
            &[0x40], // zero page base address
            0x0047,  // target address (base + Y)
            0x67,    // target value
        )?;

        // Set Y register
        cpu.registers.y = 0x07;

        // Test zero page Y addressing
        assert_address(&cpu, AddressingMode::ZeroPageY, 0x0047)?;
        assert_value(&cpu, AddressingMode::ZeroPageY, 0x67)?;

        Ok(())
    }

    #[test]
    fn test_zero_page_y_addressing_mode_wrap_around() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDX $F0,Y with Y=$30 (Zero Page,Y with wrap-around)
        // Effective address: $F0 + $30 = $20 (wrap around)
        setup_test_case(
            &mut cpu,
            0x0200,  // operand address
            &[0xF0], // zero page base address
            0x0020,  // target address (wrapped around)
            0x42,    // target value
        )?;

        // Set Y register
        cpu.registers.y = 0x30;

        // Test zero page Y addressing with wrap-around
        assert_address(&cpu, AddressingMode::ZeroPageY, 0x0020)?;
        assert_value(&cpu, AddressingMode::ZeroPageY, 0x42)?;

        Ok(())
    }

    #[test]
    fn test_absolute_addressing_mode() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDA $1234 (Absolute)
        setup_test_case(
            &mut cpu,
            0x0200,        // operand address
            &[0x34, 0x12], // absolute address $1234 (little-endian)
            0x1234,        // target address
            0x42,          // target value
        )?;

        // Test absolute addressing
        assert_address(&cpu, AddressingMode::Absolute, 0x1234)?;
        assert_value(&cpu, AddressingMode::Absolute, 0x42)?;

        Ok(())
    }

    #[test]
    fn test_absolute_x_addressing_mode() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDA $1230,X with X=$04 (Absolute,X)
        // Effective address: $1230 + $04 = $1234
        setup_test_case(
            &mut cpu,
            0x0200,        // operand address
            &[0x30, 0x12], // absolute address $1230 (little-endian)
            0x1234,        // target address (base + X)
            0x42,          // target value
        )?;

        // Set X register
        cpu.registers.x = 0x04;

        // Test absolute X addressing
        assert_address(&cpu, AddressingMode::AbsoluteX, 0x1234)?;
        assert_value(&cpu, AddressingMode::AbsoluteX, 0x42)?;

        Ok(())
    }

    #[test]
    fn test_absolute_x_addressing_mode_page_crossing() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDA $12FF,X with X=$01 (Absolute,X crossing page boundary)
        // Effective address: $12FF + $01 = $1300
        setup_test_case(
            &mut cpu,
            0x0200,        // operand address
            &[0xFF, 0x12], // absolute address $12FF (little-endian)
            0x1300,        // target address (crosses page boundary)
            0x42,          // target value
        )?;

        // Set X register
        cpu.registers.x = 0x01;

        // Test page crossing detection
        let crosses_page = AddressingMode::AbsoluteX.crosses_page_boundary(&cpu)?;
        assert!(crosses_page, "AbsoluteX should detect page boundary crossing");

        // Test address calculation and value reading
        assert_address(&cpu, AddressingMode::AbsoluteX, 0x1300)?;
        assert_value(&cpu, AddressingMode::AbsoluteX, 0x42)?;

        // Verify additional cycles for page crossing
        let additional_cycles = AddressingMode::AbsoluteX.get_additional_cycles(crosses_page);
        assert_eq!(additional_cycles, 1, "Page crossing should add 1 cycle");

        Ok(())
    }

    #[test]
    fn test_absolute_y_addressing_mode() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDA $1230,Y with Y=$05 (Absolute,Y)
        // Effective address: $1230 + $05 = $1235
        setup_test_case(
            &mut cpu,
            0x0200,        // operand address
            &[0x30, 0x12], // absolute address $1230 (little-endian)
            0x1235,        // target address (base + Y)
            0x42,          // target value
        )?;

        // Set Y register
        cpu.registers.y = 0x05;

        // Test absolute Y addressing
        assert_address(&cpu, AddressingMode::AbsoluteY, 0x1235)?;
        assert_value(&cpu, AddressingMode::AbsoluteY, 0x42)?;

        Ok(())
    }

    #[test]
    fn test_absolute_y_addressing_mode_wrap_around() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDA $12FF,Y with Y=$10 (Absolute,Y crossing page boundary)
        // Effective address: $12FF + $10 = $130F
        setup_test_case(
            &mut cpu,
            0x0200,        // operand address
            &[0xFF, 0x12], // absolute address $12FF (little-endian)
            0x130F,        // target address (crosses page boundary)
            0x42,          // target value
        )?;

        // Set Y register
        cpu.registers.y = 0x10;

        // Test page crossing detection
        let crosses_page = AddressingMode::AbsoluteY.crosses_page_boundary(&cpu)?;
        assert!(crosses_page, "AbsoluteY should detect page boundary crossing");

        // Test address calculation
        assert_address(&cpu, AddressingMode::AbsoluteY, 0x130F)?;
        assert_value(&cpu, AddressingMode::AbsoluteY, 0x42)?;

        // Verify additional cycles for page crossing
        let additional_cycles = AddressingMode::AbsoluteY.get_additional_cycles(crosses_page);
        assert_eq!(additional_cycles, 1, "Page crossing should add 1 cycle");

        Ok(())
    }

    #[test]
    fn test_indirect_addressing_mode() -> Result<()> {
        let mut cpu = setup_cpu();

        // JMP ($1234) - Jump to the address stored at $1234-$1235
        setup_test_case(
            &mut cpu,
            0x0200,        // operand address
            &[0x34, 0x12], // indirect pointer address $1234
            0,             // target address (N/A for this test)
            0,             // target value (N/A for this test)
        )?;

        // At $1234-$1235, store the target address $ABCD
        write_word_at(&mut cpu, 0x1234, 0xABCD)?;

        // Test indirect addressing
        let addr = AddressingMode::Indirect.get_operand_address(&cpu)?;
        assert_eq!(addr, 0xABCD, "Indirect addressing should return $ABCD");

        Ok(())
    }

    #[test]
    fn test_indirect_addressing_mode_page_boundary_bug() -> Result<()> {
        let mut cpu = setup_cpu();

        // JMP ($12FF) - Jump to the address stored at $12FF-$1200 (bug: wraps in page)
        setup_test_case(
            &mut cpu,
            0x0200,        // operand address
            &[0xFF, 0x12], // indirect pointer address $12FF (at page boundary)
            0,             // target address (N/A for this test)
            0,             // target value (N/A for this test)
        )?;

        // At $12FF, store the low byte $CD
        cpu.write_byte(0x12FF, 0xCD)?;

        // At $1200 (not $1300), store the high byte $AB due to the page boundary bug
        cpu.write_byte(0x1200, 0xAB)?;

        // Test indirect addressing with page boundary bug
        let addr = AddressingMode::Indirect.get_operand_address(&cpu)?;
        assert_eq!(
            addr, 0xABCD,
            "Indirect addressing with page boundary bug should return $ABCD"
        );

        Ok(())
    }

    #[test]
    fn test_indexed_indirect_addressing_mode() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDA ($40,X) with X=$05 - Load from address stored at ($40+$05)
        setup_test_case(
            &mut cpu,
            0x0200,  // operand address
            &[0x40], // zero page base pointer
            0xABCD,  // target address (stored at $45-$46)
            0x42,    // target value
        )?;

        // Set X register
        cpu.registers.x = 0x05;

        // Store target address $ABCD at $45-$46 (zero page + X)
        cpu.write_word(0x0045, 0xABCD)?; // Low byte

        // Test indexed indirect addressing
        let addr = AddressingMode::IndexedIndirect.get_operand_address(&cpu)?;
        assert_eq!(addr, 0xABCD, "Indexed indirect addressing should return $ABCD");

        let value = cpu.read_byte_using_mode(AddressingMode::IndexedIndirect)?;
        assert_eq!(value, 0x42, "Value at effective address $ABCD should be $42");

        Ok(())
    }

    #[test]
    fn test_indexed_indirect_addressing_mode_wrap_around() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDA ($FF,X) with X=$02 - Tests zero page wrap-around
        setup_test_case(
            &mut cpu,
            0x0200,  // operand address
            &[0xFF], // zero page base pointer
            0xABCD,  // target address (stored at $01-$02 after wrap)
            0x42,    // target value
        )?;

        // Set X register
        cpu.registers.x = 0x02;

        // Effective ZP pointer = $FF + $02 = $01 (with zero page wrap)
        // Store target address $ABCD at $01-$02
        cpu.write_byte(0x0001, 0xCD)?; // Low byte
        cpu.write_byte(0x0002, 0xAB)?; // High byte

        // Test indexed indirect addressing with wrap-around
        let addr = AddressingMode::IndexedIndirect.get_operand_address(&cpu)?;
        assert_eq!(addr, 0xABCD, "Indexed indirect with zero page wrap should return $ABCD");

        let value = cpu.read_byte_using_mode(AddressingMode::IndexedIndirect)?;
        assert_eq!(value, 0x42, "Value at effective address $ABCD should be $42");

        Ok(())
    }

    #[test]
    fn test_indirect_indexed_addressing_mode() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDA ($40),Y with Y=$08 - Load from address stored at $40-$41 plus Y
        setup_test_case(
            &mut cpu,
            0x0200,  // operand address
            &[0x40], // zero page pointer
            0x123C,  // target address ($1234 + $08)
            0x42,    // target value
        )?;

        // Set Y register
        cpu.registers.y = 0x08;

        // Store base address $1234 at zero page $40-$41
        cpu.write_byte(0x0040, 0x34)?; // Low byte
        cpu.write_byte(0x0041, 0x12)?; // High byte

        // Test indirect indexed addressing
        let addr = AddressingMode::IndirectIndexed.get_operand_address(&cpu)?;
        assert_eq!(addr, 0x123C, "Indirect indexed addressing should return $123C");

        let value = cpu.read_byte_using_mode(AddressingMode::IndirectIndexed)?;
        assert_eq!(value, 0x42, "Value at effective address $123C should be $42");

        Ok(())
    }

    #[test]
    fn test_indirect_indexed_addressing_mode_page_crossing() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDA ($40),Y with Y=$20 - Tests page boundary crossing
        setup_test_case(
            &mut cpu,
            0x0200,  // operand address
            &[0x40], // zero page pointer
            0x1310,  // target address ($12F0 + $20, crosses page)
            0x42,    // target value
        )?;

        // Set Y register
        cpu.registers.y = 0x20;

        // Store base address $12F0 at zero page $40-$41
        cpu.write_byte(0x0040, 0xF0)?; // Low byte
        cpu.write_byte(0x0041, 0x12)?; // High byte

        // Test indirect indexed addressing with page crossing
        let crosses_page = AddressingMode::IndirectIndexed.crosses_page_boundary(&cpu)?;
        assert!(crosses_page, "IndirectIndexed should detect page boundary crossing");

        // Verify address calculation and value
        assert_address(&cpu, AddressingMode::IndirectIndexed, 0x1310)?;
        assert_value(&cpu, AddressingMode::IndirectIndexed, 0x42)?;

        // Verify additional cycles for page crossing
        let additional_cycles = AddressingMode::IndirectIndexed.get_additional_cycles(crosses_page);
        assert_eq!(additional_cycles, 1, "Page crossing should add 1 cycle");

        Ok(())
    }

    #[test]
    fn test_indirect_indexed_addressing_mode_zero_page_wrap() -> Result<()> {
        let mut cpu = setup_cpu();

        // LDA ($FF),Y with Y=$10 - Tests zero page wrap-around for pointer
        setup_test_case(
            &mut cpu,
            0x0200,  // operand address
            &[0xFF], // zero page pointer (at boundary, will wrap)
            0x1244,  // target address ($1234 + $10)
            0x42,    // target value
        )?;

        // Set Y register
        cpu.registers.y = 0x10;

        // Store the base address split between $FF and $00 (wrap-around in zero page)
        cpu.write_byte(0x00FF, 0x34)?; // Low byte at $FF
        cpu.write_byte(0x0000, 0x12)?; // High byte at $00 (wrapped around)

        // Test indirect indexed addressing with zero page wrap-around
        let addr = AddressingMode::IndirectIndexed.get_operand_address(&cpu)?;
        assert_eq!(addr, 0x1244, "Indirect indexed with ZP wrap should return $1244");

        let value = cpu.read_byte_using_mode(AddressingMode::IndirectIndexed)?;
        assert_eq!(value, 0x42, "Value with ZP wrap-around should be $42");

        Ok(())
    }

    #[test]
    fn test_crosses_page_boundary() -> Result<()> {
        let cpu = setup_cpu();

        // 1. Test AbsoluteX
        // Cross: $12F0 + $10 = $1300
        test_page_crossing(AddressingMode::AbsoluteX, 0x0201, 0x12F0, "X", 0x10, true, None)?;
        // No cross: $1280 + $10 = $1290
        test_page_crossing(AddressingMode::AbsoluteX, 0x0201, 0x1280, "X", 0x10, false, None)?;

        // 2. Test AbsoluteY
        // Cross: $12F0 + $20 = $1310
        test_page_crossing(AddressingMode::AbsoluteY, 0x0201, 0x12F0, "Y", 0x20, true, None)?;
        // No cross: $1280 + $20 = $12A0
        test_page_crossing(AddressingMode::AbsoluteY, 0x0201, 0x1280, "Y", 0x20, false, None)?;

        // 3. Test IndirectIndexed
        // Cross: $12F0 + $20 = $1310
        test_page_crossing(
            AddressingMode::IndirectIndexed,
            0x0201,
            0x80,
            "Y",
            0x20,
            true,
            Some(0x12F0),
        )?;
        // No cross: $1280 + $20 = $12A0
        test_page_crossing(
            AddressingMode::IndirectIndexed,
            0x0201,
            0x80,
            "Y",
            0x20,
            false,
            Some(0x1280),
        )?;

        // Test modes that never cross page boundaries
        let non_crossing_modes = [
            AddressingMode::Immediate,
            AddressingMode::ZeroPage,
            AddressingMode::ZeroPageX,
            AddressingMode::ZeroPageY,
            AddressingMode::Absolute,
            AddressingMode::Indirect,
            AddressingMode::IndexedIndirect,
        ];

        for mode in non_crossing_modes {
            assert!(
                !mode.crosses_page_boundary(&cpu)?,
                "{:?} should never cross page boundaries",
                mode
            );
        }

        Ok(())
    }

    fn test_page_crossing(
        mode: AddressingMode,
        pc: u16,
        base_addr: u16,
        offset_reg: &str,
        offset_val: u8,
        should_cross: bool,
        zp_addr_value: Option<u16>,
    ) -> Result<()> {
        // Create a fresh CPU for each test to avoid borrow issues
        let mut test_cpu = setup_cpu();

        // Set PC and register values
        test_cpu.registers.pc = pc;
        match offset_reg {
            "X" => test_cpu.registers.x = offset_val,
            "Y" => test_cpu.registers.y = offset_val,
            _ => {},
        }

        // For AbsoluteX and AbsoluteY, set the base address in memory
        if mode == AddressingMode::AbsoluteX || mode == AddressingMode::AbsoluteY {
            write_word_at(&mut test_cpu, pc, base_addr)?;
        }
        // For IndirectIndexed, set up the zero page pointer and target
        else if mode == AddressingMode::IndirectIndexed {
            test_cpu.write_byte(pc, base_addr as u8)?; // ZP pointer

            // Write the value at the zero page address
            if let Some(value) = zp_addr_value {
                write_word_at(&mut test_cpu, base_addr as u16, value)?;
            } else {
                write_word_at(&mut test_cpu, base_addr as u16, 0x12F0)?; // Default base address
            }
        }

        // Test crossing detection
        let crosses = mode.crosses_page_boundary(&test_cpu)?;
        assert_eq!(
            crosses,
            should_cross,
            "{:?} with base {:04X} + {} = {:02X} should {} cross page boundary",
            mode,
            base_addr,
            offset_reg,
            offset_val,
            if should_cross { "" } else { "not " }
        );

        Ok(())
    }

    #[test]
    fn test_get_additional_cycles() -> Result<(), NesError> {
        // Test cases are (addressing_mode, page_crossed, expected_cycles)
        let test_cases = [
            // Modes with fixed additional cycles regardless of page crossing
            (AddressingMode::ZeroPageX, false, 1),
            (AddressingMode::ZeroPageX, true, 1),
            (AddressingMode::ZeroPageY, false, 1),
            (AddressingMode::ZeroPageY, true, 1),
            (AddressingMode::Indirect, false, 2),
            (AddressingMode::Indirect, true, 2),
            (AddressingMode::IndexedIndirect, false, 4),
            (AddressingMode::IndexedIndirect, true, 4),
            // Modes with additional cycles when page boundary is crossed
            (AddressingMode::AbsoluteX, true, 1),
            (AddressingMode::AbsoluteX, false, 0),
            (AddressingMode::AbsoluteY, true, 1),
            (AddressingMode::AbsoluteY, false, 0),
            (AddressingMode::IndirectIndexed, true, 1),
            (AddressingMode::IndirectIndexed, false, 0),
            // Modes with no additional cycles
            (AddressingMode::Immediate, false, 0),
            (AddressingMode::Immediate, true, 0),
            (AddressingMode::ZeroPage, false, 0),
            (AddressingMode::ZeroPage, true, 0),
            (AddressingMode::Absolute, false, 0),
            (AddressingMode::Absolute, true, 0),
        ];

        for (mode, page_crossed, expected_cycles) in test_cases {
            let actual_cycles = mode.get_additional_cycles(page_crossed);
            assert_eq!(
                actual_cycles, expected_cycles,
                "Mode {:?} with page_crossed={} should take {} additional cycles, got {}",
                mode, page_crossed, expected_cycles, actual_cycles
            );
        }

        Ok(())
    }

    #[test]
    fn test_addressing_mode_size() {
        assert_eq!(AddressingMode::Implied.size(), 1);
        assert_eq!(AddressingMode::Immediate.size(), 2);
        assert_eq!(AddressingMode::ZeroPage.size(), 2);
        assert_eq!(AddressingMode::ZeroPageX.size(), 2);
        assert_eq!(AddressingMode::ZeroPageY.size(), 2);
        assert_eq!(AddressingMode::Relative.size(), 2);
        assert_eq!(AddressingMode::Absolute.size(), 3);
        assert_eq!(AddressingMode::AbsoluteX.size(), 3);
        assert_eq!(AddressingMode::AbsoluteY.size(), 3);
        assert_eq!(AddressingMode::Indirect.size(), 3);
        assert_eq!(AddressingMode::IndexedIndirect.size(), 2);
        assert_eq!(AddressingMode::IndirectIndexed.size(), 2);
    }

    #[test]
    fn test_addressing_mode_from_str() -> Result<()> {
        // Test successful parsing
        assert_eq!(AddressingMode::from_str("#$42")?, AddressingMode::Immediate);
        assert_eq!(AddressingMode::from_str("$42")?, AddressingMode::ZeroPage);
        assert_eq!(AddressingMode::from_str("$1234")?, AddressingMode::Absolute);
        assert_eq!(AddressingMode::from_str("$42,X")?, AddressingMode::ZeroPageX);
        assert_eq!(AddressingMode::from_str("$1234,X")?, AddressingMode::AbsoluteX);
        assert_eq!(AddressingMode::from_str("$42,Y")?, AddressingMode::ZeroPageY);
        assert_eq!(AddressingMode::from_str("$1234,Y")?, AddressingMode::AbsoluteY);
        assert_eq!(AddressingMode::from_str("($1234)")?, AddressingMode::Indirect);
        assert_eq!(AddressingMode::from_str("($42,X)")?, AddressingMode::IndexedIndirect);
        assert_eq!(AddressingMode::from_str("($42),Y")?, AddressingMode::IndirectIndexed);
        assert_eq!(AddressingMode::from_str("")?, AddressingMode::Implied);
        assert_eq!(AddressingMode::from_str("A")?, AddressingMode::Implied);

        // Test with spaces and different case
        assert_eq!(AddressingMode::from_str(" #$42 ")?, AddressingMode::Immediate);
        assert_eq!(AddressingMode::from_str("$42, x")?, AddressingMode::ZeroPageX);
        assert_eq!(AddressingMode::from_str("($42), y")?, AddressingMode::IndirectIndexed);

        // Test invalid format
        assert!(matches!(
            AddressingMode::from_str("$4"),
            Err(AddressingModeError::InvalidFormat(_))
        ));
        assert!(matches!(
            AddressingMode::from_str("($42)"),
            Err(AddressingModeError::InvalidFormat(_))
        ));
        assert!(matches!(
            AddressingMode::from_str("$4,X"),
            Err(AddressingModeError::InvalidFormat(_))
        ));
        assert!(matches!(
            AddressingMode::from_str("invalid"),
            Err(AddressingModeError::InvalidFormat(_))
        ));

        Ok(())
    }

    #[test]
    fn test_addressing_mode_from_instruction() -> Result<()> {
        // Test branch instructions always use Relative addressing
        assert_eq!(
            AddressingMode::from_instruction(Instruction::BPL, "$10")?,
            AddressingMode::Relative
        );

        // Test JMP instruction with different operands
        assert_eq!(
            AddressingMode::from_instruction(Instruction::JMP, "$1234")?,
            AddressingMode::Absolute
        );
        assert_eq!(
            AddressingMode::from_instruction(Instruction::JMP, "($1234)")?,
            AddressingMode::Indirect
        );

        // Test implied and accumulator addressing
        assert_eq!(
            AddressingMode::from_instruction(Instruction::RTS, "")?,
            AddressingMode::Implied
        );

        // Test immediate addressing
        assert_eq!(
            AddressingMode::from_instruction(Instruction::LDA, "#$42")?,
            AddressingMode::Immediate
        );

        // Test zero page and absolute addressing
        assert_eq!(
            AddressingMode::from_instruction(Instruction::LDA, "$42")?,
            AddressingMode::ZeroPage
        );
        assert_eq!(
            AddressingMode::from_instruction(Instruction::LDA, "$1234")?,
            AddressingMode::Absolute
        );

        // Test indexed addressing
        assert_eq!(
            AddressingMode::from_instruction(Instruction::LDA, "$42,X")?,
            AddressingMode::ZeroPageX
        );
        assert_eq!(
            AddressingMode::from_instruction(Instruction::LDA, "$1234,X")?,
            AddressingMode::AbsoluteX
        );
        assert_eq!(
            AddressingMode::from_instruction(Instruction::LDX, "$42,Y")?,
            AddressingMode::ZeroPageY
        );
        assert_eq!(
            AddressingMode::from_instruction(Instruction::LDX, "$1234,Y")?,
            AddressingMode::AbsoluteY
        );

        // Test indirect addressing modes
        assert_eq!(
            AddressingMode::from_instruction(Instruction::LDA, "($42,X)")?,
            AddressingMode::IndexedIndirect
        );
        assert_eq!(
            AddressingMode::from_instruction(Instruction::LDA, "($42),Y")?,
            AddressingMode::IndirectIndexed
        );

        // Test label references (non-branch)
        assert_eq!(
            AddressingMode::from_instruction(Instruction::LDA, "label")?,
            AddressingMode::Absolute
        );

        Ok(())
    }
}
