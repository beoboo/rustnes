use crate::rom::Rom;

pub trait Bus {
    fn read(&self, address: u16) -> u8;

    fn write(&mut self, address: u16, data: u8);
}

pub struct BusImpl {
    rom: Rom,
}

impl BusImpl {
    pub fn new(rom: Rom) -> BusImpl {
        BusImpl {
            rom,
        }
    }
}

impl Bus for BusImpl {
    fn read(&self, address: u16) -> u8 {
        self.rom.prg_rom[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.rom.prg_rom[address as usize] = data;
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn test_read() {
        let rom = Rom::new(&[0x00], &[]);
        let bus = BusImpl::new(rom);

        assert_that!(bus.read(0x0000), eq(0));
    }

    #[test]
    fn test_write() {
        let rom = Rom::new(&[0x00; 16], &[]);
        let mut bus = BusImpl::new(rom);
        bus.write(0x0001, 0x01);

        assert_that!(bus.read(0x0001), eq(0x01));
    }
}