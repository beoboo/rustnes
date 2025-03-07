# 6502 CPU Basics

The heart of any classic computing system is its Central Processing Unit (CPU). The Nintendo Entertainment System (NES) uses a modified version of the MOS Technology 6502 processor, specifically the Ricoh 2A03 variant (2A07 for PAL regions). Understanding this CPU is fundamental to building our emulator, as it dictates how games execute their instructions.

## The 6502 Architecture

The 6502 CPU was designed in the mid-1970s as a cost-effective alternative to more expensive processors. Its simplicity and affordability led to its widespread adoption in early personal computers and gaming systems like the NES, Atari 2600, and Commodore 64.

For our NES emulator, we'll focus on the CPU's key components:

1. **Registers** - Small storage locations within the CPU
2. **Status flags** - Bits that indicate CPU state after operations
3. **Stack** - A region of memory for temporary data storage
4. **Memory interface** - How the CPU interacts with the rest of the system

## CPU Registers

The 6502 has a simple register structure consisting of:

- **Accumulator (A)**: An 8-bit register used for arithmetic and logical operations
- **Index Registers (X, Y)**: Two 8-bit registers used for indexing and counting
- **Stack Pointer (SP)**: An 8-bit register pointing to the current position in the stack
- **Program Counter (PC)**: A 16-bit register containing the address of the next instruction
- **Status Register (P)**: An 8-bit register containing processor status flags

Here's how we represent these in our Rust implementation:

```rust
pub struct Cpu {
    // Registers
    pub a: u8,      // Accumulator
    pub x: u8,      // X index register
    pub y: u8,      // Y index register
    pub sp: u8,     // Stack pointer (0x00-0xFF, maps to 0x0100-0x01FF in memory)
    pub pc: u16,    // Program counter
    pub status: u8, // Status register (flags)
    
    // CPU cycle count
    pub cycles: u64,
    
    // Memory connection
    memory: Box<dyn Memory>,
}
```

## Status Flags

The status register contains 8 individual flags that indicate the CPU's state. Each flag represents a specific condition that results from CPU operations:

```rust
pub enum CpuFlag {
    Carry      = 0b00000001,
    Zero       = 0b00000010,
    InterruptDisable = 0b00000100,
    DecimalMode = 0b00001000, // Not used in NES, but part of the 6502 spec
    Break      = 0b00010000, // Not a real flag, used during CPU stack operations
    Unused     = 0b00100000, // Bit 5 is unused, always set to 1
    Overflow   = 0b01000000,
    Negative   = 0b10000000,
}
```

These flags are important as they control program flow and indicate the results of operations:

