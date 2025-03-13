use std::collections::HashMap;

use thiserror::Error;

use super::{AddressingMode, Instruction, InstructionDecoder, InstructionMetadata};
use crate::errors::NesError;
use crate::helpers::errors::ParseError;
use crate::helpers::parse::parse_value;

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

    #[error("Directive error: {0}")]
    DirectiveError(String),

    #[error("Segment error: {0}")]
    SegmentError(String),

    #[error("NES error: {0}")]
    NesError(#[from] NesError),

    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
}

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, AssembleError>;

/// Represents an assembler directive like .segment
#[derive(Debug, Clone)]
enum Directive {
    Segment(String),
    Byte(Vec<u8>),
    Word(Vec<u16>),
    Res(u16, u8), // Size, fill value (defaults to 0)
}

/// Parses assembly language instructions into their binary representation
pub struct Assembler {
    decoder: InstructionDecoder,
    pub load_address: u16,
    segments: HashMap<String, (u16, Vec<u8>)>, // Maps segment name to (load_address, bytes)
    current_segment: Option<String>,
}

impl Assembler {
    /// Creates a new instruction parser
    pub fn new(load_address: u16) -> Self {
        Self {
            decoder: InstructionDecoder::new(),
            load_address,
            segments: HashMap::new(),
            current_segment: None,
        }
    }

    /// Configure default NES ROM segments
    pub fn with_nes_segments(mut self) -> Self {
        self.add_segment("HEADER", 0x0000); // iNES header at the start
        self.add_segment("STARTUP", 0x8000); // PRG code starting at $8000 (32KB ROM)
        self.add_segment("VECTORS", 0xFFFA); // 6502 vectors at $FFFA-$FFFF
        self.add_segment("CHARS", 0x0000); // CHR data starts after PRG data
        self
    }

    /// Add a segment with the specified name and load address
    pub fn add_segment(&mut self, name: &str, load_address: u16) {
        self.segments.insert(name.to_string(), (load_address, Vec::new()));
    }

