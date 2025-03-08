#[cfg(test)]
mod mock;

#[cfg(test)]
pub use mock::MockMemory;

/// Memory interface trait
/// 
/// This trait defines how components interact with memory in the NES.
/// The NES has a 16-bit address bus, allowing for 64KB of addressable memory.
pub trait Memory {
  /// Read a byte from memory at the specified address
  fn read_byte(&self, address: u16) -> u8;
  
  /// Write a byte to memory at the specified address
  fn write_byte(&mut self, address: u16, value: u8);
  
  /// Read a word (16-bits) from memory at the specified address
  /// NES is little-endian, so the lower byte is at the lower address
  fn read_word(&self, address: u16) -> u16 {
      let low = self.read_byte(address) as u16;
      let high = self.read_byte(address.wrapping_add(1)) as u16;
      (high << 8) | low
  }
  
  /// Write a word (16-bits) to memory at the specified address
  fn write_word(&mut self, address: u16, value: u16) {
      let low = (value & 0xFF) as u8;
      let high = (value >> 8) as u8;
      self.write_byte(address, low);
      self.write_byte(address.wrapping_add(1), high);
  }
}

/// A simple RAM implementation for testing
pub struct Ram {
  data: [u8; 0x10000], // 64KB of memory
}

impl Ram {
  pub fn new() -> Self {
      Ram {
          data: [0; 0x10000],
      }
  }
  
  pub fn reset(&mut self) {
      self.data = [0; 0x10000];
  }
}

impl Memory for Ram {
  fn read_byte(&self, address: u16) -> u8 {
      self.data[address as usize]
  }
  
  fn write_byte(&mut self, address: u16, value: u8) {
      self.data[address as usize] = value;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn test_ram_read_write_byte() {
      let mut ram = Ram::new();
      
      // Test write and read
      ram.write_byte(0x1000, 0x42);
      assert_eq!(ram.read_byte(0x1000), 0x42);
      
      // Test different address
      ram.write_byte(0x0500, 0xFF);
      assert_eq!(ram.read_byte(0x0500), 0xFF);
  }
  
  #[test]
  fn test_ram_read_write_word() {
      let mut ram = Ram::new();
      
      // Test write and read word (little-endian)
      ram.write_word(0x1000, 0x1234);
      assert_eq!(ram.read_byte(0x1000), 0x34); // Low byte
      assert_eq!(ram.read_byte(0x1001), 0x12); // High byte
      assert_eq!(ram.read_word(0x1000), 0x1234);
  }
}