pub mod simple_bus;
pub mod bus_impl;

use crate::types::{Word, Byte};

pub trait Bus {
    fn read_byte(&self, address: Word) -> Byte;

    fn read_word(&self, address: Word) -> Word {
        let low = self.read_byte(address) as Word;
        let high = self.read_byte(address + 1) as Word;

        let data = (high << 8) + low;
        println!("Read word {:#06X} from {:#06X}", data, address);
        data
    }

    fn write_byte(&mut self, address: Word, data: Byte);

    fn write_word(&mut self, address: Word, data: Word) {
        println!("Writing word {:#06X} to {:#06X}", data, address);
        self.write_byte(address, data as Byte);
        self.write_byte(address + 1, (data >> 8) as Byte);
    }
}

