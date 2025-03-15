use thiserror::Error;

use crate::cpu::InstructionDecoderError;

/// Error type for memory-related operations
#[derive(Debug, Error)]
pub enum NesError {
    /// Error for memory access issues
    #[error("Memory access error at address {0:#06X}")]
    MemoryAccessError(u16),

    /// Error for invalid memory operations
    #[error("Invalid memory operation at address {0:#06X}: {1}")]
    InvalidMemoryOperation(u16, String),

    /// Generic error
    #[error("Generic error: {0}")]
    GenericError(String),

    #[error("Instruction decoder error: {0}")]
    InstructionDecoderError(#[from] InstructionDecoderError),

    #[error("Memory not connected")]
    MemoryNotConnected,

    #[error("Cartridge not connected")]
    CartridgeNotConnected,
}
