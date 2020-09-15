use crate::rom::Rom;
use crate::types::{Byte, Word};

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
    ram: Vec<Byte>,
    rom: Rom,
}

impl BusImpl {
    pub fn new(rom: Rom) -> BusImpl {
        BusImpl {
            ram: vec![0x00; 16],
            rom,
        }
    }
}

impl Bus for BusImpl {
    fn read_byte(&self, address: Word) -> Byte {
        let address = address as usize;

        match address {
            0x0000..=16 => self.ram[address],
            0x8000..=0xFFFF if address - 0x8000 > self.rom.prg_rom.len() => self.rom.prg_rom[address - 0xC000],
            0x8000..=0xFFFF  => self.rom.prg_rom[address - 0x8000],
            _ => panic!(format!("[Bus::read_byte] Not mapped address: {:#6X}", address))
        }
    }

    fn write_byte(&mut self, address: Word, data: u8) {
        match address {
            0x0000..=16 => self.ram[address as usize] = data,
            0x8000..=0xFFFF => self.rom.prg_rom[address as usize - 0x8000] = data,
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
        let bus = BusImpl::new(rom);

        assert_that!(bus.read_byte(0x0000), eq(0));
        assert_that!(bus.read_byte(0x8000), eq(1));
    }

    #[test]
    fn test_read_word() {
        let rom = Rom::new(&[0x01, 0x02], &[]);
        let bus = BusImpl::new(rom);

        assert_that!(bus.read_word(0x8000), eq(0x0201));
    }

    #[test]
    fn test_write() {
        let rom = Rom::new(&[0x00; 16], &[]);
        let mut bus = BusImpl::new(rom);

        bus.write_byte(0x0000, 0x01);
        bus.write_byte(0x8000, 0x02);

        assert_that!(bus.read_byte(0x0000), eq(0x01));
        assert_that!(bus.read_byte(0x8000), eq(0x02));
    }
}