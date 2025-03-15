use std::{cell::RefCell, rc::Rc};

use crate::{cpu::Cpu, errors::NesError, memory::Addressable, ppu::Ppu};

/// DMA Controller for transferring data from CPU memory to PPU OAM
///
/// In the NES, writing to address $4014 triggers a DMA transfer of 256 bytes
/// from CPU memory to PPU OAM. The CPU is suspended during this transfer.
pub struct DmaController {
    /// Source high byte for DMA transfer (written to $4014)
    source_high_byte: u8,
    
    /// Number of cycles remaining in current DMA transfer
    cycles_remaining: u16,
    
    /// Whether a transfer is active
    transfer_active: bool,
    
    /// Read buffer - stores the last read value
    read_buffer: u8,

    /// CPU component
    cpu: Option<Rc<RefCell<Cpu>>>,

    /// PPU component
    ppu: Option<Rc<RefCell<Ppu>>>,
}

impl DmaController {
    /// Create a new DMA controller
    pub fn new() -> Self {
        Self {
            source_high_byte: 0,
            cycles_remaining: 0,
            transfer_active: false,
            read_buffer: 0,
            cpu: None,
            ppu: None,  
        }
    }

    /// Connect the CPU component
    pub fn connect_cpu(&mut self, cpu: Rc<RefCell<Cpu>>) {
        self.cpu = Some(cpu);
    }

    /// Connect the PPU component
    pub fn connect_ppu(&mut self, ppu: Rc<RefCell<Ppu>>) {
        self.ppu = Some(ppu);
    }

    /// Check if DMA is currently active
    ///
    /// When active, the CPU should be suspended
    pub fn is_active(&self) -> bool {
        self.transfer_active
    }
    
    /// Process a single DMA cycle
    ///
    /// Returns Some(byte_to_write) if a byte should be written to OAM on this cycle,
    /// along with the OAM index to write to. Returns None if no byte should be written.
    pub fn tick(&mut self, memory_read_fn: impl FnOnce(u16) -> Result<u8, NesError>) -> Option<(u8, u8)> {
        if !self.transfer_active {
            return None;
        }
        
        // Decrement remaining cycles
        self.cycles_remaining -= 1;
        
        // If we've reached 0, the transfer is complete
        if self.cycles_remaining == 0 {
            self.transfer_active = false;
            return None;
        }
        
        // We don't transfer data on the first 1-2 cycles (setup cycles)
        if self.cycles_remaining >= 512 {
            return None;
        }
        
        // Transfer data every other cycle (read cycle followed by write cycle)
        let cycle_index = 513 - self.cycles_remaining;
        
        // On even cycles, we read from memory
        if cycle_index % 2 == 0 {
            // Calculate the source address
            let byte_index = (cycle_index / 2) as u8;
            let source_addr = ((self.source_high_byte as u16) << 8) | (byte_index as u16);
            
            // Read from memory
            match memory_read_fn(source_addr) {
                Ok(value) => {
                    self.read_buffer = value;
                }
                Err(_) => {
                    // On error, use 0 as fallback
                    self.read_buffer = 0;
                }
            }
            
            // No write on this cycle
            None
        } else {
            // On odd cycles, we write to OAM
            let oam_index = ((cycle_index - 1) / 2) as u8;
            
            // Return the byte to write and the OAM index
            Some((self.read_buffer, oam_index))
        }
    }
    
    /// Begin a DMA transfer
    fn begin_transfer(&mut self, source_high_byte: u8) {
        self.source_high_byte = source_high_byte;
        
        // DMA takes 513 cycles (1 setup + 256 * 2 for read/write)
        // In the actual hardware, it could be 514 cycles if starting on an odd cycle
        // For simplicity, we'll use 513
        self.cycles_remaining = 513;
        self.transfer_active = true;
    }
    
    /// Execute a full DMA transfer synchronously (for testing or simple implementations)
    /// 
    /// Takes a read function to read from CPU memory and a write function to write to PPU OAM
    pub fn perform_transfer(
        &mut self,
        source_high_byte: u8,
        read_fn: impl Fn(u16) -> Result<u8, NesError>,
        mut write_fn: impl FnMut(u8, u8) -> Result<(), NesError>,
    ) -> Result<(), NesError> {
        // Set the source high byte
        self.source_high_byte = source_high_byte;
        
        // For each byte in the 256-byte page
        for i in 0..256 {
            // Calculate the source address
            let source_addr = ((source_high_byte as u16) << 8) | (i as u16);
            
            // Read from CPU memory
            let value = read_fn(source_addr)?;
            
            // Write to PPU OAM
            write_fn(i as u8, value)?;
        }
        
        Ok(())
    }
}

impl Addressable for DmaController {
    fn handles_address(&self, address: u16) -> bool {
        address == 0x4014 // DMA controller only responds to $4014
    }
    
    fn read_byte(&self, _address: u16) -> Result<u8, NesError> {
        // Reading from DMA register returns the last value written
        Ok(self.source_high_byte)
    }
    
