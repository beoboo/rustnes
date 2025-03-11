use crate::errors::NesError;

use super::Addressable;

/// A RAM implementation for the NES
///
/// This provides a RAM implementation that can be mapped to
/// a specific address range. By default, it maps to the NES's
/// main memory region ($0000-$1FFF).
#[derive(Debug)]
pub struct Ram {
    data: Vec<u8>,
    start_address: u16,
    end_address: u16,
}

impl Ram {
    /// Create a new RAM instance mapped to a specific address range
    pub fn with_range(start_address: u16, end_address: u16) -> Self {
        if end_address < start_address {
            panic!("End address must be greater than or equal to start address");
        }

        let size = end_address as usize - start_address as usize + 1;
        Self {
            data: vec![0; size],
            start_address,
            end_address,
        }
    }

    /// Get the size of the RAM in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Get the start address of this RAM in the memory map
    pub fn start_address(&self) -> u16 {
        self.start_address
    }

    /// Get the end address of this RAM in the memory map
    pub fn end_address(&self) -> u16 {
        self.end_address
    }
}

impl Default for Ram {
    fn default() -> Self {
        Self::with_range(0x0000, 0xFFFF)
    }
}

impl Addressable for Ram {
    fn handles_address(&self, address: u16) -> bool {
        address >= self.start_address && address <= self.end_address
    }

    fn read_byte(&self, address: u16) -> Result<u8, NesError> {
        if !self.handles_address(address) {
            // This shouldn't happen with proper bus routing, but return 0 just in case
            return Err(NesError::MemoryAccessError(address));
        }

        let index = (address - self.start_address) as usize;
        Ok(self.data[index])
    }

    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError> {
        if !self.handles_address(address) {
            // This shouldn't happen with proper bus routing, but silently ignore
            return Err(NesError::MemoryAccessError(address));
        }

        let index = (address - self.start_address) as usize;
        self.data[index] = value;

        Ok(())
    }

    fn reset(&mut self) {
        for byte in &mut self.data {
            *byte = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_ram_read_write_byte() -> Result<()> {
        let mut ram = Ram::default(); // Default $0000-$1FFF

        // Test write and read
        ram.write_byte(0x1000, 0x42)?;
        assert_eq!(ram.read_byte(0x1000)?, 0x42);

        // Test different address
        ram.write_byte(0x0500, 0xFF)?;
        assert_eq!(ram.read_byte(0x0500)?, 0xFF);

        Ok(())
    }

    #[test]
    fn test_ram_read_write_word() -> Result<()> {
        let mut ram = Ram::default();

        // Test write and read word (little-endian)
        ram.write_word(0x1000, 0x1234)?;
        assert_eq!(ram.read_byte(0x1000)?, 0x34); // Low byte
        assert_eq!(ram.read_byte(0x1001)?, 0x12); // High byte
        assert_eq!(ram.read_word(0x1000)?, 0x1234);

        Ok(())
    }

    #[test]
    fn test_ram_with_custom_range() -> Result<()> {
        // Create RAM for $6000-$7FFF (8KB battery-backed RAM area)
        let mut ram = Ram::with_range(0x6000, 0x7FFF);

        // Should be 8KB in size
        assert_eq!(ram.size(), 0x2000);

        // Should handle addresses in its range
        assert!(ram.handles_address(0x6000));
        assert!(ram.handles_address(0x7000));
        assert!(ram.handles_address(0x7FFF));

        // Should not handle addresses outside its range
        assert!(!ram.handles_address(0x5FFF));
        assert!(!ram.handles_address(0x8000));

        // Should read/write correctly with the offset applied
        ram.write_byte(0x6000, 0x42)?;
        assert_eq!(ram.read_byte(0x6000)?, 0x42);

        // The internal index should be 0 for address 0x6000
        assert_eq!(ram.data[0], 0x42);

        // Write to end of range
        ram.write_byte(0x7FFF, 0xFF)?;
        assert_eq!(ram.read_byte(0x7FFF)?, 0xFF);

        // The internal index should be at the end of the data
        assert_eq!(ram.data[0x1FFF], 0xFF);

        Ok(())
    }

    #[test]
    fn test_ram_out_of_bounds() -> Result<()> {
        let mut ram = Ram::with_range(0x6000, 0x7FFF);

        // Reading out of bounds should return 0
        assert_eq!(ram.read_byte(0x5FFF)?, 0);
        assert_eq!(ram.read_byte(0x8000)?, 0);

        // Writing out of bounds should be ignored (no panic)
        ram.write_byte(0x5FFF, 0x42)?;
        ram.write_byte(0x8000, 0x42)?;

        Ok(())
    }
}
