# Addressing Modes

## Understanding Memory Addressing

At the heart of any CPU's operation is the need to access data. But before a CPU can perform operations on data, it needs to know where that data is located. This is where addressing modes come into play.

Addressing modes are the various ways a CPU can calculate the effective address of the operand (data) it needs to work with. Think of them as different strategies for answering the question: "Where is the data I need to operate on?"

In the 6502 processor used by the NES, addressing modes are particularly important because they determine:

1. How many bytes an instruction uses
2. How many clock cycles an instruction takes to execute
3. What memory locations the instruction can access

## The 6502 Addressing Modes Overview

The 6502 CPU features several addressing modes, each with different capabilities and performance characteristics:

- **Immediate**: The operand is included in the instruction
- **Zero Page**: Uses only a single byte to address the first 256 bytes of memory
- **Zero Page,X and Zero Page,Y**: Indexed access to the zero page
- **Absolute**: Uses a full 16-bit address to access any memory location
- **Absolute,X and Absolute,Y**: Indexed access to any memory location
- **Indirect**: Reads the target address from a memory location
- **Indexed Indirect**: Combines indirect addressing with X register indexing
- **Indirect Indexed**: Combines indirect addressing with Y register indexing

We'll explore each of these modes in detail, starting with the simplest: Immediate addressing.

## Immediate Addressing Mode

### Concept

Immediate addressing is the simplest addressing mode conceptually. Instead of looking up data in memory, the actual value to be used is included right after the instruction opcode in the program.

In 6502 assembly language, immediate addressing is denoted by a `#` symbol before the value:

```asm
LDA #$42    ; Load the value $42 directly into the accumulator
CMP #$FF    ; Compare the accumulator with the value $FF
ADC #$01    ; Add the value $01 to the accumulator
```

This is called "immediate" because the data is immediately available in the instruction stream itself - no additional memory access is required.

### Memory Layout

Let's visualize how immediate addressing works in memory:

```
Memory Address | Content      | Description
---------------|--------------|--------------------------
$0200          | $A9          | LDA immediate opcode
$0201          | $42          | The immediate value $42
$0202          | (next opcode) | Next instruction...
```

When the CPU executes this instruction:
1. It reads the opcode $A9 at the program counter (PC) position
2. It identifies this as LDA with immediate addressing
3. It reads the next byte ($42) as the operand value
4. It updates the program counter by 2 bytes to point to the next instruction

### Advantages and Limitations

**Advantages:**
- Fast: No additional memory access required beyond instruction fetching
- Predictable: Always takes a fixed number of cycles
- Clear intent: Makes it obvious in code that you're using a specific value

**Limitations:**
- Can only use constant values known at assembly time
- Cannot modify the value (read-only)
- Limited to 8-bit values in most instructions

### Implementation in Our Emulator

In our Rust NES emulator, we've implemented immediate addressing as follows:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddressingMode {
    Immediate,
    // We'll add more modes later
}

