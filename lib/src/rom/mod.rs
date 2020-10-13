mod rom_header;

use std::fs::File;
use std::io::prelude::*;
use std::str;

use log::trace;

use crate::types::{Byte, Word};
use crate::helpers::replace_slice;
use crate::rom::rom_header::RomHeader;

#[derive(Debug, Clone)]
pub struct Rom {
    pub header: RomHeader,
    pub trainer: Vec<u8>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}

impl Default for Rom {
    fn default() -> Self {
        let prg_rom_size = 16384;
        let chr_rom_size = 8192;
        let header = RomHeader::new(prg_rom_size, chr_rom_size);

        Rom {
            header,
            trainer: vec![],
            prg_rom: vec![0; prg_rom_size],
            chr_rom: vec![0; chr_rom_size],
        }
    }
}

impl Rom {
    pub fn load_bytes(&mut self, prg_rom: &[u8], chr_rom: &[u8]) {
        replace_slice(&mut self.prg_rom[..], prg_rom);
        replace_slice(&mut self.chr_rom[..], chr_rom);
    }

    pub fn from_file(filename: &str, prg_bank_size: usize, chr_bank_size: usize) -> Rom {
        let buffer = Rom::load_file(filename);
        Self::from_bytes(buffer.as_slice(), prg_bank_size, chr_bank_size)
    }

    pub fn from_bytes(bytes: &[Byte], prg_bank_size: usize, chr_bank_size: usize) -> Rom {
        let header = RomHeader::load(&bytes[0..16]);
        trace!("Header: {:?}", header);

        let prg_rom_start = header.len();
        let prg_rom_end = prg_rom_start + prg_bank_size * header.prg_rom_size;
        let chr_rom_start = prg_rom_end;
        let chr_rom_end = chr_rom_start + chr_bank_size * header.chr_rom_size;

        Rom {
            header,
            trainer: vec![],
            prg_rom: bytes[prg_rom_start..prg_rom_end].to_vec(),
            chr_rom: bytes[chr_rom_start..chr_rom_end].to_vec(),
        }
    }

    pub fn read(&self, start: Word, end: Word) -> &[Byte] {
        let start = self.remap(start);
        let end = self.remap(end);

        trace!("[Rom::read] Reading from {:#06X}:{:#06X}", start, end);
        let data = &self.prg_rom[start as usize..=end as usize];
        data
    }

    pub fn read_byte(&self, address: Word) -> Byte {
        let address = self.remap(address);

        let data = self.prg_rom[address];
        trace!("[Rom::read_byte] Reading byte from {:#06X} -> {:#04X}", address, data);
        data
    }

    fn remap(&self, address: Word) -> usize {
        let address = address as usize;

        match address {
            0x4000..=0x7FFF if self.header.prg_rom_size <= address => address - 0x4000,
            0x8000..=0xFFFF => panic!("Invalid ROM address: {:#06X}", address),
            _ => address
        }
    }

    fn load_file(filename: &str) -> Vec<u8> {
        let mut file = match File::open(filename) {
            Ok(file) => file,
            Err(e) => panic!("Could not open {} ({})", filename, e)
        };

        // read the same file back into a Vec of bytes
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        buffer
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::path::Path;

    use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn read_header() {
        make_dir("tmp");
        let filename = "tmp/rom.nes";
        save_file(filename, &[0x4e, 0x45, 0x53, 0x1a, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        let rom = Rom::from_file(filename, 1, 1);

        assert_that!(rom.header.len(), eq(16));
        assert_that!(rom.header.nes, eq("NES"));
        assert_that!(rom.header.prg_rom_size, eq(1));
        assert_that!(rom.header.chr_rom_size, eq(1));
        assert_that!(rom.prg_rom.len(), eq(1));
        assert_that!(rom.chr_rom.len(), eq(1));
    }

    #[test]
    fn read_valid() {
        let rom = Rom::from_file("../roms/cpu/nestest/nestest.nes", 16384, 8192);

        assert_that!(rom.header.len(), eq(16));
        assert_that!(rom.header.nes, eq("NES"));
        assert_that!(rom.header.prg_rom_size, eq(1));
        assert_that!(rom.header.chr_rom_size, eq(1));
        assert_that!(rom.prg_rom.len(), eq(16384));
        assert_that!(rom.chr_rom.len(), eq(8192));
        assert_that!(rom.prg_rom[0], eq(0x4C));
    }

    #[test]
    fn read_mirror() {
        let mut rom = Rom::default();
        rom.load_bytes(&[0x01], &[]);

        assert_that!(rom.read_byte(0x0000), eq(0x01));
        assert_that!(rom.read_byte(0x4000), eq(0x01));
    }

    #[test]
    fn read() {
        let rom = Rom::default();

        assert_that!(rom.read(0x0000, 0x7FFF).len(), eq(0x4000));
    }

    fn save_file(filename: &str, content: &[u8]) {
        let path = Path::new(filename);

        if path.exists() {
            fs::remove_file(path).unwrap();
        }

        let mut file = File::create(filename).unwrap();
        file.write_all(content).unwrap();
    }

    fn make_dir(dirname: &str) {
        let path = Path::new(dirname);

        if path.exists() {
            fs::remove_dir_all(path).unwrap();
        }
        fs::create_dir(dirname).unwrap()
    }
}