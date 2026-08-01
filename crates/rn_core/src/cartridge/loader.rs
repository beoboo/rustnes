use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

/// Constants for iNES ROM format
const INES_HEADER_SIZE: usize = 16;
const INES_MAGIC: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A]; // "NES" followed by MS-DOS EOF

/// Error types for ROM loading
#[derive(Debug)]
pub enum RomLoadError {
    IoError(io::Error),
    InvalidFormat(&'static str),
}

impl std::fmt::Display for RomLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(error) => write!(f, "{error}"),
            Self::InvalidFormat(reason) => write!(f, "{reason}"),
        }
    }
}

impl std::error::Error for RomLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(error) => Some(error),
            Self::InvalidFormat(_) => None,
        }
    }
}

impl From<io::Error> for RomLoadError {
    fn from(error: io::Error) -> Self {
        RomLoadError::IoError(error)
    }
}

/// Simple iNES ROM header representation
#[derive(Debug)]
pub struct INesHeader {
    pub prg_rom_size: usize, // Size of PRG ROM in 16 KB units
    pub chr_rom_size: usize, // Size of CHR ROM in 8 KB units
    pub mapper: u8,          // Mapper number
    pub mirroring: bool,     // true = vertical, false = horizontal
    pub battery: bool,       // Has battery-backed RAM
    pub trainer: bool,       // Has trainer
    pub four_screen: bool,   // Four-screen VRAM layout
}

/// A loaded iNES ROM: its header plus both memory images.
///
/// PRG-ROM is the program the CPU executes and CHR-ROM the pattern tables the PPU draws from.
/// Previously only CHR was returned and the PRG data was read purely to seek past it, which meant
/// no `.nes` file could actually be run.
#[derive(Debug)]
pub struct Rom {
    pub header: INesHeader,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}

impl Rom {
    /// Read the byte the CPU sees at `address` in `$8000..=$FFFF`.
    ///
    /// A 16 KB PRG image is mirrored into both halves of the range, which is what NROM-128 does
    /// and what makes the reset vector at `$FFFC` resolve for a single-bank ROM.
    pub fn read_prg(&self, address: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        let offset = (address as usize).saturating_sub(0x8000);
        self.prg_rom[offset % self.prg_rom.len()]
    }

    /// The reset vector at `$FFFC`, where execution begins.
    pub fn reset_vector(&self) -> u16 {
        u16::from_le_bytes([self.read_prg(0xFFFC), self.read_prg(0xFFFD)])
    }
}

/// Load a complete iNES ROM: header, PRG-ROM and CHR-ROM.
pub fn load_rom(path: &Path) -> Result<Rom, RomLoadError> {
    let mut file = File::open(path)?;

    // Read the header only if the file is long enough to have one. Letting `read_exact` fail
    // instead reports "failed to fill whole buffer", which says nothing about the file not being
    // a ROM.
    let mut header = [0u8; INES_HEADER_SIZE];
    if file.read(&mut header)? < INES_HEADER_SIZE {
        return Err(RomLoadError::InvalidFormat(
            "file is too small to be an iNES ROM (under 16 bytes)",
        ));
    }

    if header[0..4] != INES_MAGIC {
        return Err(RomLoadError::InvalidFormat(
            "not an iNES ROM (missing the \"NES\\x1A\" signature)",
        ));
    }

    let parsed = parse_ines_header(&header)?;

    // A trainer, when present, sits between the header and the PRG data.
    if parsed.trainer {
        let mut trainer = [0u8; 512];
        file.read_exact(&mut trainer)?;
    }

    let mut prg_rom = vec![0u8; parsed.prg_rom_size * 16 * 1024];
    file.read_exact(&mut prg_rom)?;

    // A CHR size of zero means the cartridge uses CHR RAM rather than ROM.
    let chr_rom_size = parsed.chr_rom_size * 8 * 1024;
    let chr_rom = if chr_rom_size == 0 {
        vec![0u8; 8192]
    } else {
        let mut chr_rom = vec![0u8; chr_rom_size];
        file.read_exact(&mut chr_rom)?;
        chr_rom
    };

    Ok(Rom {
        header: parsed,
        prg_rom,
        chr_rom,
    })
}

