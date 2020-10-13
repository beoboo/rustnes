use crate::types::Byte;

#[derive(Debug, Clone)]
pub struct RomHeader {
    pub nes: String,
    pub prg_rom_size: usize,
    pub chr_rom_size: usize,
    pub mapper: Byte,
}

impl RomHeader {
    pub fn new(prg_rom_size: usize, chr_rom_size: usize) -> RomHeader {
        RomHeader {
            nes: "NES".to_string(),
            prg_rom_size,
            chr_rom_size,
            mapper: 0,
        }
    }

    pub fn load(buffer: &[u8]) -> RomHeader {
        let flag6 = buffer[6];
        let flag7 = buffer[7];

        let mapper = ((flag6 & 0xF0) >> 4) | (flag7 & 0xF0);
        println!("Mapper: {}", mapper);

        RomHeader {
            nes: String::from_utf8_lossy(&buffer[0..3]).to_string(),
            prg_rom_size: buffer[4] as usize,
            chr_rom_size: buffer[5] as usize,
            mapper,
        }
    }

    pub fn len(&self) -> usize {
        16
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn load() {
        let header = RomHeader::load(&[0x4e, 0x45, 0x53, 0x1a, 0x01, 0x01, 0xF0, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        assert_that!(header.len(), eq(16));
        assert_that!(header.nes, eq("NES"));
        assert_that!(header.prg_rom_size, eq(1));
        assert_that!(header.chr_rom_size, eq(1));
        assert_that!(header.mapper, eq(0xFF));
    }
}