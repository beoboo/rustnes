use thiserror::Error;

/// CPU-related errors
#[derive(Debug, Error)]
pub enum CpuError {
    /// Error for invalid or unimplemented opcodes
    #[error("Invalid opcode: {0:#04X}")]
    InvalidOpcode(u8),

    /// Error for addressing mode not implemented for a specific instruction
    #[error("Unimplemented addressing mode for instruction")]
    UnimplementedAddressingMode,

    /// Error for memory access issues
    #[error("Memory access error at address {0:#06X}")]
    MemoryAccessError(u16),
}
