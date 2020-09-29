use log::warn;

use crate::apu::Apu;
use crate::bus::Bus;
use crate::ppu::Ppu;
use crate::ram::Ram;
use crate::rom::Rom;
use crate::types::{Byte, Word};

pub struct BusImpl {
    ram: Ram,
    apu: Apu,
    ppu: Ppu,
    rom: Rom,
}

impl BusImpl {
    pub fn new(ram: Ram, apu: Apu, ppu: Ppu, rom: Rom) -> BusImpl {
        BusImpl {
            ram,
            apu,
            ppu,
            rom,
        }
    }
}

impl Bus for BusImpl {
    fn read_byte(&self, address: Word) -> Byte {
        match address {
            0x0000..=0x1FFF => self.ram.read(address & 0x07FF),
            0x2000..=0x2007 => self.ppu.read(address - 0x2000),
            0x4000..=0x401F => self.apu.read(address - 0x4000),
            0x6000..=0x7FFF => {
                warn!("Not implemented");
                0
            }
            0x8000..=0xBFFF => self.rom.read(address - 0x8000),
            0xC000..=0xFFFF if self.rom.prg_rom.len() <= 0x4000 => self.rom.read(address - 0xC000),
            0xC000..=0xFFFF => self.rom.read(address - 0x8000),
            _ => panic!(format!("[Bus::read_byte] Not mapped address: {:#6X}", address))
        }
    }

    fn write_byte(&mut self, address: Word, data: Byte) {
        match address {
            0x0000..=0x1FFF => self.ram.write(address & 0x07FF, data),
            0x2000..=0x2007 => self.ppu.write(address - 0x2000, data),
            0x4000..=0x401F => self.apu.write(address - 0x4000, data),
            0x6000..=0x7FFF => { warn!("Not implemented") }
            0x8000..=0xFFFF => { warn!("Not implemented") }
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
        let rom = Rom::new(&[0x01], &[]);
        let bus = _build_bus(rom);

        assert_that!(bus.read_byte(0x0000), eq(0));
        assert_that!(bus.read_byte(0x8000), eq(1));
    }

    #[test]
    fn test_read_word() {
        let rom = Rom::new(&[0x01, 0x02], &[]);
        let bus = _build_bus(rom);

        assert_that!(bus.read_word(0x8000), eq(0x0201));
    }

    #[test]
    fn test_write() {
        let rom = Rom::new(&[0x01, 0x02], &[]);
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
        let rom = Rom::new(&[0x01, 0x02], &[]);
        let mut bus = _build_bus(rom);

        bus.write_byte(0x0000, 0x01);

        assert_that!(bus.read_byte(0x0000), eq(0x01));
        assert_that!(bus.read_byte(0x0800), eq(0x01));
        assert_that!(bus.read_byte(0x1000), eq(0x01));
        assert_that!(bus.read_byte(0x1800), eq(0x01));
    }

    fn _build_bus(rom: Rom) -> BusImpl {
        BusImpl::new(Ram::new(16), Apu::default(), Ppu::default(), rom)
    }
}