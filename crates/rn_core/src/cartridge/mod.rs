mod loader;
mod pattern_table;

use std::path::Path;

pub use loader::{load_chr_rom, load_rom, INesHeader, Rom, RomLoadError};
pub use pattern_table::PatternTable;

/// Basic NES cartridge implementation
///
/// Handles loading and access to PRG ROM (program data) and CHR ROM (graphics data)
#[derive(Clone, Debug, Default)]
pub struct Cartridge {
    /// Pattern table containing CHR ROM data
    pattern_table: PatternTable,
    // PRG ROM and other components would go here in a complete implementation
}

impl Cartridge {
    /// Create a new empty cartridge
    pub fn new() -> Self {
        Self {
            pattern_table: PatternTable::new(),
        }
    }

    /// Load CHR ROM data from a slice
    pub fn load_chr_rom(&mut self, data: &[u8]) {
        self.pattern_table.load(data);
    }

    /// Load CHR ROM data from an iNES ROM file
    pub fn load_chr_rom_from_file(&mut self, path: &Path) -> Result<(), RomLoadError> {
        let chr_data = load_chr_rom(path)?;
        self.load_chr_rom(&chr_data);
        Ok(())
    }

    /// Read a byte from the pattern table
    pub fn read_pattern_table(&self, address: u16) -> u8 {
        self.pattern_table.read_byte(address)
    }

    /// Write a byte to the pattern table (for cartridges with CHR RAM)
    pub fn write_pattern_table(&mut self, address: u16, value: u8) {
        self.pattern_table.write_byte(address, value);
    }

    /// Get pixel data for a specific tile in the pattern table
    ///
    /// # Arguments
    ///
    /// * `tile_index` - The index of the tile (0-511) to retrieve
    ///
    /// # Returns
    ///
    /// An array of 64 values (0-3) representing the 8x8 pixel data for the tile
    pub fn get_tile_pixels(&self, tile_index: u16) -> [u8; 64] {
        self.pattern_table.get_tile_pixels(tile_index)
    }

    /// Get a single row of pixels from a tile in the pattern table
    ///
    /// # Arguments
    ///
    /// * `tile_index` - The index of the tile (0-511) to retrieve
    /// * `row` - The row within the tile (0-7)
    ///
    /// # Returns
    ///
    /// An array of 8 values (0-3) representing the pixel data for the row
    pub fn get_tile_row(&self, tile_index: u16, row: usize) -> [u8; 8] {
        self.pattern_table.get_tile_row(tile_index, row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ppu::Ppu;

    #[test]
    fn test_cartridge_pattern_table_access() {
        let mut cart = Cartridge::new();

        // Create some test data
        let test_data = [0x42; 0x2000];

        // Load the data
        cart.load_chr_rom(&test_data);

        // Read it back
        assert_eq!(cart.read_pattern_table(0x0000), 0x42);
        assert_eq!(cart.read_pattern_table(0x1FFF), 0x42);
    }

    #[test]
    fn test_load_chr_data() -> Result<(), RomLoadError> {
        // Skip this test if the ROM file is not available
        let rom_path = Path::new("../assembly/simple_sprite_test.nes");
        if !rom_path.exists() {
            return Ok(());
        }

        // Load CHR ROM data from the test file
        let chr_data = load_chr_rom(rom_path)?;

        // The CHR ROM should not be empty
        assert!(!chr_data.is_empty(), "CHR ROM data is empty");

        // Create a new cartridge and load the CHR ROM data
        let mut cartridge = Cartridge::new();
        cartridge.load_chr_rom(&chr_data);

        // Create a new PPU and connect the cartridge
        let mut ppu = Ppu::new();
        ppu.connect_cartridge(cartridge);

        // Verify we can read several addresses in the pattern table range
        for addr in [0x0000u16, 0x0100u16, 0x0800u16, 0x1000u16, 0x1F00u16] {
            let byte = ppu.read_ppu_memory(addr);
            assert!(byte != 0xFF, "Unexpected 0xFF value at 0x{:04X}", addr);
        }

        Ok(())
    }
}
