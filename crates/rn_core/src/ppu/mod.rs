
/// The Picture Processing Unit (PPU) for the NES
/// 
/// This handles all graphics rendering for the NES system.
pub struct Ppu {
    // Memory components
    vram: [u8; 2048],        // 2KB of VRAM for nametables
    palette: [u8; 32],        // 32 bytes of palette memory
    oam: [u8; 256],           // 256 bytes of Object Attribute Memory for sprites
    
    // Registers
    ctrl: u8,                 // PPUCTRL $2000
    mask: u8,                 // PPUMASK $2001
    status: u8,               // PPUSTATUS $2002
    oam_addr: u8,             // OAMADDR $2003
    scroll_x: u8,             // First write to PPUSCROLL $2005
    scroll_y: u8,             // Second write to PPUSCROLL $2005
    ppu_addr: u16,            // PPUADDR $2006 (16-bit address)
    
    // Internal state
    read_buffer: u8,          // Internal read buffer for PPUDATA reads
    write_toggle: bool,       // Tracks whether the next write is first (false) or second (true)
    frame_count: u64,         // Total frames rendered
    scanline: i16,            // Current scanline (-1 to 261)
    cycle: u16,               // Current cycle (0 to 340)
    
    // Rendering output
    frame_buffer: Vec<u8>,    // RGB data for the current frame
}

impl Ppu {
    /// Create a new PPU instance
    pub fn new() -> Self {
        Self {
            // Initialize memory components
            vram: [0; 2048],
            palette: [0; 32],
            oam: [0; 256],
            
            // Initialize registers
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            scroll_x: 0,
            scroll_y: 0,
            ppu_addr: 0,
            
            // Initialize internal state
            read_buffer: 0,
            write_toggle: false,
            frame_count: 0,
            scanline: -1,     // Start at pre-render scanline
            cycle: 0,
            
            // Initialize frame buffer (256x240 pixels, 3 bytes per pixel for RGB)
            frame_buffer: vec![0; 256 * 240 * 3],
        }
    }
    
    /// Reset the PPU to its initial state
    pub fn reset(&mut self) {
        self.ctrl = 0;
        self.mask = 0;
        self.oam_addr = 0;
        self.write_toggle = false;
        self.scanline = -1;
        self.cycle = 0;
        // Status register bits are preserved
        // Other state is preserved
    }
    
