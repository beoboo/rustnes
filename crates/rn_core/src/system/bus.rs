use std::cell::Cell;

use crate::{
    errors::NesError,
    memory::{Addressable, Ram},
};

/// Bus for routing memory access to appropriate devices
///
/// The Bus acts as a mediator between the CPU and addressable components.
/// It routes read/write operations to the appropriate component based on the address.
#[derive(Debug)]
pub struct Bus {
    /// Components attached to the bus in priority order
    /// First component that handles an address will process the request
    components: Vec<Box<dyn Addressable>>,

    /// The last value the data bus carried — what an unmapped read answers with.
    ///
    /// Large parts of the address space have nothing driving them: `$4018-$401F`, `$4020-$5FFF`,
    /// and `$6000-$7FFF` on a cartridge without RAM. A read there does not fail on hardware and it
    /// does not return zero either. Nothing drives the lines, so they hold the last value that was
    /// put on them, and the CPU reads that back. This is "open bus".
    open_bus: Cell<u8>,

    /// How many accesses have fallen through to open bus. Diagnostic only.
    ///
    /// This bus used to refuse an unmapped access outright, deliberately, so that a mistake in the
    /// memory map would be loud rather than silent. That was the right instinct and the wrong
    /// mechanism: programs read unmapped addresses *on purpose* — every indexed addressing mode
    /// performs a dummy read at an unfixed address, and four of blargg's ROMs exist specifically to
    /// check it — so refusing them stopped the emulator dead on correct programs. The count keeps
    /// the visibility without the fatality.
    open_bus_accesses: Cell<u64>,
}

impl Bus {
    /// Create a new Bus instance with standard NES memory mapping
    ///
    /// This automatically configures:
    /// - RAM at $0000-$1FFF (main system RAM)
    pub fn new() -> Self {
        let mut bus = Self {
            components: Vec::new(),
            open_bus: Cell::new(0),
            open_bus_accesses: Cell::new(0),
        };

        // Two kilobytes of work RAM, answering across $0000-$1FFF. The console decodes only
        // eleven address lines for it, so the same storage appears four times over — the comment
        // here said so for a long time while the RAM behind it was eight flat kilobytes.
        bus.attach_component(Box::new(Ram::mirrored(0x0000, 0x1FFF, 2 * 1024)));

        bus
    }

    /// Attach an addressable component to the bus
    ///
    /// Components are checked in the order they are attached, so the first
    /// component that claims an address will handle it.
    /// Attach a component ahead of everything already attached.
    ///
    /// Lookup takes the first component claiming an address, so this is how a later arrival —
    /// cartridge space, once a ROM is loaded — takes precedence over a placeholder.
    pub fn attach_component_first(&mut self, component: Box<dyn Addressable>) {
        self.components.insert(0, component);
    }

    pub fn attach_component(&mut self, component: Box<dyn Addressable>) {
        self.components.push(component);
    }

    /// Reset all components connected to the bus
    ///
    /// This method resets all components to their initial state.
    /// It should be called when the system is reset.
    pub fn reset(&mut self) {
        for component in &mut self.components {
            component.reset();
        }
    }

    /// Find the component that handles the given address
    ///
    /// Returns a reference to the first component that claims to handle the address,
    /// or None if no component handles it (which shouldn't happen with RAM fallback).
    fn find_component_for_address(&self, address: u16) -> Option<&dyn Addressable> {
        self.components
            .iter()
            .find(|component| component.handles_address(address))
            .map(|component| component.as_ref())
    }

    /// Find the component that handles the given address (mutable version)
    ///
    /// Returns a mutable reference to the first component that claims to handle the address,
    /// or None if no component handles it (which shouldn't happen with RAM fallback).
    /// Find the component that handles *writes* to an address.
    ///
    /// Uses `handles_write`, which differs from `handles_address` only for direction-split
    /// registers such as `$4017`.
    fn find_component_for_address_mut(&mut self, address: u16) -> Option<&mut Box<dyn Addressable>> {
        self.components
            .iter_mut()
            .find(|component| component.handles_write(address))
    }

