trait Bus {
    fn read(&self, address: u16) -> u8;

    fn write(&mut self, address: u16, data: u8);
}

pub struct BusImpl {
    rom: Vec<u8>,
}

impl BusImpl {
    pub fn new(rom_size: u8) -> BusImpl {
        BusImpl {
            rom: vec![0, rom_size],
        }
    }
}

impl Bus for BusImpl {
    fn read(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.rom[address as usize] = data;
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn test_read() {
        let bus = BusImpl::new(16);

        assert_that!(bus.read(0x0000), eq(0));
    }

    #[test]
    fn test_write() {
        let mut bus = BusImpl::new(16);
        bus.write(0x0001, 0x01);

        assert_that!(bus.read(0x0001), eq(0x01));
    }
}