    fn write_byte(&mut self, _address: u16, value: u8) -> Result<(), NesError> {
        // Writing to $4014 starts a DMA transfer
        self.begin_transfer(value);
        Ok(())
    }
    
    fn reset(&mut self) {
        self.source_high_byte = 0;
        self.cycles_remaining = 0;
        self.transfer_active = false;
    }
}

impl Default for DmaController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cpu::Cpu, memory::Ram, ppu::Ppu, system::Bus};
    
    #[test]
    fn test_dma_initialization() {
        let dma = DmaController::new();
        
        assert_eq!(dma.source_high_byte, 0);
        assert_eq!(dma.cycles_remaining, 0);
        assert_eq!(dma.transfer_active, false);
    }
    
    #[test]
    fn test_dma_write_starts_transfer() {
        let mut dma = DmaController::new();
        
        // Write to DMA register
        dma.write_byte(0x4014, 0x20).unwrap();
        
        assert_eq!(dma.source_high_byte, 0x20);
        assert_eq!(dma.cycles_remaining, 513);
        assert_eq!(dma.transfer_active, true);
    }
    
    #[test]
    fn test_dma_transfer_complete() {
        let mut dma = DmaController::new();
        
        // Start a transfer
        dma.write_byte(0x4014, 0x20).unwrap();
        
        // Simulate all 513 cycles, creating a dummy read function
        let mut bytes_written = 0;
        
        for _ in 0..513 {
            let read_fn = |_addr| Ok(0x42);
            
            // Tick the DMA and see if we need to write a byte
            if let Some((value, _index)) = dma.tick(read_fn) {
                assert_eq!(value, 0x42);
                bytes_written += 1;
            }
        }
        
        // Verify we wrote exactly 256 bytes
        assert_eq!(bytes_written, 256);
        
        // Verify transfer is complete
        assert_eq!(dma.transfer_active, false);
        assert_eq!(dma.cycles_remaining, 0);
    }
    
    #[test]
    fn test_dma_synchronous_transfer() {
        let mut dma = DmaController::new();
        
        // Create a test memory source
        let mut source_mem = [0; 256];
        for i in 0..256 {
            source_mem[i] = i as u8;
        }
        
        // Create a test OAM destination
        let mut oam = [0; 256];
        
        // Create read and write functions
        let read_fn = |addr: u16| {
            let index = (addr & 0xFF) as usize;
            Ok(source_mem[index])
        };
        
        let write_fn = |index: u8, value: u8| {
            oam[index as usize] = value;
            Ok(())
        };
        
        // Perform the transfer
        dma.perform_transfer(0x00, read_fn, write_fn).unwrap();
        
        // Verify all bytes were transferred correctly
        for i in 0..256 {
            assert_eq!(oam[i], i as u8);
        }
    }
    
    #[test]
    fn test_dma_integration() {
        // Create the components
        let cpu = Rc::new(RefCell::new(Cpu::new()));
        let ppu = Rc::new(RefCell::new(Ppu::new()));
        
        // Create memory with test data
        let ram = Rc::new(RefCell::new(Ram::with_range(0x0000, 0x1FFF)));
        let test_page = Rc::new(RefCell::new(Ram::with_range(0x0200, 0x02FF)));
        
        // Fill the test page with test data
        {
            let mut test_page = test_page.borrow_mut();
            for i in 0..=255 {
                test_page.write_byte(0x0200 + i, i as u8).unwrap();
            }
        }
        
        // Create a bus and connect components
        let bus = Rc::new(RefCell::new(Bus::new()));
        {
            let mut bus = bus.borrow_mut();
            bus.attach_component(ram);
            bus.attach_component(test_page);
            bus.attach_component(ppu.clone());
        }
        
        // Connect CPU to bus
        cpu.borrow_mut().connect_memory(bus.clone());
        
        // Create DMA controller and connect it to CPU and PPU
        let mut dma = DmaController::new();
        dma.connect_cpu(cpu.clone());
        dma.connect_ppu(ppu.clone());
        
        // Trigger the DMA transfer from page $02
        dma.write_byte(0x4014, 0x02).unwrap();
        
        // Run the DMA transfer for 513 cycles
        for _ in 0..513 {
            // Create a memory read function that uses the CPU
            let cpu_ref = Rc::clone(&cpu);
            let read_fn = |addr| cpu_ref.borrow().read_byte(addr);
            
            // Tick the DMA
            if let Some((value, index)) = dma.tick(read_fn) {
                // Write to PPU OAM via registers
                let mut ppu = ppu.borrow_mut();
                ppu.write_register(0x2003, index);
                ppu.write_register(0x2004, value);
            }
        }
        
        // Verify the data was transferred correctly by reading back from OAM through PPU registers
        for i in 0..=255 {
            let mut ppu_mut = ppu.borrow_mut();
            
            // Set the OAM address
            ppu_mut.write_register(0x2003, i as u8);
            
            // Read the data with the same mutable borrow
            let value = ppu_mut.read_register(0x2004);
            
            // Should match the original data
            assert_eq!(value, i as u8, "OAM byte {} should be {}", i, i);
            
            // Drop the borrow before next iteration
            drop(ppu_mut);
        }
    }
} 