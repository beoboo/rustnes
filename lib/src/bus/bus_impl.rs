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

    pub fn load_rom(&mut self, rom: Rom) {
        self.rom = rom;
    }
}

impl Bus for BusImpl {
    fn read(&self, start: Word, end: Word) -> &[Byte] {
        if start >= end {
            panic!("[BusImpl::read] Start address must be lower than end address");
        }

        match (start, end) {
            // (0x0000..=0x1FFF, 0x0000..=0x1FFF) => self.ram.read(start, end),
            // (0x2000..=0x2007, 0x2000..=0x2007) => self.ppu.read(start, end),
            // (0x4000..=0x4017, 0x4000..=0x4017) => self.apu.read(start, end),
            (0x8000..=0xFFFF, 0x8000..=0xFFFF) => self.rom.read(start - 0x8000, end - 0x8000),
            _ => panic!(format!("[BusImpl::read] Cannot read range {:#6X}:{:#6X}", start, end))
        }
    }

    fn read_byte(&mut self, address: Word) -> Byte {
        match address {
            0x0000..=0x1FFF => self.ram.read_byte(address & 0x07FF),
            0x2000..=0x3FFF => self.ppu.read_byte(address - 0x2000),
            0x4000..=0x401F => self.apu.read_byte(address - 0x4000),
            0x6000..=0x7FFF => {
                warn!("[BusImpl::read_byte] Addresses 0x6000-0x7FFF not handled");
                0
            }
            0x8000..=0xFFFF => self.rom.read_byte(address - 0x8000),
            _ => panic!(format!("[BusImpl::read_byte] Not mapped address: {:#6X}", address))
        }
    }

    fn write_byte(&mut self, address: Word, data: Byte) {
        match address {
            0x0000..=0x1FFF => self.ram.write_byte(address & 0x07FF, data),
            0x2000..=0x3FFF => self.ppu.write_byte(address - 0x2000, data),
            0x4000..=0x401F => self.apu.write_byte(address - 0x4000, data),
            0x6000..=0x7FFF => {
                warn!("[BusImpl::write_byte] Addresses 0x6000-0x7FFF not handled");
            }
            0x8000..=0xFFFF => {
                warn!("[BusImpl::write_byte] Addresses 0x8000-0xFFFF not handled");
            }
            _ => panic!(format!("[BusImpl::write_byte] Not mapped address: {:#6X}", address))
        }
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use super::*;
    use crate::ppu::status::Status;

    #[test]
    fn test_read_byte() {
        let mut rom = Rom::default();
        rom.load_bytes(&[0x01], &[]);
        let mut bus = _build_bus(rom);

        assert_that!(bus.read_byte(0x0000), eq(0));
        assert_that!(bus.read_byte(0x8000), eq(1));
    }

    #[test]
    fn test_read_word() {
        let mut rom = Rom::default();
        rom.load_bytes(&[0x01, 0x02], &[]);

        let mut bus = _build_bus(rom);

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

    #[test]
    fn mirror_ram() {
        let rom = Rom::default();
        let mut bus = _build_bus(rom);

        bus.write_byte(0x0000, 0x01);
        bus.write_byte(0x0800, 0x02);
        bus.write_byte(0x1000, 0x03);
        bus.write_byte(0x1800, 0x04);

        assert_that!(bus.read_byte(0x0000), eq(0x04));
        assert_that!(bus.read_byte(0x0800), eq(0x04));
        assert_that!(bus.read_byte(0x1000), eq(0x04));
        assert_that!(bus.read_byte(0x1800), eq(0x04));
    }

    #[test]
    fn mirror_rom() {
        let mut rom = Rom::default();
        rom.load_bytes(&[0x01, 0x02, 0x03], &[]);

        let mut bus = _build_bus(rom);

        assert_that!(bus.read_byte(0x8002), eq(0x03));
        assert_that!(bus.read_byte(0xC002), eq(0x03));
    }

    #[test]
    fn mirror_ppu() {
        env_logger::init();
        let rom = Rom::default();
        let mut bus = _build_bus(rom);
        bus.ppu.status = Status::from_byte(0x60);

        assert_that!(bus.read_byte(0x2002), eq(0x60));
        assert_that!(bus.read_byte(0x3FF2), eq(0x60));
    }

    fn _build_bus(rom: Rom) -> BusImpl {
        BusImpl::new(Ram::new(16), Ppu::default(), Apu::default(), rom)
    }
}