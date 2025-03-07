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
    pub fn get_operand_address(&self, cpu: &Cpu) -> u16 {
        match self {
            AddressingMode::Immediate => cpu.pc + 1,
        }
    }
}
```

This implementation is quite simple - for immediate addressing, the "address" of the operand is just the byte following the current instruction (PC + 1). This isn't actually a memory address we're using for a lookup, but rather the location of the immediate value in the instruction stream.

When the CPU executes an instruction with immediate addressing, it calls this method to determine where to find the operand value:

```rust
pub fn read_byte_using_mode(&self, mode: AddressingMode) -> u8 {
    let addr = mode.get_operand_address(self);
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
    assert_eq!(value, 0x42, "Immediate addressing mode should read the value after PC");
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

## Zero Page,X Addressing Mode

### Concept

Zero Page,X addressing builds on the Zero Page mode by adding the content of the X register to the zero page address. This allows for accessing a range of memory locations with a single instruction, which is particularly useful for array-like structures or tables in the zero page.

In 6502 assembly language, Zero Page,X addressing is written like this:

```asm
LDA $40,X    ; Load value from (zero page address $40 + X) into the accumulator
STA $20,X    ; Store accumulator value at (zero page address $20 + X)
INC $30,X    ; Increment value at (zero page address $30 + X)
```

An important characteristic of Zero Page,X addressing is that it always stays within the zero page. If adding X to the address would exceed $FF, it wraps around to stay within the zero page.

### Memory Layout

Let's visualize how Zero Page,X addressing works in memory:

```
Memory Address | Content      | Description
---------------|--------------|--------------------------
$0200          | $B5          | LDA Zero Page,X opcode
$0201          | $40          | Zero page base address $40
$0202          | (next opcode) | Next instruction...
... ... ...    |              |
(CPU X register) | $05        | X register contains $05
... ... ...    |              |
$0045          | $37          | Value at effective address ($40 + $05) = $45
```

When the CPU executes this instruction:
1. It reads the opcode $B5 at the program counter
2. It identifies this as LDA with Zero Page,X addressing
3. It reads the next byte ($40) as the zero page base address
4. It adds the X register ($05) to get the effective address $45
5. It reads the value at memory address $0045, which contains $37
6. It loads the value $37 into the accumulator register
7. It updates the program counter by 2 bytes

### Wrap-Around Behavior

If adding X to the zero page address exceeds $FF, the address wraps around to stay in the zero page:

```
Zero page address: $FE
X register: $05
Effective address: ($FE + $05) & $FF = $03
```

### Advantages and Limitations

**Advantages:**
- Memory efficient: Instructions are still only 2 bytes
- Flexible: Can access a range of addresses with a single instruction
- Useful for arrays: Perfect for iterating through sequential memory

**Limitations:**
- Still limited to the zero page
- Potential wrap-around can be confusing if not handled carefully

### Implementation in Our Emulator

Our implementation of Zero Page,X addressing:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    // More to come later
}

