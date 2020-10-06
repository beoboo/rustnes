use log::warn;

use crate::apu::Apu;
use crate::bus::Bus;
use crate::ppu::Ppu;
use crate::ram::Ram;
use crate::rom::Rom;
use crate::types::{Byte, Word};

#[derive(Debug)]
pub struct BusImpl {
    pub ram: Ram,
    pub ppu: Ppu,
    pub apu: Apu,
    pub rom: Rom,
}

impl BusImpl {
    pub fn new(ram: Ram, ppu: Ppu, apu: Apu, rom: Rom) -> BusImpl {
        BusImpl {
            ram,
            ppu,
            apu,
            rom,
        }
    }
}

impl Bus for BusImpl {
    fn read_byte(&self, address: Word) -> Byte {
        match address {
            0x0000..=0x1FFF => self.ram.read_byte(address & 0x07FF),
            0x2000..=0x2007 => self.ppu.read_byte(address - 0x2000),
            0x4000..=0x401F => self.apu.read_byte(address - 0x4000),
            0x6000..=0x7FFF => {
                warn!("[BusImpl::read_byte] Addresses 0x6000-0x7FFF not handled");
                0
            }
            0x8000..=0xBFFF => self.rom.read_byte(address - 0x8000),
            0xC000..=0xFFFF if self.rom.prg_rom.len() <= 0x4000 => self.rom.read_byte(address - 0xC000),
            0xC000..=0xFFFF => self.rom.read_byte(address - 0x8000),
            _ => panic!(format!("[Bus::read_byte] Not mapped address: {:#6X}", address))
        }
    }

    fn write_byte(&mut self, address: Word, data: Byte) {
        match address {
            0x0000..=0x1FFF => self.ram.write_byte(address & 0x07FF, data),
            0x2000..=0x2007 => self.ppu.write_byte(address - 0x2000, data),
            0x4000..=0x401F => self.apu.write_byte(address - 0x4000, data),
            0x6000..=0x7FFF => {
                warn!("[BusImpl::write_byte] Addresses 0x6000-0x7FFF not handled");
            }
            0x8000..=0xFFFF => {
                warn!("[BusImpl::write_byte] Addresses 0x8000-0xFFFF not handled");
            }
            _ => panic!(format!("[Bus::write_byte] Not mapped address: {:#6X}", address))
        }
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn test_read_byte() {
        let mut rom = Rom::default();
        rom.load_bytes(&[0x01], &[]);
        let bus = _build_bus(rom);

        assert_that!(bus.read_byte(0x0000), eq(0));
        assert_that!(bus.read_byte(0x8000), eq(1));
    }

    #[test]
    fn test_read_word() {
        let mut rom = Rom::default();
        rom.load_bytes(&[0x01, 0x02], &[]);

        let bus = _build_bus(rom);

        assert_that!(bus.read_word(0x8000), eq(0x0201));
    }

    #[test]
    fn test_write() {
        let mut rom = Rom::default();
        rom.load_bytes(&[0x01, 0x02], &[]);

        let mut bus = _build_bus(rom);

        bus.write_byte(0x0000, 0x01);
        // bus.write_byte(0x8000, 0x02);

        assert_that!(bus.read_byte(0x0000), eq(0x01));
        assert_that!(bus.read_byte(0x0800), eq(0x01));
        assert_that!(bus.read_byte(0x1000), eq(0x01));
        assert_that!(bus.read_byte(0x1800), eq(0x01));
        // assert_that!(bus.read_byte(0x8000), eq(0x02));
    }

    #[test]
    fn test_fetch_address() {
        let mut rom = Rom::default();
        rom.load_bytes(&[0x01, 0x02], &[]);
        let mut bus = _build_bus(rom);

        bus.write_byte(0x0000, 0x01);

        assert_that!(bus.read_byte(0x0000), eq(0x01));
        assert_that!(bus.read_byte(0x0800), eq(0x01));
        assert_that!(bus.read_byte(0x1000), eq(0x01));
        assert_that!(bus.read_byte(0x1800), eq(0x01));
    }

    fn _build_bus(rom: Rom) -> BusImpl {
        BusImpl::new(Ram::new(16), Ppu::default(), Apu::default(), rom)
    }
}