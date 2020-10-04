use crate::types::{Byte, Word};
use log::{trace, warn};
use crate::ram::Ram;

#[derive(Debug)]
pub struct Ppu {
    pub ram: Ram,
}

impl Default for Ppu {
    fn default() -> Ppu {
        Ppu {
            ram: Ram::new(2048)
        }
    }
}

impl Ppu {
    pub fn read_byte(&self, address: Word) -> Byte {
        trace!("Reading from PPU address: {:#06X}", address);
        self.ram.read_byte(address)
        // warn!("[Ppu::read] Not implemented");
        // 0xFF
    }

    pub fn write_byte(&mut self, address: Word, data: Byte) {
        trace!("Writing {:#04X} to PPU address {:#06X}", data, address);
        self.ram.write_byte(address, data);
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

        assert_that!(ppu.read_byte(0x0000), eq(0x00));
    }

    #[test]
    fn test_write() {
        let mut ppu = Ppu::default();
        ppu.write_byte(0x0000, 0x12);

        assert_that!(ppu.read_byte(0x0000), eq(0x12));
    }
}