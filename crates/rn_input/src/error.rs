use rn_core::errors::NesError;
use thiserror::Error;

/// Error type for input-related operations
#[derive(Debug, Error)]
pub enum InputError {
    /// Error for profile not found issues
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    /// Error for invalid key mappings
    #[error("Invalid key mapping: {0}")]
    InvalidKeyMapping(String),

    /// Error for serialization
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Error for deserialization
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Error from the core emulator
    #[error("NES core error: {0}")]
    CoreError(#[from] NesError),

    /// Generic error
    #[error("Input error: {0}")]
    GenericError(String),
}

impl From<InputError> for NesError {
    fn from(value: InputError) -> Self {
        match value {
            InputError::ProfileNotFound(msg) => NesError::InputError(format!("Profile not found: {}", msg)),
            InputError::InvalidKeyMapping(msg) => NesError::InputError(format!("Invalid key mapping: {}", msg)),
            InputError::SerializationError(msg) => NesError::InputError(format!("Serialization error: {}", msg)),
            InputError::DeserializationError(msg) => NesError::InputError(format!("Deserialization error: {}", msg)),
            InputError::CoreError(err) => err,
            InputError::GenericError(msg) => NesError::InputError(msg),
        }
    }
}
