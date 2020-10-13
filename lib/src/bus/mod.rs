use log::trace;

use crate::types::{Byte, Word};

pub mod simple_bus;
pub mod bus_impl;

pub trait Bus {
    fn read(&self, start: Word, end: Word) -> &[Byte];

    fn read_byte(&mut self, address: Word) -> Byte;

    fn read_word(&mut self, address: Word) -> Word {
        let low = self.read_byte(address) as Word;
        let high = self.read_byte(address + 1) as Word;

        let data = (high << 8) + low;
        trace!("Bus: Reading word from {:#06X} -> {:#04X}", address, data);
        data
    }

    fn write_byte(&mut self, address: Word, data: Byte);

    fn write_word(&mut self, address: Word, data: Word) {
        trace!("BUS: Writing word to {:#06X} <- {:#04X}", address, data);
        self.write_byte(address, data as Byte);
        self.write_byte(address + 1, (data >> 8) as Byte);
    }
}