impl AddressingMode {
    pub fn get_operand_address(&self, program_counter: u16) -> u16 {
        match self {
            AddressingMode::Immediate => program_counter + 1,
        }
    }
}
```

This implementation is quite simple - for immediate addressing, the "address" of the operand is just the byte following the current instruction (PC + 1). This isn't actually a memory address we're using for a lookup, but rather the location of the immediate value in the instruction stream.

When the CPU executes an instruction with immediate addressing, it calls this method to determine where to find the operand value:

```rust
pub fn read_byte_using_mode(&self, mode: AddressingMode) -> u8 {
    let addr = mode.get_operand_address(self.pc);
    self.read_byte(addr)
}
```

### Example Usage

Let's look at a concrete example of how immediate addressing executes:

```
Instruction: LDA #$42 (Load the value $42 into the accumulator)
```

Execution steps:
1. CPU fetches opcode $A9 at PC ($0200)
2. CPU identifies this as LDA with immediate addressing
3. CPU reads the operand at PC+1 ($0201), which contains $42
4. CPU loads the value $42 into the accumulator register
5. CPU advances PC by 2 bytes to $0202

### Testing Immediate Addressing

To ensure our immediate addressing mode works correctly, we test it by setting up a mock CPU and memory scenario:

```rust
#[test]
fn test_immediate_addressing_mode() {
    // Create CPU with mock memory
    let mut cpu = Cpu::new(Box::new(MockMemory::new()));
    
    // Set up memory with LDA #$42
    cpu.write_byte(0x0200, 0xA9); // LDA immediate opcode
    cpu.write_byte(0x0201, 0x42); // The immediate value
    
    // Configure CPU
    cpu.pc = 0x0200;
    
    // Test addressing mode
    let value = cpu.read_byte_using_mode(AddressingMode::Immediate);
    assert_eq!(value, 0x42);
}
```

This test verifies that when the CPU reads a byte using immediate addressing, it correctly retrieves the value that immediately follows the current instruction.

## Zero Page Addressing Mode

### Concept

Zero Page addressing is a memory-efficient addressing mode that allows the CPU to access the first 256 bytes of memory (addresses $0000-$00FF) using only a single byte for the address. This region is called the "zero page" because the high byte of the address is always zero.

In 6502 assembly language, zero page addressing is written simply with the address:

```asm
LDA $42    ; Load the value from zero page address $42 into the accumulator
STA $30    ; Store the accumulator value at zero page address $30
INC $55    ; Increment the value at zero page address $55
```

### Memory Layout

Let's visualize how zero page addressing works in memory:

```
Memory Address | Content      | Description
---------------|--------------|--------------------------
$0200          | $A5          | LDA zero page opcode
$0201          | $42          | Zero page address $42
$0202          | (next opcode) | Next instruction...
... ... ...    |              |
$0042          | $37          | Value stored at zero page address $42
```

When the CPU executes this instruction:
1. It reads the opcode $A5 at the program counter (PC) position
2. It identifies this as LDA with zero page addressing
3. It reads the next byte ($42) as the zero page address
4. It reads the value at memory address $0042, which contains $37
5. It loads the value $37 into the accumulator register
6. It updates the program counter by 2 bytes to point to the next instruction

### Advantages and Limitations

**Advantages:**
- Memory efficient: Instructions are only 2 bytes (vs 3 for absolute addressing)
- Faster execution: Typically requires 1 fewer cycle than absolute addressing
- Important area: The zero page was heavily used for variables and pointers in 6502 programming

**Limitations:**
- Limited range: Can only access the first 256 bytes of memory
- Prime real estate: The zero page is limited and in high demand

### Implementation in Our Emulator

In our Rust NES emulator, we've implemented zero page addressing by extending our enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    // We'll add more modes later
}

impl AddressingMode {
    pub fn get_operand_address(&self, cpu: &Cpu) -> u16 {
        match self {
            AddressingMode::Immediate => cpu.pc + 1,
            AddressingMode::ZeroPage => {
                // Read the zero page address from the byte after the opcode
                let zero_page_addr = cpu.read_byte(cpu.pc + 1) as u16;
                zero_page_addr
            }
        }
    }
}
```

For zero page addressing, we:
1. Read the byte after the opcode (at PC+1)
2. Use that byte as a memory address in the range $0000-$00FF

### Example Usage

Let's look at a concrete example of how zero page addressing executes:

```
Instruction: LDA $42 (Load the value at zero page address $42 into the accumulator)
```

Execution steps:
1. CPU fetches opcode $A5 at PC ($0200)
2. CPU identifies this as LDA with zero page addressing
3. CPU reads the zero page address at PC+1 ($0201), which contains $42
4. CPU reads memory at address $0042, getting the value $37
5. CPU loads the value $37 into the accumulator register
6. CPU advances PC by 2 bytes to $0202

### Testing Zero Page Addressing

To ensure our zero page addressing mode works correctly, we test it by setting up a CPU with memory that contains both the instruction and the target data:

```rust
#[test]
fn test_zero_page_addressing_mode() {
    // Create a CPU with mock memory
    let memory = MockMemory::new();
    let mut cpu = Cpu::new(Box::new(memory));
    
    // Setup memory:
    // At $0200: Opcode that uses zero page addressing
    // At $0201: Zero page address $42
    // At $0042: The value $37 we want to read
    cpu.write_byte(0x0200, 0xA5); // LDA Zero Page opcode
    cpu.write_byte(0x0201, 0x42); // Zero page address $42
    cpu.write_byte(0x0042, 0x37); // Value at zero page address $42
    
    // Set CPU state
    cpu.pc = 0x0200;
    
    // Test zero page addressing mode
    let addr = AddressingMode::ZeroPage.get_operand_address(&cpu);
    assert_eq!(addr, 0x0042, "Zero page address should be $0042");
    
    let value = cpu.read_byte_using_mode(AddressingMode::ZeroPage);
    assert_eq!(value, 0x37, "Value at zero page address $42 should be $37");
}
```

This test verifies that:
1. The zero page addressing mode correctly calculates the memory address ($0042)
2. When the CPU reads a byte using this addressing mode, it gets the expected value ($37)

## Next Steps

With immediate and zero page addressing understood and implemented, we'll next explore the indexed zero page addressing modes (Zero Page,X and Zero Page,Y), which add the ability to access a range of zero page memory locations with a single instruction.
