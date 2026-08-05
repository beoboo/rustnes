mod ram;
use std::fmt::Debug;

pub use ram::Ram;

use crate::errors::NesError;

/// Trait for components that can be accessed via memory addresses
///
/// This trait defines how components can be accessed via memory addresses.
/// It provides methods for reading and writing to memory, as well as
/// a method for resetting the component.
pub trait Addressable: Debug {
    /// Returns true if this component handles the specified address
    ///
    /// This is used by the memory bus to determine which component
    /// should handle a read or write operation.
    fn handles_address(&self, address: u16) -> bool;

    /// Returns true if this component handles *writes* to the specified address.
    ///
    /// Defaults to [`Addressable::handles_address`], which is right for almost everything. It is
    /// overridden for registers the hardware splits by direction: writing `$4017` sets the APU's
    /// frame counter while reading it returns controller 2, and a single address-to-component
    /// mapping cannot express that. Without this, whichever component is attached first silently
    /// swallows the other's half of the register.
    fn handles_write(&self, address: u16) -> bool {
        self.handles_address(address)
    }

    /// Read without any of the consequences of reading.
    ///
    /// A real read is an event: it moves the value on the open bus, clears `$2002`'s vblank flag,
    /// steps `$2007`'s address, acknowledges the frame IRQ. A peek is for looking — a debugger
    /// window, a test runner checking a status byte — and must leave the machine exactly as it
    /// found it, because the thing being inspected is usually the thing under test.
    ///
    /// The default forwards to [`read_byte`](Self::read_byte), which is right for storage and wrong
    /// for anything with a side effect; those override it. The bus's own peek is what keeps the
    /// open bus out of it, so a component only has to worry about its own registers.
    fn peek_byte(&self, address: u16) -> Result<u8, NesError> {
        self.read_byte(address)
    }

    /// Which bits of a read from `address` this component does not drive.
    ///
    /// Zero for almost everything — a component that answers an address usually drives all eight
    /// lines. The controller ports do not: `$4016` and `$4017` put the shift register's output on
    /// the bottom bits and leave the top three floating, so those keep whatever the bus last
    /// carried. Reading `$4016` right after a `JMP $4016` therefore returns `$40`, the jump's own
    /// high address byte, and `cpu_exec_space/test_cpu_exec_space_apu` is built on that: it
    /// executes the byte it gets back, and `$40` is `RTI`.
    fn open_bus_mask(&self, _address: u16) -> u8 {
        0
    }

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