impl AddressingMode {
    pub fn get_operand_address(&self, cpu: &Cpu) -> u16 {
        match self {
            AddressingMode::Immediate => cpu.pc + 1,
            AddressingMode::ZeroPage => {
                let zero_page_addr = cpu.read_byte(cpu.pc + 1) as u16;
                zero_page_addr
            },
            AddressingMode::ZeroPageX => {
                // Get the zero page address from the byte after the opcode
                let zero_page_addr = cpu.read_byte(cpu.pc + 1);
                
                // Add the X register to it (with wrap-around in the zero page)
                let effective_addr = (zero_page_addr.wrapping_add(cpu.x)) as u16;
                
                // The high byte is always 0 since we stay in the zero page
                effective_addr
            }
        }
    }
}
```

The critical part here is using `wrapping_add` to ensure we get the correct wrap-around behavior when the sum exceeds 255.

### Testing Zero Page,X Addressing

We test both normal addressing and wrap-around behavior:

```rust
#[test]
fn test_zero_page_x_addressing_mode() {
    // Create a CPU with mock memory
    let memory = MockMemory::new();
    let mut cpu = Cpu::new(Box::new(memory));
    
    // Setup memory:
    // At $0200: Opcode using Zero Page,X addressing
    // At $0201: Zero page address $40
    // CPU X register: $05
    // Effective address: $45
    // At $0045: The value $37 we want to read
    cpu.write_byte(0x0200, 0xB5); // LDA Zero Page,X opcode
    cpu.write_byte(0x0201, 0x40); // Zero page address $40
    cpu.write_byte(0x0045, 0x37); // Value at effective address $45
    
    // Set CPU state
    cpu.pc = 0x0200;
    cpu.x = 0x05;  // Set X register
    
    // Test zero page,X addressing mode
    let addr = AddressingMode::ZeroPageX.get_operand_address(&cpu);
    assert_eq!(addr, 0x0045, "Zero page,X address should be $0045");
    
    let value = cpu.read_byte_using_mode(AddressingMode::ZeroPageX);
    assert_eq!(value, 0x37, "Value at zero page,X address $45 should be $37");
    
    // Test wrap-around behavior
    cpu.write_byte(0x0201, 0xFE); // Zero page address $FE
    cpu.x = 0x05;  // X register is $05
    // Effective address should be $03 (0xFE + 0x05 = 0x103, which wraps to 0x03)
    cpu.write_byte(0x0003, 0x42); // Value at wrapped address $03
    
    let addr = AddressingMode::ZeroPageX.get_operand_address(&cpu);
    assert_eq!(addr, 0x0003, "Zero page,X address should wrap to $0003");
    
    let value = cpu.read_byte_using_mode(AddressingMode::ZeroPageX);
    assert_eq!(value, 0x42, "Value at wrapped address $03 should be $42");
}
```

This ensures our implementation correctly handles both normal cases and the special wrap-around behavior of the 6502 CPU.

## Zero Page,Y Addressing Mode

### Concept

Zero Page,Y addressing is similar to Zero Page,X, but it uses the Y register for indexing instead of the X register. This mode allows accessing data in the zero page with an offset stored in the Y register, which is useful for different types of data structures that are indexed by Y.

In 6502 assembly language, Zero Page,Y addressing is written like this:

```asm
LDX $40,Y    ; Load value from (zero page address $40 + Y) into the X register
STX $20,Y    ; Store X register value at (zero page address $20 + Y)
INC $30,Y    ; Increment value at (zero page address $30 + Y)
```

Note that fewer instructions support Zero Page,Y compared to Zero Page,X. For example, the LDA instruction doesn't have a Zero Page,Y mode, but LDX does.

### Memory Layout

Let's visualize how Zero Page,Y addressing works in memory:

```
Memory Address | Content      | Description
---------------|--------------|--------------------------
$0200          | $B6          | LDX Zero Page,Y opcode
$0201          | $40          | Zero page base address $40
$0202          | (next opcode) | Next instruction...
... ... ...    |              |
(CPU Y register) | $07        | Y register contains $07
... ... ...    |              |
$0047          | $37          | Value at effective address ($40 + $07) = $47
```

When the CPU executes this instruction:
1. It reads the opcode $B6 at the program counter
2. It identifies this as LDX with Zero Page,Y addressing
3. It reads the next byte ($40) as the zero page base address
4. It adds the Y register ($07) to get the effective address $47
5. It reads the value at memory address $0047, which contains $37
6. It loads the value $37 into the X register
7. It updates the program counter by 2 bytes

### Wrap-Around Behavior

Just like Zero Page,X, the Zero Page,Y addressing mode also exhibits wrap-around behavior when the sum of the base address and Y register exceeds $FF:

```
Zero page address: $FB
Y register: $07
Effective address: ($FB + $07) & $FF = $02
```

### Advantages and Limitations

**Advantages:**
- Memory efficient: Instructions are only 2 bytes
- Provides Y-indexed access to the zero page
- Complements Zero Page,X for different access patterns

**Limitations:**
- Available in fewer instructions than Zero Page,X
- Still limited to the zero page
- Same wrap-around considerations as Zero Page,X

### Implementation in Our Emulator

Our implementation of Zero Page,Y addressing:

```rust
impl AddressingMode {
    pub fn get_operand_address(&self, cpu: &Cpu) -> u16 {
        match self {
            // Previous addressing modes...
            AddressingMode::ZeroPageY => {
                // Get the zero page address from the byte after the opcode
                let zero_page_addr = cpu.read_byte(cpu.pc + 1);
                
                // Add the Y register to it (with wrap-around in the zero page)
                let effective_addr = (zero_page_addr.wrapping_add(cpu.y)) as u16;
                
                // The high byte is always 0 since we stay in the zero page
                effective_addr
            }
        }
    }
}
```

Note how similar this is to the Zero Page,X implementation, with the only difference being the use of `cpu.y` instead of `cpu.x`.

### Testing Zero Page,Y Addressing

We test both normal addressing and wrap-around behavior:

```rust
#[test]
fn test_zero_page_y_addressing_mode() {
    // Create a CPU with mock memory
    let memory = MockMemory::new();
    let mut cpu = Cpu::new(Box::new(memory));
    
    // Setup memory:
    // At $0200: Opcode using Zero Page,Y addressing
    // At $0201: Zero page address $40
    // CPU Y register: $07
    // Effective address: $47
    // At $0047: The value $37 we want to read
    cpu.write_byte(0x0200, 0xB6); // LDX Zero Page,Y opcode
    cpu.write_byte(0x0201, 0x40); // Zero page address $40
    cpu.write_byte(0x0047, 0x37); // Value at effective address $47
    
    // Set CPU state
    cpu.pc = 0x0200;
    cpu.y = 0x07;  // Set Y register
    
    // Test zero page,Y addressing mode
    let addr = AddressingMode::ZeroPageY.get_operand_address(&cpu);
    assert_eq!(addr, 0x0047, "Zero page,Y address should be $0047");
    
    let value = cpu.read_byte_using_mode(AddressingMode::ZeroPageY);
    assert_eq!(value, 0x37, "Value at zero page,Y address $47 should be $37");
    
    // Test wrap-around behavior
    cpu.write_byte(0x0201, 0xFB); // Zero page address $FB
    cpu.y = 0x07;  // Y register is $07
    // Effective address should be $02 (0xFB + 0x07 = 0x102, which wraps to 0x02)
    cpu.write_byte(0x0002, 0x42); // Value at wrapped address $02
    
    let addr = AddressingMode::ZeroPageY.get_operand_address(&cpu);
    assert_eq!(addr, 0x0002, "Zero page,Y address should wrap to $0002");
    
    let value = cpu.read_byte_using_mode(AddressingMode::ZeroPageY);
    assert_eq!(value, 0x42, "Value at wrapped address $02 should be $42");
}
```

This test ensures our implementation correctly handles both normal cases and the special wrap-around behavior for the Zero Page,Y addressing mode.

## Next Steps

With immediate, zero page, zero page,X, and zero page,Y addressing modes understood and implemented, we'll next explore the absolute addressing modes, which use a full 16-bit address to access any memory location in the system.

## Absolute Addressing Mode

### Concept

Absolute addressing is a powerful mode that allows the CPU to access any memory location within the entire 64KB address space of the 6502. Unlike the zero page modes that can only access the first 256 bytes with a single byte address, absolute addressing uses a full 16-bit address (two bytes) to specify the target memory location.

In 6502 assembly language, absolute addressing is written with a full address:

```asm
LDA $1234    ; Load the value from address $1234 into the accumulator
STA $5678    ; Store the accumulator value at address $5678
JMP $ABCD    ; Jump to address $ABCD
```

### Memory Layout

Let's visualize how absolute addressing works in memory:

```
Memory Address | Content      | Description
---------------|--------------|--------------------------
$0200          | $AD          | LDA Absolute opcode
$0201          | $34          | Low byte of address ($34)
$0202          | $12          | High byte of address ($12)
$0203          | (next opcode) | Next instruction...
... ... ...    |              |
$1234          | $42          | Value stored at absolute address $1234
```

When the CPU executes this instruction:
1. It reads the opcode $AD at the program counter (PC) position
2. It identifies this as LDA with absolute addressing
3. It reads the next two bytes ($34, $12) as the low and high bytes of the address
4. It forms the full address $1234 (in little-endian order)
5. It reads the value at memory address $1234, which contains $42
6. It loads the value $42 into the accumulator register
7. It updates the program counter by 3 bytes to point to the next instruction

### Little-Endian Byte Order

The 6502 uses little-endian byte order, which means that the least significant byte (the low byte) comes first in memory, followed by the most significant byte (the high byte). For example, the 16-bit address $1234 is stored in memory as $34 $12.

### Advantages and Limitations

**Advantages:**
- Full access: Can reference any memory location in the entire 64KB address space
- Versatility: Used by almost all instructions and essential for accessing memory outside the zero page
- Direct: Provides a straightforward way to access a specific known address

**Limitations:**
- Larger instruction size: Takes 3 bytes (vs 2 bytes for zero page addressing)
- Slower execution: Typically requires 1 more cycle than zero page addressing
- Fixed target: Points to a specific address rather than a calculated one (without indexing)

### Implementation in Our Emulator

Here's how we've implemented absolute addressing:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    // We'll add more modes later
}

impl AddressingMode {
    pub fn get_operand_address(&self, cpu: &Cpu) -> u16 {
        match self {
            // Previous addressing modes...
            AddressingMode::Absolute => {
                // Read the low byte and high byte from the instruction
                let low_byte = cpu.read_byte(cpu.pc + 1) as u16;
                let high_byte = cpu.read_byte(cpu.pc + 2) as u16;
                
                // Combine them into a 16-bit address (little-endian)
                (high_byte << 8) | low_byte
            }
        }
    }
}
```

