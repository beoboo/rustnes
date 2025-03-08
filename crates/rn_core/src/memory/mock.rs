use super::Memory;

pub struct MockMemory {
    data: [u8; 0x10000], // 64KB of memory
}

impl MockMemory {
    pub fn new() -> Self {
        MockMemory { data: [0; 0x10000] }
    }
}

impl Memory for MockMemory {
    fn read_byte(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        self.data[addr as usize] = value;
    }
}
