use log::trace;

use crate::bus::Bus;
use crate::types::{Byte, Word};

pub struct SimpleBus {
    data: Vec<u8>,
}

fn replace_slice<T>(source: &mut [T], from: &[T])
    where
        T: Clone + PartialEq,
{
    source[..from.len()].clone_from_slice(from);
}

impl Default for SimpleBus {
    fn default() -> SimpleBus {
        SimpleBus {
            data: vec![0; 0xFFFF + 1],
        }
    }
}


impl SimpleBus {

    pub fn load(&mut self, program: Vec<u8>, starting_pos: usize) {
        // let mut data = self.data;
        replace_slice(&mut self.data[starting_pos..], program.as_slice());
    }
}

impl Bus for SimpleBus {
    fn read_byte(&self, address: Word) -> Byte {
        let address = address as usize;

        let data = self.data[address];
        trace!("Reading byte from {:#06X} -> {:#04X}", address, data);
        data
    }

    fn write_byte(&mut self, address: Word, data: Byte) {
        trace!("Writing byte {:#04X} to {:#06X}", data, address);
        let address = address as usize;

        self.data[address] = data
    }
}