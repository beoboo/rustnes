/// Pattern table memory implementation for the NES
///
/// This represents the CHR ROM/RAM data that contains the sprite and background patterns.
/// The NES has two pattern tables, each 4KB in size:
/// - 0x0000-0x0FFF: Pattern table 0
/// - 0x1000-0x1FFF: Pattern table 1
///
/// Each 8x8 tile takes up 16 bytes (8 bytes for the low bit plane, 8 bytes for the high bit plane)
#[derive(Clone)]
pub struct PatternTable {
    /// The raw pattern table data (8KB)
    data: Vec<u8>,
}

impl PatternTable {
    /// Create a new empty pattern table
    pub fn new() -> Self {
        Self {
            // Initialize with 8KB of zeros
            data: vec![0; 0x2000],
        }
    }

    /// Create a pattern table from existing data
    pub fn from_data(data: Vec<u8>) -> Self {
        // Ensure the data is exactly 8KB
        assert_eq!(data.len(), 0x2000, "Pattern table data must be 8KB");
        Self { data }
    }

    /// Read a byte from the pattern table
    pub fn read_byte(&self, address: u16) -> u8 {
        let addr = address & 0x1FFF; // Ensure address is within range
        self.data[addr as usize]
    }

    /// Write a byte to the pattern table (for CHR-RAM support)
    pub fn write_byte(&mut self, address: u16, value: u8) {
        let addr = address & 0x1FFF; // Ensure address is within range
        self.data[addr as usize] = value;
    }

    /// Load pattern table data from a slice
    pub fn load(&mut self, data: &[u8]) {
        // If less than 8KB is provided, only load what's available
        let len = std::cmp::min(data.len(), 0x2000);
        self.data[..len].copy_from_slice(&data[..len]);
    }

    /// Get the pixel data for a specific tile in the pattern table.
    ///
    /// # Arguments
    ///
    /// * `tile_index` - The index of the tile (0-511) to retrieve
    ///
    /// # Returns
    ///
    /// An array of 64 values (0-3) representing the 8x8 pixel data for the tile.
    /// Each value is a 2-bit number formed by combining the corresponding bits
    /// from the low and high bit planes.
    pub fn get_tile_pixels(&self, tile_index: u16) -> [u8; 64] {
        // Calculate the address of the tile in pattern memory
        // Each tile is 16 bytes (8 bytes low plane + 8 bytes high plane)
        let tile_addr = (tile_index as usize) * 16;

        // Initialize the pixel data array
        let mut pixels = [0u8; 64];

        // Process each row of the tile (8 rows)
        for row in 0..8 {
            // Get the byte from the low bit plane for this row
            let low_byte = self.data[tile_addr + row];

            // Get the byte from the high bit plane for this row
            let high_byte = self.data[tile_addr + row + 8];

            #[cfg(test)]
            println!("Row {}: low_byte={:08b}, high_byte={:08b}", row, low_byte, high_byte);

            // Process each bit in the row (8 bits, from MSB to LSB)
            for bit in 0..8 {
                // Calculate pixel value by combining bits from both planes
                // Bit 0 from low plane, Bit 1 from high plane
                let low_bit = (low_byte >> (7 - bit)) & 0x01;
                let high_bit = (high_byte >> (7 - bit)) & 0x01;

                // Combine the bits (high bit in bit position 1, low bit in bit position 0)
                let pixel_value = (high_bit << 1) | low_bit;

                #[cfg(test)]
                if row == 0 && (bit == 0 || bit == 7) {
                    println!(
                        "Row {}, Bit {}: low_bit={}, high_bit={}, pixel_value={}",
                        row, bit, low_bit, high_bit, pixel_value
                    );
                }

                // Store the pixel value in the array
                pixels[row * 8 + bit] = pixel_value;
            }
        }

        pixels
    }

    /// Get a single 8x1 row of pixels from a tile
    ///
    /// This is useful for sprite rendering when you need to access a specific row.
    ///
    /// # Arguments
    ///
    /// * `tile_index` - The index of the tile (0-511) to retrieve
    /// * `row` - The row within the tile (0-7)
    ///
    /// # Returns
    ///
    /// An array of 8 values (0-3) representing the pixel data for the row.
    pub fn get_tile_row(&self, tile_index: u16, row: usize) -> [u8; 8] {
        // Calculate the address of the tile in pattern memory
        let tile_addr = (tile_index as usize) * 16;

        // Get the bytes for this row from both planes
        let low_byte = self.data[tile_addr + row];
        let high_byte = self.data[tile_addr + row + 8];

        // Process each pixel in the row
        let mut row_pixels = [0u8; 8];
        for bit in 0..8 {
            // Extract and combine the bits
            let low_bit = (low_byte >> (7 - bit)) & 0x01;
            let high_bit = (high_byte >> (7 - bit)) & 0x01;
            row_pixels[bit] = (high_bit << 1) | low_bit;
        }

        row_pixels
    }
}

