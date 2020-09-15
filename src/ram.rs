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
        self.data[address as usize]
    }

    pub fn write(&mut self, address: Word, data: Byte) {
        self.data[address as usize] = data;
    }
}

#[cfg(test)]
mod tests {
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