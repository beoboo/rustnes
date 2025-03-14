use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use log::{debug, error, info, warn};

use crate::{
    cartridge::Cartridge,
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
        let old_state = self.state;
        self.state = SystemState::Ready;
        debug!("System state transition: {:?} -> {:?}", old_state, self.state);
        self.error_message = None;

        Ok(())
    }

    /// Load a program into memory
    pub fn load_program(&mut self, program: &[u8], address: u16) -> Result<(), NesError> {
        self.cpu.load_program(program, address)?;
        let old_state = self.state;
        self.state = SystemState::Loaded;
        debug!("System state transition: {:?} -> {:?}", old_state, self.state);
        self.error_message = None;
        info!("Program loaded at ${:04X}, size: {} bytes", address, program.len());
        Ok(())
    }

    /// Step the system by one CPU instruction
    ///
    /// Returns the number of CPU cycles used
    pub fn step(&mut self) -> Result<u8, NesError> {
        if self.state == SystemState::Finished || matches!(self.state, SystemState::Error(_)) {
            debug!("Skipping step in terminal state: {:?}", self.state);
            return Ok(0); // Don't step if already finished or in error state
        }

        // Set state to running
        let old_state = self.state;
        self.state = SystemState::Running;
        if old_state != self.state {
            debug!("System state transition: {:?} -> {:?}", old_state, self.state);
        }

        // Step the CPU and get cycles
        match self.cpu.step() {
            Ok(cpu_cycles) => {
                // Run the PPU at 3x the CPU speed
                for _ in 0..cpu_cycles * 3 {
                    self.ppu.borrow_mut().tick();
                }

                // Check if we've hit a BRK instruction (end of program)
                if self.cpu.read_byte(self.cpu.pc)? == 0x00 {
                    let old_state = self.state;
                    self.state = SystemState::Finished;
                    debug!("System state transition: {:?} -> {:?}", old_state, self.state);
                    info!("BRK instruction encountered at ${:04X}, halting", self.cpu.pc);
                }

                Ok(cpu_cycles)
            },
            Err(err) => {
                // Store the error and set error state
                self.error_message = Some(format!("Execution error: {}", err));
                let old_state = self.state;
                self.state = SystemState::Error(self.cpu.pc);
                error!(
                    "System state transition: {:?} -> Error({:04X}) - {}",
                    old_state, self.cpu.pc, err
                );
                Err(err)
            },
        }
    }

    /// Run the system until completion or error
    ///
    /// Takes a maximum number of steps to prevent infinite loops
    pub fn run(&mut self, max_steps: usize) -> Result<usize, NesError> {
        if self.state != SystemState::Loaded && self.state != SystemState::Running {
            debug!("Skipping run in state: {:?}", self.state);
            return Ok(0); // Don't run if not loaded or already finished
        }

        let mut steps = 0;
        info!("Running program from ${:04X}", self.cpu.pc);

        while steps < max_steps {
            match self.step() {
                Ok(0) => break, // Got 0 cycles, means we're finished
                Ok(_) => steps += 1,
                Err(err) => {
                    error!("Error at step {}: {}", steps, err);
                    return Err(err);
                },
            }

            // Check if we've reached the finished state
            if self.state == SystemState::Finished || matches!(self.state, SystemState::Error(_)) {
                break;
            }
        }

        if steps >= max_steps {
            warn!("Program reached maximum step limit of {}", max_steps);
            self.error_message = Some(format!("Program reached maximum step limit of {}", max_steps));
            let old_state = self.state;
            self.state = SystemState::Error(self.cpu.pc);
            debug!(
                "System state transition: {:?} -> Error({:04X}) - step limit reached",
                old_state, self.cpu.pc
            );
        } else if self.state == SystemState::Running {
            // If we broke out of the loop without error or finishing, consider it finished
            let old_state = self.state;
            self.state = SystemState::Finished;
            debug!("System state transition: {:?} -> {:?}", old_state, self.state);
            info!("Program terminated after {} steps at ${:04X}", steps, self.cpu.pc);
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

    /// Get access to the cartridge from the PPU (if connected)
    pub fn cartridge(&self) -> Option<Rc<RefCell<Cartridge>>> {
        // Get a reference to the PPU
        let ppu = self.ppu.borrow();

        // Use the PPU's cartridge method and clone the Rc if present
        ppu.cartridge().cloned()
    }

    /// Load CHR ROM data into the cartridge
    pub fn load_chr_rom(&mut self, chr_data: &[u8]) {
        // Create a cartridge if one doesn't exist
        let mut ppu = self.ppu.borrow_mut();
        if ppu.cartridge().is_none() {
            // Create a new cartridge
            let cart = Rc::new(RefCell::new(Cartridge::new()));
            // Connect it to the PPU
            ppu.connect_cartridge(cart);
            println!("Created and connected new cartridge");
        }
        drop(ppu); // Release the borrow before the next one

        // Get the PPU's cartridge and load the CHR ROM data
        let mut ppu = self.ppu.borrow_mut();
        if let Some(cart_rc) = ppu.cartridge_mut() {
            // Need to borrow_mut() the RefCell to get mutable access to the cartridge
            cart_rc.borrow_mut().load_chr_rom(chr_data);
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;
    use crate::cpu::Assembler;

    // Create a utility function to assemble code for tests
    fn assemble_code(code: &str, load_address: u16) -> Vec<u8> {
        let mut assembler = Assembler::new(load_address);
        // For tests, we just use the STARTUP segment which is the default
        assembler
            .assemble_program(code)
            .expect("Failed to assemble test code")
            .get("STARTUP")
            .cloned()
            .unwrap_or_default()
    }

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

        // Use assembly code instead of raw bytes
        let program = assemble_code(
            "
            LDA #$37    ; Load $37 into accumulator
            STA $0200   ; Store it in memory
            LDA #$42    ; Load $42 into accumulator
        ",
            0x8000,
        );

        // Load the program
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

        // Use assembly code
        let program = assemble_code(
            "
            NOP    ; A NOP takes 2 cycles
        ",
            0x8000,
        );

        system.cpu_mut().load_program(&program, 0x8000)?;

        let cpu_cycles = system.step()?;
        assert_eq!(cpu_cycles, 2, "NOP should take 2 CPU cycles");

        // The ratio verification would ideally check that PPU
        // advanced by 6 cycles (3x the CPU cycles)
        // For now, we're just verifying the step returns the correct CPU cycles
        Ok(())
    }

    #[test]
    fn test_initial_state() {
        let system = NesSystem::new();
        assert_eq!(system.state(), SystemState::Ready, "Initial state should be Ready");
        assert_eq!(
            system.error_message(),
            None,
            "No error message should be present initially"
        );
    }

    #[test]
    fn test_state_transitions() -> Result<()> {
        let mut system = NesSystem::new();

        // Initial state should be Ready
        assert_eq!(system.state(), SystemState::Ready);

        // After loading a program, state should be Loaded
        let program = assemble_code(
            "
            LDA #$42   ; Load $42 into accumulator
            LDA #$43   ; Load $43 into accumulator
            BRK        ; Break instruction
        ",
            0x8000,
        );

        system.load_program(&program, 0x8000)?;
        assert_eq!(
            system.state(),
            SystemState::Loaded,
            "State should be Loaded after loading program"
        );

        // First step executes LDA #$42, but PC advances to another instruction, not BRK
        system.step()?;
        assert_eq!(
            system.state(),
            SystemState::Running,
            "State should be Running after first step"
        );

        // Second step executes LDA #$43, and now PC points to BRK
        system.step()?;
        // System detects BRK is next and transitions to Finished
        assert_eq!(
            system.state(),
            SystemState::Finished,
            "State should be Finished when PC points to BRK"
        );

        Ok(())
    }

    #[test]
    fn test_run_completion() -> Result<()> {
        let mut system = NesSystem::new();

        // Use only instructions we know are implemented
        let program = assemble_code(
            "
            LDA #$01   ; Load $01 into accumulator
            LDX #$02   ; Load $02 into X register
            LDY #$03   ; Load $03 into Y register
            BRK        ; Break instruction
        ",
            0x8000,
        );

        system.load_program(&program, 0x8000)?;
        assert_eq!(system.state(), SystemState::Loaded);

        // Run the program - should complete and transition to Finished
        let steps = system.run(100)?;
        assert!(steps < 100, "Program should complete in fewer than 100 steps");
        assert_eq!(
            system.state(),
            SystemState::Finished,
            "State should be Finished after run completes"
        );

        // Verify registers have expected values
        assert_eq!(system.cpu().a, 0x01, "A register should contain $01");
        assert_eq!(system.cpu().x, 0x02, "X register should contain $02");
        assert_eq!(system.cpu().y, 0x03, "Y register should contain $03");

        Ok(())
    }

    #[test]
    fn test_error_state() -> Result<()> {
        let mut system = NesSystem::new();

        // Attempt to execute from unmapped memory
        let pc = 0x4000; // Typically unmapped in our system

        // Manually set PC to unmapped region
        system.cpu_mut().pc = pc;

        // Step should fail and set Error state
        let result = system.step();
        assert!(result.is_err(), "Step should fail when PC is in unmapped memory");
        assert!(
            matches!(system.state(), SystemState::Error(error_pc) if error_pc == pc),
            "State should be Error with correct PC"
        );
        assert!(system.error_message().is_some(), "Error message should be present");

        Ok(())
    }

    #[test]
    fn test_terminal_states() -> Result<()> {
        let mut system = NesSystem::new();

        // Set up program that executes BRK immediately
        let program = assemble_code(
            "
            BRK        ; Immediate break
        ",
            0x8000,
        );

        system.load_program(&program, 0x8000)?;

        // Execute to reach Finished state
        system.step()?;
        assert_eq!(system.state(), SystemState::Finished);

        // Attempting to step again should do nothing
        let original_pc = system.cpu().pc;
        let cycles = system.step()?;
        assert_eq!(cycles, 0, "Step should return 0 cycles when in Finished state");
        assert_eq!(
            system.cpu().pc,
            original_pc,
            "PC should not change when stepping in Finished state"
        );
        assert_eq!(system.state(), SystemState::Finished, "State should remain Finished");

        // Reset system
        system.reset()?;
        assert_eq!(system.state(), SystemState::Ready);

        // Create an error state
        system.cpu_mut().pc = 0x4000; // Unmapped memory
        let step_result = system.step();
        assert!(step_result.is_err());
        assert!(matches!(system.state(), SystemState::Error(_)));

        // Attempting to step in Error state should do nothing
        let error_pc = match system.state() {
            SystemState::Error(pc) => pc,
            _ => panic!("Expected Error state"),
        };
        let cycles = system.step()?;
        assert_eq!(cycles, 0, "Step should return 0 cycles when in Error state");
        assert!(
            matches!(system.state(), SystemState::Error(pc) if pc == error_pc),
            "State should remain Error with same PC"
        );

        Ok(())
    }

    #[test]
    fn test_reset_clears_state() -> Result<()> {
        let mut system = NesSystem::new();

        // Put system in Error state
        system.cpu_mut().pc = 0x4000; // Unmapped memory
        let _ = system.step();
        assert!(matches!(system.state(), SystemState::Error(_)));
        assert!(system.error_message().is_some());

        // Reset should clear state back to Ready
        system.reset()?;
        assert_eq!(system.state(), SystemState::Ready);
        assert_eq!(system.error_message(), None, "Error message should be cleared on reset");

        Ok(())
    }
}