impl Default for PatternTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_table_new() {
        let pt = PatternTable::new();
        assert_eq!(pt.data.len(), 0x2000);
        assert_eq!(pt.data[0], 0);
    }

    #[test]
    fn test_pattern_table_read_write() {
        let mut pt = PatternTable::new();

        // Write a value
        pt.write_byte(0x1234, 0xAB);

        // Read it back
        assert_eq!(pt.read_byte(0x1234), 0xAB);

        // Check address masking
        assert_eq!(pt.read_byte(0x3234), 0xAB); // 0x3234 & 0x1FFF = 0x1234
    }

    #[test]
    fn test_pattern_table_load() {
        let mut pt = PatternTable::new();
        let test_data = [0xAA; 0x1000]; // 4KB of 0xAA

        pt.load(&test_data);

        // First 4KB should be 0xAA, rest should be 0
        assert_eq!(pt.read_byte(0x0000), 0xAA);
        assert_eq!(pt.read_byte(0x0FFF), 0xAA);
        assert_eq!(pt.read_byte(0x1000), 0);
    }

    #[test]
    fn test_get_tile_pixels() {
        let mut pt = PatternTable::new();

        println!("\nSetting up test pattern...");

        // Create a simple test pattern for a single tile
        // This pattern will be a simple square outline:
        // ■ ■ ■ ■ ■ ■ ■ ■
        // ■             ■
        // ■             ■
        // ■             ■
        // ■             ■
        // ■             ■
        // ■             ■
        // ■ ■ ■ ■ ■ ■ ■ ■

        // Low bit plane: All bits are 1 for top and bottom rows, and the first and last bits
        // for middle rows (10000001)
        pt.write_byte(0, 0xFF); // Row 1: 11111111
        pt.write_byte(1, 0x81); // Row 2: 10000001
        pt.write_byte(2, 0x81); // Row 3: 10000001
        pt.write_byte(3, 0x81); // Row 4: 10000001
        pt.write_byte(4, 0x81); // Row 5: 10000001
        pt.write_byte(5, 0x81); // Row 6: 10000001
        pt.write_byte(6, 0x81); // Row 7: 10000001
        pt.write_byte(7, 0xFF); // Row 8: 11111111

        // High bit plane: Set only the corners to 1 to give them color 3
        pt.write_byte(8, 0x81); // Row 1: 10000001 (corners only)
        pt.write_byte(9, 0x81); // Row 2: 10000001 (corners only)
        pt.write_byte(10, 0); // Row 3: 00000000
        pt.write_byte(11, 0); // Row 4: 00000000
        pt.write_byte(12, 0); // Row 5: 00000000
        pt.write_byte(13, 0); // Row 6: 00000000
        pt.write_byte(14, 0x81); // Row 7: 10000001 (corners only)
        pt.write_byte(15, 0x81); // Row 8: 10000001 (corners only)

        // Verify the data was written correctly
        println!("\nVerifying data was written correctly...");
        println!("Low plane row 1: {:08b}", pt.read_byte(0));
        println!("High plane row 1: {:08b}", pt.read_byte(8));

        // Get the full tile's pixel data
        println!("\nGetting tile pixels...");
        let pixels = pt.get_tile_pixels(0);

        // Display the first row of pixels
        print!("First row of pixels: ");
        for i in 0..8 {
            print!("{} ", pixels[i]);
        }
        println!();

        // Check the pattern matches what we expect
        println!("\nChecking expected values...");

        // Row 1: corners should be 3, rest should be 1
        println!(
            "Checking first row corners: pixels[0]={}, pixels[7]={}",
            pixels[0], pixels[7]
        );
        assert_eq!(pixels[0], 3, "Top-left corner should be 3"); // top-left: low=1, high=1 -> 3
        assert_eq!(pixels[1], 1); // top row: low=1, high=0 -> 1
        assert_eq!(pixels[6], 1); // top row: low=1, high=0 -> 1
        assert_eq!(pixels[7], 3, "Top-right corner should be 3"); // top-right: low=1, high=1 -> 3

        // Middle rows: edges should be 1, inside should be 0
        println!("\nChecking middle rows...");
        assert_eq!(pixels[8], 3, "Row 1 left edge should be 3"); // left edge: low=1, high=1 -> 3
        assert_eq!(pixels[9], 0, "Inside should be 0"); // inside: low=0, high=0 -> 0
        assert_eq!(pixels[15], 3, "Row 1 right edge should be 3"); // right edge: low=1, high=1 -> 3

        // Last row: corners should be 3, rest should be 1
        println!("\nChecking last row...");
        println!("Bottom corners: pixels[56]={}, pixels[63]={}", pixels[56], pixels[63]);
        assert_eq!(pixels[56], 3, "Bottom-left corner should be 3"); // bottom-left: low=1, high=1 -> 3
        assert_eq!(pixels[57], 1, "Bottom row should be 1"); // bottom row: low=1, high=0 -> 1
        assert_eq!(pixels[62], 1, "Bottom row should be 1"); // bottom row: low=1, high=0 -> 1
        assert_eq!(pixels[63], 3, "Bottom-right corner should be 3"); // bottom-right: low=1, high=1 -> 3

        // Test getting a single row
        let row_pixels = pt.get_tile_row(0, 0);
        assert_eq!(row_pixels, [3, 1, 1, 1, 1, 1, 1, 3]); // First row

        let row_pixels = pt.get_tile_row(0, 7);
        assert_eq!(row_pixels, [3, 1, 1, 1, 1, 1, 1, 3]); // Last row

        let row_pixels = pt.get_tile_row(0, 3);
        assert_eq!(row_pixels, [1, 0, 0, 0, 0, 0, 0, 1]); // Middle row
    }
}
