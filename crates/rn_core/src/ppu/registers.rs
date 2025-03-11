use std::{cell::RefCell, rc::Rc};

use crate::{memory::Addressable, ppu::Ppu};

/// Adapter to connect PPU registers to the memory bus
///
/// This component handles memory-mapped I/O for the PPU registers
/// at addresses $2000-$2007.
pub struct PpuRegisters {
    /// Reference to the PPU
    ppu: Rc<RefCell<Ppu>>,
}

impl PpuRegisters {
    /// Create a new PPU registers adapter
    pub fn new(ppu: Rc<RefCell<Ppu>>) -> Self {
        Self { ppu }
    }
}

impl Addressable for PpuRegisters {
    /// Check if the address is in the PPU register range ($2000-$2007)
    fn handles_address(&self, address: u16) -> bool {
        address >= 0x2000 && address <= 0x2007
    }
    
    /// Read from a PPU register
    ///
    /// This forwards the read operation to the PPU's read_register method.
    /// Note that reading from some PPU registers may have side effects.
    fn read_byte(&self, address: u16) -> u8 {
        self.ppu.borrow_mut().read_register(address)
    }
    
    /// Write to a PPU register
    ///
    /// This forwards the write operation to the PPU's write_register method.
    /// Note that writing to some PPU registers may have side effects.
    fn write_byte(&mut self, address: u16, value: u8) {
        self.ppu.borrow_mut().write_register(address, value);
    }
    
    /// Reset the PPU registers
    ///
    /// This is called when the system is reset. It forwards the reset
    /// operation to the PPU.
    fn reset(&mut self) {
        self.ppu.borrow_mut().reset();
    }
} 