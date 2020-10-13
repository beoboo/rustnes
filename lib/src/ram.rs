use log::trace;

use crate::types::{Byte, Word};

#[derive(Debug)]
pub struct Ram {
    pub data: Vec<Byte>,
}

impl Ram {
    pub fn new(size: usize) -> Ram {
        Ram {
            data: vec![0x00; size],
        }
    }

    pub fn read_byte(&self, address: Word) -> Byte {
        let data = self.data[address as usize];
        trace!("RAM: Reading from {:#06X} -> {:#04X}", address, data);
        data
    }

    pub fn write_byte(&mut self, address: Word, data: Byte) {
        println!("RAM: Writing to {:#06X} <- {:#04X}", address, data);
        trace!("RAM: Writing to {:#06X} <- {:#04X}", address, data);
        self.data[address as usize] = data;
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn test_read() {
        let ram = Ram::new(16);

        assert_that!(ram.read_byte(0x0000), eq(0));
    }

    #[test]
    fn test_write() {
        let mut ram = Ram::new(16);
        ram.write_byte(0x0000, 0x12);

        assert_that!(ram.read_byte(0x0000), eq(0x12));
    }
}