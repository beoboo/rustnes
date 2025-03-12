use std::collections::HashMap;

use thiserror::Error;

use super::{AddressingMode, Instruction, InstructionDecoder, InstructionMetadata};
use crate::errors::NesError;

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

    #[error("Label error: {0}")]
    LabelError(String),

    #[error("NES error: {0}")]
    NesError(#[from] NesError),
}

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, AssembleError>;

/// Parses assembly language instructions into their binary representation
pub struct Assembler {
    decoder: InstructionDecoder,
    pub load_address: u16,
}

impl Assembler {
    /// Creates a new instruction parser
    pub fn new(load_address: u16) -> Self {
        Self {
            decoder: InstructionDecoder::new(),
            load_address,
        }
    }

    /// Splits an instruction string into mnemonic and operand parts
    fn split_instruction(&self, input: &str) -> ParseResult<(String, Option<String>)> {
        let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
        if parts.is_empty() {
            return Err(AssembleError::InvalidSyntax("Empty input".to_string()));
        }

        let mnemonic = parts[0].to_string();
        let operand = if parts.len() > 1 {
            Some(parts[1].trim().to_string())
        } else {
            None
        };

        Ok((mnemonic, operand))
    }

    /// Checks if instruction uses implied addressing mode (no operand)
    fn is_implied_addressing(&self, instruction: Instruction) -> bool {
        matches!(instruction, Instruction::BRK | Instruction::RTS | Instruction::NOP)
    }

    /// Handles an instruction with implied addressing mode
    fn handle_implied_instruction(&self, instruction: Instruction) -> ParseResult<InstructionMetadata> {
        self.decoder
            .lookup(instruction, AddressingMode::Implied)
            .map_err(|_| AssembleError::InvalidAddressingMode(format!("{instruction} does not support implied mode")))
    }

    /// Checks if an operand is a label reference (not starting with $ or #)
    fn is_label_reference(&self, operand: &str) -> bool {
        !operand.starts_with('$') && !operand.starts_with('#')
    }

    /// Checks if a label exists in the labels map and handles the reference
    fn handle_label_reference(
        &self,
        instruction: Instruction,
        operand: &str,
        labels: &HashMap<String, u16>,
    ) -> ParseResult<Option<(InstructionMetadata, u16)>> {
        if !self.is_label_reference(operand) {
            return Ok(None); // Not a label reference
        }

        // It's a label reference - look it up in the labels map
        let address = labels
            .get(operand)
            .ok_or_else(|| AssembleError::LabelError(format!("Undefined label: {}", operand)))?;

        // Use absolute addressing mode for the label
        let metadata = self
            .decoder
            .lookup(instruction, AddressingMode::Absolute)
            .map_err(|_| {
                AssembleError::InvalidAddressingMode(format!("{instruction} does not support absolute mode for labels"))
            })?;

        Ok(Some((metadata, *address)))
    }

