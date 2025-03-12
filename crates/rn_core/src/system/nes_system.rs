use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use crate::{
    cpu::Cpu,
    errors::NesError,
    memory::Ram,
    ppu::{registers::PpuRegisters, Ppu},
    system::Bus,
};

/// NesSystem coordinates the main components of the NES
pub struct NesSystem {
    /// The CPU component
    cpu: Cpu,

    /// The PPU component
    ppu: Rc<RefCell<Ppu>>,
}

impl NesSystem {
    /// Create a new NesSystem
    pub fn new() -> Self {
        // Create a PPU instance with RefCell for sharing
        let ppu = Rc::new(RefCell::new(Ppu::new()));

        // Create a bus with basic memory mapping
        let mut bus = Bus::new();

        // Attach PPU registers to the CPU bus at $2000-$2007
        let ppu_regs = Box::new(PpuRegisters::new(ppu.clone()));
        bus.attach_component(ppu_regs);

        // Add ROM mapping for program memory (0x8000-0xFFFF)
        // This ensures we have a proper place to load programs
        let rom = Box::new(Ram::with_range(0x8000, 0xFFFF));
        bus.attach_component(rom);

        // Debug: Print the memory map before attaching to CPU
        // This will help diagnose missing memory components during development
        #[cfg(debug_assertions)]
        {
            println!("\n=== NesSystem Memory Map ===");
            println!("{}", bus.debug_memory_map());
            println!("===========================\n");
        }

        // Create the CPU with its bus
        let cpu = Cpu::new(Box::new(bus));

        Self { cpu, ppu }
    }

    /// Reset the system
    pub fn reset(&mut self) -> Result<(), NesError> {
        self.cpu.reset()?;
        self.ppu.borrow_mut().reset();

        Ok(())
    }

    /// Step the system by one CPU instruction
    ///
    /// Returns the number of CPU cycles used
    pub fn step(&mut self) -> Result<u8, NesError> {
        // Step the CPU and get cycles
        let cpu_cycles = self.cpu.step()?;

        // Run the PPU at 3x the CPU speed
        for _ in 0..cpu_cycles * 3 {
            self.ppu.borrow_mut().tick();
        }

        Ok(cpu_cycles)
    }

    /// Get the CPU
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    /// Get mutable access to the CPU
    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    /// Get the PPU
    pub fn ppu(&self) -> Ref<Ppu> {
        self.ppu.borrow()
    }

    /// Get mutable access to the PPU
    pub fn ppu_mut(&mut self) -> RefMut<Ppu> {
        self.ppu.borrow_mut()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;
    #[test]
    fn test_system_creation() {
        let _system = NesSystem::new();
        // Just verify we can create one without panicking
    }

    #[test]
    fn test_component_interaction() -> Result<()> {
        // Test 1: Memory operations through CPU
        let mut system = NesSystem::new();

        // Write a value to memory using CPU
        system.cpu_mut().write_byte(0x0200, 0x42)?;

        // Read it back and verify
        let value = system.cpu().read_byte(0x0200)?;
        assert_eq!(value, 0x42, "CPU should be able to read value it wrote");

        // Test 2: Program execution and CPU state
        let mut system = NesSystem::new();

        // Program: LDA #$37, STA $0200, LDA #$42
        let program = [0xA9, 0x37, 0x8D, 0x00, 0x02, 0xA9, 0x42];
        // Use CPU's load_program method directly
        system.cpu_mut().load_program(&program, 0x8000)?;

        // Execute first instruction (LDA #$37)
        let _ = system.step(); // Ignoring the result for now
        assert_eq!(system.cpu().a, 0x37, "A register should contain $37");

        // Execute second instruction (STA $0200)
        let _ = system.step(); // Ignoring the result for now
        assert_eq!(
            system.cpu().read_byte(0x0200)?,
            0x37,
            "Memory at $0200 should contain $37"
        );

        // Execute third instruction (LDA #$42)
        let _ = system.step(); // Ignoring the result for now
        assert_eq!(system.cpu().a, 0x42, "A register should contain $42");

        Ok(())
    }

    #[test]
    fn test_timing_ratio() -> Result<()> {
        // This test assumes the PPU has a method to count ticks or some observable
        // effect of ticks that we can verify. For now, we'll just test the concept.

        let mut system = NesSystem::new();

        // Execute an instruction (e.g., NOP) that takes 2 cycles
        let program = [0xEA]; // NOP takes 2 cycles
        system.cpu_mut().load_program(&program, 0x8000)?;

        let cpu_cycles = system.step()?;
        assert_eq!(cpu_cycles, 2, "NOP should take 2 CPU cycles");

        // The ratio verification would ideally check that PPU
        // advanced by 6 cycles (3x the CPU cycles)
        // For now, we're just verifying the step returns the correct CPU cycles
        Ok(())
    }
}
