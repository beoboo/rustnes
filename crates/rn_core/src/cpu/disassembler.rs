use thiserror::Error;

use crate::errors::NesError;

use super::{AddressingMode, Instruction, InstructionDecoder, InstructionMetadata};

/// Errors that can occur during instruction disassembly
#[derive(Debug, Error)]
pub enum DisassembleError {
    #[error("Invalid opcode: {0:#04X}")]
    InvalidOpcode(u8),

    #[error("Unknown instruction type: {0:?}")]
    UnknownInstruction(Instruction),

    #[error("NES error: {0}")]
    NesError(#[from] NesError),
}

/// Result type for disassembly operations
pub type DisassembleResult<T> = Result<T, DisassembleError>;

/// Converts binary machine code into assembly language
pub struct Disassembler {
    decoder: InstructionDecoder,
}

impl Disassembler {
    /// Creates a new disassembler
    pub fn new() -> Self {
        Self {
            decoder: InstructionDecoder::new(),
        }
    }

    /// Disassembles a single byte into an instruction metadata object
    fn decode_opcode(&self, opcode: u8) -> DisassembleResult<InstructionMetadata> {
        self.decoder.decode(opcode).map_err(|err| match err {
            NesError::InvalidOpcode(op) => DisassembleError::InvalidOpcode(op),
            _ => DisassembleError::NesError(err),
        })
    }

    /// Formats the operand according to the addressing mode
    fn format_operand(&self, addressing_mode: AddressingMode, operand_bytes: &[u8]) -> String {
        match addressing_mode {
            AddressingMode::Immediate => {
                if !operand_bytes.is_empty() {
                    format!("#${:02X}", operand_bytes[0])
                } else {
                    "#$??".to_string() // Error case - missing operand
                }
            },
            AddressingMode::ZeroPage => {
                if !operand_bytes.is_empty() {
                    format!("${:02X}", operand_bytes[0])
                } else {
                    "$??".to_string() // Error case - missing operand
                }
            },
            AddressingMode::Absolute => {
                if operand_bytes.len() >= 2 {
                    format!("${:02X}{:02X}", operand_bytes[1], operand_bytes[0])
                } else {
                    "$????".to_string() // Error case - missing operand
                }
            },
            AddressingMode::Implied => "".to_string(),
            _ => format!("${:?}", addressing_mode), // Placeholder for other addressing modes
        }
    }

    /// Disassembles a single instruction at the given memory location
    ///
    /// Returns the disassembled instruction string and the number of bytes used.
    pub fn disassemble_instruction(&self, memory: &[u8], offset: usize) -> DisassembleResult<(String, usize)> {
        if memory.is_empty() || offset >= memory.len() {
            return Err(DisassembleError::NesError(NesError::MemoryAccessError(offset as u16)));
        }

        // Decode the opcode byte
        let opcode = memory[offset];
        let metadata = self.decode_opcode(opcode)?;

        // Extract operand bytes if any
        let operand_len = metadata.bytes as usize - 1; // -1 for the opcode byte
        let end_offset = std::cmp::min(offset + 1 + operand_len, memory.len());
        let operand_bytes = &memory[offset + 1..end_offset];

        // Format the operand according to addressing mode
        let operand_str = self.format_operand(metadata.addressing_mode, operand_bytes);

        // Format the full instruction
        let instruction_str = if operand_str.is_empty() {
            format!("{}", metadata.instruction)
        } else {
            format!("{} {}", metadata.instruction, operand_str)
        };

        Ok((instruction_str, metadata.bytes as usize))
    }

    /// Disassembles a range of memory into assembly instructions
    ///
    /// The result is a vector of (address, bytes, instruction) tuples.
    pub fn disassemble_program(
        &self,
        memory: &[u8],
        start_offset: usize,
        length: usize,
    ) -> Vec<(usize, Vec<u8>, String)> {
        let mut result = Vec::new();
        let mut offset = start_offset;
        let end_offset = std::cmp::min(start_offset + length, memory.len());

        while offset < end_offset {
            // Attempt to disassemble the current instruction
            match self.disassemble_instruction(memory, offset) {
                Ok((instruction_str, bytes_used)) => {
                    // Calculate actual bytes used (handle end of buffer)
                    let actual_bytes_used = std::cmp::min(bytes_used, end_offset - offset);

                    // Extract the raw bytes for this instruction
                    let raw_bytes = memory[offset..offset + actual_bytes_used].to_vec();

                    // Add this instruction to the result
                    result.push((offset, raw_bytes, instruction_str));

                    // Move to the next instruction
                    offset += actual_bytes_used;
                },
                Err(_) => {
                    // If we can't decode the instruction, treat it as data byte
                    let raw_bytes = vec![memory[offset]];
                    result.push((offset, raw_bytes, format!(".byte ${:02X}", memory[offset])));
                    offset += 1;
                },
            }
        }

        result
    }