    /// Parses an instruction string into metadata
    /// If labels map is provided, label references in operands will be resolved
    fn parse_instruction(
        &self,
        input: &str,
        labels: Option<&HashMap<String, u16>>,
    ) -> ParseResult<InstructionMetadata> {
        // Split input into mnemonic and operand
        let (mnemonic, operand_opt) = self.split_instruction(input)?;

        // Parse the instruction mnemonic using FromStr
        let instruction = mnemonic
            .parse::<Instruction>()
            .map_err(|_| AssembleError::UnknownMnemonic(mnemonic))?;

        // Check for implied addressing mode instructions (no operand)
        if self.is_implied_addressing(instruction) {
            return self.handle_implied_instruction(instruction);
        }

        // For other instructions, we need an operand
        let operand = operand_opt.ok_or_else(|| AssembleError::InvalidSyntax("Missing operand".to_string()))?;

        // Check if this is a label reference
        if let Some(labels_map) = labels {
            // First check if this is a known label
            if labels_map.contains_key(&operand) {
                // It's a verified label reference - use Absolute addressing
                return self.decoder.lookup(instruction, AddressingMode::Absolute).map_err(|_| {
                    AssembleError::InvalidAddressingMode(format!(
                        "{instruction} does not support absolute mode for labels"
                    ))
                });
            }
        }

        // Handle standard addressing modes
        let (addressing_mode, _) = self.parse_addressing_mode(&operand)?;

        self.decoder.lookup(instruction, addressing_mode).map_err(|_| {
            AssembleError::InvalidAddressingMode(format!("{instruction} does not support {addressing_mode:?}"))
        })
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

    /// Calculates the size of an instruction in bytes
    fn calculate_instruction_size(&self, line: &str, labels: Option<&HashMap<String, u16>>) -> ParseResult<u16> {
        let (mnemonic, operand_opt) = self.split_instruction(line)?;

        // Parse the instruction mnemonic
        let instruction = mnemonic
            .parse::<Instruction>()
            .map_err(|_| AssembleError::UnknownMnemonic(mnemonic))?;

        // Implied addressing mode (just the opcode)
        if self.is_implied_addressing(instruction) {
            return Ok(1);
        }

        // For other instructions, we need an operand
        let operand = operand_opt.ok_or_else(|| AssembleError::InvalidSyntax("Missing operand".to_string()))?;

        // For potential label references, assume Absolute addressing (3 bytes)
        if self.is_label_reference(&operand) {
            return Ok(3);
        }

        // Regular instruction, get its size from metadata
        let metadata = self.parse_instruction(line, labels)?;
        Ok(metadata.addressing_mode.size())
    }

    /// Process a single line of assembly, extract labels and clean code
    fn process_line(&self, line: &str) -> (String, Option<String>) {
        // Remove comments and trim whitespace
        let line = match line.find(';') {
            Some(idx) => &line[0..idx],
            None => line,
        }
        .trim();

        // Check if this line is a label declaration
        if let Some(idx) = line.find(':') {
            let label = line[0..idx].trim().to_string();

            // If there's code after the label, return it as well
            let remainder = line[idx + 1..].trim();
            if !remainder.is_empty() {
                return (label, Some(remainder.to_string()));
            }
            return (label, None);
        }

        // No label, just a line of code
        if !line.is_empty() {
            return (String::new(), Some(line.to_string()));
        }

        // Empty line
        (String::new(), None)
    }

    /// Assembles an instruction string into bytes
    /// If labels map is provided, label references in operands will be resolved
    pub fn assemble_instruction(&self, input: &str, labels: Option<&HashMap<String, u16>>) -> ParseResult<Vec<u8>> {
        // Split input into mnemonic and operand
        let (mnemonic, operand_opt) = self.split_instruction(input)?;

        // Parse the instruction mnemonic
        let instruction = mnemonic
            .parse::<Instruction>()
            .map_err(|_| AssembleError::UnknownMnemonic(mnemonic))?;

        // Handle implied addressing mode (no operand)
        if self.is_implied_addressing(instruction) {
            let metadata = self.handle_implied_instruction(instruction)?;
            return Ok(vec![metadata.opcode]);
        }

        // For other instructions, we need an operand
        let operand = operand_opt.ok_or_else(|| AssembleError::InvalidSyntax("Missing operand".to_string()))?;

        // Check if this is a label reference
        if let Some(labels_map) = labels {
            // Try to handle as a label reference first
            if let Some((metadata, address)) = self.handle_label_reference(instruction, &operand, labels_map)? {
                // Return opcode + 16-bit address (little endian)
                return Ok(vec![
                    metadata.opcode,
                    (address & 0xFF) as u8, // Low byte first
                    (address >> 8) as u8,   // High byte second
                ]);
            }
            // For potential forward references or invalid labels
            else if self.is_label_reference(&operand) {
                // During assembly of a complete program, all labels should be in the map at this point
                // This check should fail only when directly calling assemble_instruction with a label
                // that doesn't exist in the provided map
                return Err(AssembleError::LabelError(format!("Undefined label: {}", operand)));
            }
        }

        // Handle standard addressing modes
        let (addressing_mode, operand_value) = self.parse_addressing_mode(&operand)?;

        let metadata = self.decoder.lookup(instruction, addressing_mode).map_err(|_| {
            AssembleError::InvalidAddressingMode(format!("{instruction} does not support {addressing_mode:?}"))
        })?;

        self.encode_instruction(metadata.opcode, addressing_mode, operand_value)
    }

    /// Encodes an instruction with its operand bytes based on addressing mode
    fn encode_instruction(
        &self,
        opcode: u8,
        addressing_mode: AddressingMode,
        operand_value: u16,
    ) -> ParseResult<Vec<u8>> {
        let mut bytes = vec![opcode];

        match addressing_mode {
            AddressingMode::Immediate | AddressingMode::ZeroPage => {
                bytes.push(operand_value as u8);
            },
            AddressingMode::Absolute => {
                bytes.push((operand_value & 0xFF) as u8);
                bytes.push((operand_value >> 8) as u8);
            },
            AddressingMode::Implied => {}, // No operand bytes
            _ => {
                return Err(AssembleError::InvalidAddressingMode(format!(
                    "Unsupported addressing mode: {:?}",
                    addressing_mode
                )))
            },
        }

        Ok(bytes)
    }

    /// Collects labels and processed instructions from a program
    fn collect_labels_and_instructions(
        &self,
        program: &str,
    ) -> ParseResult<(HashMap<String, u16>, Vec<(String, u16)>)> {
        let mut labels = HashMap::new();
        let mut current_address: u16 = self.load_address;
        let mut processed_lines = Vec::new();

        // Process each line, collecting labels and cleaned instruction lines
        for line in program.lines() {
            let (label, code_opt) = self.process_line(line);

            // If we found a label, record it
            if !label.is_empty() {
                // Check for duplicate labels
                if labels.contains_key(&label) {
                    return Err(AssembleError::LabelError(format!("Duplicate label: {}", label)));
                }

                // Record the label's position
                labels.insert(label, current_address);
            }

            // If we have code to process
            if let Some(code) = code_opt {
                processed_lines.push((code.clone(), current_address));

                // Calculate instruction size to update current_address
                current_address += self.calculate_instruction_size(&code, None)?;
            }
        }

        Ok((labels, processed_lines))
    }

    /// Assembles a multi-line program, handling comments, empty lines, and labels
    ///
    /// This method processes a complete program with multiple instructions.
    /// It ignores:
    /// - Empty lines
    /// - Comments (lines starting with ';')
    /// - Inline comments (text after ';' on a line)
    ///
    /// Returns the assembled bytes for all valid instructions.
    pub fn assemble_program(&self, program: &str) -> ParseResult<Vec<u8>> {
        // First pass: collect all labels and calculate instruction sizes
        let (labels, processed_lines) = self.collect_labels_and_instructions(program)?;

        // Second pass: assemble instructions with resolved labels
        let mut result = Vec::new();
        for (line, _) in processed_lines {
            // Now that we have all labels collected, we can assemble each instruction
            let bytes = self.assemble_instruction(&line, Some(&labels))?;
            result.extend(bytes);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

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
        let parser = Assembler::new(0);

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
        let parser = Assembler::new(0);

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
        let parser = Assembler::new(0);

        // Test LDA immediate
        let metadata = parser.parse_instruction("LDA #$42", None)?;
        assert_eq!(metadata.instruction, Instruction::LDA);
        assert_eq!(metadata.addressing_mode, AddressingMode::Immediate);
        assert_eq!(metadata.opcode, 0xA9);

        // Test LDA zero page
        let metadata = parser.parse_instruction("LDA $42", None)?;
        assert_eq!(metadata.instruction, Instruction::LDA);
        assert_eq!(metadata.addressing_mode, AddressingMode::ZeroPage);
        assert_eq!(metadata.opcode, 0xA5);

        // Test LDA absolute
        let metadata = parser.parse_instruction("LDA $1234", None)?;
        assert_eq!(metadata.instruction, Instruction::LDA);
        assert_eq!(metadata.addressing_mode, AddressingMode::Absolute);
        assert_eq!(metadata.opcode, 0xAD);

        Ok(())
    }

    /// Tests for error conditions
    #[test]
    fn test_error_conditions() -> Result<()> {
        let parser = Assembler::new(0);

        // Test invalid mnemonic
        let result = parser.parse_instruction("XYZ #$42", None);
        assert!(result.is_err());

        // Test invalid addressing mode syntax
        let result = parser.parse_instruction("LDA xyz", None);
        assert!(result.is_err());

        // Test invalid operand value
        let result = parser.parse_instruction("LDA #$ZZ", None);
        assert!(result.is_err());

        // Test missing operand
        let result = parser.parse_instruction("LDA", None);
        assert!(result.is_err());

        Ok(())
    }

    /// Tests for label declaration and usage
    #[test]
    fn test_label_declaration() -> Result<()> {
        let assembler = Assembler::new(0x0600);

        // Test simple label declaration and usage
        let program = r#"
            start:
            LDA #$42
            JMP start
        "#;

        let bytes = assembler.assemble_program(program)?;
        // JMP absolute is 0x4C, and should point to address 0x0600 (start of program)
        assert_eq!(bytes, vec![0xA9, 0x42, 0x4C, 0x00, 0x06]);

        Ok(())
    }

    /// Tests for forward reference of labels (used before defined)
    #[test]
    fn test_forward_reference() -> Result<()> {
        let assembler = Assembler::new(0x0600);

        // Test forward reference
        let program = r#"
            JMP end    ; Jump to label defined later
            LDA #$42
        end:
            NOP
        "#;

        let bytes = assembler.assemble_program(program)?;
        // JMP should point to address 0x0605 (where NOP is)
        assert_eq!(bytes, vec![0x4C, 0x05, 0x06, 0xA9, 0x42, 0xEA]); // 0xEA is NOP

        Ok(())
    }

    /// Tests for multiple labels in a program
    #[test]
    fn test_multiple_labels() -> Result<()> {
        let assembler = Assembler::new(0x0600);

        let program = r#"
        start:
            LDA #$10
        middle:
            LDA #$20
        end:
            LDA #$30
            JMP start
            JMP middle
        "#;

        let bytes = assembler.assemble_program(program)?;
        // First three LDAs, then JMP to 0x0600, then JMP to 0x0602
        assert_eq!(
            bytes,
            vec![
                0xA9, 0x10, // LDA #$10 at start
                0xA9, 0x20, // LDA #$20 at middle
                0xA9, 0x30, // LDA #$30 at end
                0x4C, 0x00, 0x06, // JMP to start (address 0x0600)
                0x4C, 0x02, 0x06 // JMP to middle (address 0x0602)
            ]
        );

        Ok(())
    }

    /// Tests for error conditions with labels
    #[test]
    fn test_label_errors() -> Result<()> {
        let assembler = Assembler::new(0);

        // Test undefined label
        let program = "JMP nonexistent";
        assert!(assembler.assemble_program(program).is_err());

        // Test duplicate label definition
        let program = r#"
        start:
            LDA #$10
        start:    ; Duplicate label
            LDA #$20
        "#;
        assert!(assembler.assemble_program(program).is_err());

        Ok(())
    }

    /// Test correct handling of labels with comments and whitespace
    #[test]
    fn test_label_formatting() -> Result<()> {
        let assembler = Assembler::new(0x0600);

        let program = r#"
            ; Comment before label
        start:  ; Comment after label
            LDA #$42
            
            JMP start ; With comment
        "#;

        let bytes = assembler.assemble_program(program)?;
        assert_eq!(bytes, vec![0xA9, 0x42, 0x4C, 0x00, 0x06]);

        Ok(())
    }
}
