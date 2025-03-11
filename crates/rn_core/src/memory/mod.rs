/// Trait for components that can be accessed via memory addresses
///
/// This trait defines how components can be accessed via memory addresses.
/// It provides methods for reading and writing to memory, as well as
/// a method for resetting the component.
pub trait Addressable {
    /// Returns true if this component handles the specified address
    ///
    /// This is used by the memory bus to determine which component
    /// should handle a read or write operation.
    fn handles_address(&self, address: u16) -> bool;

    /// Read a byte from the specified address
    ///
    /// # Arguments
    /// * `address` - The address to read from
    ///
    /// # Returns
    /// The byte read from the address
    fn read_byte(&self, address: u16) -> Result<u8, NesError>;

    /// Write a byte to the specified address
    ///
    /// # Arguments
    /// * `address` - The address to write to
    /// * `value` - The byte value to write
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError>;

    /// Read a 16-bit word (little-endian) from the specified address
    ///
    /// # Arguments
    /// * `address` - The address to read from
    ///
    /// # Returns
    /// The 16-bit word read from the address
    fn read_word(&self, address: u16) -> Result<u16, NesError> {
        let lo = self.read_byte(address)? as u16;
        let hi = self.read_byte(address.wrapping_add(1))? as u16;
        Ok((hi << 8) | lo)
    }

    /// Write a 16-bit word (little-endian) to the specified address
    ///
    /// # Arguments
    /// * `address` - The address to write to
    /// * `value` - The 16-bit word to write
    fn write_word(&mut self, address: u16, value: u16) -> Result<(), NesError> {
        let lo = (value & 0xFF) as u8;
        let hi = (value >> 8) as u8;
        self.write_byte(address, lo)?;
        self.write_byte(address.wrapping_add(1), hi)?;
        Ok(())
    }

    /// Reset the component to its initial state
    ///
    /// This is called when the system is reset.
    fn reset(&mut self) {
        // Default implementation does nothing
    }
}

/// Helper function to assert that a memory access is valid
/// 
/// This function can be used to add debug assertions for memory accesses.
/// It will panic in debug mode if the assertion fails, but will do nothing
/// in release mode.
/// 
/// # Arguments
/// * `condition` - The condition to assert
/// * `message` - The message to display if the assertion fails
/// 
/// # Example
/// ```
/// use rn_core::memory::assert_memory;
/// 
/// // Assert that the address is valid
/// assert_memory(address < 0x2000, format!("Invalid address: {:#06X}", address));
/// ```
#[inline]
pub fn assert_memory(condition: bool, message: impl Into<String>) {
    debug_assert!(condition, "{}", message.into());
}

mod ram;
pub use ram::Ram;

use crate::errors::NesError;
