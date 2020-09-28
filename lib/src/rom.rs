use std::fs::File;
use std::io::prelude::*;
use std::str;
use crate::types::{Word, Byte};
use log::trace;

#[derive(Debug)]
pub struct RomHeader {
    pub nes: String,
    pub prg_rom_size: usize,
    pub chr_rom_size: usize,
}

impl RomHeader {
    fn new(prg_rom_size: usize, chr_rom_size: usize) -> RomHeader {
        RomHeader {
            nes: "NES".to_string(),
            prg_rom_size,
            chr_rom_size,
        }
    }

    fn load(buffer: &[u8]) -> RomHeader {
        RomHeader {
            nes: String::from_utf8_lossy(&buffer[0..3]).to_string(),
            prg_rom_size: buffer[4] as usize,
            chr_rom_size: buffer[5] as usize,
        }
    }

    fn len(&self) -> usize {
        16
    }
}

#[derive(Debug)]
pub struct Rom {
    pub header: RomHeader,
    pub trainer: Vec<u8>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}

impl Rom {
    pub fn new(prg_rom: &[u8], chr_rom: &[u8]) -> Rom {
        let header = RomHeader::new(prg_rom.len(), chr_rom.len());

        Rom {
            header,
            trainer: vec![],
            prg_rom: prg_rom.to_vec(),
            chr_rom: chr_rom.to_vec(),
        }
    }

    pub fn load(filename: &str, prg_bank_size: usize, chr_bank_size: usize) -> Rom {
        let buffer = Rom::load_file(filename);
        let header = RomHeader::load(&buffer[0..16]);

        let prg_rom_start = header.len();
        // println!("Header length: {}", header.len());
        let prg_rom_end = prg_rom_start + prg_bank_size * header.prg_rom_size;
        let chr_rom_start = prg_rom_end;
        let chr_rom_end = chr_rom_start + chr_bank_size * header.chr_rom_size;

        Rom {
            header,
            trainer: vec![],
            prg_rom: buffer[prg_rom_start..prg_rom_end].to_vec(),
            chr_rom: buffer[chr_rom_start..chr_rom_end].to_vec(),
        }
    }

    pub fn read(&self, address: Word) -> Byte {
        let data = self.prg_rom[address as usize];
        trace!("ROM: Reading from {:#06X} -> {:#04X}", address, data);
        data
    }
    //
    // pub fn write(&mut self, address: Word, data: Byte) {
    //     self.prg_rom[address as usize] = data;
    // }

    fn load_file(filename: &str) -> Vec<u8> {
        let mut file = File::open(filename).unwrap_or_else(|_| panic!("Could not open {}", filename));

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

        let rom = Rom::load(filename, 1, 1);

        assert_that!(rom.header.len(), eq(16));
        assert_that!(rom.header.nes, eq("NES"));
        assert_that!(rom.header.prg_rom_size, eq(1));
        assert_that!(rom.header.chr_rom_size, eq(1));
        assert_that!(rom.prg_rom.len(), eq(1));
        assert_that!(rom.chr_rom.len(), eq(1));
    }

    #[test]
    fn read_valid() {
        let rom = Rom::load("../roms/cpu/nestest/nestest.nes", 16384, 8192);

        assert_that!(rom.header.len(), eq(16));
        assert_that!(rom.header.nes, eq("NES"));
        assert_that!(rom.header.prg_rom_size, eq(1));
        assert_that!(rom.header.chr_rom_size, eq(1));
        assert_that!(rom.prg_rom.len(), eq(16384));
        assert_that!(rom.chr_rom.len(), eq(8192));
        assert_that!(rom.prg_rom[0], eq(0x4C));
    }


    //
    // #[test]
    // fn read_from_file() {
    //     fs::create_dir("tmp");
    //     let filename = "tmp/rom.nes";
    //     save_file(filename, "");
    //
    //     let rom = Rom::load(filename);
    //
    //     assert_that!(rom.header.len(), eq(16));
    //     assert_that!(rom.data.len(), eq(0));
    // }

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