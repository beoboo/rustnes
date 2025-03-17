use std::{cell::RefCell, rc::Rc};

use crate::{cpu::CpuInterface, errors::NesError, memory::Addressable, ppu::PpuInterface};

#[derive(Clone)]
pub struct DmaControllerWrapper<C: CpuInterface, P: PpuInterface> {
    dma: Rc<RefCell<DmaController<C, P>>>,
}
impl<C: CpuInterface, P: PpuInterface> DmaControllerWrapper<C, P> {
    pub(crate) fn new(dma: DmaController<C, P>) -> Self {
        Self {
            dma: Rc::new(RefCell::new(dma)),
        }
    }

    pub fn connect_cpu(&mut self, cpu: C) {
        self.dma.borrow_mut().connect_cpu(cpu);
    }

    pub fn connect_ppu(&mut self, ppu: P) {
        self.dma.borrow_mut().connect_ppu(ppu);
    }

    pub fn is_active(&self) -> bool {
        self.dma.borrow().is_active()
    }

    pub fn tick(&mut self, memory_read_fn: impl FnOnce(u16) -> Result<u8, NesError>) -> Option<(u8, u8)> {
        self.dma.borrow_mut().tick(memory_read_fn)
    }
}

impl<C: CpuInterface, P: PpuInterface> Addressable for DmaControllerWrapper<C, P> {
    fn handles_address(&self, address: u16) -> bool {
        self.dma.borrow().handles_address(address)
    }

    fn read_byte(&self, address: u16) -> Result<u8, NesError> {
        self.dma.borrow().read_byte(address)
    }

    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError> {
        self.dma.borrow_mut().write_byte(address, value)
    }
}

/// DMA Controller for transferring data from CPU memory to PPU OAM
///
/// In the NES, writing to address $4014 triggers a DMA transfer of 256 bytes
/// from CPU memory to PPU OAM. The CPU is suspended during this transfer.
pub struct DmaController<C: CpuInterface, P: PpuInterface> {
    /// Source high byte for DMA transfer (written to $4014)
    source_high_byte: u8,

    /// Number of cycles remaining in current DMA transfer
    cycles_remaining: u16,

    /// Whether a transfer is active
    transfer_active: bool,

    /// Read buffer - stores the last read value
    read_buffer: u8,

    /// CPU component
    cpu: Option<C>,

    /// PPU component
    ppu: Option<P>,
}

impl<C: CpuInterface, P: PpuInterface> DmaController<C, P> {
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
    pub fn connect_cpu(&mut self, cpu: C) {
        self.cpu = Some(cpu);
    }

