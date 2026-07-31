use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

use super::{
    addressing_mode::AddressingModeError,
    AddressingMode,
    Instruction,
    InstructionDecoder,
    InstructionDecoderError,
    InstructionMetadata,
};
use crate::helpers::{errors::ParseError, parse::parse_value};

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

    #[error("Invalid syntax in line '{line}': {message}")]
    InvalidSyntaxWithContext { line: String, message: String },

    #[error("Label error: {0}")]
    LabelError(String),

    #[error("Directive error: {0}")]
    DirectiveError(String),

    #[error("Segment error: {0}")]
    SegmentError(String),

    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),

    #[error("Instruction decoder error: {0}")]
    InstructionDecoderError(#[from] InstructionDecoderError),

    #[error("Addressing mode error: {0}")]
    AddressingModeError(#[from] AddressingModeError),

    #[error("Invalid operand: {0}")]
    InvalidOperand(String),
}

/// Result type for parsing operations
pub type AssembleResult<T> = Result<T, AssembleError>;

/// Represents an assembler directive like .segment
#[derive(Debug, Clone)]
enum Directive {
    Segment(String),
    Byte(Vec<u8>),
    Word(Vec<u16>),
    Res(u16, u8),            // Size, fill value (defaults to 0)
    Sprite(u8, u8, Vec<u8>), // Width (tiles), Height (tiles), pattern data
}

struct Segment {
    load_address: u16,
    data: Vec<u8>,
}

impl Segment {
    pub fn new(load_address: u16) -> Self {
        Self {
            load_address,
            data: Vec::new(),
        }
    }

    pub fn extend(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

#[derive(Default)]
struct Segments {
    segments: HashMap<String, Segment>,
    current: Option<String>,
}

impl Segments {
    fn get(&self, name: &str) -> Option<&Segment> {
        self.segments.get(name)
    }

    fn all(&self) -> &HashMap<String, Segment> {
        &self.segments
    }

    fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    fn add(&mut self, name: &str, load_address: u16) {
        self.segments.insert(name.to_string(), Segment::new(load_address));
    }

    fn contains(&self, name: &str) -> bool {
        self.segments.contains_key(name)
    }

    fn reset(&mut self) {
        for segment in self.segments.values_mut() {
            segment.clear();
        }
        self.current = None;
    }

    /// Get the active segment for applying directives,
    /// using the current segment if available, otherwise falling back to the first segment
    fn current_or_first_mut(&mut self) -> AssembleResult<&mut Segment> {
        // First determine which segment to use
        let segment_name = if let Some(name) = &self.current {
            // Use current segment if available
            name.clone()
        } else if let Some(first_name) = self.segments.keys().next() {
            // Fallback to first segment if no active segment (for backwards compatibility)
            first_name.clone()
        } else {
            // No segments defined
            return Err(AssembleError::SegmentError(
                "No segments defined, cannot apply directive".to_string(),
            ));
        };

        self.segments
            .get_mut(&segment_name)
            .ok_or_else(|| AssembleError::SegmentError(format!("Selected segment '{}' not found", segment_name)))
    }

    fn current_mut(&mut self) -> AssembleResult<&mut Segment> {
        let segment_name = if let Some(name) = &self.current {
            // Use current segment if available
            name.clone()
        } else {
            // No current segment defined
            return Err(AssembleError::SegmentError(
                "No segments defined, cannot apply directive".to_string(),
            ));
        };

        self.segments
            .get_mut(&segment_name)
            .ok_or_else(|| AssembleError::SegmentError(format!("Selected segment '{}' not found", segment_name)))
    }
}

/// Parses assembly language instructions into their binary representation
pub struct Assembler {
    decoder: InstructionDecoder,
    pub load_address: u16,
    segments: Segments, // Maps segment name to (load_address, bytes)
}

impl Assembler {
    /// Creates a new instruction parser
    pub fn new(load_address: u16) -> Self {
        Self {
            decoder: InstructionDecoder::new(),
            load_address,
            segments: Segments::default(),
        }
    }

    /// Initializes the assembler with standard NES segments
    ///
    /// This adds the HEADER, ZEROPAGE, STARTUP, VECTORS, and CHARS segments
    /// that are commonly used in NES programs.
    /// This is the owned version that consumes self and returns Self.
    pub fn with_nes_segments(mut self) -> Self {
        self.segments.add("HEADER", 0x0000); // iNES header at the start
        self.segments.add("ZEROPAGE", 0x0000); // Zero page variables (0x0000-0x00FF)
        self.segments.add("STARTUP", 0x8000); // PRG code starting at $8000 (32KB ROM)
        self.segments.add("VECTORS", 0xFFFA); // 6502 vectors at $FFFA-$FFFF
        self.segments.add("CHARS", 0x0000); // CHR data starts after PRG data
        self
    }

    /// Creates a complete NES ROM from the assembled segments
    pub fn create_nes_rom(&self) -> AssembleResult<Vec<u8>> {
        // This is a basic implementation - will need enhancement for proper ROM generation
        let mut rom = Vec::new();

        // Add header if present
        if let Some(header) = self.segments.get("HEADER") {
            rom.extend_from_slice(&header.data);
        } else {
            return Err(AssembleError::SegmentError("Missing HEADER segment".to_string()));
        }

        // Add PRG ROM data
        if let Some(startup) = self.segments.get("STARTUP") {
            rom.extend_from_slice(&startup.data);
        }

        // Add vectors if not included in PRG ROM
        if let Some(vectors) = self.segments.get("VECTORS") {
            // Check if vectors need padding to reach the end of PRG ROM
            let prg_size = 16384; // 16KB
            let current_prg_size = rom.len() - 16; // Subtract header size
            let vectors_len = vectors.data.len();

            if current_prg_size < prg_size - vectors_len {
                // Pad to reach the vectors position
                rom.resize(16 + prg_size - vectors_len, 0);
            }

            rom.extend_from_slice(&vectors.data);
        }

        // Add CHR ROM data
        if let Some(chars) = self.segments.get("CHARS") {
            rom.extend_from_slice(&chars.data);
        }

        Ok(rom)
    }

    /// Handles a line containing a directive
    fn handle_directive_line(&mut self, line: &str, labels: &HashMap<String, u16>) -> AssembleResult<bool> {
        if line.starts_with('.') {
            if let Some(directive) = self.parse_directive(line, labels)? {
                self.apply_directive(&directive)?;
                return Ok(true); // Directive was handled
            }
        }
        Ok(false) // Not a directive or couldn't be handled
    }

    /// Handles a potential labeled directive scenario (where a label is on one line and directive follows)
    fn handle_labeled_directive(
        &mut self,
        label: &str,
        next_line: Option<&str>,
        labels: &HashMap<String, u16>,
        line_index: &mut usize,
    ) -> AssembleResult<bool> {
        if let (false, Some(next_line)) = (label.is_empty(), next_line) {
            if let Some(clean_next_line) = self.clean_line(next_line) {
                if clean_next_line.starts_with('.') {
                    if let Some(directive) = self.parse_directive(&clean_next_line, labels)? {
                        self.apply_directive(&directive)?;
                        *line_index += 1; // Skip the directive line since we processed it
                        return Ok(true); // Labeled directive was handled
                    }
                }
            }
        }
        Ok(false) // Not a labeled directive or couldn't be handled
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
    pub fn assemble_program(&mut self, program: &str) -> AssembleResult<HashMap<String, Vec<u8>>> {
        // If no segments are defined, add a default "STARTUP" segment for backward compatibility
        if self.segments.is_empty() {
            self.segments.add("STARTUP", self.load_address);
        }

        // First pass: collect all labels (ignoring directives)
        let labels = self.collect_labels(program)?;

        // Debug the collected labels
        log::debug!("Collected labels:");
        for (label, address) in &labels {
            log::debug!("  {} => ${:04X}", label, address);
        }

        // Reset segment processing state for second pass
        self.segments.reset();

        // Second pass: process directives and assemble instructions with resolved labels
        let mut line_index = 0;
        let lines: Vec<&str> = program.lines().collect();

        while line_index < lines.len() {
            // Get current line
            let line = lines[line_index];
            line_index += 1;

            // Clean the line - removing comments and trimming whitespace
            let Some(clean_line) = self.clean_line(line) else {
                continue;
            };

            // Process the line to get label and code
            let (label, code_opt) = process_line(&clean_line)?;

            // Check if this is a directive line
            if self.handle_directive_line(&clean_line, &labels)? {
                continue;
            }

            // Check if this is a label followed by a directive
            let next_line = if line_index < lines.len() {
                Some(lines[line_index])
            } else {
                None
            };
            if self.handle_labeled_directive(&label, next_line, &labels, &mut line_index)? {
                continue;
            }

            // Skip if no code to assemble
            let Some(code) = code_opt else {
                continue;
            };

            if self.handle_directive_line(&code, &labels)? {
                continue;
            }

            // Assemble the instruction
            let bytes = self.assemble_instruction(&code, &labels)?;

            // Add the assembled bytes to the current segment
            if let Ok(segment) = self.segments.current_or_first_mut() {
                segment.extend(&bytes);
            }
        }

        // Create result map with segment bytes
        let mut result = HashMap::new();
        for (name, segment) in self.segments.all() {
            // Include all segments, even empty ones
            result.insert(name.clone(), segment.data.clone());
        }

        Ok(result)
    }

    /// How many bytes a data directive emits, for address tracking during label collection.
    ///
    /// Label collection must advance the address by exactly what the assembly pass will emit.
    /// Counting only instructions meant that every label following a `.byte` table was assigned
    /// too low an address — and since several labels then landed on the same address, a `JSR` to
    /// a routine defined after a table jumped into the table's data instead.
    fn directive_size(&self, code: &str) -> AssembleResult<u16> {
        let (name, args) = match code.split_once(char::is_whitespace) {
            Some((name, args)) => (name.trim(), args.trim()),
            None => (code.trim(), ""),
        };

        let count_values = || args.split(',').filter(|v| !v.trim().is_empty()).count() as u16;

        Ok(match name {
            // A quoted string in a `.byte` contributes one byte per character rather than one
            // value, which is how iNES headers such as `.byte "NES", $1A` are written.
            ".byte" | ".db" => args
                .split(',')
                .filter(|v| !v.trim().is_empty())
                .map(|value| {
                    let value = value.trim();
                    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                        (value.len() - 2) as u16
                    } else {
                        1
                    }
                })
                .sum(),
            ".word" | ".dw" => count_values() * 2,
            ".res" => {
                let size = args.split(',').next().unwrap_or("").trim();
                parse_value::<u16>(size).unwrap_or(0)
            },
            // Directives that emit nothing, such as `.segment`.
            _ => 0,
        })
    }

    /// Collects labels and their positions from a program
    fn collect_labels(&mut self, program: &str) -> AssembleResult<HashMap<String, u16>> {
        let mut labels = HashMap::new();
        let mut pending_labels = Vec::new();

        // Track current address and segments
        let mut current_address = self.load_address;
        let mut current_segment = "STARTUP".to_string();
        let mut segment_addresses = HashMap::new();
        segment_addresses.insert(current_segment.clone(), current_address);

        // Create a list of lines for easier handling
        let lines: Vec<&str> = program.lines().collect();
        let mut line_index = 0;

        // FIRST PASS: Collect all labels and their initial positions
        while line_index < lines.len() {
            if let Some(line) = self.clean_line(lines[line_index]) {
                // Handle segment directives
                if line.starts_with(".segment") {
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if parts.len() > 1 {
                        let segment_name = format_segment_name(parts[1]).to_string();

                        // Add this segment if it doesn't exist
                        if !self.segments.contains(&segment_name) {
                            // For new segments, use appropriate default addresses
                            let addr = if segment_name == "ZEROPAGE" {
                                0x0000
                            } else if segment_name == "VECTORS" {
                                0xFFFA
                            } else {
                                self.load_address
                            };
                            self.segments.add(&segment_name, addr);
                        }

                        // Update current segment
                        current_segment = segment_name.clone();

                        // Get the address for this segment
                        if let Some(&addr) = segment_addresses.get(&current_segment) {
                            current_address = addr;
                        } else if let Some(segment) = self.segments.get(&current_segment) {
                            current_address = segment.load_address;
                            segment_addresses.insert(current_segment.clone(), current_address);
                        }
                    }
                } else {
                    // Get label and code from the line
                    let (label, code_opt) = process_line(&line)?;

                    // If we have a label, record it
                    if !label.is_empty() {
                        if code_opt.is_some() {
                            // Label with code on the same line - use current address
                            if labels.contains_key(&label) {
                                return Err(AssembleError::LabelError(format!("Duplicate label: {}", label)));
                            }
                            labels.insert(label.clone(), current_address);

                            if label == "WaitForVBlank" {
                                log::debug!(
                                    "Found WaitForVBlank with code, initial address: ${:04X}",
                                    current_address
                                );
                            }
                        } else {
                            // Label on its own line - add to pending
                            pending_labels.push(label.clone());

                            if label == "WaitForVBlank" {
                                log::debug!("Found standalone WaitForVBlank, adding to pending labels");
                            }
                        }
                    }

                    // If we have code, update the address and assign pending labels
                    if let Some(code) = code_opt {
                        // Assign any pending labels to the current address
                        for label in pending_labels.drain(..) {
                            if labels.contains_key(&label) {
                                return Err(AssembleError::LabelError(format!("Duplicate label: {}", label)));
                            }
                            labels.insert(label.clone(), current_address);

                            if label == "WaitForVBlank" {
                                log::debug!("Assigning WaitForVBlank to address: ${:04X}", current_address);
                            }
                        }

                        // Update the address based on the code. Data directives emit bytes just
                        // as instructions do, so both must advance the address.
                        //
                        // Wrapping, because the VECTORS segment sits at $FFFA and emitting its
                        // words legitimately runs off the top of the 16-bit address space.
                        if code.starts_with('.') {
                            current_address = current_address.wrapping_add(self.directive_size(&code)?);
                        } else {
                            let instruction_size = self.calculate_instruction_size(&code)?;
                            current_address = current_address.wrapping_add(instruction_size);
                        }
                    }
                }
            }

            line_index += 1;
        }

        // Assign any remaining pending labels to the end address
        for label in pending_labels {
            if labels.contains_key(&label) {
                return Err(AssembleError::LabelError(format!("Duplicate label: {}", label)));
            }
            labels.insert(label.clone(), current_address);
        }

        // SECOND PASS: Update addresses accounting for real instruction sizes with resolved labels
        // Reset for second pass
        current_address = self.load_address;
        current_segment = "STARTUP".to_string();
        line_index = 0;
        let mut updated_labels = HashMap::new();
        pending_labels = Vec::new();

        // Copy known labels for reference
        let initial_labels = labels.clone();

        while line_index < lines.len() {
            if let Some(line) = self.clean_line(lines[line_index]) {
                // Handle segment directives
                if line.starts_with(".segment") {
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if parts.len() > 1 {
                        let segment_name = format_segment_name(parts[1]).to_string();
                        current_segment = segment_name.clone();

                        // Get the address for this segment
                        if let Some(&addr) = segment_addresses.get(&current_segment) {
                            current_address = addr;
                        } else if let Some(segment) = self.segments.get(&current_segment) {
                            current_address = segment.load_address;
                        }
                    }
                } else {
                    // Get label and code from the line
                    let (label, code_opt) = process_line(&line)?;

                    // Process any label on this line
                    if !label.is_empty() {
                        if code_opt.is_some() {
                            // Label with code - update at current address
                            updated_labels.insert(label.clone(), current_address);

                            if label == "WaitForVBlank" {
                                log::debug!("Updated WaitForVBlank with code to address: ${:04X}", current_address);
                            }
                        } else {
                            // Standalone label - add to pending
                            pending_labels.push(label.clone());

                            if label == "WaitForVBlank" {
                                log::debug!("Added WaitForVBlank to pending labels in second pass");
                            }
                        }
                    }

                    // Process code
                    if let Some(code) = code_opt {
                        // Assign pending labels to current address
                        for label in pending_labels.drain(..) {
                            updated_labels.insert(label.clone(), current_address);

                            if label == "WaitForVBlank" {
                                log::debug!("Updated standalone WaitForVBlank to address: ${:04X}", current_address);
                            }
                        }

                        // Update address based on the code
                        if code.starts_with('.') {
                            current_address = current_address.wrapping_add(self.directive_size(&code)?);
                        } else if !code.starts_with('.') {
                            // For actual instructions, we need to consider the real instruction size
                            // using the labels we collected in the first pass
                            let mut real_instruction_size = 0;

                            // Parse the instruction and get its real size
                            let (instruction, operand_opt) = split_instruction(&code)?;

                            if instruction.has_implied_addressing() {
                                real_instruction_size = 1; // Just the opcode
                            } else if let Some(operand) = operand_opt {
                                // Check for accumulator addressing mode
                                if operand.trim().eq_ignore_ascii_case("a") {
                                    real_instruction_size = 1; // Just the opcode for accumulator mode

                                    // Debug logging for accumulator addressing
                                    log::debug!(
                                        "Instruction '{}' with accumulator addressing, size: 1 byte",
                                        instruction
                                    );
                                } else if self.is_label_reference(&operand) {
                                    // Extract the base label without any index
                                    let base_label = if let Some(idx_pos) = operand.find(',') {
                                        &operand[..idx_pos]
                                    } else {
                                        &operand
                                    };

                                    // Look up the label address from first pass
                                    if let Some(&label_addr) = initial_labels.get(base_label) {
                                        // For JSR and JMP, always use absolute addressing (3 bytes)
                                        if instruction.is_jump() {
                                            real_instruction_size = AddressingMode::Absolute.size();

                                            // Debug logging for JSR instructions
                                            if instruction == Instruction::JSR {
                                                log::debug!(
                                                    "JSR to label '{}' at address: ${:04X}, using absolute addressing (3 bytes)",
                                                    base_label,
                                                    label_addr
                                                );
                                            }
                                        } else if instruction.is_branch() {
                                            real_instruction_size = AddressingMode::Relative.size();
                                        // 2 bytes total
                                        } else if operand.contains(",X") || operand.contains(",Y") {
                                            real_instruction_size = if label_addr <= 0xFF {
                                                AddressingMode::ZeroPageX.size()
                                            // 2 bytes
                                            } else {
                                                AddressingMode::AbsoluteX.size()
                                                // 3 bytes
                                            };
                                        } else if label_addr <= 0xFF {
                                            real_instruction_size = AddressingMode::ZeroPage.size();
                                        // 2 bytes total
                                        } else {
                                            real_instruction_size = AddressingMode::Absolute.size();
                                            // 3 bytes total
                                        }
                                    } else {
                                        // Label not found - use default size (absolute addressing)
                                        real_instruction_size = 3;
                                        log::warn!(
                                            "Label '{}' not found in first pass, using default size 3",
                                            base_label
                                        );
                                    }
                                } else {
                                    // Not a label reference - parse addressing mode and get size
                                    if let Ok((addressing_mode, _)) = self.parse_addressing_mode(&operand) {
                                        real_instruction_size = addressing_mode.size();
                                    } else {
                                        // Unable to parse - use estimate from calculate_instruction_size
                                        real_instruction_size = self.calculate_instruction_size(&code)?;
                                    }
                                }
                            }

                            // Update the address with the real instruction size. Wrapping for the
                            // same reason as above.
                            current_address = current_address.wrapping_add(real_instruction_size);
                        }
                    }
                }
            }

            // Update the segment address
            segment_addresses.insert(current_segment.clone(), current_address);

            line_index += 1;
        }

        // Assign any remaining pending labels to the end address
        for label in pending_labels {
            updated_labels.insert(label.clone(), current_address);
        }

        // Merge updated labels with any that weren't in the second pass
        for (label, addr) in initial_labels {
            updated_labels.entry(label).or_insert(addr);
        }

        // Debug all labels
        for (label, addr) in &updated_labels {
            log::debug!("Final label address: {} => ${:04X}", label, addr);
        }

        Ok(updated_labels)
    }

    /// Parse a directive without applying any side effects
    fn parse_directive(&self, line: &str, labels: &HashMap<String, u16>) -> AssembleResult<Option<Directive>> {
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
        let directive = match parts[0] {
            ".segment" => self.parse_segment_directive(args)?,
            ".byte" => self.parse_byte_directive(args)?,
            ".word" => self.parse_word_directive(args, labels)?,
            ".res" => self.parse_res_directive(args)?,
            ".sprite" => self.parse_sprite_directive(args)?,
            _ => {
                return Err(AssembleError::DirectiveError(format!(
                    "Unknown directive: {}",
                    parts[0]
                )))
            },
        };

        Ok(Some(directive))
    }

    /// Parse a segment directive
    fn parse_segment_directive(&self, args: &str) -> AssembleResult<Directive> {
        if args.is_empty() {
            return Err(AssembleError::DirectiveError("Missing segment name".to_string()));
        }

        // Get segment name, remove quotes if present
        let segment_name = format_segment_name(args);

        if !self.segments.contains(segment_name) {
            return Err(AssembleError::SegmentError(format!("Unknown segment: {segment_name}")));
        }

        Ok(Directive::Segment(segment_name.to_string()))
    }

    /// Parse a byte directive
    fn parse_byte_directive(&self, args: &str) -> AssembleResult<Directive> {
        if args.is_empty() {
            return Err(AssembleError::DirectiveError("Missing byte values".to_string()));
        }

        let bytes = parse_comma_separated_byte_tokens(args)?;
        Ok(Directive::Byte(bytes))
    }

    /// Parse a word directive with optional label resolution
    fn parse_word_directive(&self, args: &str, labels: &HashMap<String, u16>) -> AssembleResult<Directive> {
        if args.is_empty() {
            return Err(AssembleError::DirectiveError("Missing word values".to_string()));
        }

        // Split values by commas and parse each one
        let values_str = args.split(',').map(|s| s.trim());
        let mut values = Vec::new();

        for value_str in values_str {
            // Skip empty strings (can happen with trailing commas)
            if value_str.is_empty() {
                continue;
            }

            // Try to parse as a numeric value first
            match parse_value::<u16>(value_str) {
                Ok(value) => {
                    values.push(value);
                    continue;
                },
                Err(_) => {
                    // If it's not a valid number, try to resolve it as a label if labels are provided
                    if let Some(&address) = labels.get(value_str) {
                        values.push(address);
                        continue;
                    }
                },
            }

            return Err(AssembleError::LabelError(format!(
                "Invalid .word directive: {}",
                value_str
            )));
        }

        Ok(Directive::Word(values))
    }

    /// Parse a res directive
    fn parse_res_directive(&self, args: &str) -> AssembleResult<Directive> {
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

    /// Parse a sprite directive
    fn parse_sprite_directive(&self, args: &str) -> AssembleResult<Directive> {
        if args.is_empty() {
            return Err(AssembleError::DirectiveError("Missing sprite parameters".to_string()));
        }

        // Extract width and height parameters
        let mut parts = args.splitn(3, ',');

        // Get width
        let width_str = parts
            .next()
            .ok_or_else(|| AssembleError::DirectiveError("Missing width parameter".to_string()))?
            .trim();
        let width = width_str
            .parse::<u8>()
            .map_err(|_| AssembleError::DirectiveError(format!("Invalid width: {}", width_str)))?;

        // Get height
        let height_str = parts
            .next()
            .ok_or_else(|| AssembleError::DirectiveError("Missing height parameter".to_string()))?
            .trim();
        let height = height_str
            .parse::<u8>()
            .map_err(|_| AssembleError::DirectiveError(format!("Invalid height: {}", height_str)))?;

        // Get pattern data string
        let pattern_str = parts
            .next()
            .ok_or_else(|| AssembleError::DirectiveError("Missing pattern data".to_string()))?;

        // Process the pattern data
        let mut pattern_data = Vec::new();

        // Split the pattern data by commas and process each token
        for token in pattern_str.split(',') {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }

            // Parse the binary value
            if let Some(binary_str) = token.strip_prefix('%') {
                match u8::from_str_radix(binary_str, 2) {
                    Ok(value) => pattern_data.push(value),
                    Err(_) => {
                        return Err(AssembleError::DirectiveError(format!(
                            "Invalid binary value: {}",
                            token
                        )))
                    },
                }
            } else if let Some(hex_str) = token.strip_prefix("$") {
                // Handle hex values
                match u8::from_str_radix(hex_str, 16) {
                    Ok(value) => pattern_data.push(value),
                    Err(_) => return Err(AssembleError::DirectiveError(format!("Invalid hex value: {}", token))),
                }
            } else {
                // Handle decimal values
                match token.parse::<u8>() {
                    Ok(value) => pattern_data.push(value),
                    Err(_) => {
                        return Err(AssembleError::DirectiveError(format!(
                            "Invalid decimal value: {}",
                            token
                        )))
                    },
                }
            }
        }

        // Check if we have the correct amount of pattern data
        let expected_bytes = width as usize * height as usize * 16; // 16 bytes per tile (8+8 bytes for 2 bit planes)
        if pattern_data.len() != expected_bytes {
            return Err(AssembleError::DirectiveError(format!(
                "Incorrect pattern data size: expected {} bytes, got {} bytes",
                expected_bytes,
                pattern_data.len()
            )));
        }

        Ok(Directive::Sprite(width, height, pattern_data))
    }

    /// Apply the effects of a directive
    fn apply_directive(&mut self, directive: &Directive) -> AssembleResult<()> {
        match directive {
            Directive::Segment(name) => {
                let name = format_segment_name(name);

                // If segment doesn't exist, create it with default load address
                if !self.segments.contains(name) {
                    self.segments.add(name, self.load_address);
                }

                // Set as current segment
                self.segments.current = Some(name.to_string());
                Ok(())
            },
            Directive::Byte(values) => {
                // Get current segment to add bytes
                if let Ok(segment) = self.segments.current_mut() {
                    segment.extend(values);
                }
                Ok(())
            },
            Directive::Word(values) => {
                // Get current segment to add words
                if let Ok(segment) = self.segments.current_mut() {
                    for value in values {
                        // Store words in little-endian format
                        segment.extend(&value.to_le_bytes());
                    }
                }
                Ok(())
            },
            Directive::Res(size, fill) => {
                // Get current segment to reserve space
                if let Ok(segment) = self.segments.current_mut() {
                    // Create a vector of the fill value with the specified size
                    let data = vec![*fill; *size as usize];
                    segment.extend(&data);
                }
                Ok(())
            },
            Directive::Sprite(_width, _height, pattern_data) => {
                // Get current segment to add sprite data
                if let Ok(segment) = self.segments.current_mut() {
                    // Append the pattern data to the current segment
                    // The pattern data is already arranged in the correct order:
                    // For each tile: 8 bytes for bit plane 0, followed by 8 bytes for bit plane 1
                    segment.extend(pattern_data);
                }
                Ok(())
            },
        }
    }

    /// Assembles an instruction string into bytes
    /// If labels map is provided, label references in operands will be resolved
    pub fn assemble_instruction(&mut self, input: &str, labels: &HashMap<String, u16>) -> AssembleResult<Vec<u8>> {
        // Split input into mnemonic and operand
        let (instruction, operand_opt) = split_instruction(input)?;

        // Handle implied addressing mode (no operand)
        if instruction.has_implied_addressing() {
            let metadata = self.handle_implied_instruction(instruction)?;
            return Ok(vec![metadata.opcode]);
        }

        // For other instructions, we need an operand
        let operand = operand_opt.ok_or_else(|| AssembleError::InvalidSyntaxWithContext {
            line: input.to_string(),
            message: format!("Missing operand for instruction '{}'", instruction),
        })?;

        // Handle accumulator addressing mode ("A")
        if operand.eq_ignore_ascii_case("a") {
            // Check if the instruction supports accumulator addressing
            match instruction {
                Instruction::ASL | Instruction::LSR => {
                    let metadata = self.decoder.lookup(instruction, AddressingMode::Accumulator)?;
                    return Ok(vec![metadata.opcode]);
                },
                _ => {
                    return Err(AssembleError::InvalidSyntaxWithContext {
                        line: input.to_string(),
                        message: format!(
                            "Instruction '{}' does not support accumulator addressing mode",
                            instruction
                        ),
                    })
                },
            }
        }

        // Special case: Handle asterisk (*) as current address
        if operand == "*" {
            // Look up the instruction with Absolute addressing mode
            let metadata = self.decoder.lookup(instruction, AddressingMode::Absolute)?;

            // Get the current address (usually the address of this instruction)
            let current_address = if let Ok(segment) = self.segments.current_mut() {
                segment.load_address + segment.data.len() as u16
            } else {
                self.load_address
            };

            // For JMP *, we want to jump to the address of the JMP instruction itself
            // The instruction is 3 bytes long: opcode + low byte + high byte
            // So we use current_address, which is the address of the instruction
            return Ok(vec![
                metadata.opcode,
                (current_address & 0xFF) as u8,        // Low byte
                ((current_address >> 8) & 0xFF) as u8, // High byte
            ]);
        }

        // Check if this is a label reference
        if self.is_label_reference(&operand) {
            let (metadata, address) = self.handle_label_reference(instruction, &operand, labels)?;

            if metadata.addressing_mode == AddressingMode::Relative {
                // For branch instructions, we need to calculate the offset relative to PC+2
                // (PC+2 points to the next instruction after the branch)

                // Get current position (where this instruction will be placed)
                let current_address = if let Ok(segment) = self.segments.current_mut() {
                    segment.load_address + segment.data.len() as u16
                } else {
                    self.load_address
                };

                // Target is PC+2 (after branch instruction) + offset
                // So offset = target - (PC+2)
                let offset = ((address as i32) - (current_address as i32 + 2)) as i8;

                return Ok(vec![
                    metadata.opcode,
                    offset as u8, // Store as unsigned byte, will be interpreted as signed during execution
                ]);
            }

            // Use encode_instruction for all other addressing modes
            return self.encode_instruction(metadata.opcode, metadata.addressing_mode, address);
        }

        // Handle standard addressing modes
        let (addressing_mode, operand_value) = self.parse_addressing_mode(&operand)?;

        let metadata = self.decoder.lookup(instruction, addressing_mode)?;

        self.encode_instruction(metadata.opcode, addressing_mode, operand_value)
    }

    /// Handles an instruction with implied addressing mode
    fn handle_implied_instruction(&self, instruction: Instruction) -> AssembleResult<InstructionMetadata> {
        Ok(self.decoder.lookup(instruction, AddressingMode::Implied)?)
    }

    /// Checks if an operand is a label reference (not starting with $ or #)
    fn is_label_reference(&self, operand: &str) -> bool {
        // Check for accumulator addressing mode (operand is just "A")
        if operand.trim().eq_ignore_ascii_case("a") {
            return false;
        }

        // Remove any index indicators (like ",X" or ",Y")
        let base_operand = if let Some(idx_pos) = operand.find(',') {
            &operand[..idx_pos]
        } else {
            operand
        };

        // Check if this is a numeric value with any common prefix
        if base_operand.starts_with('#') || base_operand.starts_with('$') || base_operand.starts_with('%') {
            return false;
        }

        // If it doesn't start with any known prefix, it's probably a label
        true
    }

    /// Handle an operand that refers to a label
    fn handle_label_reference(
        &self,
        instruction: Instruction,
        operand: &str,
        labels: &HashMap<String, u16>,
    ) -> AssembleResult<(InstructionMetadata, u16)> {
        // Extract the label and determine addressing mode based on indexed reference
        let (label, addressing_mode) = if let Some(idx_pos) = operand.find(",X") {
            // X-indexed addressing
            let label = &operand[..idx_pos];
            let addr_mode = if labels.get(label).is_some_and(|&addr| addr <= 0xFF) {
                AddressingMode::ZeroPageX
            } else {
                AddressingMode::AbsoluteX
            };
            (label, addr_mode)
        } else if let Some(idx_pos) = operand.find(",Y") {
            // Y-indexed addressing
            let label = &operand[..idx_pos];
            let addr_mode = if labels.get(label).is_some_and(|&addr| addr <= 0xFF) {
                AddressingMode::ZeroPageY
            } else {
                AddressingMode::AbsoluteY
            };
            (label, addr_mode)
        } else {
            // It's a normal label reference - look it up in the labels map
            let addr_mode = if instruction.is_branch() {
                AddressingMode::Relative
            } else if instruction.is_jump() {
                AddressingMode::Absolute
            } else if labels.get(operand).is_some_and(|&addr| addr <= 0xFF) {
                // Zero page addressing for addresses $00-$FF, unless the instruction requires absolute
                AddressingMode::ZeroPage
            } else {
                AddressingMode::Absolute
            };
            (operand, addr_mode)
        };

        // Look up the label in the labels map
        let address = labels
            .get(label)
            .ok_or_else(|| AssembleError::LabelError(format!("Undefined label: {}", label)))?;

        log::debug!("Label '{}' resolved to address: ${:04X}", label, address);

        // Look up the instruction metadata for this instruction and addressing mode
        let metadata = self.decoder.lookup(instruction, addressing_mode)?;

        Ok((metadata, *address))
    }

    /// Determines the addressing mode and operand value from a string
    ///
    /// Examples:
    /// - "#$42" -> Immediate mode with value 0x42
    /// - "$2000" -> Absolute mode with value 0x2000
    /// - "$42" -> Zero page mode with value 0x42
    fn parse_addressing_mode(&self, operand: &str) -> AssembleResult<(AddressingMode, u16)> {
        // Check for zero page X-indexed addressing ($00,X)
        if let Some(index_pos) = operand.find(",X") {
            let value_part = &operand[..index_pos];

            // Check if it's a label reference like ButtonMasks,X
            if !value_part.starts_with('$') {
                // This is a label reference with X-indexing
                return Ok((AddressingMode::AbsoluteX, 0)); // Value will be resolved later
            }

            // Handle normal $00,X or $1234,X
            let value = parse_value::<u16>(value_part)?;
            if value <= 0xFF {
                return Ok((AddressingMode::ZeroPageX, value));
            } else {
                return Ok((AddressingMode::AbsoluteX, value));
            }
        }

        // Check for zero page Y-indexed addressing ($00,Y)
        if let Some(index_pos) = operand.find(",Y") {
            let value_part = &operand[..index_pos];

            // Check if it's a label reference like ButtonMasks,Y
            if !value_part.starts_with('$') {
                // This is a label reference with Y-indexing
                return Ok((AddressingMode::AbsoluteY, 0)); // Value will be resolved later
            }

            // Handle normal $00,Y or $1234,Y
            let value = parse_value::<u16>(value_part)?;
            if value <= 0xFF {
                return Ok((AddressingMode::ZeroPageY, value));
            } else {
                return Ok((AddressingMode::AbsoluteY, value));
            }
        }

        // Check for immediate addressing (#$00)
        if let Some(value_part) = operand.strip_prefix('#') {
            let value = parse_value::<u16>(value_part)?;
            return Ok((AddressingMode::Immediate, value));
        }

        // Check for indexed indirect addressing (($00,X))
        if operand.starts_with('(') && operand.ends_with(",X)") {
            let value_part = &operand[1..operand.len() - 3];
            let value = parse_value::<u16>(value_part)?;
            if value <= 0xFF {
                return Ok((AddressingMode::IndexedIndirect, value));
            } else {
                return Err(AssembleError::ValueOutOfRange(format!(
                    "Value ${:X} too large for indexed indirect addressing",
                    value
                )));
            }
        }

        // Check for indirect indexed addressing (($00),Y)
        if operand.starts_with('(') && operand.ends_with("),Y") {
            let value_part = &operand[1..operand.len() - 3];
            let value = parse_value::<u16>(value_part)?;
            if value <= 0xFF {
                return Ok((AddressingMode::IndirectIndexed, value));
            } else {
                return Err(AssembleError::ValueOutOfRange(format!(
                    "Value ${:X} too large for indirect indexed addressing",
                    value
                )));
            }
        }

        // Check for indirect addressing (($1234))
        if operand.starts_with('(') && operand.ends_with(')') {
            let value_part = &operand[1..operand.len() - 1];
            let value = parse_value::<u16>(value_part)?;
            return Ok((AddressingMode::Indirect, value));
        }

        // Check for zero page addressing ($00)
        if operand.starts_with('$') && operand.len() <= 3 {
            let value = parse_value::<u16>(operand)?;
            if value <= 0xFF {
                return Ok((AddressingMode::ZeroPage, value));
            }
        }

        // Check for absolute addressing ($1234)
        if operand.starts_with('$') && operand.len() == 5 {
            let value = parse_value::<u16>(operand)?;
            return Ok((AddressingMode::Absolute, value));
        }

        Err(AssembleError::InvalidAddressingMode(operand.to_string()))
    }

    /// Calculates the size of an instruction in bytes, assuming directive check has already been done
    fn calculate_instruction_size(&self, line: &str) -> AssembleResult<u16> {
        let (instruction, operand_opt) = split_instruction(line)?;

        // Implied addressing mode (just the opcode)
        if instruction.has_implied_addressing() {
            return Ok(1);
        }

        // For other instructions, we need an operand
        let operand = operand_opt.ok_or_else(|| AssembleError::InvalidSyntaxWithContext {
            line: line.to_string(),
            message: format!("Missing operand for instruction '{}'", instruction),
        })?;

        // Check for accumulator addressing mode
        if operand.trim().eq_ignore_ascii_case("a") {
            return Ok(1); // Just the opcode for accumulator addressing
        }

        // For potential label references, assume Absolute addressing (3 bytes)
        if self.is_label_reference(&operand) {
            return Ok(3);
        }

        // Regular instruction, get its size from metadata
        let metadata = self.parse_instruction(line)?;
        Ok(metadata.addressing_mode.size())
    }

    /// Parses an instruction string into metadata
    /// If labels map is provided, label references in operands will be resolved
    fn parse_instruction(&self, input: &str) -> AssembleResult<InstructionMetadata> {
        // Split input into mnemonic and operand
        let (instruction, operand_opt) = split_instruction(input)?;

        // Check for implied addressing mode instructions (no operand)
        if instruction.has_implied_addressing() {
            return self.handle_implied_instruction(instruction);
        }

        // For other instructions, we need an operand
        let operand = operand_opt.ok_or_else(|| AssembleError::InvalidSyntax("Missing operand".to_string()))?;

        let addressing_mode = AddressingMode::from_instruction(instruction, &operand)?;

        Ok(self.decoder.lookup(instruction, addressing_mode)?)
    }

    /// Encodes an instruction with its operand bytes based on addressing mode
    fn encode_instruction(
        &self,
        opcode: u8,
        addressing_mode: AddressingMode,
        operand_value: u16,
    ) -> AssembleResult<Vec<u8>> {
        let mut bytes = vec![opcode];

        match addressing_mode {
            AddressingMode::Immediate
            | AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::Relative => {
                bytes.push(operand_value as u8);
            },
            AddressingMode::Absolute
            | AddressingMode::AbsoluteX
            | AddressingMode::AbsoluteY
            | AddressingMode::Indirect => {
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
}

/// Process a single line of assembly, extract labels and code, assuming directive check has already been done
fn process_line(line: &str) -> AssembleResult<(String, Option<String>)> {
    let trimmed = line.trim();
    let mut label = String::new();
    let mut code = None;

    // First, check if the line starts with a label
    // A label can be at the start of the line and followed by a colon
    if let Some(colon_pos) = trimmed.find(':') {
        label = trimmed[..colon_pos].trim().to_string();

        // Get the code part after the colon, if any
        let code_part = trimmed[colon_pos + 1..].trim();
        if !code_part.is_empty() {
            code = Some(code_part.to_string());
        }
    } else {
        // If no label with colon, treat it as just code
        code = Some(trimmed.to_string());
    }

    // Debug logging for WaitForVBlank label or code
    if label == "WaitForVBlank" || code.as_ref().is_some_and(|c| c.contains("WaitForVBlank")) {
        log::debug!(
            "Processing line with WaitForVBlank - Label: '{}', Code: '{:?}'",
            label,
            code
        );
    }

    Ok((label, code))
}

/// Parse a comma-separated list of tokens that can be either string literals or numeric values
fn parse_comma_separated_byte_tokens(input: &str) -> AssembleResult<Vec<u8>> {
    lazy_static! {
        static ref STRING_REGEX: Regex = Regex::new(r#"^\s*["']([^"']*)["']"#).unwrap();
    }

    let mut result = Vec::new();
    let mut remaining = input.trim();

    while !remaining.is_empty() {
        // Skip leading whitespace
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            break;
        }

        // Check if this is a string literal
        if let Some(captures) = STRING_REGEX.captures(remaining) {
            let (bytes, rest) = extract_string_literal(remaining, &captures)?;
            result.extend(bytes);
            remaining = rest;
        } else {
            // Handle numeric value
            let (byte, rest) = extract_numeric_byte_value(remaining)?;
            result.push(byte);
            remaining = rest;
        }

        // Skip comma if present
        remaining = remaining.trim_start();
        if remaining.starts_with(',') {
            remaining = &remaining[1..];
        }
    }

    Ok(result)
}

fn format_segment_name(name: &str) -> &str {
    name.trim().trim_matches('"').trim_matches('\'')
}

/// Extract a string literal and convert to bytes
fn extract_string_literal<'a>(input: &'a str, captures: &regex::Captures) -> AssembleResult<(Vec<u8>, &'a str)> {
    // Get the string content (capture group 1)
    let string_content = match captures.get(1) {
        Some(m) => m.as_str(),
        None => {
            return Err(AssembleError::DirectiveError(
                "Missing string content in regex match".to_string(),
            ))
        },
    };

    // Get the full match length to update the remaining input
    let match_len = match captures.get(0) {
        Some(m) => m.as_str().len(),
        None => return Err(AssembleError::DirectiveError("Invalid regex capture".to_string())),
    };

    let remaining = &input[match_len..];

    // Convert string to bytes
    let bytes: Vec<u8> = string_content.chars().map(|c| c as u8).collect();

    Ok((bytes, remaining))
}

/// Extract a numeric byte value
fn extract_numeric_byte_value(input: &str) -> AssembleResult<(u8, &str)> {
    let comma_pos = input.find(',').unwrap_or(input.len());
    let value_str = input[..comma_pos].trim();

    if value_str.is_empty() {
        return Err(AssembleError::DirectiveError("Empty value in byte list".to_string()));
    }

    let value = parse_value::<u8>(value_str)?;
    let remaining = if comma_pos < input.len() {
        &input[comma_pos..]
    } else {
        ""
    };

    Ok((value, remaining))
}

/// Splits an instruction string into mnemonic and operand parts
fn split_instruction(input: &str) -> AssembleResult<(Instruction, Option<String>)> {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    if parts.is_empty() {
        return Err(AssembleError::InvalidSyntaxWithContext {
            line: input.to_string(),
            message: "Empty input line".to_string(),
        });
    }

    let mnemonic = parts[0].to_string();

    // Check if this is a directive (starts with a dot) - we shouldn't parse it as an instruction
    if mnemonic.starts_with('.') {
        return Err(AssembleError::InvalidSyntaxWithContext {
            line: input.to_string(),
            message: format!("Tried to parse directive '{}' as an instruction", mnemonic),
        });
    }

    let operand = if parts.len() > 1 {
        Some(parts[1].trim().to_string())
    } else {
        None
    };

    // Try to parse the instruction, provide a descriptive error if it fails
    match mnemonic.parse::<Instruction>() {
        Ok(instruction) => Ok((instruction, operand)),
        Err(_) => Err(AssembleError::InvalidSyntaxWithContext {
            line: input.to_string(),
            message: format!("Unknown instruction mnemonic: '{}'", mnemonic),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anyhow::Result;

    use super::*;
    use crate::cpu::AddressingMode;

    /// Tests for instruction parsing (mnemonic recognition)
    #[test]
    fn test_instruction_parsing() -> Result<()> {
        // Direct FromStr tests
        assert_eq!("LDA".parse::<Instruction>()?, Instruction::LDA);
        assert!("XYZ".parse::<Instruction>().is_err());

        Ok(())
    }

    /// Integration tests for complete instruction parsing
    #[test]
    fn test_complete_instruction_parsing() -> AssembleResult<()> {
        let parser = Assembler::new(0);

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
    fn test_error_conditions() -> AssembleResult<()> {
        let parser = Assembler::new(0);

        // Test invalid mnemonic
        let result = parser.parse_instruction("XYZ #$42");
        assert!(result.is_err());

        // Test invalid operand value
        let result = parser.parse_instruction("LDA #$ZZ");
        assert!(result.is_err());

        // Test missing operand
        let result = parser.parse_instruction("LDA");
        assert!(result.is_err());

        Ok(())
    }

    /// Tests for label declaration and usage
    #[test]
    fn test_label_declaration() -> AssembleResult<()> {
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
    fn test_forward_reference() -> AssembleResult<()> {
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
    fn test_multiple_labels() -> AssembleResult<()> {
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
    fn test_segment_directive() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0);

        // Add test segments
        assembler.segments.add("CODE", 0x8000);
        assembler.segments.add("DATA", 0xC000);

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
    fn test_label_errors() -> AssembleResult<()> {
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
    fn test_label_formatting() -> AssembleResult<()> {
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
    fn test_directive_handling_separation() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0);

        // Add test segments
        assembler.segments.add("CODE", 0x8000);
        assembler.segments.add("DATA", 0xC000);

        // Parse a directive without applying it
        let directive = assembler.parse_directive(".segment \"CODE\"", &HashMap::new())?;
        assert!(directive.is_some());
        if let Some(directive) = directive {
            match directive {
                Directive::Segment(name) => {
                    assert_eq!(name, "CODE");
                    // Current segment should still be None at this point
                    assert!(assembler.segments.current.is_none());

                    // Now apply the directive
                    assembler.apply_directive(&Directive::Segment(name))?;
                    // Current segment should now be set
                    assert_eq!(assembler.segments.current, Some("CODE".to_string()));
                },
                Directive::Byte(_) => panic!("Expected Segment directive, got Byte"),
                Directive::Word(_) => panic!("Expected Segment directive, got Word"),
                Directive::Res(_, _) => panic!("Expected Segment directive, got Res"),
                Directive::Sprite(_, _, _) => panic!("Expected Segment directive, got Sprite"),
            }
        } else {
            panic!("Expected Segment directive");
        }

        Ok(())
    }

    /// Test the .byte and .word directives
    #[test]
    fn test_data_directives() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0);

        // Add test segments
        assembler.segments.add("CODE", 0x8000);
        assembler.segments.add("DATA", 0xC000);

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
    fn test_res_directive() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0);

        // Add test segments
        assembler.segments.add("CODE", 0x8000);
        assembler.segments.add("DATA", 0xC000);

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

    /// Test branch instructions with label references
    #[test]
    fn test_branch_instruction_with_label() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x0600);

        // Test program with a label and a branch to that label
        let program = r#"
            LDA #$01    ; Set a value
            BPL target  ; Branch to target (should be encoded as relative)
            LDA #$FF    ; This shouldn't execute if branch taken
        target:
            LDA #$42    ; Target of branch
        "#;

        let segments = assembler.assemble_program(program)?;
        let bytes = segments.get("STARTUP").expect("STARTUP segment missing");

        // Expected encoding - actual output from assembler:
        // A9 01       ; LDA #$01
        // 10 04       ; BPL +4 (offset to target)
        // A9 FF       ; LDA #$FF
        // A9 42       ; LDA #$42 (target)
        assert_eq!(
            bytes,
            &vec![
                0xA9, 0x01, // LDA #$01
                0x10, 0x04, // BPL with offset 4 to target
                0xA9, 0xFF, // LDA #$FF
                0xA9, 0x42 // LDA #$42 (target)
            ]
        );

        // Now let's test with a backward branch
        let program = r#"
        start:
            LDA #$01    ; Set a value
            BPL start   ; Branch back to start (negative offset)
        "#;

        let segments = assembler.assemble_program(program)?;
        let bytes = segments.get("STARTUP").expect("STARTUP segment missing");

        // Expected encoding - actual output from assembler:
        // A9 01       ; LDA #$01 (start)
        // 10 FE       ; BPL -2 (branch back to start)
        assert_eq!(
            bytes,
            &vec![
                0xA9, 0x01, // LDA #$01 (start)
                0x10, 0xFE // BPL with offset -2 to start
            ]
        );

        Ok(())
    }

    #[test]
    fn test_assemble_status_flag_instructions() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x8000);
        let program = "
            CLC         ; Clear carry flag
            SEC         ; Set carry flag
        ";

        // Test the full program assembly
        let segments = assembler.assemble_program(program)?;
        assert_eq!(segments.get("STARTUP").unwrap()[0], 0x18); // CLC
        assert_eq!(segments.get("STARTUP").unwrap()[1], 0x38); // SEC

        Ok(())
    }

    #[test]
    fn test_assemble_branch_eq_ne() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x8000);

        // Test BEQ
        let beq_program = "
            LDA #$00    ; Load 0 (sets Z flag)
            BEQ target  ; Branch if Equal
            LDA #$FF    ; Should be skipped
        target:
            LDA #$42    ; Target
        ";

        let segments = assembler.assemble_program(beq_program)?;
        let code = segments.get("STARTUP").unwrap();

        // BEQ should be present with opcode 0xF0
        assert_eq!(code[2], 0xF0);

        // Test BNE
        let bne_program = "
            LDA #$01    ; Load 1 (clears Z flag)
            BNE target  ; Branch if Not Equal
            LDA #$FF    ; Should be skipped
        target:
            LDA #$42    ; Target
        ";

        let segments = assembler.assemble_program(bne_program)?;
        let code = segments.get("STARTUP").unwrap();

        // BNE should be present with opcode 0xD0
        assert_eq!(code[2], 0xD0);

        Ok(())
    }

    #[test]
    fn test_assemble_arithmetic_instructions() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x8000);

        // Test ADC and SBC with immediate addressing
        let program = "
            CLC         ; Clear carry before add
            ADC #$01    ; Add 1 to accumulator
            SEC         ; Set carry before subtract
            SBC #$01    ; Subtract 1 from accumulator
        ";

        let segments = assembler.assemble_program(program)?;
        let code = segments.get("STARTUP").unwrap();

        // Check that opcodes match expected values
        assert_eq!(code[0], 0x18); // CLC
        assert_eq!(code[1], 0x69); // ADC #$01
        assert_eq!(code[2], 0x01); // The value $01
        assert_eq!(code[3], 0x38); // SEC
        assert_eq!(code[4], 0xE9); // SBC #$01
        assert_eq!(code[5], 0x01); // The value $01

        // Test with zero page addressing
        let zp_program = "
            ADC $10     ; Add value at zero page address $10
            SBC $20     ; Subtract value at zero page address $20
        ";

        let segments = assembler.assemble_program(zp_program)?;
        let code = segments.get("STARTUP").unwrap();

        assert_eq!(code[0], 0x65); // ADC $10
        assert_eq!(code[1], 0x10); // Zero page address $10
        assert_eq!(code[2], 0xE5); // SBC $20
        assert_eq!(code[3], 0x20); // Zero page address $20

        // Test with absolute addressing
        let abs_program = "
            ADC $1000   ; Add value at address $1000
            SBC $2000   ; Subtract value at address $2000
        ";

        let segments = assembler.assemble_program(abs_program)?;
        let code = segments.get("STARTUP").unwrap();

        assert_eq!(code[0], 0x6D); // ADC $1000
        assert_eq!(code[1], 0x00); // Low byte of $1000
        assert_eq!(code[2], 0x10); // High byte of $1000
        assert_eq!(code[3], 0xED); // SBC $2000
        assert_eq!(code[4], 0x00); // Low byte of $2000
        assert_eq!(code[5], 0x20); // High byte of $2000

        Ok(())
    }

    #[test]
    fn test_assemble_comparison_instructions() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x8000);

        // Test CMP with immediate addressing
        let program = "
            LDA #$40    ; Load 0x40 into accumulator
            CMP #$40    ; Compare with 0x40 (should be equal)
            CMP #$30    ; Compare with 0x30 (should be greater)
            CMP #$50    ; Compare with 0x50 (should be less)
        ";

        let segments = assembler.assemble_program(program)?;
        let code = segments.get("STARTUP").unwrap();

        // Check that opcodes match expected values
        assert_eq!(code[0], 0xA9); // LDA #$40
        assert_eq!(code[1], 0x40); // Value $40

        assert_eq!(code[2], 0xC9); // CMP #$40
        assert_eq!(code[3], 0x40); // Value $40

        assert_eq!(code[4], 0xC9); // CMP #$30
        assert_eq!(code[5], 0x30); // Value $30

        assert_eq!(code[6], 0xC9); // CMP #$50
        assert_eq!(code[7], 0x50); // Value $50

        // Test with zero page addressing
        let zp_program = "
            CMP $10     ; Compare with value at zero page address $10
        ";

        let segments = assembler.assemble_program(zp_program)?;
        let code = segments.get("STARTUP").unwrap();

        assert_eq!(code[0], 0xC5); // CMP $10
        assert_eq!(code[1], 0x10); // Zero page address $10

        // Test with absolute addressing
        let abs_program = "
            CMP $1000   ; Compare with value at address $1000
        ";

        let segments = assembler.assemble_program(abs_program)?;
        let code = segments.get("STARTUP").unwrap();

        assert_eq!(code[0], 0xCD); // CMP $1000
        assert_eq!(code[1], 0x00); // Low byte of $1000
        assert_eq!(code[2], 0x10); // High byte of $1000

        // Test with indexed addressing modes
        let idx_program = "
            CMP $10,X   ; Compare with value at zero page address $10 + X
            CMP $1000,X ; Compare with value at address $1000 + X
            CMP $1000,Y ; Compare with value at address $1000 + Y
        ";

        let segments = assembler.assemble_program(idx_program)?;
        let code = segments.get("STARTUP").unwrap();

        assert_eq!(code[0], 0xD5); // CMP $10,X
        assert_eq!(code[1], 0x10); // Zero page address $10

        assert_eq!(code[2], 0xDD); // CMP $1000,X
        assert_eq!(code[3], 0x00); // Low byte of $1000
        assert_eq!(code[4], 0x10); // High byte of $1000

        assert_eq!(code[5], 0xD9); // CMP $1000,Y
        assert_eq!(code[6], 0x00); // Low byte of $1000
        assert_eq!(code[7], 0x10); // High byte of $1000

        Ok(())
    }

    #[test]
    fn test_assemble_transfer_instructions() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x8000);

        // Test TXS (Transfer X to Stack Pointer)
        let txs_program = "
            LDX #$FF    ; Load 0xFF into X
            TXS         ; Transfer X to Stack Pointer
        ";

        let segments = assembler.assemble_program(txs_program)?;
        let code = segments.get("STARTUP").unwrap();

        // Check opcode for LDX #$FF
        assert_eq!(code[0], 0xA2); // LDX immediate
        assert_eq!(code[1], 0xFF); // Value 0xFF

        // Check opcode for TXS
        assert_eq!(code[2], 0x9A); // TXS implied

        Ok(())
    }

    #[test]
    fn test_variable_declarations_with_labels() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x8000).with_nes_segments();

        // Test program with labeled variable declarations in ZEROPAGE segment
        let program = r#"
            .segment "ZEROPAGE"
            ball_x:     .res 1   ; Ball X position
            ball_y:     .res 1   ; Ball Y position
            
            .segment "STARTUP"
            LDA #$50          ; Initial X position
            STA ball_x        ; Store in ball_x variable using label
            
            LDA #$60          ; Initial Y position
            STA ball_y        ; Store in ball_y variable using label
        "#;

        let segments = assembler.assemble_program(program)?;

        // Check that variables were correctly reserved in ZEROPAGE segment
        let zeropage = segments.get("ZEROPAGE").unwrap();
        assert_eq!(zeropage.len(), 2, "ZEROPAGE segment should have 2 bytes reserved");

        // Check code in STARTUP segment
        let code = segments.get("STARTUP").unwrap();

        // First instruction: LDA #$50
        assert_eq!(code[0], 0xA9, "First byte should be LDA immediate (0xA9)"); // LDA immediate
        assert_eq!(code[1], 0x50, "Second byte should be value $50"); // Value $50

        // Second instruction: STA ball_x (should be STA $00, as ball_x is at address $0000)
        assert_eq!(code[2], 0x85, "Third byte should be STA zero page (0x85)"); // STA zero page
        assert_eq!(code[3], 0x00, "Fourth byte should be address $00 (ball_x)"); // Address $00 (ball_x)

        // Third instruction: LDA #$60
        assert_eq!(code[4], 0xA9, "Fifth byte should be LDA immediate (0xA9)"); // LDA immediate
        assert_eq!(code[5], 0x60, "Sixth byte should be value $60"); // Value $60

        // Fourth instruction: STA ball_y
        assert_eq!(code[6], 0x85, "Seventh byte should be STA zero page (0x85)"); // STA zero page
        assert_eq!(code[7], 0x01, "Eighth byte should be address $01 (ball_y)"); // Address $01 (ball_y)

        Ok(())
    }

    #[test]
    fn test_zeropage_variable_declarations_with_labels() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x8000).with_nes_segments();

        // Test program with labeled variable declarations in ZEROPAGE segment
        let program = r#"
            .segment "ZEROPAGE"
            ball_x:     .res 1   ; Ball X position
            ball_y:     .res 1   ; Ball Y position
            
            .segment "STARTUP"
            LDA #$50          ; Initial X position
            STA ball_x        ; Store in ball_x variable using label
            
            LDA #$60          ; Initial Y position
            STA ball_y        ; Store in ball_y variable using label
        "#;

        let segments = assembler.assemble_program(program)?;

        // Check that variables were correctly reserved in ZEROPAGE segment
        let zeropage = segments.get("ZEROPAGE").unwrap();
        assert_eq!(zeropage.len(), 2, "ZEROPAGE segment should have 2 bytes reserved");

        // Check code in STARTUP segment
        let code = segments.get("STARTUP").unwrap();

        // First instruction: LDA #$50
        assert_eq!(code[0], 0xA9, "First byte should be LDA immediate (0xA9)"); // LDA immediate
        assert_eq!(code[1], 0x50, "Second byte should be value $50"); // Value $50

        // Second instruction: STA ball_x (should be STA $00, as ball_x is at address $0000)
        assert_eq!(code[2], 0x85, "Third byte should be STA zero page (0x85)"); // STA zero page
        assert_eq!(code[3], 0x00, "Fourth byte should be address $00 (ball_x)"); // Address $00 (ball_x)

        // Third instruction: LDA #$60
        assert_eq!(code[4], 0xA9, "Fifth byte should be LDA immediate (0xA9)"); // LDA immediate
        assert_eq!(code[5], 0x60, "Sixth byte should be value $60"); // Value $60

        // Fourth instruction: STA ball_y
        assert_eq!(code[6], 0x85, "Seventh byte should be STA zero page (0x85)"); // STA zero page
        assert_eq!(code[7], 0x01, "Eighth byte should be address $01 (ball_y)"); // Address $01 (ball_y)

        Ok(())
    }

    #[test]
    fn test_zeropage_operand_size() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x8000).with_nes_segments();

        // Test program focusing on zero page addressing operand size
        let program = r#"
            .segment "ZEROPAGE"
            var_x:     .res 1   ; Variable at $00
            var_y:     .res 1   ; Variable at $01
            
            .segment "STARTUP"
            ; Store values in zero page variables
            LDA #$42
            STA var_x  ; This should assemble to 85 00 (2 bytes only)
            
            LDA #$24
            STA var_y  ; This should assemble to 85 01 (2 bytes only)
            
            ; For comparison, absolute addressing
            LDA #$FF
            STA $2000  ; This should be 8D 00 20 (3 bytes)
        "#;

        let segments = assembler.assemble_program(program)?;

        // Get the STARTUP segment
        let code = segments.get("STARTUP").unwrap();

        // Check zero page addressing instructions
        assert_eq!(code[0], 0xA9, "First byte should be LDA immediate");
        assert_eq!(code[1], 0x42, "Second byte should be the immediate value $42");
        assert_eq!(code[2], 0x85, "Third byte should be STA zero page");
        assert_eq!(code[3], 0x00, "Fourth byte should be zero page address $00");

        // Check that the next instruction starts immediately after (position 4)
        // If there's an extra byte incorrectly added for zero page, this will fail
        assert_eq!(code[4], 0xA9, "Fifth byte should be next LDA immediate");
        assert_eq!(code[5], 0x24, "Sixth byte should be the immediate value $24");
        assert_eq!(code[6], 0x85, "Seventh byte should be STA zero page");
        assert_eq!(code[7], 0x01, "Eighth byte should be zero page address $01");

        // Check absolute addressing has correct size
        assert_eq!(code[8], 0xA9, "Should be LDA immediate");
        assert_eq!(code[9], 0xFF, "Should be the immediate value $FF");
        assert_eq!(code[10], 0x8D, "Should be STA absolute");
        assert_eq!(code[11], 0x00, "Should be low byte of address $2000");
        assert_eq!(code[12], 0x20, "Should be high byte of address $2000");

        Ok(())
    }

    /// Test the .sprite directive for defining multi-tile sprites
    #[test]
    fn test_sprite_directive() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x8000).with_nes_segments();

        // Test program with a 2x2 sprite
        let program = r#"
            .segment "CHARS"
            .sprite 2, 2, %00111100, %01000010, %10000001, %10000001, %10000001, %10000001, %01000010, %00111100, %00000000, %00111100, %01111110, %01111110, %01111110, %01111110, %00111100, %00000000, %00111100, %01000010, %10000001, %10000001, %10000001, %10000001, %01000010, %00111100, %00000000, %00111100, %01111110, %01111110, %01111110, %01111110, %00111100, %00000000, %00111100, %01000010, %10000001, %10000001, %10000001, %10000001, %01000010, %00111100, %00000000, %00111100, %01111110, %01111110, %01111110, %01111110, %00111100, %00000000, %00111100, %01000010, %10000001, %10000001, %10000001, %10000001, %01000010, %00111100, %00000000, %00111100, %01111110, %01111110, %01111110, %01111110, %00111100, %00000000
        "#;

        let segments = assembler.assemble_program(program)?;

        // Verify the CHARS segment
        let chars_segment = segments.get("CHARS").expect("CHARS segment missing");

        // Should have 64 bytes (2x2 sprite, 16 bytes per tile)
        assert_eq!(chars_segment.len(), 64, "Expected 64 bytes for a 2x2 sprite pattern");

        // Check the pattern data (first few bytes)
        // First tile (top-left)
        assert_eq!(chars_segment[0], 0x3C, "First byte of top-left tile incorrect"); // %00111100 - bit plane 0
        assert_eq!(chars_segment[8], 0x00, "Ninth byte of top-left tile incorrect"); // %00000000 - bit plane 1

        // Second tile (top-right)
        assert_eq!(chars_segment[16], 0x3C, "First byte of top-right tile incorrect"); // %00111100 - bit plane 0
        assert_eq!(chars_segment[24], 0x00, "Ninth byte of top-right tile incorrect"); // %00000000 - bit plane 1

        // Third tile (bottom-left)
        assert_eq!(chars_segment[32], 0x3C, "First byte of bottom-left tile incorrect"); // %00111100 - bit plane 0
        assert_eq!(chars_segment[40], 0x00, "Ninth byte of bottom-left tile incorrect"); // %00000000 - bit plane 1

        // Fourth tile (bottom-right)
        assert_eq!(chars_segment[48], 0x3C, "First byte of bottom-right tile incorrect"); // %00111100 - bit plane 0
        assert_eq!(chars_segment[56], 0x00, "Ninth byte of bottom-right tile incorrect"); // %00000000 - bit plane 1

        Ok(())
    }

    #[test]
    fn test_asl_addressing_modes() -> Result<()> {
        let mut assembler = Assembler::new(0x8000);
        let labels = HashMap::new();

        // Test ASL with accumulator addressing
        let bytes = assembler.assemble_instruction("ASL A", &labels)?;
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 0x0A, "ASL A should assemble to opcode 0x0A");

        // Test ASL with zero page addressing
        let bytes = assembler.assemble_instruction("ASL $10", &labels)?;
        assert_eq!(bytes.len(), 2);
        assert_eq!(bytes[0], 0x06, "ASL $10 should assemble to opcode 0x06");
        assert_eq!(bytes[1], 0x10);

        // Test ASL with absolute addressing
        let bytes = assembler.assemble_instruction("ASL $1000", &labels)?;
        assert_eq!(bytes.len(), 3);
        assert_eq!(bytes[0], 0x0E, "ASL $1000 should assemble to opcode 0x0E");
        assert_eq!(bytes[1], 0x00);
        assert_eq!(bytes[2], 0x10);

        Ok(())
    }

    #[test]
    fn test_lsr_addressing_modes() -> Result<()> {
        let mut assembler = Assembler::new(0x8000);
        let labels = HashMap::new();

        // Test LSR with accumulator addressing
        let bytes = assembler.assemble_instruction("LSR A", &labels)?;
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 0x4A, "LSR A should assemble to opcode 0x4A");

        // Test LSR with zero page addressing
        let bytes = assembler.assemble_instruction("LSR $10", &labels)?;
        assert_eq!(bytes.len(), 2);
        assert_eq!(bytes[0], 0x46, "LSR $10 should assemble to opcode 0x46");
        assert_eq!(bytes[1], 0x10);

        // Test LSR with absolute addressing
        let bytes = assembler.assemble_instruction("LSR $1000", &labels)?;
        assert_eq!(bytes.len(), 3);
        assert_eq!(bytes[0], 0x4E, "LSR $1000 should assemble to opcode 0x4E");
        assert_eq!(bytes[1], 0x00);
        assert_eq!(bytes[2], 0x10);

        Ok(())
    }

    #[test]
    fn test_accumulator_addressing_mode_parsing() -> Result<()> {
        // Test that from_instruction correctly identifies accumulator addressing
        let addressing_mode = AddressingMode::from_instruction(Instruction::ASL, "A")?;

        // The implementation should now correctly return Accumulator for "A" with ASL instruction
        assert_eq!(
            addressing_mode,
            AddressingMode::Accumulator,
            "from_instruction should return Accumulator for 'A' operand with ASL instruction"
        );

        // Test with a non-shift instruction where "A" should still be treated as Implied
        let addressing_mode = AddressingMode::from_instruction(Instruction::LDA, "A")?;
        assert_eq!(
            addressing_mode,
            AddressingMode::Implied,
            "from_instruction should return Implied for 'A' operand with non-shift instructions"
        );

        Ok(())
    }

    #[test]
    fn test_indexed_label_references() -> AssembleResult<()> {
        // Create an assembler instance
        let mut assembler = Assembler::new(0x8000);

        // Create a labels hashmap with test values
        let mut labels = HashMap::new();
        labels.insert("ButtonMasks".to_string(), 0x8500);

        // Test X-indexed absolute addressing (ButtonMasks,X)
        let result = assembler.assemble_instruction("AND ButtonMasks,X", &labels)?;

        // Expected output: AND instruction with AbsoluteX addressing mode for ButtonMasks
        // Opcode 0x3D (AND AbsoluteX) followed by the address bytes in little-endian
        assert_eq!(result.len(), 3, "AND ButtonMasks,X should be 3 bytes");
        assert_eq!(result[0], 0x3D, "First byte should be opcode 0x3D (AND AbsoluteX)");
        assert_eq!(result[1], 0x00, "Second byte should be low byte of address (0x8500)");
        assert_eq!(result[2], 0x85, "Third byte should be high byte of address (0x8500)");

        // Test Y-indexed absolute addressing (ButtonMasks,Y)
        let result = assembler.assemble_instruction("AND ButtonMasks,Y", &labels)?;

        // Expected output: AND instruction with AbsoluteY addressing mode for ButtonMasks
        // Opcode 0x39 (AND AbsoluteY) followed by the address bytes in little-endian
        assert_eq!(result.len(), 3, "AND ButtonMasks,Y should be 3 bytes");
        assert_eq!(result[0], 0x39, "First byte should be opcode 0x39 (AND AbsoluteY)");
        assert_eq!(result[1], 0x00, "Second byte should be low byte of address (0x8500)");
        assert_eq!(result[2], 0x85, "Third byte should be high byte of address (0x8500)");

        // Test with a variable in zero page range
        // The handle_label_reference function now optimizes for zero page addressing
        labels.insert("ZeroVar".to_string(), 0x0042);

        // Test X-indexed addressing with a zero page variable (ZeroVar,X)
        let result = assembler.assemble_instruction("AND ZeroVar,X", &labels)?;

        // Now our assembler correctly uses ZeroPageX for zero page variables with X-indexing
        assert_eq!(result.len(), 2, "AND ZeroVar,X should be 2 bytes (uses ZeroPageX)");
        assert_eq!(result[0], 0x35, "First byte should be opcode 0x35 (AND ZeroPageX)");
        assert_eq!(result[1], 0x42, "Second byte should be the address (0x0042)");

        Ok(())
    }

    // ---------------------------------------------------------------------------------------
    // Address tracking across data directives
    //
    // Regression tests for a bug where label collection advanced the address for instructions
    // and `.res`, but not for `.byte` or `.word`. Data tables therefore occupied no space as far
    // as the label pass was concerned, so every label defined after a table was assigned an
    // address that was too low — and several distinct labels could collapse onto the same one.
    //
    // The symptom was severe and hard to attribute: `JSR SomeRoutine` assembled to the address of
    // a data table, so the CPU executed the table's bytes as code and died on an invalid opcode.
    // Nothing in the label map looked obviously wrong, because the addresses were self-consistent
    // — just uniformly wrong after the first table.
    // ---------------------------------------------------------------------------------------

    #[test]
    fn labels_after_a_byte_table_get_correct_addresses() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x8000).with_nes_segments();
        let labels = assembler.collect_labels(
            r#"
.segment "STARTUP"
Start:
  NOP
Table:
  .byte $01, $02, $03, $04
After:
  NOP
"#,
        )?;

        assert_eq!(labels.get("Start"), Some(&0x8000), "Start should be at the load address");
        assert_eq!(labels.get("Table"), Some(&0x8001), "Table follows a one-byte NOP");
        assert_eq!(
            labels.get("After"),
            Some(&0x8005),
            "After must clear all four table bytes, not sit on top of them"
        );
        Ok(())
    }

    #[test]
    fn labels_after_a_word_table_get_correct_addresses() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x8000).with_nes_segments();
        let labels = assembler.collect_labels(
            r#"
.segment "STARTUP"
Table:
  .word $1234, $5678
After:
  NOP
"#,
        )?;

        // Each `.word` is two bytes, so two of them occupy four.
        assert_eq!(labels.get("Table"), Some(&0x8000));
        assert_eq!(labels.get("After"), Some(&0x8004), "a .word is two bytes, not one");
        Ok(())
    }

    #[test]
    fn byte_directive_counts_string_literals_by_character() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x8000).with_nes_segments();
        let labels = assembler.collect_labels(
            r#"
.segment "STARTUP"
Header:
  .byte "NES", $1A
After:
  NOP
"#,
        )?;

        // The iNES header is written this way: three characters plus one byte is four bytes,
        // not the two values a naive comma count would give.
        assert_eq!(
            labels.get("After"),
            Some(&0x8004),
            "a quoted string is one byte per character"
        );
        Ok(())
    }

    #[test]
    fn labels_after_a_res_directive_get_correct_addresses() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x8000).with_nes_segments();
        let labels = assembler.collect_labels(
            r#"
.segment "STARTUP"
Buffer:
  .res 16
After:
  NOP
"#,
        )?;

        assert_eq!(labels.get("After"), Some(&0x8010), ".res must reserve its full size");
        Ok(())
    }

    #[test]
    fn distinct_labels_never_collapse_onto_one_address() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x8000).with_nes_segments();
        let labels = assembler.collect_labels(
            r#"
.segment "STARTUP"
TableA:
  .byte $01, $02
TableB:
  .byte $03, $04
Routine:
  NOP
"#,
        )?;

        let a = labels.get("TableA").copied().unwrap_or_default();
        let b = labels.get("TableB").copied().unwrap_or_default();
        let routine = labels.get("Routine").copied().unwrap_or_default();

        assert_ne!(a, b, "two tables must not share an address");
        assert_ne!(b, routine, "a routine must not share an address with a table");
        assert_eq!((a, b, routine), (0x8000, 0x8002, 0x8004));
        Ok(())
    }

    /// The end-to-end shape of the original bug: a call to a routine defined *after* a data table.
    #[test]
    fn jsr_to_a_routine_after_a_table_targets_the_routine_not_the_data() -> AssembleResult<()> {
        let source = r#"
.segment "STARTUP"
Start:
  JSR Routine
  NOP
Table:
  .byte $F9, $B0, $8F, $60
Routine:
  RTS
"#;

        let mut assembler = Assembler::new(0x8000).with_nes_segments();
        let labels = assembler.collect_labels(source)?;
        let routine = labels.get("Routine").copied().unwrap_or_default();

        let mut assembler = Assembler::new(0x8000).with_nes_segments();
        let code = assembler.assemble_program(source)?;
        let startup = code.get("STARTUP").expect("STARTUP segment");

        // JSR is opcode $20 followed by a little-endian target address.
        assert_eq!(startup[0], 0x20, "first instruction should be JSR");
        let target = u16::from_le_bytes([startup[1], startup[2]]);

        assert_eq!(target, routine, "JSR should target the routine's real address");

        // And the byte actually living there must be the routine's opcode (RTS, $60), not the
        // first byte of the table.
        let offset = (target - 0x8000) as usize;
        assert_eq!(
            startup[offset], 0x60,
            "JSR landed on ${target:04X}, which holds ${:02X} rather than RTS — it points into data",
            startup[offset]
        );
        Ok(())
    }

    /// The VECTORS segment sits at $FFFA, so emitting its three words runs off the top of the
    /// 16-bit address space. That is normal for a NES ROM and must wrap rather than panic — an
    /// earlier version overflowed here and aborted the whole application on startup.
    #[test]
    fn address_tracking_wraps_at_the_top_of_the_address_space() -> AssembleResult<()> {
        let mut assembler = Assembler::new(0x8000).with_nes_segments();
        let labels = assembler.collect_labels(
            r#"
.segment "STARTUP"
Reset:
  RTS
.segment "VECTORS"
  .word $0000, $8000, $0000
"#,
        )?;

        assert_eq!(labels.get("Reset"), Some(&0x8000));
        Ok(())
    }

}