/// Load CHR ROM data from an iNES format file
pub fn load_chr_rom(path: &Path) -> Result<Vec<u8>, RomLoadError> {
    // Open the file
    let mut file = File::open(path)?;

    // Read the header (16 bytes)
    let mut header = [0u8; INES_HEADER_SIZE];
    file.read_exact(&mut header)?;

    // Verify the magic number ("NES" followed by MS-DOS EOF)
    if header[0..4] != INES_MAGIC {
        return Err(RomLoadError::InvalidFormat("Not a valid iNES ROM file"));
    }

    // Parse the header
    let parsed_header = parse_ines_header(&header)?;

    // Skip the trainer if present (512 bytes)
    if parsed_header.trainer {
        let mut trainer = [0u8; 512];
        file.read_exact(&mut trainer)?;
    }

    // Skip the PRG ROM data
    let prg_rom_size = parsed_header.prg_rom_size * 16 * 1024; // Convert to bytes
    let mut prg_rom = vec![0u8; prg_rom_size];
    file.read_exact(&mut prg_rom)?;

    // Read the CHR ROM data
    let chr_rom_size = parsed_header.chr_rom_size * 8 * 1024; // Convert to bytes

    // If CHR ROM size is 0, it means the game uses CHR RAM
    if chr_rom_size == 0 {
        return Ok(vec![0u8; 8192]); // Return 8KB of zeros for CHR RAM
    }

    let mut chr_rom = vec![0u8; chr_rom_size];
    file.read_exact(&mut chr_rom)?;

    Ok(chr_rom)
}

/// Parse an iNES header into a structured format
fn parse_ines_header(header: &[u8; INES_HEADER_SIZE]) -> Result<INesHeader, RomLoadError> {
    // Validate header size
    if header.len() < INES_HEADER_SIZE {
        return Err(RomLoadError::InvalidFormat("Header too small"));
    }

    // Extract basic information
    let prg_rom_size = header[4] as usize;
    let chr_rom_size = header[5] as usize;

    // Extract flags
    let mirroring = (header[6] & 0x01) != 0;
    let battery = (header[6] & 0x02) != 0;
    let trainer = (header[6] & 0x04) != 0;
    let four_screen = (header[6] & 0x08) != 0;

    // Extract mapper number (low and high nibbles)
    let mapper_low = (header[6] & 0xF0) >> 4;
    let mapper_high = header[7] & 0xF0;
    let mapper = mapper_high | mapper_low;

    Ok(INesHeader {
        prg_rom_size,
        chr_rom_size,
        mapper,
        mirroring,
        battery,
        trainer,
        four_screen,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ines_header() {
        // Create a mock iNES header
        let mut header = [0u8; INES_HEADER_SIZE];

        // Set magic number
        header[0..4].copy_from_slice(&INES_MAGIC);

        // Set PRG ROM size (2 * 16KB = 32KB)
        header[4] = 2;

        // Set CHR ROM size (1 * 8KB = 8KB)
        header[5] = 1;

        // Set flags (mapper 1, vertical mirroring, no battery, no trainer)
        header[6] = 0b00010001; // mapper low nibble: 0001, vertical mirroring: 1
        header[7] = 0b00000000; // mapper high nibble: 0000

        // Parse the header
        let parsed = parse_ines_header(&header).unwrap();

        // Verify the parsed data
        assert_eq!(parsed.prg_rom_size, 2);
        assert_eq!(parsed.chr_rom_size, 1);
        assert_eq!(parsed.mapper, 1);
        assert!(parsed.mirroring);
        assert!(!parsed.battery);
        assert!(!parsed.trainer);
        assert!(!parsed.four_screen);
    }
}