    /// How many accesses have found nothing driving the bus.
    ///
    /// Expected to be non-zero on a working machine — dummy reads land here — so this is for
    /// noticing a *change*, not for asserting zero.
    pub fn open_bus_accesses(&self) -> u64 {
        self.open_bus_accesses.get()
    }

    /// Returns a debugging string showing all attached components and their address ranges
    pub fn debug_memory_map(&self) -> String {
        let mut result = String::new();
        result.push_str("Memory Map:\n");

        // For debugging, test a set of critical addresses and see which component handles them
        let test_addresses = [
            (0x0000, "Zero Page"),
            (0x0100, "Stack"),
            (0x0200, "RAM"),
            (0x2000, "PPU Registers"),
            (0x4000, "APU Registers"),
            (0x4015, "APU Status"),
            (0x4016, "Controller 1"),
            (0x4017, "Controller 2/APU Frame Counter"),
            (0x8000, "Program Memory (Low)"),
            (0xC000, "Program Memory (High)"),
            (0xFFFA, "NMI Vector"),
            (0xFFFC, "Reset Vector"),
            (0xFFFE, "IRQ Vector"),
        ];

        for (addr, desc) in test_addresses.iter() {
            let component = self.find_component_for_address(*addr);
            result.push_str(&format!(
                "{}: {:#06X} - {}\n",
                desc,
                addr,
                if component.is_some() { "Mapped" } else { "UNMAPPED!" }
            ));
        }

        result
    }
}

impl Addressable for Bus {
    fn handles_address(&self, address: u16) -> bool {
        // The bus handles any address that one of its components can handle
        self.components
            .iter()
            .any(|component| component.handles_address(address))
    }

    /// Likewise for writes, which are not the same set.
    ///
    /// Needed as soon as a component claims an address in one direction only. `$4014` is the case
    /// that found it: the DMA controller takes writes there and nothing takes reads, so a bus that
    /// answered this from `handles_address` alone reported the address unhandled and dropped every
    /// sprite DMA the moment reads of it were given back to the open bus.
    fn handles_write(&self, address: u16) -> bool {
        self.components
            .iter()
            .any(|component| component.handles_write(address))
    }

    fn read_byte(&self, address: u16) -> Result<u8, NesError> {
        // Find the component that handles this address
        if let Some(component) = self.find_component_for_address(address) {
            // Only the lines the component actually drives come from it; the rest still hold what
            // the bus last carried. See `Addressable::open_bus_mask`.
            let mask = component.open_bus_mask(address);
            let value = (component.read_byte(address)? & !mask) | (self.open_bus.get() & mask);
            self.open_bus.set(value);
            return Ok(value);
        }

        // Nothing drives these lines, so they still hold whatever was last put on them.
        self.open_bus_accesses.set(self.open_bus_accesses.get().saturating_add(1));
        Ok(self.open_bus.get())
    }

