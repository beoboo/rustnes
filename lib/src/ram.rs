use log::trace;

use crate::types::{Byte, Word};

pub struct Ram {
    data: Vec<Byte>,
}

impl Ram {
    pub fn new(size: usize) -> Ram {
        Ram {
            data: vec![0x00; size],
        }
    }

    pub fn read(&self, address: Word) -> Byte {
        let data = self.data[address as usize];
        trace!("RAM: Reading from {:#06X} -> {:#04X}", address, data);
        data
    }

    pub fn write(&mut self, address: Word, data: Byte) {
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

        assert_that!(ram.read(0x0000), eq(0));
    }

    #[test]
    fn test_write() {
        let mut ram = Ram::new(16);
        ram.write(0, 0x12);

        assert_that!(ram.read(0), eq(0x12));
    }
}