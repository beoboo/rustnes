use thiserror::Error;

use crate::cpu::InstructionDecoderError;

/// Error type for memory-related operations
#[derive(Debug, Error)]
pub enum NesError {
    /// Error for invalid or unimplemented opcodes
    #[error("Invalid opcode: {0:#04X}")]
    InvalidOpcode(u8),

    /// Error for addressing mode not implemented for a specific instruction
    #[error("Unimplemented addressing mode for instruction")]
    UnimplementedAddressingMode,

    /// Error for memory access issues
    #[error("Memory access error at address {0:#06X}")]
    MemoryAccessError(u16),

    /// Error for invalid memory operations
    #[error("Invalid memory operation at address {0:#06X}: {1}")]
    InvalidMemoryOperation(u16, String),

    /// Generic error
    #[error("Generic error: {0}")]
    GenericError(String),
}

impl From<InstructionDecoderError> for NesError {
    fn from(error: InstructionDecoderError) -> Self {
        match error {
            InstructionDecoderError::InvalidOpcode(opcode) => NesError::InvalidOpcode(opcode),
            InstructionDecoderError::UnimplementedAddressingMode => NesError::UnimplementedAddressingMode,
        }
    }
}
