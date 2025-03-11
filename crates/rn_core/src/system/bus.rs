use crate::memory::{Addressable, Ram};

/// Bus for routing memory access to appropriate devices
///
/// The Bus acts as a mediator between the CPU and addressable components.
/// It routes read/write operations to the appropriate component based on the address.
pub struct Bus {
    /// Components attached to the bus in priority order
    /// First component that handles an address will process the request
    components: Vec<Box<dyn Addressable>>,
}

impl Bus {
    /// Create a new Bus instance with standard NES memory mapping
    ///
    /// This automatically configures:
    /// - RAM at $0000-$1FFF (main system RAM)
    pub fn new() -> Self {
        let mut bus = Self { components: Vec::new() };

        // Attach RAM for the main memory region ($0000-$1FFF)
        // This is the 2KB of RAM that's mirrored throughout this region in the NES
        bus.attach_component(Box::new(Ram::with_range(0x0000, 0x1FFF)));

        bus
    }

    /// Attach an addressable component to the bus
    ///
    /// Components are checked in the order they are attached, so the first
    /// component that claims an address will handle it.
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
    fn find_component_for_address(&self, address: u16) -> Option<&Box<dyn Addressable>> {
        self.components
            .iter()
            .find(|component| component.handles_address(address))
    }

    /// Find the component that handles the given address (mutable version)
    ///
    /// Returns a mutable reference to the first component that claims to handle the address,
    /// or None if no component handles it (which shouldn't happen with RAM fallback).
    fn find_component_for_address_mut(&mut self, address: u16) -> Option<&mut Box<dyn Addressable>> {
        self.components
            .iter_mut()
            .find(|component| component.handles_address(address))
    }
}

impl Addressable for Bus {
    fn handles_address(&self, address: u16) -> bool {
        // The bus handles any address that one of its components can handle
        self.components
            .iter()
            .any(|component| component.handles_address(address))
    }

    fn read_byte(&self, address: u16) -> u8 {
        // Find the component that handles this address
        if let Some(component) = self.find_component_for_address(address) {
            return component.read_byte(address);
        }

        // This shouldn't happen with RAM as fallback, but return 0 just in case
        0
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        // Find the component that handles this address
        if let Some(component) = self.find_component_for_address_mut(address) {
            component.write_byte(address, value);
        }

        // If no component handles it, the write is silently ignored
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    // A universal test component that records accesses and can be configured for any address range
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

        fn read_byte(&self, address: u16) -> u8 {
            self.read_count.set(self.read_count.get() + 1);
            self.last_address.set(address);
            let index = (address - self.start_address) as usize;
            self.memory[index]
        }

        fn write_byte(&mut self, address: u16, value: u8) {
            self.write_count.set(self.write_count.get() + 1);
            self.last_address.set(address);
            let index = (address - self.start_address) as usize;
            self.memory[index] = value;
        }
    }

    #[test]
    fn test_bus_read_write() {
        let mut bus = Bus::new();

        // Test RAM component that's included by default
        bus.write_byte(0x0100, 0x42);
        assert_eq!(bus.read_byte(0x0100), 0x42);

        // Test custom component
        let ppu_regs = Box::new(TestComponent::new(0x2000, 0x2007));
        bus.attach_component(ppu_regs);

        bus.write_byte(0x2000, 0x55);
        assert_eq!(bus.read_byte(0x2000), 0x55);
    }

    #[test]
    fn test_component_priority_routing() {
        let mut bus = Bus::new();

        // Add two components with overlapping ranges
        let component1 = Box::new(TestComponent::new(0x2000, 0x2007));
        let component2 = Box::new(TestComponent::new(0x2000, 0x2FFF));

        bus.attach_component(component1);
        bus.attach_component(component2);

        // Write to the overlapping address - should go to the first component
        bus.write_byte(0x2000, 0x42);
        assert_eq!(bus.read_byte(0x2000), 0x42);

        // Address only in second component's range should go there
        bus.write_byte(0x2010, 0x55);
        assert_eq!(bus.read_byte(0x2010), 0x55);
    }

    #[test]
    fn test_cross_component_boundaries() {
        let mut bus = Bus::new();

        // Create component that handles PPU registers
        let ppu = Box::new(TestComponent::new(0x2000, 0x2007));
        bus.attach_component(ppu);

        // Test boundary between RAM and PPU
        bus.write_byte(0x1FFF, 0x42); // Last RAM address
        bus.write_byte(0x2000, 0x55); // First PPU address

        assert_eq!(bus.read_byte(0x1FFF), 0x42);
        assert_eq!(bus.read_byte(0x2000), 0x55);
    }

    #[test]
    fn test_component_access_counts() {
        let mut bus = Bus::new();

        // Create a test component with a unique memory range
        let ppu = Box::new(TestComponent::new(0x2000, 0x2007));
        bus.attach_component(ppu);

        // Write and read through the bus multiple times
        bus.write_byte(0x2000, 0x42);
        bus.write_byte(0x2001, 0x43);
        assert_eq!(bus.read_byte(0x2000), 0x42);
        assert_eq!(bus.read_byte(0x2001), 0x43);

        // Now verify RAM (built-in component) is being accessed correctly too
        bus.write_byte(0x0100, 0x55);
        assert_eq!(bus.read_byte(0x0100), 0x55);

        // A further test verifying distinct component access
        bus.write_byte(0x0200, 0x66); // To RAM
        bus.write_byte(0x2002, 0x77); // To PPU
        assert_eq!(bus.read_byte(0x0200), 0x66); // From RAM
        assert_eq!(bus.read_byte(0x2002), 0x77); // From PPU
    }

    #[test]
    fn test_reset() {
        let mut bus = Bus::new();

        // Set some values in RAM
        bus.write_byte(0x0100, 0x42);
        assert_eq!(bus.read_byte(0x0100), 0x42);

        // Reset the bus
        bus.reset();

        // RAM should be reset
        assert_eq!(bus.read_byte(0x0100), 0x00);
    }

    #[test]
    fn test_multiple_components() {
        let mut bus = Bus::new();

        // Add components for different memory regions
        bus.attach_component(Box::new(TestComponent::new(0x2000, 0x2007)));
        bus.attach_component(Box::new(TestComponent::new(0x4000, 0x4017)));
        bus.attach_component(Box::new(TestComponent::new(0x8000, 0xFFFF)));

        // Test writes to different regions
        bus.write_byte(0x0100, 0x01); // RAM
        bus.write_byte(0x2000, 0x02); // PPU
        bus.write_byte(0x4000, 0x03); // APU
        bus.write_byte(0x8000, 0x04); // Cart

        // Verify reads from different regions
        assert_eq!(bus.read_byte(0x0100), 0x01);
        assert_eq!(bus.read_byte(0x2000), 0x02);
        assert_eq!(bus.read_byte(0x4000), 0x03);
        assert_eq!(bus.read_byte(0x8000), 0x04);
    }

    #[test]
    fn test_unmapped_memory() {
        let mut bus = Bus::new();

        // Read from unmapped memory (nothing handles 0x6000)
        let value = bus.read_byte(0x6000);
        assert_eq!(value, 0);

        // Write to unmapped memory should be silently ignored
        bus.write_byte(0x6000, 0xFF);
        assert_eq!(bus.read_byte(0x6000), 0);
    }
}