- **Carry (C)**: Set when an addition results in a carry, or a subtraction doesn't result in a borrow
- **Zero (Z)**: Set when an operation results in zero
- **Interrupt Disable (I)**: When set, the CPU ignores hardware interrupts
- **Decimal Mode (D)**: Controls decimal arithmetic (not used in the NES)
- **Break (B)**: Not a real flag, but used internally during certain stack operations
- **Unused**: Always set to 1
- **Overflow (V)**: Set when an arithmetic operation produces an invalid two's complement result
- **Negative (N)**: Set when the result of an operation has bit 7 set (i.e., is negative in two's complement)

We implement methods to manipulate these flags:

```rust
impl Cpu {
    /// Get the value of a specific CPU flag
    pub fn get_flag(&self, flag: CpuFlag) -> bool {
        (self.status & flag as u8) != 0
    }
    
    /// Set a specific CPU flag to the given value
    pub fn set_flag(&mut self, flag: CpuFlag, value: bool) {
        if value {
            self.status |= flag as u8;
        } else {
            self.status &= !(flag as u8);
        }
    }
}
```

## Memory Interaction

The 6502 uses a 16-bit address bus, allowing it to access up to 64KB of memory. In our emulator, we define a `Memory` trait to handle all memory operations:

```rust
pub trait Memory {
    /// Read a byte from memory at the specified address
    fn read_byte(&self, address: u16) -> u8;
    
    /// Write a byte to memory at the specified address
    fn write_byte(&mut self, address: u16, value: u8);
    
    /// Read a word (16-bits) from memory at the specified address
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
```

Notice that the 6502 is little-endian, meaning the least significant byte is stored at the lower memory address. This is reflected in our implementation of `read_word` and `write_word`.

To test our memory implementation, we create a simple RAM class:

```rust
pub struct Ram {
    data: [u8; 0x10000], // 64KB of memory
}

impl Memory for Ram {
    fn read_byte(&self, address: u16) -> u8 {
        self.data[address as usize]
    }
    
    fn write_byte(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }
}
```

## The Stack

The 6502 has a 256-byte stack located in memory at addresses `0x0100` through `0x01FF`. The stack pointer (SP) is an 8-bit register that holds the low byte of the stack address, with the high byte fixed at `0x01`.

The stack grows downward in memory, meaning that when a value is pushed onto the stack, the stack pointer decreases. Here's our implementation of stack operations:

```rust
impl Cpu {
    /// Push a byte onto the stack
    pub fn push_byte(&mut self, value: u8) {
        let stack_addr = 0x0100 | (self.sp as u16);
        self.write_byte(stack_addr, value);
        self.sp = self.sp.wrapping_sub(1);
    }
    
    /// Pop a byte from the stack
    pub fn pop_byte(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let stack_addr = 0x0100 | (self.sp as u16);
        self.read_byte(stack_addr)
    }
    
    /// Push a word onto the stack (high byte first, then low byte)
    pub fn push_word(&mut self, value: u16) {
        let high = (value >> 8) as u8;
        let low = (value & 0xFF) as u8;
        self.push_byte(high);
        self.push_byte(low);
    }
    
    /// Pop a word from the stack (low byte first, then high byte)
    pub fn pop_word(&mut self) -> u16 {
        let low = self.pop_byte() as u16;
        let high = self.pop_byte() as u16;
        (high << 8) | low
    }
}
```

## CPU Initialization and Reset

When the NES is powered on, the CPU starts with specific values in its registers. We implement this in our `new` function:

```rust
impl Cpu {
    /// Create a new CPU instance initialized to power-up state with the provided memory
    pub fn new(memory: Box<dyn Memory>) -> Self {
        // Initial state according to NES specs
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD, // Initial stack pointer
            pc: 0,    // Will be set to the reset vector
            status: 0x34, // 0b00110100 - Unused bit and Interrupt disable set
            cycles: 0,
            memory,
        }
    }
}
```

When a reset occurs (either at power-up or when the reset button is pressed), the CPU performs specific operations:

```rust
impl Cpu {
    /// Reset the CPU
    pub fn reset(&mut self) {
        // Set registers to their initial values
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.status = 0x34;
        
        // Read the reset vector from 0xFFFC-0xFFFD
        self.pc = self.read_word(0xFFFC);
        
        // Reset takes 7 cycles
        self.cycles = 7;
    }
}
```

The reset vector is stored at addresses `0xFFFC` and `0xFFFD`. The CPU reads these addresses to determine where to start executing code.

## Testing Our Implementation

One of the most important aspects of building an emulator is ensuring accuracy through testing. We've written tests to verify our CPU flag operations, memory interactions, and stack operations:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Ram;
    
    #[test]
    fn test_cpu_flags() {
        let ram = Ram::new();
        let mut cpu = Cpu::new(Box::new(ram));
        
        // Test flag is initially not set
        assert!(!cpu.get_flag(CpuFlag::Zero));
        
        // Test setting a flag
        cpu.set_flag(CpuFlag::Zero, true);
        assert!(cpu.get_flag(CpuFlag::Zero));
        
        // Test clearing a flag
        cpu.set_flag(CpuFlag::Zero, false);
        assert!(!cpu.get_flag(CpuFlag::Zero));
    }
    
    #[test]
    fn test_cpu_memory_interaction() {
        let ram = Ram::new();
        let mut cpu = Cpu::new(Box::new(ram));
        
        // Test writing and reading bytes
        cpu.write_byte(0x1000, 0x42);
        assert_eq!(cpu.read_byte(0x1000), 0x42);
        
        // Test writing and reading words
        cpu.write_word(0x2000, 0x1234);
        assert_eq!(cpu.read_word(0x2000), 0x1234);
    }
    
    #[test]
    fn test_stack_operations() {
        let ram = Ram::new();
        let mut cpu = Cpu::new(Box::new(ram));
        
        // Test push and pop byte
        cpu.push_byte(0x42);
        assert_eq!(cpu.sp, 0xFC);
        assert_eq!(cpu.pop_byte(), 0x42);
        assert_eq!(cpu.sp, 0xFD);
        
        // Test push and pop word
        cpu.push_word(0x1234);
        assert_eq!(cpu.sp, 0xFB);
        assert_eq!(cpu.pop_word(), 0x1234);
        assert_eq!(cpu.sp, 0xFD);
    }
    
    #[test]
    fn test_reset() {
        let mut ram = Ram::new();
        
        // Set reset vector
        ram.write_byte(0xFFFC, 0x34);
        ram.write_byte(0xFFFD, 0x12);
        
        let mut cpu = Cpu::new(Box::new(ram));
        cpu.reset();
        
        // Check if PC was set to the reset vector
        assert_eq!(cpu.pc, 0x1234);
        // Check if SP was set to 0xFD
        assert_eq!(cpu.sp, 0xFD);
        // Check if cycles were set to 7
        assert_eq!(cpu.cycles, 7);
    }
}
```

## Next Steps

With our basic CPU implementation complete, we can now move on to more complex features. In the next chapter, we'll implement addressing modes, which define how the CPU accesses data in memory.

The 6502 has several addressing modes, each with its own way of calculating the effective address for an operation. These addressing modes are crucial for implementing the CPU's instruction set.

By the end of this chapter, we've established the foundation for our NES emulator by implementing the basic components of the 6502 CPU. We now have a CPU with registers, status flags, and memory interaction capabilities, all verified through comprehensive tests.
