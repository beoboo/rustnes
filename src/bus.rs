use crate::rom::Rom;
use crate::types::{Byte, Word};
use crate::ppu::Ppu;
use crate::ram::Ram;

pub trait Bus {
    fn read_byte(&self, address: Word) -> Byte;

    fn read_word(&self, address: Word) -> Word {
        let low = self.read_byte(address) as Word;
        let high = self.read_byte(address + 1) as Word;

        (high << 8) + low
    }

    fn write_byte(&mut self, address: Word, data: Byte);
}

pub struct BusImpl {
    ram: Ram,
    ppu: Ppu,
    rom: Rom,
}

impl BusImpl {
    pub fn new(ram: Ram, ppu: Ppu, rom: Rom) -> BusImpl {
        BusImpl {
            ram,
            ppu,
            rom,
        }
    }
}

impl Bus for BusImpl {
    fn read_byte(&self, address: Word) -> Byte {
        match address {
            0x0000..=16 => self.ram.read(address),
            0x2000..=0x2007 => self.ppu.read(address - 0x2000),
            0x8000..=0xFFFF if address - 0x8000 > self.rom.prg_rom.len() as Word => self.rom.read(address - 0xC000),
            0x8000..=0xFFFF  => self.rom.read(address - 0x8000),
            _ => panic!(format!("[Bus::read_byte] Not mapped address: {:#6X}", address))
        }
    }

    fn write_byte(&mut self, address: Word, data: Byte) {
        match address {
            0x0000..=16 => self.ram.write(address, data),
            _ => panic!(format!("[Bus::write_byte] Not mapped address: {:#6X}", address))
        }
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn test_read_byte() {
        let rom = Rom::new(&[0x01], &[]);
        let mut bus = _build_bus(rom);

        assert_that!(bus.read_byte(0x0000), eq(0));
        assert_that!(bus.read_byte(0x8000), eq(1));
    }

    #[test]
    fn test_read_word() {
        let rom = Rom::new(&[0x01, 0x02], &[]);
        let mut bus = _build_bus(rom);

        assert_that!(bus.read_word(0x8000), eq(0x0201));
    }

    #[test]
    fn test_write() {
        let rom = Rom::new(&[0x01, 0x02], &[]);
        let mut bus = _build_bus(rom);

        bus.write_byte(0x0000, 0x01);
        // bus.write_byte(0x8000, 0x02);

        assert_that!(bus.read_byte(0x0000), eq(0x01));
        // assert_that!(bus.read_byte(0x8000), eq(0x02));
    }

    fn _build_bus(rom: Rom) -> BusImpl {
        BusImpl::new(Ram::new(16), Ppu::new(), rom)
    }
}