    /// Execute a single PPU cycle
    /// 
    /// The PPU runs at 3x the speed of the CPU, so this will be called
    /// three times for each CPU cycle.
    pub fn tick(&mut self) {
        // Update cycle and scanline counters
        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = -1;
                self.frame_count += 1;
            }
        }
        
        // TODO: Implement actual rendering logic
        // This will involve sprite evaluation, background fetching, etc.
    }
    
    /// Get the current frame buffer
    pub fn frame_buffer(&self) -> &[u8] {
        &self.frame_buffer
    }
    
    // --- PPU Register Access Methods ---
    
    /// Read from a PPU register (mapped at $2000-$2007)
    pub fn read_register(&mut self, address: u16) -> u8 {
        match address & 0x7 {
            0x2 => self.read_status(),
            0x4 => self.read_oam_data(),
            0x7 => self.read_data(),
            _ => {
                // Most PPU registers are write-only
                // Reading from write-only registers returns the internal read buffer
                self.read_buffer
            }
        }
    }
    
    /// Write to a PPU register (mapped at $2000-$2007)
    pub fn write_register(&mut self, address: u16, value: u8) {
        match address & 0x7 {
            0x0 => self.write_control(value),
            0x1 => self.write_mask(value),
            0x3 => self.write_oam_address(value),
            0x4 => self.write_oam_data(value),
            0x5 => self.write_scroll(value),
            0x6 => self.write_address(value),
            0x7 => self.write_data(value),
            _ => {} // Writes to PPUSTATUS ($2002) are ignored
        }
    }
    
    // --- Individual Register Handlers ---
    
    /// Read from PPUSTATUS ($2002)
    fn read_status(&mut self) -> u8 {
        let result = self.status;
        
        // Reading status resets the write toggle
        self.write_toggle = false;
        
        // Clear bit 7 (VBlank flag) after reading
        self.status &= 0x7F;
        
        result
    }
    
    /// Read from OAMDATA ($2004)
    fn read_oam_data(&self) -> u8 {
        self.oam[self.oam_addr as usize]
    }
    
    /// Read from PPUDATA ($2007)
    fn read_data(&mut self) -> u8 {
        let addr = self.ppu_addr;
        
        // Increment address after read
        self.ppu_addr = self.ppu_addr.wrapping_add(
            if (self.ctrl & 0x04) != 0 { 32 } else { 1 }
        );
        
        // Palette memory reads are not buffered
        if addr >= 0x3F00 {
            return self.read_palette(addr);
        }
        
        // Other memory reads are buffered
        let result = self.read_buffer;
        self.read_buffer = self.read_ppu_memory(addr);
        result
    }
    
    /// Write to PPUCTRL ($2000)
    fn write_control(&mut self, value: u8) {
        self.ctrl = value;
        
        // TODO: Update internal nametable select bits from ctrl
    }
    
    /// Write to PPUMASK ($2001)
    fn write_mask(&mut self, value: u8) {
        self.mask = value;
    }
    
    /// Write to OAMADDR ($2003)
    fn write_oam_address(&mut self, value: u8) {
        self.oam_addr = value;
    }
    
    /// Write to OAMDATA ($2004)
    fn write_oam_data(&mut self, value: u8) {
        self.oam[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }
    
    /// Write to PPUSCROLL ($2005)
    fn write_scroll(&mut self, value: u8) {
        if !self.write_toggle {
            // First write: X scroll
            self.scroll_x = value;
        } else {
            // Second write: Y scroll
            self.scroll_y = value;
        }
        
        self.write_toggle = !self.write_toggle;
    }
    
    /// Write to PPUADDR ($2006)
    fn write_address(&mut self, value: u8) {
        if !self.write_toggle {
            // First write: high byte
            self.ppu_addr = (self.ppu_addr & 0x00FF) | ((value as u16) << 8);
        } else {
            // Second write: low byte
            self.ppu_addr = (self.ppu_addr & 0xFF00) | (value as u16);
        }
        
        self.write_toggle = !self.write_toggle;
    }
    
    /// Write to PPUDATA ($2007)
    fn write_data(&mut self, value: u8) {
        let addr = self.ppu_addr;
        
        // Increment address after write
        self.ppu_addr = self.ppu_addr.wrapping_add(
            if (self.ctrl & 0x04) != 0 { 32 } else { 1 }
        );
        
        self.write_ppu_memory(addr, value);
    }
    
    // --- Internal Memory Access ---
    
    /// Read from PPU address space
    fn read_ppu_memory(&self, address: u16) -> u8 {
        let addr = address & 0x3FFF;  // Mirror down to 14 bits
        
        match addr {
            0x0000..=0x1FFF => {
                // Pattern tables (CHR ROM/RAM) - External
                // TODO: Implement CHR ROM/RAM access
                0
            }
            0x2000..=0x3EFF => {
                // Nametables and mirrors
                self.read_nametable(addr)
            }
            0x3F00..=0x3FFF => {
                // Palette RAM
                self.read_palette(addr)
            }
            _ => unreachable!(),
        }
    }
    
    /// Write to PPU address space
    fn write_ppu_memory(&mut self, address: u16, value: u8) {
        let addr = address & 0x3FFF;  // Mirror down to 14 bits
        
        match addr {
            0x0000..=0x1FFF => {
                // Pattern tables (CHR ROM/RAM) - External
                // TODO: Implement CHR ROM/RAM access
            }
            0x2000..=0x3EFF => {
                // Nametables and mirrors
                self.write_nametable(addr, value);
            }
            0x3F00..=0x3FFF => {
                // Palette RAM
                self.write_palette(addr, value);
            }
            _ => unreachable!(),
        }
    }
    
    /// Read from nametable memory (including mirrors)
    fn read_nametable(&self, address: u16) -> u8 {
        // Map the address to the internal VRAM
        // Currently just implemented with a single nametable mirrored
        let addr = (address & 0x0FFF) % 0x0800;
        self.vram[addr as usize]
    }
    
    /// Write to nametable memory (including mirrors)
    fn write_nametable(&mut self, address: u16, value: u8) {
        // Map the address to the internal VRAM
        // Currently just implemented with a single nametable mirrored
        let addr = (address & 0x0FFF) % 0x0800;
        self.vram[addr as usize] = value;
    }
    
    /// Read from palette memory (including mirrors)
    fn read_palette(&self, address: u16) -> u8 {
        let addr = address & 0x001F;
        
        // Addresses $3F10, $3F14, $3F18, $3F1C are mirrors of $3F00, $3F04, $3F08, $3F0C
        let addr = match addr {
            0x10 => 0x00,
            0x14 => 0x04,
            0x18 => 0x08,
            0x1C => 0x0C,
            _ => addr,
        };
        
        self.palette[addr as usize]
    }
    
    /// Write to palette memory (including mirrors)
    fn write_palette(&mut self, address: u16, value: u8) {
        let addr = address & 0x001F;
        
        // Addresses $3F10, $3F14, $3F18, $3F1C are mirrors of $3F00, $3F04, $3F08, $3F0C
        let addr = match addr {
            0x10 => 0x00,
            0x14 => 0x04,
            0x18 => 0x08,
            0x1C => 0x0C,
            _ => addr,
        };
        
        self.palette[addr as usize] = value;
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

// Define register bit constants
pub mod registers {
    // PPUCTRL ($2000) bits
    pub const CTRL_NAMETABLE_X: u8 = 0x01;       // 0: Select nametable at $2000; 1: Select nametable at $2400
    pub const CTRL_NAMETABLE_Y: u8 = 0x02;       // 0: Select nametable at $2000; 1: Select nametable at $2800
    pub const CTRL_INCREMENT_MODE: u8 = 0x04;    // 0: Add 1; 1: Add 32
    pub const CTRL_SPRITE_PATTERN: u8 = 0x08;    // 0: $0000; 1: $1000
    pub const CTRL_BACKGROUND_PATTERN: u8 = 0x10; // 0: $0000; 1: $1000
    pub const CTRL_SPRITE_SIZE: u8 = 0x20;       // 0: 8x8; 1: 8x16
    pub const CTRL_MASTER_SLAVE: u8 = 0x40;      // Not used in NES
    pub const CTRL_NMI_ENABLE: u8 = 0x80;        // Generate NMI at start of vblank
    
    // PPUMASK ($2001) bits
    pub const MASK_GRAYSCALE: u8 = 0x01;         // 0: Color; 1: Grayscale
    pub const MASK_SHOW_LEFT_BACKGROUND: u8 = 0x02; // Show background in leftmost 8 pixels
    pub const MASK_SHOW_LEFT_SPRITES: u8 = 0x04; // Show sprites in leftmost 8 pixels
    pub const MASK_SHOW_BACKGROUND: u8 = 0x08;   // Show background
    pub const MASK_SHOW_SPRITES: u8 = 0x10;      // Show sprites
    pub const MASK_EMPHASIZE_RED: u8 = 0x20;     // Emphasize red
    pub const MASK_EMPHASIZE_GREEN: u8 = 0x40;   // Emphasize green
    pub const MASK_EMPHASIZE_BLUE: u8 = 0x80;    // Emphasize blue
    
    // PPUSTATUS ($2002) bits
    pub const STATUS_SPRITE_OVERFLOW: u8 = 0x20; // Sprite overflow occurred
    pub const STATUS_SPRITE_ZERO_HIT: u8 = 0x40; // Sprite 0 hit occurred
    pub const STATUS_VBLANK: u8 = 0x80;          // In vblank
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ppu_init() {
        let ppu = Ppu::new();
        
        // Check initial register values
        assert_eq!(ppu.ctrl, 0);
        assert_eq!(ppu.mask, 0);
        assert_eq!(ppu.status, 0);
        assert_eq!(ppu.oam_addr, 0);
        
        // Check initial internal state
        assert_eq!(ppu.write_toggle, false);
        assert_eq!(ppu.scanline, -1);
        assert_eq!(ppu.cycle, 0);
    }
    
    #[test]
    fn test_ppu_register_write_toggle() {
        let mut ppu = Ppu::new();
        
        // Write to scroll register
        ppu.write_scroll(0x12);
        assert_eq!(ppu.scroll_x, 0x12);
        assert_eq!(ppu.write_toggle, true);
        
        // Write again to scroll register
        ppu.write_scroll(0x34);
        assert_eq!(ppu.scroll_y, 0x34);
        assert_eq!(ppu.write_toggle, false);
        
        // Test reset of write toggle when reading status
        ppu.write_scroll(0x56);
        assert_eq!(ppu.write_toggle, true);
        ppu.read_status();
        assert_eq!(ppu.write_toggle, false);
    }
    
    #[test]
    fn test_ppu_oam_access() {
        let mut ppu = Ppu::new();
        
        // Write to OAM
        ppu.write_oam_address(0x10);
        ppu.write_oam_data(0xAB);
        
        // OAM address should auto-increment
        assert_eq!(ppu.oam_addr, 0x11);
        
        // Read from OAM
        ppu.write_oam_address(0x10);
        assert_eq!(ppu.read_oam_data(), 0xAB);
    }
    
    #[test]
    fn test_ppu_data_access() {
        let mut ppu = Ppu::new();
        
        // Write to VRAM
        ppu.write_address(0x20);  // High byte
        ppu.write_address(0x05);  // Low byte
        ppu.write_data(0xCD);     // Write to $2005
        
        // Address should increment by 1 (ctrl bit 2 is 0)
        assert_eq!(ppu.ppu_addr, 0x2006);
        
        // Set address increment to 32
        ppu.write_control(0x04);
        
        // Read from VRAM
        ppu.write_address(0x20);  // High byte
        ppu.write_address(0x05);  // Low byte
        
        // First read is buffered (except palette)
        let _ = ppu.read_data();
        
        // Address should increment by 32 (ctrl bit 2 is 1)
        assert_eq!(ppu.ppu_addr, 0x2025);
        
        // Second read should return the actual value
        ppu.write_address(0x20);  // High byte
        ppu.write_address(0x05);  // Low byte
        let _ = ppu.read_data();  // Buffered read
        ppu.write_address(0x20);  // High byte
        ppu.write_address(0x05);  // Low byte
        assert_eq!(ppu.read_data(), 0xCD);  // Actual read
    }
}
