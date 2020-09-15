use crate::types::{Byte, Word};

pub struct Ppu {

}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {}
    }

    pub fn read(&self, address: Word) -> Byte {
        0x00
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn test_read_byte() {
        let ppu = Ppu::new();

        assert_that!(ppu.read(0x0000), eq(0));
    }
}