use super::{Instruction, AddressingMode, InstructionMetadata, InstructionDecoder, CpuError};
use thiserror::Error;

/// Errors that can occur during instruction parsing
#[derive(Debug, Error)]
pub enum AssembleError {
    #[error("Unknown instruction mnemonic: {0}")]
    UnknownMnemonic(String),
    
    #[error("Invalid addressing mode for instruction: {0}")]
    InvalidAddressingMode(String),
    
    #[error("Invalid operand format: {0}")]
    InvalidOperandFormat(String),
    
    #[error("Value out of range: {0}")]
    ValueOutOfRange(String),
    
    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),
    
    #[error("CPU error: {0}")]
    CpuError(#[from] CpuError),
}

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, AssembleError>;

/// Parses assembly language instructions into their binary representation
pub struct Assembler {
    decoder: InstructionDecoder,
}

impl Assembler {
    /// Creates a new instruction parser
    pub fn new() -> Self {
        Self {
            decoder: InstructionDecoder::new(),
        }
    }
    
    /// Assembles a string into an instruction metadata object
    /// 
    /// Examples:
    /// - "LDA #$42" -> Load accumulator with immediate value $42
    /// - "LDA $2000" -> Load accumulator from address $2000
    /// - "LDA $42" -> Load accumulator from zero page address $42
    fn parse_instruction(&self, input: &str) -> ParseResult<InstructionMetadata> {
        // Split input into mnemonic and operand
        let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
        if parts.is_empty() {
            return Err(AssembleError::InvalidSyntax("Empty input".to_string()));
        }
        
        // Parse the instruction mnemonic using FromStr
        let instruction = parts[0].parse::<Instruction>()
            .map_err(|_| AssembleError::UnknownMnemonic(parts[0].to_string()))?;
        
        // Check for implied addressing mode instructions (no operand)
        if instruction == Instruction::BRK || instruction == Instruction::RTS {
            return self.decoder.lookup(instruction, AddressingMode::Implied)
                .map_err(|_| AssembleError::InvalidAddressingMode(format!("{instruction} does not support implied mode")));
        }
        
        // Extract and parse the operand if it exists
        if parts.len() < 2 {
            return Err(AssembleError::InvalidSyntax("Missing operand".to_string()));
        }
        
        let operand = parts[1].trim();
        let (addressing_mode, _operand_value) = self.parse_addressing_mode(operand)?;
        
        // Look up the instruction metadata using the decoder
        let metadata = self.decoder.lookup(instruction, addressing_mode)
            .map_err(|_| AssembleError::InvalidAddressingMode(format!("{instruction} does not support {addressing_mode:?}")))?;
        
        Ok(metadata)
    }
    
    /// Determines the addressing mode and operand value from a string
    /// 
    /// Examples:
    /// - "#$42" -> Immediate mode with value 0x42
    /// - "$2000" -> Absolute mode with value 0x2000
    /// - "$42" -> Zero page mode with value 0x42
    fn parse_addressing_mode(&self, operand: &str) -> ParseResult<(AddressingMode, u16)> {
        // Immediate: #$xx
        if operand.starts_with("#$") {
            let value = u8::from_str_radix(&operand[2..], 16)
                .map_err(|_| AssembleError::InvalidOperandFormat(format!("Invalid hex value: {}", &operand[2..])))?;
            return Ok((AddressingMode::Immediate, value as u16));
        }
        
        // Zero Page: $xx (where xx is 00-FF)
        if operand.starts_with('$') && operand.len() == 3 {
            let value = u8::from_str_radix(&operand[1..], 16)
                .map_err(|_| AssembleError::InvalidOperandFormat(format!("Invalid hex value: {}", &operand[1..])))?;
            return Ok((AddressingMode::ZeroPage, value as u16));
        }
        
        // Absolute: $xxxx (where xxxx is 0000-FFFF)
        if operand.starts_with('$') && operand.len() == 5 {
            let value = u16::from_str_radix(&operand[1..], 16)
                .map_err(|_| AssembleError::InvalidOperandFormat(format!("Invalid hex value: {}", &operand[1..])))?;
            return Ok((AddressingMode::Absolute, value));
        }
        
        Err(AssembleError::InvalidAddressingMode(operand.to_string()))
    }
    
    /// Assembles a string into the raw opcode and operand bytes
    pub fn assemble_instruction(&self, input: &str) -> ParseResult<Vec<u8>> {
        let metadata = self.parse_instruction(input)?;
        let mut bytes = vec![metadata.opcode];
        
        // For implied addressing mode, no operands to add
        if metadata.addressing_mode == AddressingMode::Implied {
            return Ok(bytes);
        }
        
        // Extract operand value from input string to add correct bytes
        let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
        if parts.len() < 2 {
            return Err(AssembleError::InvalidSyntax("Missing operand".to_string()));
        }
        
        let operand = parts[1].trim();
        let (_, operand_value) = self.parse_addressing_mode(operand)?;
        
        match metadata.addressing_mode {
            AddressingMode::Immediate | 
            AddressingMode::ZeroPage => {
                bytes.push(operand_value as u8);
            }
            AddressingMode::Absolute => {
                bytes.push((operand_value & 0xFF) as u8);
                bytes.push((operand_value >> 8) as u8);
            }
            _ => return Err(AssembleError::InvalidAddressingMode(format!("Unsupported addressing mode: {:?}", metadata.addressing_mode))),
        }
        
        Ok(bytes)
    }