    /// Formats a disassembled program as a string with each instruction on a new line
    ///
    /// Each line includes the address, raw bytes, and the disassembled instruction.
    pub fn format_disassembly(&self, disassembly: &[(usize, Vec<u8>, String)]) -> String {
        let mut result = String::new();

        for (addr, bytes, instruction) in disassembly {
            // Format the address as a 4-digit hex number
            let addr_str = format!("{:04X}", addr);

            // Format the raw bytes as space-separated hex values
            let bytes_str = bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");

            // Ensure the bytes column is consistent width with padding
            let padded_bytes = format!("{:8}", bytes_str);

            // Combine all parts into a single line
            result.push_str(&format!("{}: {} {}\n", addr_str, padded_bytes, instruction));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn test_disassemble_single_instruction() -> Result<()> {
        let disassembler = Disassembler::new();

        // Test LDA #$42 (A9 42)
        let memory = [0xA9, 0x42];
        let (instruction, bytes) = disassembler.disassemble_instruction(&memory, 0)?;
        assert_eq!(instruction, "LDA #$42");
        assert_eq!(bytes, 2);

        // Test STA $2000 (8D 00 20)
        let memory = [0x8D, 0x00, 0x20];
        let (instruction, bytes) = disassembler.disassemble_instruction(&memory, 0)?;
        assert_eq!(instruction, "STA $2000");
        assert_eq!(bytes, 3);

        // Test BRK (00)
        let memory = [0x00];
        let (instruction, bytes) = disassembler.disassemble_instruction(&memory, 0)?;
        assert_eq!(instruction, "BRK");
        assert_eq!(bytes, 1);

        Ok(())
    }

    #[test]
    fn test_disassemble_program() -> Result<()> {
        let disassembler = Disassembler::new();

        // Simple program: LDA #$42, STA $0200, BRK
        let memory = [0xA9, 0x42, 0x8D, 0x00, 0x02, 0x00];

        let disassembly = disassembler.disassemble_program(&memory, 0, memory.len());
        assert_eq!(disassembly.len(), 3);

        // Check individual instructions
        assert_eq!(disassembly[0].0, 0); // First instruction address
        assert_eq!(disassembly[0].1, vec![0xA9, 0x42]); // First instruction bytes
        assert_eq!(disassembly[0].2, "LDA #$42"); // First instruction text

        assert_eq!(disassembly[1].0, 2); // Second instruction address
        assert_eq!(disassembly[1].1, vec![0x8D, 0x00, 0x02]); // Second instruction bytes
        assert_eq!(disassembly[1].2, "STA $0200"); // Second instruction text

        assert_eq!(disassembly[2].0, 5); // Third instruction address
        assert_eq!(disassembly[2].1, vec![0x00]); // Third instruction bytes
        assert_eq!(disassembly[2].2, "BRK"); // Third instruction text

        Ok(())
    }

    #[test]
    fn test_format_disassembly() -> Result<()> {
        let disassembler = Disassembler::new();

        // Create a simple disassembly
        let disassembly = vec![
            (0x8000, vec![0xA9, 0x42], "LDA #$42".to_string()),
            (0x8002, vec![0x8D, 0x00, 0x02], "STA $0200".to_string()),
            (0x8005, vec![0x00], "BRK".to_string()),
        ];

        let formatted = disassembler.format_disassembly(&disassembly);

        // The output should look like:
        // 8000: A9 42    LDA #$42
        // 8002: 8D 00 02 STA $0200
        // 8005: 00       BRK
        let expected = "\
8000: A9 42    LDA #$42
8002: 8D 00 02 STA $0200
8005: 00       BRK
";

        assert_eq!(formatted, expected);

        Ok(())
    }

    #[test]
    fn test_invalid_opcode() -> Result<()> {
        let disassembler = Disassembler::new();

        // Test an invalid opcode (0xFF)
        let memory = [0xFF];
        let result = disassembler.disassemble_instruction(&memory, 0);

        // Should return an error
        assert!(result.is_err());
        if let Err(DisassembleError::InvalidOpcode(opcode)) = result {
            assert_eq!(opcode, 0xFF);
        } else {
            anyhow::bail!("Expected InvalidOpcode error, got: {:?}", result);
        }

        // But we can still disassemble a program with invalid opcodes
        let disassembly = disassembler.disassemble_program(&memory, 0, memory.len());
        assert_eq!(disassembly.len(), 1);
        assert_eq!(disassembly[0].2, ".byte $FF");

        Ok(())
    }
}
