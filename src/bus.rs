use crate::rom::Rom;
use crate::types::Byte;

pub trait Bus {
    fn read(&self, address: u16) -> u8;

    fn write(&mut self, address: u16, data: u8);
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
    fn read(&self, address: u16) -> u8 {
        let address = address as usize;

        match address {
            0x0000..=16 => self.ram[address],
            0x8000..=0xFFFF if address - 0x8000 > self.rom.prg_rom.len() => self.rom.prg_rom[address - 0xC000],
            0x8000..=0xFFFF  => self.rom.prg_rom[address - 0x8000],
            _ => panic!(format!("Not mapped address: {:#6X}", address))

        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=16 => self.ram[address as usize] = data,
            0x8000..=0xFFFF => self.rom.prg_rom[address as usize - 0x8000] = data,
            _ => panic!(format!("Not mapped address: {:#6X}", address))

        }
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn test_read() {
        let rom = Rom::new(&[0x01], &[]);
        let bus = BusImpl::new(rom);

        assert_that!(bus.read(0x0000), eq(0));
        assert_that!(bus.read(0x8000), eq(1));
    }

    #[test]
    fn test_write() {
        let rom = Rom::new(&[0x00; 16], &[]);
        let mut bus = BusImpl::new(rom);

        bus.write(0x0000, 0x01);
        bus.write(0x8000, 0x02);

        assert_that!(bus.read(0x0000), eq(0x01));
        assert_that!(bus.read(0x8000), eq(0x02));
    }
}