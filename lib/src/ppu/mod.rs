use crate::types::{Byte, Word};

pub struct Ppu {}

impl Default for Ppu {
    fn default() -> Ppu {
        Ppu {}
    }
}

impl Ppu {
    pub fn read(&self, address: Word) -> Byte {
        println!("Reading from PPU address: {:#06X}", address);
        0xFF
    }

    pub fn write(&mut self, address: Word, data: Byte) {
        println!("Writing {:#04X} to PPU address {:#06X}", data, address);
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn test_read() {
        let ppu = Ppu::default();

        assert_that!(ppu.read(0x0000), eq(0xFF));
    }
    //
    // #[test]
    // fn test_write() {
    //     let mut ppu = Ppu::new();
    //     ppu.write(0x0000, 0x123);
    //
    //     assert_that!(ppu.read(0x0000), eq(0x123));
    // }
}