    /// Splits an instruction string into mnemonic and operand parts
    fn split_instruction(&self, input: &str) -> ParseResult<(String, Option<String>)> {
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
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

    /// Parse a segment directive
    fn parse_segment_directive(&self, args: &str) -> ParseResult<Directive> {
        if args.is_empty() {
            return Err(AssembleError::DirectiveError("Missing segment name".to_string()));
        }

        // Get segment name, remove quotes if present
        let segment_name = args.trim().trim_matches('"').trim_matches('\'').to_string();

        if !self.segments.contains_key(&segment_name) {
            return Err(AssembleError::SegmentError(format!(
                "Unknown segment: {}",
                segment_name
            )));
        }

        Ok(Directive::Segment(segment_name))
    }

    /// Parse a byte directive
    fn parse_byte_directive(&self, args: &str) -> ParseResult<Directive> {
        if args.is_empty() {
            return Err(AssembleError::DirectiveError("Missing byte values".to_string()));
        }

        // Parse bytes
        let bytes = self.parse_comma_separated_values::<u8>(args)?;
        Ok(Directive::Byte(bytes))
    }

    /// Parse a word directive
    fn parse_word_directive(&self, args: &str) -> ParseResult<Directive> {
        if args.is_empty() {
            return Err(AssembleError::DirectiveError("Missing word values".to_string()));
        }

        // Parse words
        let words = self.parse_comma_separated_values::<u16>(args)?;
        Ok(Directive::Word(words))
    }

    /// Parse a res directive
    fn parse_res_directive(&self, args: &str) -> ParseResult<Directive> {
        if args.is_empty() {
            return Err(AssembleError::DirectiveError("Missing size parameter".to_string()));
        }

        // Parse parameters (up to two: size and optional fill value)
        let params: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
        if params.is_empty() {
            return Err(AssembleError::DirectiveError("Missing size parameter".to_string()));
        }

        // Parse size (required)
        let size = parse_value::<u16>(params[0])?;

        // Parse fill value (optional, defaults to 0)
        let fill = if params.len() > 1 {
            parse_value::<u8>(params[1])?
        } else {
            0 // Default fill value is 0
        };

        Ok(Directive::Res(size, fill))
    }

    /// Parse a directive without applying any side effects
    fn parse_directive(&self, line: &str) -> ParseResult<Option<Directive>> {
        if !line.starts_with('.') {
            return Ok(None);
        }

        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.is_empty() {
            return Err(AssembleError::InvalidSyntax("Empty directive".to_string()));
        }

        // Extract directive arguments (if any)
        let args = if parts.len() > 1 { parts[1] } else { "" };

        // Match directive type and call appropriate handler
        match parts[0] {
            ".segment" => self.parse_segment_directive(args).map(Some),
            ".byte" => self.parse_byte_directive(args).map(Some),
            ".word" => self.parse_word_directive(args).map(Some),
            ".res" => self.parse_res_directive(args).map(Some),
            _ => Err(AssembleError::DirectiveError(format!(
                "Unknown directive: {}",
                parts[0]
            ))),
        }
    }

    /// Get the active segment for applying directives,
    /// using the current segment if available, otherwise falling back to the first segment
    fn get_active_segment(&mut self) -> ParseResult<&mut Vec<u8>> {
        // First determine which segment to use
        let segment_name = if let Some(name) = &self.current_segment {
            // Use current segment if available
            name.clone()
        } else if let Some(first_name) = self.segments.keys().next() {
            // Fallback to first segment if no active segment (for backwards compatibility)
            first_name.clone()
        } else {
            // No segments defined
            return Err(AssembleError::SegmentError("No segments defined, cannot apply directive".to_string()));
        };
        
        // Get and return the segment data
        if let Some(segment) = self.segments.get_mut(&segment_name) {
            Ok(&mut segment.1)
        } else {
            // This would be unusual but possible if segment was removed after being selected
            Err(AssembleError::SegmentError(format!("Selected segment '{}' not found", segment_name)))
        }
    }

    /// Apply the effects of a directive
    fn apply_directive(&mut self, directive: &Directive) -> ParseResult<()> {
        match directive {
            Directive::Segment(name) => {
                self.current_segment = Some(name.clone());
                Ok(())
            },
            Directive::Byte(bytes) => {
                // Add bytes to the current segment
                let segment_data = self.get_active_segment()?;
                segment_data.extend_from_slice(bytes);
                Ok(())
            },
            Directive::Word(words) => {
                // Add words as bytes (little-endian) to the current segment
                let segment_data = self.get_active_segment()?;
                for &word in words {
                    segment_data.push((word & 0xFF) as u8);         // Low byte
                    segment_data.push(((word >> 8) & 0xFF) as u8);  // High byte
                }
                Ok(())
            },
            Directive::Res(size, fill) => {
                // Add reserved bytes to the current segment
                let segment_data = self.get_active_segment()?;
                segment_data.resize(segment_data.len() + *size as usize, *fill);
                Ok(())
            },
        }
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
        if operand.starts_with('#') {
            let value = parse_value::<u8>(&operand)?;
            return Ok((AddressingMode::Immediate, value as u16));
        }

        // Zero Page: $xx (where xx is 00-FF)
        if operand.starts_with('$') && operand.len() == 3 {
            let value = parse_value::<u8>(operand)?;
            return Ok((AddressingMode::ZeroPage, value as u16));
        }

        // Absolute: $xxxx (where xxxx is 0000-FFFF)
        if operand.starts_with('$') && operand.len() == 5 {
            let value = parse_value::<u16>(operand)?;
            return Ok((AddressingMode::Absolute, value));
        }

        Err(AssembleError::InvalidAddressingMode(operand.to_string()))
    }

    /// Calculates the size of an instruction in bytes, assuming directive check has already been done
    fn calculate_instruction_size_internal(
        &self,
        line: &str,
        labels: Option<&HashMap<String, u16>>,
    ) -> ParseResult<u16> {
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

    /// Process a single line of assembly, extract labels and code, assuming directive check has already been done
    fn process_line_internal(&mut self, line: &str) -> ParseResult<(String, Option<String>)> {
        // Check if this line is a label declaration
        if let Some(idx) = line.find(':') {
            let label = line[0..idx].trim().to_string();

            // If there's code after the label, return it as well
            let remainder = line[idx + 1..].trim();
            if !remainder.is_empty() {
                return Ok((label, Some(remainder.to_string())));
            }
            return Ok((label, None));
        }

        // No label, just a line of code
        Ok((String::new(), Some(line.to_string())))
    }

    /// Process a single line of assembly, extract labels, code and handle directives
    fn process_line(&mut self, line: &str) -> ParseResult<(String, Option<String>)> {

        // Check if it's a directive
        if let Some(_) = self.parse_directive(&line)? {
            return Ok((String::new(), None)); // Directive handled, no code to assemble
        }

        // Not a directive, process as label or code
        self.process_line_internal(&line)
    }

    /// Collects labels and processed instructions from a program
    fn collect_labels_and_instructions(
        &mut self,
        program: &str,
    ) -> ParseResult<(HashMap<String, u16>, Vec<(String, u16)>)> {
        let mut labels = HashMap::new();
        let mut processed_lines = Vec::new();

        // Initialize with default load address
        let mut current_address = self.load_address;

        // Process each line, collecting labels and cleaned instruction lines
        for line in program.lines() {
            // Clean the line - removing comments and trimming whitespace
            let Some(line) = self.clean_line(line) else {
                continue;
            };

            // Check if it's a directive first
            if let Some(directive) = self.parse_directive(&line)? {
                match &directive {
                    Directive::Segment(name) => {
                        // When switching segments, update the current address
                        if let Some(segment) = self.segments.get(name) {
                            current_address = segment.0;
                        }
                    },
                    Directive::Byte(bytes) => {
                        // Add the length of bytes to current_address, checking for overflow
                        current_address = current_address.saturating_add(bytes.len() as u16);
                    },
                    Directive::Word(words) => {
                        // Each word is 2 bytes, check for overflow
                        let bytes_len = words.len().saturating_mul(2);
                        current_address = current_address.saturating_add(bytes_len as u16);
                    },
                    Directive::Res(size, _) => {
                        // Add the size of reserved bytes to current_address, checking for overflow
                        current_address = current_address.saturating_add(*size);
                    },
                }
                continue;
            }

            // Process the line to get label and code (not a directive at this point)
            let (label, code_opt) = self.process_line_internal(&line)?;

            // If we found a label, record it with current address
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

                // Calculate instruction size to update current_address (not a directive at this point)
                current_address += self.calculate_instruction_size_internal(&code, None)?;
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
    /// Returns assembled bytes for each segment.
    pub fn assemble_program(&mut self, program: &str) -> ParseResult<HashMap<String, Vec<u8>>> {
        // If no segments are defined, add a default "STARTUP" segment for backward compatibility
        if self.segments.is_empty() {
            self.add_segment("STARTUP", self.load_address);
        }

        // First pass: collect all labels and calculate instruction sizes
        let (labels, _processed_lines) = self.collect_labels_and_instructions(program)?;

        // Reset segment processing state
        self.current_segment = None;
        for segment in self.segments.values_mut() {
            segment.1.clear();
        }

        // Second pass: assemble instructions with resolved labels
        for line in program.lines() {
            // Clean the line - removing comments and trimming whitespace
            let Some(line) = self.clean_line(line) else {
                continue;
            };

            // Handle directives (especially .segment) first
            if let Some(directive) = self.parse_directive(&line)? {
                self.apply_directive(&directive)?;
                continue;
            }

            // Process the line to get label and code
            let (_label, code_opt) = self.process_line(&line)?;

            // Skip if no code to assemble
            if code_opt.is_none() {
                continue;
            }

            // Assemble the instruction
            let bytes = self.assemble_instruction(&code_opt.unwrap(), Some(&labels))?;

            // Add the assembled bytes to the current segment
            if let Some(segment_name) = &self.current_segment {
                if let Some(segment) = self.segments.get_mut(segment_name) {
                    segment.1.extend_from_slice(&bytes);
                }
            } else if let Some(first_segment) = self.segments.keys().cloned().next() {
                // If no segment is active, use the first defined segment as default
                if let Some(segment) = self.segments.get_mut(&first_segment) {
                    segment.1.extend_from_slice(&bytes);
                }
            }
        }

        // Create result map with segment bytes
        let mut result = HashMap::new();
        for (name, (_, bytes)) in &self.segments {
            // Include all segments, even empty ones
            result.insert(name.clone(), bytes.clone());
        }

        Ok(result)
    }

    /// Creates a complete NES ROM from the assembled segments
    pub fn create_nes_rom(&self) -> ParseResult<Vec<u8>> {
        // This is a basic implementation - will need enhancement for proper ROM generation
        let mut rom = Vec::new();

        // Add header if present
        if let Some(header) = self.segments.get("HEADER") {
            rom.extend_from_slice(&header.1);
        } else {
            return Err(AssembleError::SegmentError("Missing HEADER segment".to_string()));
        }

        // Add PRG ROM data
        if let Some(startup) = self.segments.get("STARTUP") {
            rom.extend_from_slice(&startup.1);
        }

        // Add vectors if not included in PRG ROM
        if let Some(vectors) = self.segments.get("VECTORS") {
            // Check if vectors need padding to reach the end of PRG ROM
            let prg_size = 16384; // 16KB
            let current_prg_size = rom.len() - 16; // Subtract header size

            if current_prg_size < prg_size - vectors.1.len() {
                // Pad to reach the vectors position
                rom.resize(16 + prg_size - vectors.1.len(), 0);
            }

            rom.extend_from_slice(&vectors.1);
        }

        // Add CHR ROM data
        if let Some(chars) = self.segments.get("CHARS") {
            rom.extend_from_slice(&chars.1);
        }

        Ok(rom)
    }

    /// Assembles an instruction string into bytes
    /// If labels map is provided, label references in operands will be resolved
    pub fn assemble_instruction(&mut self, input: &str, labels: Option<&HashMap<String, u16>>) -> ParseResult<Vec<u8>> {
        // Check if this is a directive
        if let Some(directive) = self.parse_directive(&input)? {
            // Apply the directive
            self.apply_directive(&directive)?;
            // For now, directives don't directly produce bytes
            return Ok(Vec::new());
        }

        // Split input into mnemonic and operand
        let (mnemonic, operand_opt) = self.split_instruction(&input)?;

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

    /// Clean a line by removing comments and excess whitespace
    fn clean_line(&self, line: &str) -> Option<String> {
        // First find and remove comments
        let line = match line.find(';') {
            Some(idx) => &line[0..idx],
            None => line,
        };

        // Then trim whitespace
        let line = line.trim();
        if line.is_empty() {
            None
        } else {
            Some(line.to_string())
        }
    }

    /// Parse comma-separated values into a vector of the specified numeric type
    fn parse_comma_separated_values<T>(&self, values_str: &str) -> ParseResult<Vec<T>> 
    where 
        T: Copy + crate::helpers::parse::ParseOperand, // Add the required trait
    {
        // Split values by commas and parse each one
        let values_str = values_str.split(',').map(|s| s.trim());
        let mut values = Vec::new();

        for value_str in values_str {
            // Skip empty strings (can happen with trailing commas)
            if value_str.is_empty() {
                continue;
            }

            // Parse using our helper
            let value = parse_value::<T>(value_str)?;
            values.push(value);
        }

        Ok(values)
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
        let mut assembler = Assembler::new(0x0600);

        // Test simple label declaration and usage
        let program = r#"
            start:
            LDA #$42
            JMP start
        "#;

        let segments = assembler.assemble_program(program)?;
        let bytes = segments.get("STARTUP").expect("STARTUP segment missing");
        // JMP absolute is 0x4C, and should point to address 0x0600 (start of program)
        assert_eq!(bytes, &vec![0xA9, 0x42, 0x4C, 0x00, 0x06]);

        Ok(())
    }

    /// Tests for forward reference of labels (used before defined)
    #[test]
    fn test_forward_reference() -> Result<()> {
        let mut assembler = Assembler::new(0x0600);

        // Test forward reference
        let program = r#"
            JMP end    ; Jump to label defined later
            LDA #$42
        end:
            NOP
        "#;

        let segments = assembler.assemble_program(program)?;
        let bytes = segments.get("STARTUP").expect("STARTUP segment missing");
        // JMP should point to address 0x0605 (where NOP is)
        assert_eq!(bytes, &vec![0x4C, 0x05, 0x06, 0xA9, 0x42, 0xEA]); // 0xEA is NOP

        Ok(())
    }

    /// Tests for multiple labels in a program
    #[test]
    fn test_multiple_labels() -> Result<()> {
        let mut assembler = Assembler::new(0x0600);

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

        let segments = assembler.assemble_program(program)?;
        let bytes = segments.get("STARTUP").expect("STARTUP segment missing");
        // First three LDAs, then JMP to 0x0600, then JMP to 0x0602
        assert_eq!(
            bytes,
            &vec![
                0xA9, 0x10, // LDA #$10 at start
                0xA9, 0x20, // LDA #$20 at middle
                0xA9, 0x30, // LDA #$30 at end
                0x4C, 0x00, 0x06, // JMP to start (address 0x0600)
                0x4C, 0x02, 0x06 // JMP to middle (address 0x0602)
            ]
        );

        Ok(())
    }

    /// Test the .segment directive
    #[test]
    fn test_segment_directive() -> Result<()> {
        let mut assembler = Assembler::new(0);

        // Add test segments
        assembler.add_segment("CODE", 0x8000);
        assembler.add_segment("DATA", 0xC000);

        let program = r#"
        .segment "CODE"
        start:
            LDA #$10
            
        .segment "DATA"
        data:
            ; Should be at $C000
            
        .segment "CODE"
            JMP start
        "#;

        let segments = assembler.assemble_program(program)?;

        // Verify CODE segment contains our code at the proper address
        let code_segment = segments.get("CODE").expect("CODE segment missing");
        assert_eq!(
            code_segment,
            &vec![
                0xA9, 0x10, // LDA #$10
                0x4C, 0x00, 0x80 // JMP to start ($8000)
            ]
        );

        // Verify DATA segment exists but is empty in this case
        let data_segment = segments.get("DATA").expect("DATA segment missing");
        assert!(data_segment.is_empty());

        Ok(())
    }

    /// Tests for error conditions with labels
    #[test]
    fn test_label_errors() -> Result<()> {
        let mut assembler = Assembler::new(0);

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
        let mut assembler = Assembler::new(0x0600);

        let program = r#"
            ; Comment before label
        start:  ; Comment after label
            LDA #$42
            
            JMP start ; With comment
        "#;

        let segments = assembler.assemble_program(program)?;
        let bytes = segments.get("STARTUP").expect("STARTUP segment missing");
        assert_eq!(bytes, &vec![0xA9, 0x42, 0x4C, 0x00, 0x06]);

        Ok(())
    }

    /// Test the new directive parsing and application separation
    #[test]
    fn test_directive_handling_separation() -> Result<()> {
        let mut assembler = Assembler::new(0);

        // Add test segments
        assembler.add_segment("CODE", 0x8000);
        assembler.add_segment("DATA", 0xC000);

        // Parse a directive without applying it
        let directive = assembler.parse_directive(".segment \"CODE\"")?;
        assert!(directive.is_some());
        if let Some(directive) = directive {
            match directive {
                Directive::Segment(name) => {
                    assert_eq!(name, "CODE");
                    // Current segment should still be None at this point
                    assert!(assembler.current_segment.is_none());

                    // Now apply the directive
                    assembler.apply_directive(&Directive::Segment(name))?;
                    // Current segment should now be set
                    assert_eq!(assembler.current_segment, Some("CODE".to_string()));
                },
                Directive::Byte(_) => panic!("Expected Segment directive, got Byte"),
                Directive::Word(_) => panic!("Expected Segment directive, got Word"),
                Directive::Res(_, _) => panic!("Expected Segment directive, got Res"),
            }
        } else {
            panic!("Expected Segment directive");
        }

        Ok(())
    }

    /// Test the .byte and .word directives
    #[test]
    fn test_data_directives() -> Result<()> {
        let mut assembler = Assembler::new(0);

        // Add test segments
        assembler.add_segment("CODE", 0x8000);
        assembler.add_segment("DATA", 0xC000);

        let program = r#"
        .segment "CODE"
            LDA #$10
            
        .segment "DATA"
            ; Define some bytes
            .byte $01, $02, $03
            
            ; Define some words (16-bit values)
            .word $1234, $5678
        "#;

        let segments = assembler.assemble_program(program)?;

        // Verify CODE segment with LDA instruction
        let code_segment = segments.get("CODE").expect("CODE segment missing");
        assert_eq!(code_segment, &vec![0xA9, 0x10]);

        // Verify DATA segment with .byte and .word values
        let data_segment = segments.get("DATA").expect("DATA segment missing");
        assert_eq!(
            data_segment,
            &vec![
                // .byte values
                0x01, 0x02, 0x03, // .word values (little-endian)
                0x34, 0x12, // $1234 stored as 34 12
                0x78, 0x56 // $5678 stored as 78 56
            ]
        );

        Ok(())
    }

    /// Test the .res directive for reserving space
    #[test]
    fn test_res_directive() -> Result<()> {
        let mut assembler = Assembler::new(0);

        // Add test segments
        assembler.add_segment("CODE", 0x8000);
        assembler.add_segment("DATA", 0xC000);

        let program = r#"
        .segment "CODE"
            LDA #$10
            
        .segment "DATA"
            ; Define some bytes
            .byte $01, $02, $03
            
            ; Reserve 10 bytes initialized to 0
            .res 10
            
            ; Reserve 5 bytes initialized to $FF
            .res 5, $FF
            
            ; Define some words (16-bit values)
            .word $1234, $5678
        "#;

        let segments = assembler.assemble_program(program)?;

        // Verify CODE segment with LDA instruction
        let code_segment = segments.get("CODE").expect("CODE segment missing");
        assert_eq!(code_segment, &vec![0xA9, 0x10]);

        // Verify DATA segment with .byte, .res, and .word values
        let data_segment = segments.get("DATA").expect("DATA segment missing");

        // Create expected data segment:
        // 3 bytes from .byte directive
        let mut expected = vec![0x01, 0x02, 0x03];
        // 10 bytes of 0 from .res 10
        expected.extend(vec![0x00; 10]);
        // 5 bytes of 0xFF from .res 5, $FF
        expected.extend(vec![0xFF; 5]);
        // 4 bytes from .word directive (2 words)
        expected.extend(vec![0x34, 0x12, 0x78, 0x56]);

        assert_eq!(data_segment, &expected);

        Ok(())
    }
}
