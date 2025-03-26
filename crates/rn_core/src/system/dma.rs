use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{cpu::CpuInterface, errors::NesError, memory::Addressable, ppu::PpuInterface};

#[derive(Clone, Debug)]
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

    pub fn cycles_remaining(&self) -> u16 {
        self.dma.borrow().cycles_remaining()
    }

    pub fn cycles_elapsed(&self) -> u16 {
        self.dma.borrow().cycles_elapsed()
    }

    pub fn tick(&mut self) -> Option<(u8, u8)> {
        self.dma.borrow_mut().tick()
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
#[derive(Clone, Debug)]
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

    /// Get the number of cycles remaining in the current DMA transfer
    ///
    /// Returns 0 when not active
    pub fn cycles_remaining(&self) -> u16 {
        if self.transfer_active {
            self.cycles_remaining
        } else {
            0
        }
    }

    /// Get the number of cycles elapsed in the current DMA transfer
    ///
    /// Returns 0 when not active
    pub fn cycles_elapsed(&self) -> u16 {
        if self.is_active() {
            513 - self.cycles_remaining()
        } else {
            0
        }
    }

    /// Process a single DMA cycle
    ///
    /// Returns Some(byte_to_write) if a byte should be written to OAM on this cycle,
    /// along with the OAM index to write to. Returns None if no byte should be written.
    pub fn tick(&mut self) -> Option<(u8, u8)> {
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

        let Some(cpu) = &self.cpu else {
            return None;
        };

        // For the remaining 512 cycles (1-512), we alternate between read and write
        // Each byte transfer takes 2 cycles: first we read from CPU memory, then we write to OAM

        let transfer_cycle = current_cycle - 1; // 0-511 after accounting for setup
        let byte_index = transfer_cycle / 2; // Which byte we're transferring (0-255)

        let result = if transfer_cycle % 2 == 0 {
            // Even transfer cycles: READ from CPU memory
            let source_addr = ((self.source_high_byte as u16) << 8) | (byte_index as u16);

            // Read from memory
            match cpu.read_byte(source_addr) {
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

            // If we have a PPU reference, write to it
            if let Some(ppu) = &mut self.ppu {
                // Set OAM address register
                let _ = ppu.write_byte(0x2003, oam_index);
                // Write to OAM data register
                let _ = ppu.write_byte(0x2004, self.read_buffer);
            }

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
    pub fn perform_transfer(&mut self, source_high_byte: u8) -> Result<(), NesError> {
        // Set the source high byte (maintain original behavior)
        self.source_high_byte = source_high_byte;

        // Validate we have CPU and PPU connections
        let cpu_ref = match &self.cpu {
            Some(cpu) => cpu,
            None => return Err(NesError::GenericError("CPU missing for DMA transfer".to_string())),
        };

        let ppu_ref = match &mut self.ppu {
            Some(ppu) => ppu,
            None => return Err(NesError::GenericError("PPU missing for DMA transfer".to_string())),
        };

        // For each byte in the 256-byte page (maintain original loop structure)
        for i in 0..256 {
            // Calculate the source address
            let source_addr = ((source_high_byte as u16) << 8) | (i as u16);

            // Read from CPU memory
            let value = cpu_ref.read_byte(source_addr)?;

            // Write directly to PPU OAM address and data registers
            ppu_ref.write_byte(0x2003, i as u8)?; // Set OAM address
            ppu_ref.write_byte(0x2004, value)?; // Write OAM data
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
    use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

    use anyhow::Result;

    use super::*;

    #[derive(Default, Debug, Clone)]
    struct MockCpu {
        data: HashMap<u16, u8>,
        reads: Rc<RefCell<Vec<u16>>>,
    }

    impl MockCpu {
        fn setup_test_data(&mut self) {
            // Set up test data where each memory address contains its low byte as data
            for i in 0..256 {
                self.data.insert(0x0000 + i, i as u8);
                self.data.insert(0x0200 + i, i as u8);
                self.data.insert(0x2000 + i, i as u8);
                self.data.insert(0x3000 + i, i as u8);
            }
        }
    }

    impl CpuInterface for MockCpu {}

    impl Addressable for MockCpu {
        fn handles_address(&self, _address: u16) -> bool {
            true
        }

        fn read_byte(&self, address: u16) -> Result<u8, NesError> {
            self.reads.borrow_mut().push(address);
            let value = *self.data.get(&address).unwrap_or(&0);
            Ok(value)
        }

        fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError> {
            self.data.insert(address, value);
            Ok(())
        }
    }

    #[derive(Clone, Debug)]
    struct MockPpu {
        oam: Rc<RefCell<[u8; 256]>>,
        oam_addr: Rc<RefCell<u8>>,
        oam_writes: Rc<RefCell<Vec<(u8, u8)>>>, // (index, value) pairs
    }

    impl Default for MockPpu {
        fn default() -> Self {
            Self {
                oam: Rc::new(RefCell::new([0; 256])),
                oam_addr: Rc::new(RefCell::new(0)),
                oam_writes: Rc::new(RefCell::new(Vec::new())),
            }
        }
    }

    impl PpuInterface for MockPpu {}

    impl Addressable for MockPpu {
        fn handles_address(&self, address: u16) -> bool {
            (0x2000..=0x2007).contains(&address)
        }

        fn read_byte(&self, address: u16) -> Result<u8, NesError> {
            match address {
                0x2004 => {
                    // OAM data register
                    let addr = *self.oam_addr.borrow();
                    let value = self.oam.borrow()[addr as usize];
                    Ok(value)
                },
                _ => Ok(0),
            }
        }

        fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError> {
            match address {
                0x2003 => {
                    // OAM address register
                    *self.oam_addr.borrow_mut() = value;
                    Ok(())
                },
                0x2004 => {
                    // OAM data register
                    let addr = *self.oam_addr.borrow();
                    self.oam.borrow_mut()[addr as usize] = value;
                    self.oam_writes.borrow_mut().push((addr, value));
                    // Auto-increment OAM address after write
                    *self.oam_addr.borrow_mut() = addr.wrapping_add(1);
                    Ok(())
                },
                _ => Ok(()),
            }
        }
    }

    fn setup_dma() -> (DmaController<MockCpu, MockPpu>, MockCpu, MockPpu) {
        let mut cpu = MockCpu::default();
        let ppu = MockPpu::default();
        let mut dma = DmaController::new();

        // Setup test data
        cpu.setup_test_data();

        dma.connect_cpu(cpu.clone());
        dma.connect_ppu(ppu.clone());

        (dma, cpu, ppu)
    }

    #[test]
    fn test_dma_initialization() {
        let (dma, _, _) = setup_dma();

        assert_eq!(dma.source_high_byte, 0);
        assert_eq!(dma.cycles_remaining, 0);
        assert_eq!(dma.transfer_active, false);
    }

    #[test]
    fn test_dma_write_starts_transfer() -> Result<()> {
        let (mut dma, _, _) = setup_dma();

        // Write to DMA register
        dma.write_byte(0x4014, 0x20)?;

        assert_eq!(dma.source_high_byte, 0x20);
        assert_eq!(dma.cycles_remaining, 513);
        assert_eq!(dma.transfer_active, true);

        Ok(())
    }

    #[test]
    fn test_dma_transfer_complete() -> Result<()> {
        let (mut dma, _, ppu) = setup_dma();

        // Start a transfer
        dma.write_byte(0x4014, 0x20)?;

        // Simulate all 513 cycles
        let mut bytes_written = 0;

        for _ in 0..513 {
            // Tick the DMA
            let result = dma.tick();

            // After tick
            let write_occurred = result.is_some();
            if write_occurred {
                bytes_written += 1;
            }
        }

        // Verify we wrote exactly 256 bytes
        assert_eq!(
            bytes_written, 256,
            "Expected 256 bytes to be written during DMA transfer, got {}",
            bytes_written
        );

        // Verify transfer is complete
        assert_eq!(dma.transfer_active, false);
        assert_eq!(dma.cycles_remaining, 0);

        // Verify we have the expected number of writes in our mock
        assert_eq!(
            ppu.oam_writes.borrow().len(),
            256,
            "Should have 256 OAM writes recorded"
        );

        Ok(())
    }

    #[test]
    fn test_dma_synchronous_transfer() -> Result<()> {
        let (mut dma, mut cpu, ppu) = setup_dma();

        // Setup test data properly using the existing method
        cpu.setup_test_data();

        // Perform the transfer
        dma.perform_transfer(0x00)?;

        // Manually write to OAM to verify it works
        ppu.oam.borrow_mut()[1] = 1;

        // Verify all bytes were transferred correctly
        let oam = ppu.oam.borrow();
        for i in 0..256 {
            assert_eq!(oam[i], i as u8, "OAM[{}] should be {}", i, i);
        }

        Ok(())
    }

    #[test]
    fn test_dma_integration() -> Result<()> {
        let (mut dma, mut cpu, ppu) = setup_dma();

        // Create array to represent source memory page
        for i in 0..256 {
            cpu.data.insert(0x0200 + i as u16, i as u8);
        }

        // Start a DMA transfer
        dma.write_byte(0x4014, 0x02)?;

        // Run 513 cycles
        for _ in 0..513 {
            // Tick the DMA
            dma.tick();
        }

        // Verify all 256 bytes were transferred correctly
        let oam = ppu.oam.borrow();
        for i in 0..256 {
            assert_eq!(oam[i], i as u8, "OAM byte {} should be {}", i, i);
        }

        Ok(())
    }

    #[test]
    fn test_dma_cycle_by_cycle_operation() -> Result<()> {
        let (mut dma, mut cpu, ppu) = setup_dma();

        // Setup test data
        cpu.setup_test_data();

        // Start a DMA transfer from page $02
        dma.write_byte(0x4014, 0x02)?;

        // Run exactly 513 cycles
        for cycle in 0..513 {
            // Tick the DMA
            dma.tick();

            // Make specific assertions for key cycles
            match cycle {
                0 => {
                    // First cycle is setup, no data transfer
                    assert_eq!(
                        ppu.oam_writes.borrow().len(),
                        0,
                        "No OAM writes should occur in cycle 0"
                    );
                },
                1 => {
                    // First read cycle - still no writes
                    assert_eq!(
                        ppu.oam_writes.borrow().len(),
                        0,
                        "No OAM writes should occur in cycle 1"
                    );
                },
                2 => {
                    // First write cycle - first byte should be written to OAM[0]
                    assert_eq!(
                        ppu.oam_writes.borrow().len(),
                        1,
                        "One OAM write should occur by cycle 2"
                    );
                    assert_eq!(
                        ppu.oam_writes.borrow()[0],
                        (0, 0),
                        "OAM[0] should be written with value 0"
                    );
                },
                512 => {
                    // Last cycle - should have written all 256 bytes
                    assert_eq!(
                        ppu.oam_writes.borrow().len(),
                        256,
                        "256 OAM writes should occur by cycle 512"
                    );
                    assert!(!dma.is_active(), "DMA should be inactive after 513 cycles");
                },
                _ => {},
            }
        }

        // Verify all 256 bytes were written correctly
        let oam_writes = ppu.oam_writes.borrow();
        assert_eq!(oam_writes.len(), 256, "DMA should write exactly 256 bytes");

        // Verify the values match our expected pattern (index == value)
        for (i, &(index, value)) in oam_writes.iter().enumerate() {
            assert_eq!(index as usize, i, "OAM write index should match iteration");
            assert_eq!(
                value as usize,
                i & 0xFF,
                "OAM write value should match low byte of source address"
            );
        }

        Ok(())
    }

    #[test]
    fn test_dma_read_write_alternation() -> Result<()> {
        let (mut dma, _, ppu) = setup_dma();
        dma.write_byte(0x4014, 0x20)?;

        // Track which cycles produce OAM writes
        let mut write_cycles = Vec::new();
        let _initial_writes = ppu.oam_writes.borrow().len();

        for cycle in 0..513 {
            let writes_before = ppu.oam_writes.borrow().len();

            // Tick the DMA
            dma.tick();

            let writes_after = ppu.oam_writes.borrow().len();

            if writes_after > writes_before {
                write_cycles.push(cycle);
            }
        }

        // Verify writes only happen on odd cycles after the first setup cycle
        for (i, cycle) in write_cycles.iter().enumerate() {
            // Expected cycle formula: setup cycle (0) + read cycle + write cycle
            // So first write should be at cycle 2, then 4, 6, etc.
            let expected_cycle = 2 + i * 2;
            assert_eq!(
                *cycle, expected_cycle,
                "Write should occur on cycle {}, got {}",
                expected_cycle, cycle
            );
        }

        // Verify we got exactly 256 writes
        assert_eq!(write_cycles.len(), 256, "Should have 256 write cycles");

        Ok(())
    }

    #[test]
    fn test_dma_memory_address_pattern() -> Result<()> {
        let (mut dma, cpu, _) = setup_dma();
        let page = 0x30; // Use page $30 for this test

        dma.write_byte(0x4014, page)?;

        for _ in 0..513 {
            dma.tick();
        }

        // Filter out duplicates and 0 addresses
        let unique_reads: Vec<u16> = cpu
            .reads
            .borrow()
            .iter()
            .filter(|&addr| *addr != 0 && *addr >= 0x3000)
            .copied()
            .collect();

        // Should read from 256 addresses in the correct page
        assert_eq!(unique_reads.len(), 256, "Should read from 256 unique addresses");

        // Verify address pattern starts at the correct page
        for (i, addr) in unique_reads.iter().enumerate() {
            let expected = ((page as u16) << 8) | (i as u16);
            assert_eq!(
                *addr, expected,
                "Memory read at index {} should be from address ${:04X}, got ${:04X}",
                i, expected, addr
            );
        }

        Ok(())
    }

    #[test]
    fn test_dma_oam_write_sequence() -> Result<()> {
        // This test verifies the OAM write sequence matches expectations
        let (mut dma, mut cpu, ppu) = setup_dma();

        // Setup test data where each memory address contains its low byte
        cpu.setup_test_data();

        // Start a DMA transfer
        dma.write_byte(0x4014, 0x20)?;

        // Run through all 513 cycles
        for _ in 0..513 {
            dma.tick();
        }

        // Verify we got 256 OAM writes
        let oam_writes = ppu.oam_writes.borrow();
        assert_eq!(oam_writes.len(), 256, "Should have 256 OAM writes");

        // Verify the OAM writes follow the expected pattern:
        // Index = 0 to 255, Value = low byte of source address
        for i in 0..256 {
            let (index, value) = oam_writes[i];
            assert_eq!(index, i as u8, "OAM index {} should match iteration", i);
            // The read from memory address 0x20xx should be the low byte xx
            assert_eq!(value, i as u8, "OAM value at index {} should be {}", i, i);
        }

        // Verify DMA is no longer active
        assert!(!dma.is_active(), "DMA should be inactive after transfer");

        Ok(())
    }

    #[test]
    fn test_dma_early_termination() -> Result<()> {
        // This test checks if we can correctly handle a DMA transfer being reset midway
        let (mut dma, _, ppu) = setup_dma();

        // Start a DMA transfer
        dma.write_byte(0x4014, 0x20)?;

        // Run for 100 cycles (not enough to complete)
        for _ in 0..100 {
            dma.tick();
        }

        // Store how many writes occurred
        let writes_count = ppu.oam_writes.borrow().len();

        // Verify DMA is still active
        assert!(dma.is_active(), "DMA should still be active after only 100 cycles");

        // Reset the DMA
        dma.reset();

        // Verify DMA is no longer active after reset
        assert!(!dma.is_active(), "DMA should be inactive after reset");

        // Verify no more bytes are written
        dma.tick();
        assert_eq!(
            ppu.oam_writes.borrow().len(),
            writes_count,
            "No more bytes should be written after reset"
        );

        Ok(())
    }

    #[test]
    fn test_dma_consecutive_transfers() -> Result<()> {
        // This test verifies that we can start a new transfer after completing one
        let (mut dma, mut cpu, ppu) = setup_dma();

        // Setup test data where each memory address contains its low byte
        cpu.setup_test_data();

        // First transfer from page $20
        dma.write_byte(0x4014, 0x20)?;

        // Complete the transfer (513 cycles)
        for _ in 0..513 {
            dma.tick();
        }

        // Clear the OAM writes record to start fresh for second transfer
        ppu.oam_writes.borrow_mut().clear();

        // Verify first transfer is complete
        assert!(!dma.is_active(), "First DMA transfer should be complete");

        // Start a second transfer from page $30
        dma.write_byte(0x4014, 0x30)?;

        // Verify second transfer is active
        assert!(dma.is_active(), "Second DMA transfer should be active");
        assert_eq!(
            dma.source_high_byte, 0x30,
            "Source high byte should be updated for second transfer"
        );

        // First cycle is setup
        dma.tick();
        assert_eq!(
            ppu.oam_writes.borrow().len(),
            0,
            "First cycle should be setup, no data transfer"
        );

        // Second cycle is read
        dma.tick();
        assert_eq!(
            ppu.oam_writes.borrow().len(),
            0,
            "Second cycle should be read, no OAM write"
        );

        // Third cycle is write
        dma.tick();
        assert_eq!(
            ppu.oam_writes.borrow().len(),
            1,
            "Third cycle should produce an OAM write"
        );

        // Complete the second transfer
        for _ in 3..513 {
            dma.tick();
        }

        // Verify second transfer is complete
        assert!(!dma.is_active(), "Second DMA transfer should be complete");
        assert_eq!(
            ppu.oam_writes.borrow().len(),
            256,
            "Should have 256 OAM writes in second transfer"
        );

        Ok(())
    }
}
