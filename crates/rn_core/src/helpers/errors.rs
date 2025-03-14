use thiserror::Error;

/// Errors that can occur during parsing operations
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid operand format: {0}")]
    InvalidFormat(String),

    #[error("Value out of range: {0}")]
    ValueOutOfRange(String),
}

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;