    /// Connect the PPU component
    pub fn connect_ppu(&mut self, ppu: P) {
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
        // Check if transfer is active
        if !self.transfer_active {
            return None;
        }

        // Calculate the current cycle index (0-512)
        let current_cycle = 513 - self.cycles_remaining;

        // First cycle (0) is just setup, no data transfer
        if current_cycle == 0 {
            self.cycles_remaining -= 1;
            return None;
        }

        // For the remaining 512 cycles (1-512), we alternate between read and write
        // Each byte transfer takes 2 cycles: first we read from CPU memory, then we write to OAM

        let transfer_cycle = current_cycle - 1; // 0-511 after accounting for setup
        let byte_index = transfer_cycle / 2; // Which byte we're transferring (0-255)

        let result = if transfer_cycle % 2 == 0 {
            // Even transfer cycles: READ from CPU memory
            let source_addr = ((self.source_high_byte as u16) << 8) | (byte_index as u16);

            // Read from memory
            match memory_read_fn(source_addr) {
                Ok(value) => {
                    self.read_buffer = value;
                },
                Err(_) => {
                    // On error, use 0 as fallback
                    self.read_buffer = 0;
                },
            }

            // No OAM write on read cycles
            None
        } else {
            // Odd transfer cycles: WRITE to OAM
            let oam_index = byte_index as u8;

            // Return the value to write to OAM
            Some((self.read_buffer, oam_index))
        };

        // Decrement remaining cycles AFTER processing the current cycle
        self.cycles_remaining -= 1;

        // If we've reached 0, the transfer is complete
        if self.cycles_remaining == 0 {
            self.transfer_active = false;
        }

        result
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

impl<C: CpuInterface, P: PpuInterface> Addressable for DmaController<C, P> {
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

impl<C: CpuInterface, P: PpuInterface> Default for DmaController<C, P> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct MockCpu;
    impl CpuInterface for MockCpu {}

    struct MockPpu;
    impl PpuInterface for MockPpu {}

    fn setup_dma() -> DmaController<MockCpu, MockPpu> {
        let mut dma = DmaController::new();
        dma.connect_cpu(MockCpu);
        dma
    }

    #[test]
    fn test_dma_initialization() {
        let dma = setup_dma();

        assert_eq!(dma.source_high_byte, 0);
        assert_eq!(dma.cycles_remaining, 0);
        assert_eq!(dma.transfer_active, false);
    }

    #[test]
    fn test_dma_write_starts_transfer() {
        let mut dma = setup_dma();

        // Write to DMA register
        dma.write_byte(0x4014, 0x20).unwrap();

        assert_eq!(dma.source_high_byte, 0x20);
        assert_eq!(dma.cycles_remaining, 513);
        assert_eq!(dma.transfer_active, true);
    }

    #[test]
    fn test_dma_transfer_complete() {
        let mut dma = setup_dma();

        // Start a transfer
        dma.write_byte(0x4014, 0x20).unwrap();

        // Simulate all 513 cycles, creating a dummy read function
        let mut bytes_written = 0;

        // Uncomment to see what's happening in each cycle:
        // println!("Cycle | Remaining | Active | Result");
        // println!("----- | --------- | ------ | ------");

        for _ in 0..513 {
            let read_fn = |_addr| Ok(0x42);

            // Tick the DMA
            let result = dma.tick(read_fn);

            // After tick
            let write_occurred = result.is_some();
            if write_occurred {
                bytes_written += 1;
            }

            // Debug print:
            // println!("{:5} | {:9} | {:6} | {:?}",
            //         i, dma.cycles_remaining, dma.transfer_active,
            //         if write_occurred { "Write" } else { "None" });
        }

        // Verify we wrote exactly 256 bytes (this is what was failing)
        assert_eq!(
            bytes_written, 256,
            "Expected 256 bytes to be written during DMA transfer, got {}",
            bytes_written
        );

        // Verify transfer is complete
        assert_eq!(dma.transfer_active, false);
        assert_eq!(dma.cycles_remaining, 0);
    }

    #[test]
    fn test_dma_synchronous_transfer() {
        let mut dma = setup_dma();

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
        // Create a simplified test with direct memory functions instead of CPU/bus complexity

        // Create array to represent source memory page
        let mut source_page = [0u8; 256];
        for i in 0..256 {
            source_page[i] = i as u8;
        }

        // Create an array for OAM
        let mut oam = [0u8; 256];

        // Create the DMA controller
        let mut dma = setup_dma();

        // Start a DMA transfer
        dma.write_byte(0x4014, 0x02).unwrap();

        // Run 513 cycles using a simple direct-access function
        for _ in 0..513 {
            let read_fn = |addr: u16| {
                // Only handle addresses we expect from DMA (0x0200-0x02FF)
                if addr >= 0x0200 && addr <= 0x02FF {
                    let index = (addr - 0x0200) as usize;
                    Ok(source_page[index])
                } else {
                    // For unexpected addresses, return 0
                    Ok(0)
                }
            };

            // Tick the DMA
            if let Some((value, index)) = dma.tick(read_fn) {
                // Write directly to OAM array
                oam[index as usize] = value;
            }
        }

        // Verify all 256 bytes were transferred correctly
        for i in 0..256 {
            assert_eq!(oam[i], i as u8, "OAM byte {} should be {}", i, i);
        }
    }

    #[test]
    fn test_dma_cycle_by_cycle_operation() {
        let mut dma = setup_dma();
        
        // Start a DMA transfer from page $02
        dma.write_byte(0x4014, 0x02).unwrap();
        
        // Track the values being written to OAM
        let mut oam_writes = Vec::new();
        
        // Run exactly 513 cycles
        for cycle in 0..513 {
            // Create a read function that returns the low byte of the address as the value
            let read_fn = |addr: u16| {
                let page = (addr >> 8) as u8;
                let offset = addr as u8;
                assert_eq!(page, 0x02, "DMA should read from page $02");
                Ok(offset) // Return the low byte as the test value
            };
            
            // Tick the DMA and track OAM writes
            if let Some((value, index)) = dma.tick(read_fn) {
                oam_writes.push((index, value));
            }
            
            // Make specific assertions for key cycles
            match cycle {
                0 => {
                    // First cycle is setup, no data transfer
                    assert!(oam_writes.is_empty(), "No OAM writes should occur in cycle 0");
                },
                1 => {
                    // First read cycle - still no writes
                    assert!(oam_writes.is_empty(), "No OAM writes should occur in cycle 1");
                },
                2 => {
                    // First write cycle - first byte should be written to OAM[0]
                    assert_eq!(oam_writes.len(), 1, "One OAM write should occur by cycle 2");
                    assert_eq!(oam_writes[0], (0, 0), "OAM[0] should be written with value 0");
                },
                512 => {
                    // Last cycle - should have written all 256 bytes
                    assert_eq!(oam_writes.len(), 256, "256 OAM writes should occur by cycle 512");
                    assert!(!dma.is_active(), "DMA should be inactive after 513 cycles");
                },
                _ => {}
            }
        }
        
        // Verify all 256 bytes were written correctly
        assert_eq!(oam_writes.len(), 256, "DMA should write exactly 256 bytes");
        
        // Verify the values match our expected pattern (index == value)
        for (i, (index, value)) in oam_writes.iter().enumerate() {
            assert_eq!(*index as usize, i, "OAM write index should match iteration");
            assert_eq!(*value as usize, i & 0xFF, "OAM write value should match low byte of source address");
        }
    }

    #[test]
    fn test_dma_read_write_alternation() {
        let mut dma = setup_dma();
        dma.write_byte(0x4014, 0x20).unwrap();
        
        // Track which cycles produce OAM writes
        let mut write_cycles = Vec::new();
        
        for cycle in 0..513 {
            let read_fn = |_addr| Ok(0x42);
            let result = dma.tick(read_fn);
            
            if result.is_some() {
                write_cycles.push(cycle);
            }
        }
        
        // Verify writes only happen on odd cycles after the first setup cycle
        for (i, cycle) in write_cycles.iter().enumerate() {
            // Expected cycle formula: setup cycle (0) + read cycle + write cycle
            // So first write should be at cycle 2, then 4, 6, etc.
            let expected_cycle = 2 + i * 2;
            assert_eq!(*cycle, expected_cycle, 
                "Write should occur on cycle {}, got {}", expected_cycle, cycle);
        }
        
        // Verify we got exactly 256 writes
        assert_eq!(write_cycles.len(), 256, "Should have 256 write cycles");
    }

    #[test]
    fn test_dma_memory_address_pattern() {
        let mut dma = setup_dma();
        let page = 0x30; // Use page $30 for this test
        
        dma.write_byte(0x4014, page).unwrap();
        
        // Track memory reads
        let mut read_addresses = Vec::new();
        
        for _ in 0..513 {
            let read_fn = |addr: u16| {
                read_addresses.push(addr);
                Ok(0x42)
            };
            
            dma.tick(read_fn);
        }
        
        // Filter out duplicates and 0 addresses
        // (Since we're calling the read function even on write cycles,
        // but with a default address of 0)
        let unique_reads: Vec<u16> = read_addresses.into_iter()
            .filter(|&addr| addr != 0)
            .collect();
        
        // Should read exactly 256 unique addresses
        assert_eq!(unique_reads.len(), 256, "Should read from 256 unique addresses");
        
        // Verify address pattern starts at the correct page
        for (i, addr) in unique_reads.iter().enumerate() {
            let expected = ((page as u16) << 8) | (i as u16);
            assert_eq!(*addr, expected, 
                "Memory read at index {} should be from address ${:04X}, got ${:04X}", 
                i, expected, addr);
        }
    }

    #[test]
    fn test_dma_oam_write_sequence() {
        // This test verifies the OAM write sequence matches expectations
        let mut dma = setup_dma();
        
        // Start a DMA transfer
        dma.write_byte(0x4014, 0x20).unwrap();
        
        // Collect all OAM writes
        let mut oam_writes = Vec::new();
        
        // Run through all 513 cycles
        for _ in 0..513 {
            let read_fn = |addr: u16| {
                // Return the low byte as the data value
                // This gives us a predictable pattern to verify
                Ok((addr & 0xFF) as u8)
            };
            
            if let Some((value, index)) = dma.tick(read_fn) {
                oam_writes.push((index, value));
            }
        }
        
        // Verify we got 256 OAM writes
        assert_eq!(oam_writes.len(), 256, "Should have 256 OAM writes");
        
        // Verify the OAM writes follow the expected pattern:
        // Index = 0 to 255, Value = low byte of source address
        for i in 0..256 {
            let (index, value) = oam_writes[i];
            assert_eq!(index, i as u8, "OAM index {} should match iteration", i);
            assert_eq!(value, i as u8, "OAM value at index {} should be {}", i, i);
        }
        
        // Verify DMA is no longer active
        assert!(!dma.is_active(), "DMA should be inactive after transfer");
    }

    #[test]
    fn test_dma_early_termination() {
        // This test checks if we can correctly handle a DMA transfer being reset midway
        let mut dma = setup_dma();
        
        // Start a DMA transfer
        dma.write_byte(0x4014, 0x20).unwrap();
        
        // Run for 100 cycles (not enough to complete)
        let mut oam_writes = Vec::new();
        for _ in 0..100 {
            let read_fn = |_addr| Ok(0x42);
            if let Some((value, index)) = dma.tick(read_fn) {
                oam_writes.push((index, value));
            }
        }
        
        // Verify DMA is still active
        assert!(dma.is_active(), "DMA should still be active after only 100 cycles");
        
        // Reset the DMA
        dma.reset();
        
        // Verify DMA is no longer active after reset
        assert!(!dma.is_active(), "DMA should be inactive after reset");
        
        // Verify no more bytes are written
        let read_fn = |_addr| Ok(0x42);
        let result = dma.tick(read_fn);
        assert!(result.is_none(), "No more bytes should be written after reset");
    }

    #[test]
    fn test_dma_consecutive_transfers() {
        // This test verifies that we can start a new transfer after completing one
        let mut dma = setup_dma();
        
        // First transfer from page $20
        dma.write_byte(0x4014, 0x20).unwrap();
        
        // Complete the transfer (513 cycles)
        for _ in 0..513 {
            let read_fn = |_addr| Ok(0x42);
            dma.tick(read_fn);
        }
        
        // Verify first transfer is complete
        assert!(!dma.is_active(), "First DMA transfer should be complete");
        
        // Start a second transfer from page $30
        dma.write_byte(0x4014, 0x30).unwrap();
        
        // Verify second transfer is active
        assert!(dma.is_active(), "Second DMA transfer should be active");
        assert_eq!(dma.source_high_byte, 0x30, "Source high byte should be updated for second transfer");
        
        // Check that the initial cycles are correct
        let read_fn = |addr: u16| {
            let page = (addr >> 8) as u8;
            assert_eq!(page, 0x30, "Second transfer should read from page $30");
            Ok(0x42)
        };
        
        // First cycle is setup
        let result = dma.tick(read_fn);
        assert!(result.is_none(), "First cycle should be setup, no data transfer");
        
        // Second cycle is read
        let result = dma.tick(read_fn);
        assert!(result.is_none(), "Second cycle should be read, no OAM write");
        
        // Third cycle is write
        let result = dma.tick(read_fn);
        assert!(result.is_some(), "Third cycle should produce an OAM write");
        
        // Complete the second transfer
        for _ in 3..513 {
            let read_fn = |_addr| Ok(0x42);
            dma.tick(read_fn);
        }
        
        // Verify second transfer is complete
        assert!(!dma.is_active(), "Second DMA transfer should be complete");
    }
}
