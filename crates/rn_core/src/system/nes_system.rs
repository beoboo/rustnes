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

/// The possible states of the NES system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemState {
    Ready,      // System is reset, no program loaded
    Loaded,     // Program loaded but not running
    Running,    // Program is actively running
    Finished,   // Program has finished execution (hit BRK or error)
    Error(u16), // System encountered an error (with PC where error occurred)
}

/// NesSystem coordinates the main components of the NES
pub struct NesSystem {
    /// The CPU component
    cpu: Cpu,

    /// The PPU component
    ppu: Rc<RefCell<Ppu>>,
    
    /// Current system state
    state: SystemState,
    
    /// Error message if in Error state
    error_message: Option<String>,
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

        Self { 
            cpu, 
            ppu,
            state: SystemState::Ready,
            error_message: None,
        }
    }

    /// Reset the system
    pub fn reset(&mut self) -> Result<(), NesError> {
        self.cpu.reset()?;
        self.ppu.borrow_mut().reset();
        self.state = SystemState::Ready;
        self.error_message = None;

        Ok(())
    }

    /// Load a program into memory
    pub fn load_program(&mut self, program: &[u8], address: u16) -> Result<(), NesError> {
        self.cpu.load_program(program, address)?;
        self.state = SystemState::Loaded;
        self.error_message = None;
        Ok(())
    }

    /// Step the system by one CPU instruction
    ///
    /// Returns the number of CPU cycles used
    pub fn step(&mut self) -> Result<u8, NesError> {
        if self.state == SystemState::Finished || matches!(self.state, SystemState::Error(_)) {
            return Ok(0); // Don't step if already finished or in error state
        }
        
        // Set state to running
        self.state = SystemState::Running;

        // Step the CPU and get cycles
        match self.cpu.step() {
            Ok(cpu_cycles) => {
                // Run the PPU at 3x the CPU speed
                for _ in 0..cpu_cycles * 3 {
                    self.ppu.borrow_mut().tick();
                }

                // Check if we've hit a BRK instruction (end of program)
                if self.cpu.read_byte(self.cpu.pc)? == 0x00 {
                    self.state = SystemState::Finished;
                    println!("BRK instruction encountered at ${:04X}, halting", self.cpu.pc);
                }
                
                Ok(cpu_cycles)
            },
            Err(err) => {
                // Store the error and set error state
                self.error_message = Some(format!("Execution error: {}", err));
                self.state = SystemState::Error(self.cpu.pc);
                Err(err)
            }
        }
    }

    /// Run the system until completion or error
    /// 
    /// Takes a maximum number of steps to prevent infinite loops
    pub fn run(&mut self, max_steps: usize) -> Result<usize, NesError> {
        if self.state != SystemState::Loaded && self.state != SystemState::Running {
            return Ok(0); // Don't run if not loaded or already finished
        }
        
        let mut steps = 0;
        println!("Running program from ${:04X}", self.cpu.pc);
        
        while steps < max_steps {
            match self.step() {
                Ok(0) => break, // Got 0 cycles, means we're finished
                Ok(_) => steps += 1,
                Err(err) => {
                    println!("Error at step {}: {}", steps, err);
                    return Err(err);
                }
            }
            
            // Check if we've reached the finished state
            if self.state == SystemState::Finished || matches!(self.state, SystemState::Error(_)) {
                break;
            }
        }
        
        if steps >= max_steps {
            println!("Program reached maximum step limit of {}", max_steps);
            self.error_message = Some(format!("Program reached maximum step limit of {}", max_steps));
            self.state = SystemState::Error(self.cpu.pc);
        } else if self.state == SystemState::Running {
            // If we broke out of the loop without error or finishing, consider it finished
            self.state = SystemState::Finished;
            println!("Program terminated after {} steps at ${:04X}", steps, self.cpu.pc);
        }
        
        Ok(steps)
    }

    /// Get the current system state
    pub fn state(&self) -> SystemState {
        self.state
    }
    
    /// Get the current error message if any
    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
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