    /// Assembles a multi-line program, handling comments and empty lines
    /// 
    /// This method processes a complete program with multiple instructions.
    /// It ignores:
    /// - Empty lines
    /// - Comments (lines starting with ';')
    /// - Inline comments (text after ';' on a line)
    /// 
    /// Returns the assembled bytes for all valid instructions.
    pub fn assemble_program(&self, program: &str) -> ParseResult<Vec<u8>> {
        let mut result = Vec::new();
        
        // Process each line
        for (line_num, line) in program.lines().enumerate() {
            // Remove comments (anything after ';')
            let line = match line.find(';') {
                Some(idx) => &line[0..idx],
                None => line,
            }.trim();
            
            // Skip empty lines
            if line.is_empty() {
                continue;
            }
            
            // Parse the line and append bytes
            match self.assemble_instruction(line) {
                Ok(bytes) => {
                    result.extend(bytes);
                },
                Err(err) => {
                    // Add line number to error for better debugging
                    return Err(AssembleError::InvalidSyntax(
                        format!("Line {}: {} - '{}'", line_num + 1, err, line)
                    ));
                }
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    
    /// Tests for instruction parsing (mnemonic recognition)
    #[test]
    fn test_instruction_parsing() -> Result<()> {
        // Direct FromStr tests
        assert_eq!("LDA".parse::<Instruction>()?, Instruction::LDA);
        assert!("XYZ".parse::<Instruction>().is_err());
        
        Ok(())
    }
    
    /// Tests for addressing mode recognition
    #[test]
    fn test_addressing_mode_parsing() -> Result<()> {
        let parser = Assembler::new();
        
        // Test immediate mode
        let (mode, _) = parser.parse_addressing_mode("#$42")?;
        assert_eq!(mode, AddressingMode::Immediate);
        
        // Test zero page
        let (mode, _) = parser.parse_addressing_mode("$42")?;
        assert_eq!(mode, AddressingMode::ZeroPage);
        
        // Test absolute
        let (mode, _) = parser.parse_addressing_mode("$1234")?;
        assert_eq!(mode, AddressingMode::Absolute);
        
        // Test invalid format
        assert!(parser.parse_addressing_mode("xyz").is_err());
        
        Ok(())
    }
    
    /// Tests for operand value extraction
    #[test]
    fn test_operand_parsing() -> Result<()> {
        let parser = Assembler::new();
        
        // Test immediate operand
        let (_, value) = parser.parse_addressing_mode("#$42")?;
        assert_eq!(value, 0x42);
        
        // Test zero page operand
        let (_, value) = parser.parse_addressing_mode("$42")?;
        assert_eq!(value, 0x42);
        
        // Test absolute operand
        let (_, value) = parser.parse_addressing_mode("$1234")?;
        assert_eq!(value, 0x1234);
        
        // Test invalid hex value
        assert!(parser.parse_addressing_mode("#$ZZ").is_err());
        
        Ok(())
    }
    
    /// Integration tests for complete instruction parsing
    #[test]
    fn test_complete_instruction_parsing() -> Result<()> {
        let parser = Assembler::new();
        
        // Test LDA immediate
        let metadata = parser.parse_instruction("LDA #$42")?;
        assert_eq!(metadata.instruction, Instruction::LDA);
        assert_eq!(metadata.addressing_mode, AddressingMode::Immediate);
        assert_eq!(metadata.opcode, 0xA9);
        
        // Test LDA zero page
        let metadata = parser.parse_instruction("LDA $42")?;
        assert_eq!(metadata.instruction, Instruction::LDA);
        assert_eq!(metadata.addressing_mode, AddressingMode::ZeroPage);
        assert_eq!(metadata.opcode, 0xA5);
        
        // Test LDA absolute
        let metadata = parser.parse_instruction("LDA $1234")?;
        assert_eq!(metadata.instruction, Instruction::LDA);
        assert_eq!(metadata.addressing_mode, AddressingMode::Absolute);
        assert_eq!(metadata.opcode, 0xAD);
        
        Ok(())
    }
    
    /// Tests for error conditions
    #[test]
    fn test_error_conditions() -> Result<()> {
        let parser = Assembler::new();
        
        // Test invalid mnemonic
        let result = parser.parse_instruction("XYZ #$42");
        assert!(result.is_err());
        
        // Test invalid addressing mode syntax
        let result = parser.parse_instruction("LDA xyz");
        assert!(result.is_err());
        
        // Test invalid operand value
        let result = parser.parse_instruction("LDA #$ZZ");
        assert!(result.is_err());
        
        // Test missing operand
        let result = parser.parse_instruction("LDA");
        assert!(result.is_err());
        
        Ok(())
    }
} 