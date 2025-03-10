/// Addressable interface trait
///
/// This trait defines how components can be accessed via memory addresses in the NES.
/// Components can either handle the entire address space (like RAM) or specific
/// address ranges (like memory-mapped devices).
pub trait Addressable {
    /// Returns true if this component handles the specified address
    ///
    /// Default implementation returns true for all addresses, which is appropriate
    /// for components like RAM that handle the entire address space.
    ///
    /// Components that only handle specific address ranges should override this method.
    fn handles_address(&self, _address: u16) -> bool {
        true
    }

    /// Read a byte from the specified address
    ///
    /// This method should only be called for addresses where `handles_address`
    /// returns true. Implementations can assume the address is valid.
    ///
    /// # Side Effects
    ///
    /// Reading from certain addresses may have side effects in hardware devices.
    /// For example, reading from certain PPU registers clears status flags.
    fn read_byte(&self, address: u16) -> u8;

    /// Write a byte to the specified address
    ///
    /// This method should only be called for addresses where `handles_address`
    /// returns true. Implementations can assume the address is valid.
    ///
    /// # Side Effects
    ///
    /// Writing to certain addresses may have side effects in hardware devices.
    /// For example, writing to certain registers might trigger DMA transfers.
    fn write_byte(&mut self, address: u16, value: u8);

    /// Read a word (16-bits) from the specified address
    /// NES is little-endian, so the lower byte is at the lower address
    fn read_word(&self, address: u16) -> u16 {
        let low = self.read_byte(address) as u16;
        let high = self.read_byte(address.wrapping_add(1)) as u16;
        (high << 8) | low
    }

    /// Write a word (16-bits) to the specified address
    fn write_word(&mut self, address: u16, value: u16) {
        let low = (value & 0xFF) as u8;
        let high = (value >> 8) as u8;
        self.write_byte(address, low);
        self.write_byte(address.wrapping_add(1), high);
    }

    /// Reset the component to its initial state
    ///
    /// This method is called when the system is reset. The default
    /// implementation does nothing.
    fn reset(&mut self) {}
}

// Submodules and re-exports
mod bus;
mod ram;

pub use bus::Bus;
pub use ram::Ram;