For absolute addressing, we:
1. Read the low byte from the byte after the opcode (at PC+1)
2. Read the high byte from the byte after that (at PC+2)
3. Combine them into a 16-bit address with the high byte shifted left by 8 bits
4. This gives us a full 16-bit address anywhere in the 6502's address space

### Example Usage

Let's look at a concrete example of how absolute addressing executes:

```
Instruction: LDA $1234 (Load the value at address $1234 into the accumulator)
```

Execution steps:
1. CPU fetches opcode $AD at PC ($0200)
2. CPU identifies this as LDA with absolute addressing
3. CPU reads the low byte at PC+1 ($0201), which contains $34
4. CPU reads the high byte at PC+2 ($0202), which contains $12
5. CPU forms the address $1234 and reads memory at that address, getting the value $42
6. CPU loads the value $42 into the accumulator register
7. CPU advances PC by 3 bytes to $0203

### Testing Absolute Addressing

Here's how we test our absolute addressing mode implementation:

```rust
#[test]
fn test_absolute_addressing_mode() {
    // Create a CPU with mock memory
    let memory = MockMemory::new();
    let mut cpu = Cpu::new(Box::new(memory));
    
    // Setup memory:
    // At $0200: Opcode using Absolute addressing
    // At $0201-$0202: Absolute address $1234 (low byte first)
    // At $1234: The value $42 we want to read
    cpu.write_byte(0x0200, 0xAD); // LDA Absolute opcode
    cpu.write_byte(0x0201, 0x34); // Low byte of address
    cpu.write_byte(0x0202, 0x12); // High byte of address
    cpu.write_byte(0x1234, 0x42); // Value at absolute address $1234
    
    // Set CPU state
    cpu.pc = 0x0200;
    
    // Test absolute addressing mode
    let addr = AddressingMode::Absolute.get_operand_address(&cpu);
    assert_eq!(addr, 0x1234, "Absolute address should be $1234");
    
    let value = cpu.read_byte_using_mode(AddressingMode::Absolute);
    assert_eq!(value, 0x42, "Value at absolute address $1234 should be $42");
}
```

This test verifies that:
1. The absolute addressing mode correctly calculates the 16-bit address from the two bytes following the opcode
2. When the CPU reads a byte using this addressing mode, it gets the expected value from that address

## Next Steps

With immediate, zero page, zero page,X, zero page,Y, and absolute addressing modes understood and implemented, we'll next explore the indexed absolute addressing modes (Absolute,X and Absolute,Y), which combine the power of absolute addressing with the flexibility of indexing.
