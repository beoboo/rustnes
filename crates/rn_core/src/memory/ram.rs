use super::Addressable;
use crate::errors::NesError;

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

    /// RAM that answers a larger address range than it has storage for, repeating within it.
    ///
    /// The NES has two kilobytes of work RAM and decodes only eleven address lines for it, so the
    /// same two kilobytes answer four times over across `$0000-$1FFF`. A program writing `$0000`
    /// and reading `$0800` gets the byte it wrote.
    ///
    /// Nothing much depends on it — a game has no reason to use the mirrors, and eight kilobytes of
    /// flat storage behaves identically until something looks — which is why this went unnoticed
    /// until `ppu_read_buffer` looked, in its test 60.
    pub fn mirrored(start_address: u16, end_address: u16, size: usize) -> Self {
        assert!(size > 0, "mirrored RAM needs some storage to mirror");
        if end_address < start_address {
            panic!("End address must be greater than or equal to start address");
        }

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

        // Modulo the storage, which is what makes a mirrored range repeat. For RAM whose storage
        // matches its range — every other use of this type — it changes nothing.
        let index = (address - self.start_address) as usize % self.data.len();
        Ok(self.data[index])
    }

    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError> {
        if !self.handles_address(address) {
            // This shouldn't happen with proper bus routing, but silently ignore
            return Err(NesError::MemoryAccessError(address));
        }

        let index = (address - self.start_address) as usize % self.data.len();
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
    use anyhow::Result;

    use super::*;

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

        // Reading out of bounds should return an error
        let read_result_low = ram.read_byte(0x5FFF);
        let read_result_high = ram.read_byte(0x8000);

        assert!(read_result_low.is_err());
        assert!(read_result_high.is_err());

        if let Err(NesError::MemoryAccessError(addr)) = read_result_low {
            assert_eq!(addr, 0x5FFF);
        } else {
            panic!("Expected MemoryAccessError for low address");
        }

        if let Err(NesError::MemoryAccessError(addr)) = read_result_high {
            assert_eq!(addr, 0x8000);
        } else {
            panic!("Expected MemoryAccessError for high address");
        }

        // Writing out of bounds should return an error
        let write_result_low = ram.write_byte(0x5FFF, 0x42);
        let write_result_high = ram.write_byte(0x8000, 0x42);

        assert!(write_result_low.is_err());
        assert!(write_result_high.is_err());

        if let Err(NesError::MemoryAccessError(addr)) = write_result_low {
            assert_eq!(addr, 0x5FFF);
        } else {
            panic!("Expected MemoryAccessError for low address");
        }

        if let Err(NesError::MemoryAccessError(addr)) = write_result_high {
            assert_eq!(addr, 0x8000);
        } else {
            panic!("Expected MemoryAccessError for high address");
        }

        Ok(())
    }

    /// Work RAM answers four times across $0000-$1FFF, because only eleven address lines reach it.
    ///
    /// A program writing $0000 and reading $0800 gets the byte it wrote. Nothing much depends on
    /// it — a game has no reason to touch the mirrors — which is why eight flat kilobytes behaved
    /// identically until `ppu_read_buffer` looked, in its test 60.
    #[test]
    fn mirrored_ram_repeats_across_its_range() -> Result<(), NesError> {
        let mut ram = Ram::mirrored(0x0000, 0x1FFF, 2 * 1024);

        ram.write_byte(0x0000, 0x5A)?;
        for mirror in [0x0800, 0x1000, 0x1800] {
            assert_eq!(ram.read_byte(mirror)?, 0x5A, "${mirror:04X} is the same storage as $0000");
        }

        // And back the other way: writing through a mirror is writing the original.
        ram.write_byte(0x1801, 0x3C)?;
        assert_eq!(ram.read_byte(0x0001)?, 0x3C);

        // The last byte of the physical RAM and the last byte of the range are the same one.
        ram.write_byte(0x07FF, 0x11)?;
        assert_eq!(ram.read_byte(0x1FFF)?, 0x11);

        Ok(())
    }

    /// RAM whose storage matches its range is unaffected: every address is its own byte.
    #[test]
    fn unmirrored_ram_gives_every_address_its_own_byte() -> Result<(), NesError> {
        let mut ram = Ram::with_range(0x0000, 0x1FFF);

        ram.write_byte(0x0000, 0x5A)?;
        ram.write_byte(0x0800, 0xA5)?;

        assert_eq!(ram.read_byte(0x0000)?, 0x5A, "not mirrored, so these are different bytes");
        assert_eq!(ram.read_byte(0x0800)?, 0xA5);

        Ok(())
    }
}
