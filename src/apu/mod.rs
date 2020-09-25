use crate::types::{Byte, Word};
use log::warn;

pub struct Apu {

}

impl Apu {
    pub fn new() -> Apu {
        Apu {}
    }

    pub fn read(&self, address: Word) -> Byte {
        warn!("APU: Reading from {:#06X} -> (not implemented)", address);
        0xFF
    }

    pub fn write(&mut self, address: Word, data: Byte) {
        warn!("APU: Writing to {:#06X} <- {:#04X} (not implemented)", address, data);
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn test_read() {
        let apu = Apu::new();

        assert_that!(apu.read(0x0000), eq(0xFF));
    }
    //
    // #[test]
    // fn test_write() {
    //     let mut apu = Apu::new();
    //     apu.write(0x0000, 123);
    //
    //     assert_that!(apu.read(0x0000), eq(123));
    // }
}