    /// Look without touching. See [`Addressable::peek_byte`].
    ///
    /// The open bus is deliberately neither read from nor written to here. `rom_test`'s runner
    /// polls `$6000` after every instruction to see whether a ROM has finished, and doing that
    /// through a real read left the status byte sitting on the lines — which is invisible to almost
    /// every ROM and fatal to `cpu_exec_space`, whose whole subject is what an undriven read
    /// returns. It reported a failure the emulator was not making.
    fn peek_byte(&self, address: u16) -> Result<u8, NesError> {
        match self.find_component_for_address(address) {
            Some(component) => component.peek_byte(address),
            None => Ok(self.open_bus.get()),
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError> {
        // The CPU drives the data bus for the whole of a write, whether or not anything is
        // listening, so the value stays on the lines either way.
        self.open_bus.set(value);

        // Find the component that handles this address
        if let Some(component) = self.find_component_for_address_mut(address) {
            component.write_byte(address, value)?;
            return Ok(());
        }

        // A write to nothing is simply lost, as it is on hardware.
        self.open_bus_accesses.set(self.open_bus_accesses.get().saturating_add(1));
        Ok(())
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use anyhow::Result;

    use super::*;

    // A universal test component that records accesses and can be configured for any address range
    #[derive(Debug)]
    struct TestComponent {
        start_address: u16,
        end_address: u16,
        memory: Vec<u8>,          // Stores actual memory values
        read_count: Cell<usize>,  // For internal tracking
        write_count: Cell<usize>, // For internal tracking
        last_address: Cell<u16>,  // For internal tracking
    }

    impl TestComponent {
        fn new(start_address: u16, end_address: u16) -> Self {
            let size = (end_address - start_address + 1) as usize;
            Self {
                start_address,
                end_address,
                memory: vec![0; size],
                read_count: Cell::new(0),
                write_count: Cell::new(0),
                last_address: Cell::new(0),
            }
        }
    }

    impl Addressable for TestComponent {
        fn handles_address(&self, address: u16) -> bool {
            address >= self.start_address && address <= self.end_address
        }

        fn read_byte(&self, address: u16) -> Result<u8, NesError> {
            self.read_count.set(self.read_count.get() + 1);
            self.last_address.set(address);
            let index = (address - self.start_address) as usize;

            Ok(self.memory[index])
        }

        fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError> {
            self.write_count.set(self.write_count.get() + 1);
            self.last_address.set(address);
            let index = (address - self.start_address) as usize;
            self.memory[index] = value;

            Ok(())
        }
    }

    #[test]
    fn test_bus_read_write() -> Result<()> {
        let mut bus = Bus::new();

        // Test RAM component that's included by default
        bus.write_byte(0x0100, 0x42)?;
        assert_eq!(bus.read_byte(0x0100)?, 0x42);

        // Test custom component
        let ppu_regs = Box::new(TestComponent::new(0x2000, 0x2007));
        bus.attach_component(ppu_regs);

        bus.write_byte(0x2000, 0x55)?;
        assert_eq!(bus.read_byte(0x2000)?, 0x55);

        Ok(())
    }

    #[test]
    fn test_component_priority_routing() -> Result<()> {
        let mut bus = Bus::new();

        // Add two components with overlapping ranges
        let component1 = Box::new(TestComponent::new(0x2000, 0x2007));
        let component2 = Box::new(TestComponent::new(0x2000, 0x2FFF));

        bus.attach_component(component1);
        bus.attach_component(component2);

        // Write to the overlapping address - should go to the first component
        bus.write_byte(0x2000, 0x42)?;
        assert_eq!(bus.read_byte(0x2000)?, 0x42);

        // Address only in second component's range should go there
        bus.write_byte(0x2010, 0x55)?;
        assert_eq!(bus.read_byte(0x2010)?, 0x55);

        Ok(())
    }

    #[test]
    fn test_cross_component_boundaries() -> Result<()> {
        let mut bus = Bus::new();

        // Create component that handles PPU registers
        let ppu = Box::new(TestComponent::new(0x2000, 0x2007));
        bus.attach_component(ppu);

        // Test boundary between RAM and PPU
        bus.write_byte(0x1FFF, 0x42)?; // Last RAM address
        bus.write_byte(0x2000, 0x55)?; // First PPU address

        assert_eq!(bus.read_byte(0x1FFF)?, 0x42);
        assert_eq!(bus.read_byte(0x2000)?, 0x55);

        Ok(())
    }

    #[test]
    fn test_component_access_counts() -> Result<()> {
        let mut bus = Bus::new();

        // Create a test component with a unique memory range
        let ppu = Box::new(TestComponent::new(0x2000, 0x2007));
        bus.attach_component(ppu);

        // Write and read through the bus multiple times
        bus.write_byte(0x2000, 0x42)?;
        bus.write_byte(0x2001, 0x43)?;
        assert_eq!(bus.read_byte(0x2000)?, 0x42);
        assert_eq!(bus.read_byte(0x2001)?, 0x43);

        // Now verify RAM (built-in component) is being accessed correctly too
        bus.write_byte(0x0100, 0x55)?;
        assert_eq!(bus.read_byte(0x0100)?, 0x55);

        // A further test verifying distinct component access
        bus.write_byte(0x0200, 0x66)?; // To RAM
        bus.write_byte(0x2002, 0x77)?; // To PPU
        assert_eq!(bus.read_byte(0x0200)?, 0x66); // From RAM
        assert_eq!(bus.read_byte(0x2002)?, 0x77); // From PPU

        Ok(())
    }

    #[test]
    fn test_reset() -> Result<()> {
        let mut bus = Bus::new();

        // Set some values in RAM
        bus.write_byte(0x0100, 0x42)?;
        assert_eq!(bus.read_byte(0x0100)?, 0x42);

        // Reset the bus
        bus.reset();

        // RAM should be reset
        assert_eq!(bus.read_byte(0x0100)?, 0x00);

        Ok(())
    }

    #[test]
    fn test_multiple_components() -> Result<()> {
        let mut bus = Bus::new();

        // Add components for different memory regions
        bus.attach_component(Box::new(TestComponent::new(0x2000, 0x2007)));
        bus.attach_component(Box::new(TestComponent::new(0x4000, 0x4017)));
        bus.attach_component(Box::new(TestComponent::new(0x8000, 0xFFFF)));

        // Test writes to different regions
        bus.write_byte(0x0100, 0x01)?; // RAM
        bus.write_byte(0x2000, 0x02)?; // PPU
        bus.write_byte(0x4000, 0x03)?; // APU
        bus.write_byte(0x8000, 0x04)?; // Cart

        // Verify reads from different regions
        assert_eq!(bus.read_byte(0x0100)?, 0x01);
        assert_eq!(bus.read_byte(0x2000)?, 0x02);
        assert_eq!(bus.read_byte(0x4000)?, 0x03);
        assert_eq!(bus.read_byte(0x8000)?, 0x04);

        Ok(())
    }

    /// An unmapped address reads back the last value the bus carried, and a write to one is lost.
    ///
    /// This test used to assert the opposite — that both fail with `MemoryAccessError` — which was
    /// a deliberate choice to make a hole in the memory map loud. It had to go, because programs
    /// read unmapped addresses on purpose: every indexed addressing mode performs a dummy read at
    /// an unfixed address, and four of blargg's ROMs stopped the emulator dead doing exactly that.
    /// Refusing an access hardware answers is not strictness, it is a different machine.
    #[test]
    fn unmapped_addresses_read_back_the_last_value_on_the_bus() -> Result<()> {
        let mut bus = Bus::new();

        // Put a known value on the bus by way of a mapped address.
        bus.write_byte(0x0010, 0x5A)?;
        assert_eq!(bus.read_byte(0x0010)?, 0x5A);

        // Nothing handles $6000 on a bare bus, so the lines still hold that value.
        assert_eq!(bus.read_byte(0x6000)?, 0x5A, "an unmapped read sees what was last on the bus");

        // A write drives the bus even where nothing is listening, and is otherwise lost.
        bus.write_byte(0x6000, 0x3C)?;
        assert_eq!(bus.read_byte(0x6000)?, 0x3C, "the write left its value on the lines");

        // And it really was lost rather than stored: a mapped read moves the bus on, and the
        // unmapped address has nothing of its own to give back.
        bus.write_byte(0x0010, 0x11)?;
        assert_eq!(bus.read_byte(0x0010)?, 0x11);
        assert_eq!(bus.read_byte(0x6000)?, 0x11, "nothing was remembered at $6000");

        assert!(bus.open_bus_accesses() > 0, "and the fall-throughs are counted");

        Ok(())
    }

    #[test]
    fn test_debug_memory_map() {
        let mut bus = Bus::new();

        // By default only RAM is mapped
        let map = bus.debug_memory_map();
        assert!(map.contains("Zero Page"));
        assert!(map.contains("Stack"));
        assert!(map.contains("RAM"));
        assert!(map.contains("PPU Registers"));
        assert!(map.contains("APU Registers"));
        assert!(map.contains("Program Memory (Low)"));
        assert!(map.contains("Program Memory (High)"));
        assert!(map.contains("NMI Vector"));
        assert!(map.contains("Reset Vector"));
        assert!(map.contains("IRQ Vector"));

        // Add PPU registers
        let ppu_regs = Box::new(TestComponent::new(0x2000, 0x2007));
        bus.attach_component(ppu_regs);

        // Add Program ROM
        let program_rom = Box::new(TestComponent::new(0x8000, 0xFFFF));
        bus.attach_component(program_rom);

        // Now verify the mappings
        let map = bus.debug_memory_map();
        assert!(map.contains("PPU Registers"));
        assert!(map.contains("Reset Vector"));
    }
}
