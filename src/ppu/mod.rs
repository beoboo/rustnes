use crate::types::{Byte, Word};

pub struct Ppu {

}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {}
    }

    pub fn read(&self, address: Word) -> Byte {
        println!("Raading from PPU address: {}", address);
        0xFF
    }

    pub fn write(&mut self, address: Word, data: Byte) {
        println!("Writing to PPU address: {}", address);
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn test_read_byte() {
        let ppu = Ppu::new();

        assert_that!(ppu.read(0x0000), eq(0xFF));
    }
}