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
    // At $0201-$0202: Absolute address $1234
    // At $1234: The value $42 we want to read
    cpu.write_byte(0x0200, 0xAD); // LDA Absolute opcode
    cpu.write_word(0x0201, 0x1234); // Address to read from
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

## Indexed Absolute Addressing Modes

The Absolute,X and Absolute,Y addressing modes build on the Absolute addressing mode by adding an index register (X or Y) to the 16-bit address. These modes provide flexible access to arrays or tables anywhere in memory.

### Absolute,X Addressing Mode

#### Concept

Absolute,X addressing adds the value in the X register to a 16-bit address. In 6502 assembly language, it's written with a full address followed by ",X":

```asm
LDA $1234,X    ; Load from (address $1234 + X) into the accumulator
STA $5678,X    ; Store accumulator at (address $5678 + X)
INC $ABCD,X    ; Increment value at (address $ABCD + X)
```

#### Memory Layout

Let's visualize how Absolute,X addressing works in memory:

```
Memory Address | Content      | Description
---------------|--------------|--------------------------
$0200          | $BD          | LDA Absolute,X opcode
$0201          | $34          | Low byte of address ($34)
$0202          | $12          | High byte of address ($12)
$0203          | (next opcode) | Next instruction...
... ... ...    |              |
(CPU X register) | $10        | X register contains $10
... ... ...    |              |
$1244          | $42          | Value at effective address ($1234 + $10 = $1244)
```

When the CPU executes this instruction:
1. It reads the opcode $BD at the program counter
2. It identifies this as LDA with Absolute,X addressing
3. It reads the next two bytes ($34, $12) as the base address $1234
4. It adds the X register ($10) to get the effective address $1244
5. It reads the value at memory address $1244, which contains $42
6. It loads the value $42 into the accumulator register
7. It updates the program counter by 3 bytes

#### Page Crossing Behavior

When the addition of the index register crosses a page boundary (changes the high byte of the address), many instructions take an additional CPU cycle. For example:

```
Base address: $12F0
X register: $20
Effective address: $1310 (crosses from page $12 to page $13)
```

#### Implementation and Testing

Our implementation uses a simple approach of reading the base address and adding the X register to it:

```rust
AddressingMode::AbsoluteX => {
    // Read the base address and add X register
    let base_addr = cpu.read_word(cpu.pc + 1);
    base_addr.wrapping_add(cpu.x as u16)
}
```

We use `wrapping_add` to handle address overflow correctly when the addition crosses the 64KB boundary.

Testing example:

```rust
#[test]
fn test_absolute_x_addressing_mode() {
    let memory = MockMemory::new();
    let mut cpu = Cpu::new(Box::new(memory));
    
    // Base address $1234, X=$10, effective address=$1244
    cpu.write_byte(0x0200, 0xBD); // LDA Absolute,X opcode
    cpu.write_word(0x0201, 0x1234); // Base address
    cpu.write_byte(0x1244, 0x42); // Value at effective address $1244
    
    cpu.pc = 0x0200;
    cpu.x = 0x10;
    
    let addr = AddressingMode::AbsoluteX.get_operand_address(&cpu);
    assert_eq!(addr, 0x1244);
    
    let value = cpu.read_byte_using_mode(AddressingMode::AbsoluteX);
    assert_eq!(value, 0x42);
}

#[test]
fn test_absolute_x_addressing_mode_page_crossing() {
    let memory = MockMemory::new();
    let mut cpu = Cpu::new(Box::new(memory));
    
    // Base address $12F0, X=$20, effective address=$1310 (page boundary crossed)
    cpu.write_byte(0x0200, 0xBD); // LDA Absolute,X opcode
    cpu.write_word(0x0201, 0x12F0); // Base address
    cpu.write_byte(0x1310, 0x42); // Value at effective address $1310
    
    cpu.pc = 0x0200;
    cpu.x = 0x20;
    
    let addr = AddressingMode::AbsoluteX.get_operand_address(&cpu);
    assert_eq!(addr, 0x1310);
    
    let value = cpu.read_byte_using_mode(AddressingMode::AbsoluteX);
    assert_eq!(value, 0x42);
}
```

### Absolute,Y Addressing Mode

Absolute,Y addressing works identically to Absolute,X, except it uses the Y register for indexing instead of the X register. This provides flexibility for different algorithms or when the X register is already in use.

#### Syntax and Usage

```asm
LDA $1234,Y    ; Load from (address $1234 + Y) into the accumulator
STA $5678,Y    ; Store accumulator at (address $5678 + Y)
CMP $ABCD,Y    ; Compare accumulator with value at (address $ABCD + Y)
```

#### Implementation

The implementation is nearly identical to Absolute,X:

```rust
AddressingMode::AbsoluteY => {
    // Read the base address and add Y register
    let base_addr = cpu.read_word(cpu.pc + 1);
    base_addr.wrapping_add(cpu.y as u16)
}
```

Both Absolute,X and Absolute,Y have the same key properties:
- Can access the entire 64KB address space
- Take 3 bytes for the instruction
- May incur an extra cycle when crossing page boundaries
- Will wrap around if the sum exceeds the address space

## Indirect Addressing Mode

### Concept

Indirect addressing is a powerful mode where the instruction contains a 16-bit pointer to the actual address to be used. Think of it as a memory-based pointer dereference: the instruction specifies where to find the address, not the address itself.

In the 6502, indirect addressing is primarily used by the JMP instruction, allowing for dynamic jumps to addresses determined at runtime.

In 6502 assembly language, indirect addressing is written with parentheses around the address:

```asm
JMP ($1234)    ; Jump to the address stored at memory locations $1234-$1235
```

### Memory Layout

Let's visualize how indirect addressing works in memory:

```
Memory Address | Content      | Description
---------------|--------------|--------------------------
$0200          | $6C          | JMP Indirect opcode
$0201          | $34          | Low byte of pointer address ($34)
$0202          | $12          | High byte of pointer address ($12)
$0203          | (next opcode) | Next instruction (not executed due to jump)
... ... ...    |              |
$1234          | $CD          | Low byte of target address
$1235          | $AB          | High byte of target address
... ... ...    |              |
$ABCD          | (opcode)     | Jump target (execution continues here)
```

When the CPU executes this instruction:
1. It reads the opcode $6C at the program counter
2. It identifies this as JMP with indirect addressing
3. It reads the next two bytes ($34, $12) as the pointer address $1234
4. It reads two bytes from that address: the low byte $CD from $1234 and the high byte $AB from $1235
5. It forms the effective address $ABCD
6. It jumps to that address, setting the program counter to $ABCD

### The JMP Indirect Bug

The 6502 CPU has a well-known hardware bug in the implementation of the JMP indirect instruction. If the indirect pointer address falls on a page boundary (e.g., $12FF), the processor incorrectly fetches the high byte from the start of the same page ($1200) rather than the start of the next page ($1300).

For example:
```
Instruction: JMP ($12FF)
Low byte fetched from $12FF (correct)
High byte fetched from $1200 (bug - should be $1300)
```

This bug must be emulated for accurate behavior.

### Advantages and Limitations

**Advantages:**
- Dynamic execution: Enables jumping to addresses determined at runtime
- Facilitates function pointers and jump tables
- Essential for more complex programming techniques

**Limitations:**
- Limited to JMP instruction only (not used by other instructions)
- Hardware bug requires special handling for page boundary cases
- No indexed indirect variants (those are different addressing modes)

### Implementation in Our Emulator

Our implementation of Indirect addressing includes special handling for the JMP indirect bug:

```rust
impl AddressingMode {
    pub fn get_operand_address(&self, cpu: &Cpu) -> u16 {
        match self {
            // Previous cases...
            AddressingMode::Indirect => {
                // Get the pointer address from the instruction
                let ptr_addr = cpu.read_word(cpu.pc + 1);
                
                // Handle the 6502 JMP indirect bug:
                // If the pointer address ends in $xxFF (page boundary),
                // the second byte is fetched from $xx00 instead of $xx+1:00
                if (ptr_addr & 0x00FF) == 0x00FF {
                    // Get the low byte from the given address
                    let low_byte = cpu.read_byte(ptr_addr) as u16;
                    
                    // Get the high byte from the same page (wrap around)
                    let high_byte = cpu.read_byte(ptr_addr & 0xFF00) as u16;
                    
                    // Combine into the effective address
                    (high_byte << 8) | low_byte
                } else {
                    // Normal case - just read the word from the pointer address
                    cpu.read_word(ptr_addr)
                }
            }
        }
    }
}
```

### Testing Indirect Addressing

We test both the normal case and the page boundary bug case:

```rust
#[test]
fn test_indirect_addressing_mode() {
    let memory = MockMemory::new();
    let mut cpu = Cpu::new(Box::new(memory));
    
    // JMP ($1234) - Jump to the address stored at $1234
    cpu.write_byte(0x0200, 0x6C); // JMP Indirect opcode
    cpu.write_word(0x0201, 0x1234); // Pointer address
    
    // At $1234-$1235, store the target address $ABCD
    cpu.write_word(0x1234, 0xABCD); // Target address
    
    cpu.pc = 0x0200;
    
    // Test indirect addressing
    let addr = AddressingMode::Indirect.get_operand_address(&cpu);
    assert_eq!(addr, 0xABCD, "Indirect addressing should return $ABCD");
}

#[test]
fn test_indirect_addressing_mode_page_boundary_bug() {
    let memory = MockMemory::new();
    let mut cpu = Cpu::new(Box::new(memory));
    
    // JMP ($12FF) - Jump to the address formed by $12FF and $1200
    // due to the 6502 JMP indirect bug
    cpu.write_byte(0x0200, 0x6C); // JMP Indirect opcode
    cpu.write_word(0x0201, 0x12FF); // Pointer address
    
    // The pointer straddles a page boundary - need to keep this as individual bytes
    // due to the hardware bug we're testing
    cpu.write_byte(0x12FF, 0xCD); // Low byte comes from $12FF
    cpu.write_byte(0x1200, 0xAB); // High byte comes from $1200 (same page, not $1300)
    // For comparison, what would be expected without the bug:
    cpu.write_byte(0x1300, 0xEF); // This should NOT be used
    
    cpu.pc = 0x0200;
    
    // Test the JMP indirect bug
    let addr = AddressingMode::Indirect.get_operand_address(&cpu);
    assert_eq!(addr, 0xABCD, "Indirect addressing with page boundary bug should return $ABCD");
}
```

## Indexed Indirect (X,ind) Addressing Mode

### Concept

Indexed Indirect addressing (also called "pre-indexed indirect" or "Indirect,X") is a complex mode that combines zero page addressing, indexing with the X register, and indirection. This mode is particularly useful for working with tables of pointers stored in the zero page.

The operation sequence is:
1. Take a zero page address from the instruction
2. Add the X register to this address (with zero page wrap-around)
3. Read two bytes from the resulting zero page location as a 16-bit address
4. Use that 16-bit address to access memory

In 6502 assembly language, Indexed Indirect addressing is written with parentheses and an ",X" suffix:

```asm
LDA ($80,X)    ; Load the accumulator from the address stored at ($80 + X)
STA ($40,X)    ; Store the accumulator to the address stored at ($40 + X)
EOR ($FF,X)    ; XOR accumulator with value at address stored at ($FF + X)
```

### Memory Layout

Let's visualize how Indexed Indirect addressing works in memory:

```
Memory Address | Content      | Description
---------------|--------------|--------------------------
$0200          | $A1          | LDA (Indirect,X) opcode
$0201          | $80          | Zero page base address ($80)
$0202          | (next opcode) | Next instruction...
... ... ...    |              |
(CPU X register) | $04        | X register contains $04
... ... ...    |              |
$0084          | $34          | Low byte of pointer ($84 = $80 + $04)
$0085          | $12          | High byte of pointer
... ... ...    |              |
$1234          | $42          | The target value at $1234
```

When the CPU executes this instruction:
1. It reads the opcode $A1 at the program counter
2. It identifies this as LDA with Indexed Indirect addressing
3. It reads the next byte ($80) as the zero page base address
4. It adds the X register ($04) to get the effective zero page pointer $84
5. It reads two bytes from $84-$85, giving the address $1234
6. It reads the value at memory address $1234, which contains $42
7. It loads the value $42 into the accumulator register

### Zero Page Wrap-Around Behavior

A key aspect of Indexed Indirect addressing is that the addition of the X register to the zero page address always wraps around within the zero page:

```
Zero page base address: $FF
X register: $02
Effective zero page pointer: $01 (wraps from $FF + $02 = $101 to $01)
```

The pointer address can never leave the zero page. This means that the two bytes of the pointer are always read from addresses $00-$FF.

### Advantages and Limitations

**Advantages:**
- Flexible indirection: Perfect for working with tables of pointers
- Efficient for implementing data structures like arrays of records
- Compact instruction size (2 bytes)
- Full 16-bit addressing range for the final memory access

**Limitations:**
- Complex to understand and use
- Limited to the X register only (not Y)
- Pointers must be stored in zero page
- Usually slower than direct addressing modes
- More prone to coding errors due to complexity

### Implementation in Our Emulator

Here's how we've implemented Indexed Indirect addressing:

```rust
impl AddressingMode {
    pub fn get_operand_address(&self, cpu: &Cpu) -> u16 {
        match self {
            // Previous cases...
            AddressingMode::IndexedIndirect => {
                // 1. Get the zero page pointer base from the instruction
                let base_ptr = cpu.read_byte(cpu.pc + 1);
                
                // 2. Add X register to get the effective pointer (with zero page wrap-around)
                let eff_ptr = base_ptr.wrapping_add(cpu.x);
                
                // 3. Read the target address from the zero page (with wrap-around for the high byte)
                let low_byte = cpu.read_byte(eff_ptr as u16) as u16;
                let high_byte = cpu.read_byte(eff_ptr.wrapping_add(1) as u16) as u16;
                
                // 4. Combine to form the final address
                (high_byte << 8) | low_byte
            }
        }
    }
}
```

Note that we use `wrapping_add` to handle address overflow correctly for the zero page pointer calculation and for reading the high byte of the indirect address, ensuring correct wrap-around behavior.

### Testing Indexed Indirect Addressing

Here's how we test normal operation and zero page wrap-around:

```rust
#[test]
fn test_indexed_indirect_addressing_mode() {
    let memory = MockMemory::new();
    let mut cpu = Cpu::new(Box::new(memory));
    
    // LDA ($80,X) with X=$04
    // So the pointer address is at zero page address $84
    cpu.write_byte(0x0200, 0xA1); // LDA (Indirect,X) opcode
    cpu.write_byte(0x0201, 0x80); // Zero page pointer base
    
    // At zero page address $84-$85 (after adding X), we store the target address $1234
    cpu.write_word(0x0084, 0x1234); // Target address at effective zero page location
    
    // The actual value we want to read is at $1234
    cpu.write_byte(0x1234, 0x42); 
    
    cpu.pc = 0x0200;
    cpu.x = 0x04;
    
    // Test indexed indirect addressing
    let addr = AddressingMode::IndexedIndirect.get_operand_address(&cpu);
    assert_eq!(addr, 0x1234, "Indexed indirect address should be $1234");
    
    let value = cpu.read_byte_using_mode(AddressingMode::IndexedIndirect);
    assert_eq!(value, 0x42, "Value at indexed indirect address $1234 should be $42");
}

#[test]
fn test_indexed_indirect_addressing_mode_wrap_around() {
    let memory = MockMemory::new();
    let mut cpu = Cpu::new(Box::new(memory));
    
    // LDA ($FF,X) with X=$02
    // So the pointer wraps around to zero page address $01-$02
    cpu.write_byte(0x0200, 0xA1); // LDA (Indirect,X) opcode
    cpu.write_byte(0x0201, 0xFF); // Zero page pointer base
    
    // At zero page address $01-$02 (after adding X and wrap-around), 
    // we store the target address $ABCD
    cpu.write_word(0x0001, 0xABCD); // Target address
    
    // The actual value we want to read is at $ABCD
    cpu.write_byte(0xABCD, 0x42); 
    
    cpu.pc = 0x0200;
    cpu.x = 0x02;
    
    // Test indexed indirect addressing with wrap-around
    let addr = AddressingMode::IndexedIndirect.get_operand_address(&cpu);
    assert_eq!(addr, 0xABCD, "Indexed indirect with wrap-around should point to $ABCD");
    
    let value = cpu.read_byte_using_mode(AddressingMode::IndexedIndirect);
    assert_eq!(value, 0x42, "Value at wrapped indexed indirect address $ABCD should be $42");
}
```

## Indirect Indexed (ind,Y) Addressing Mode

### Concept

Indirect Indexed addressing (also called "post-indexed indirect" or "Indirect,Y") is another complex mode that combines zero page indirection with Y-register indexing. Unlike Indexed Indirect, the indexing happens *after* the indirection, which makes it particularly useful for working with arrays where the base address is stored in zero page.

The operation sequence is:
1. Take a zero page address from the instruction
2. Read two bytes from that zero page location as a 16-bit base address
3. Add the Y register to this 16-bit address
4. Use the resulting address to access memory

In 6502 assembly language, Indirect Indexed addressing is written with parentheses around the zero page address, followed by ",Y":

```asm
LDA ($80),Y    ; Load the accumulator from the address stored at $80 plus Y
STA ($40),Y    ; Store the accumulator to the address stored at $40 plus Y
CMP ($FF),Y    ; Compare accumulator with value at address stored at $FF plus Y
```

### Memory Layout

Let's visualize how Indirect Indexed addressing works in memory:

```
Memory Address | Content      | Description
---------------|--------------|--------------------------
$0200          | $B1          | LDA (Indirect),Y opcode
$0201          | $80          | Zero page pointer address ($80)
$0202          | (next opcode) | Next instruction...
... ... ...    |              |
(CPU Y register) | $10        | Y register contains $10
... ... ...    |              |
$0080          | $34          | Low byte of base address
$0081          | $12          | High byte of base address
... ... ...    |              |
$1244          | $42          | The target value ($1234 + $10 = $1244)
```

When the CPU executes this instruction:
1. It reads the opcode $B1 at the program counter
2. It identifies this as LDA with Indirect Indexed addressing
3. It reads the next byte ($80) as the zero page pointer address
4. It reads two bytes from $80-$81, giving the base address $1234
5. It adds the Y register ($10) to get the effective address $1244
6. It reads the value at memory address $1244, which contains $42
7. It loads the value $42 into the accumulator register

### Page Crossing and Zero Page Wrap-Around

Indirect Indexed addressing has two notable behaviors:

1. **Page Crossing Penalty**: When the addition of the Y register causes a page boundary to be crossed, many instructions will take an extra cycle to execute. This is important for cycle-accurate emulation.

2. **Zero Page Pointer Wrap-Around**: If the zero page pointer address is at the end of the zero page (e.g., $FF), the high byte for the indirect address is read from address $00, not from $100.

```
Zero page pointer: $FF
Low byte read from: $FF
High byte read from: $00 (wraps around within zero page)
```

### Advantages and Limitations

**Advantages:**
- Perfect for array access: Pointer to array base + Y as index
- Efficient for string operations and table lookups
- Compact instruction size (2 bytes)
- Full 16-bit addressing range for the final memory access

**Limitations:**
- Complex to understand and use
- Limited to the Y register only (not X)
- Base address pointer must be stored in zero page
- Usually slower than direct addressing modes
- More prone to coding errors due to complexity

### Implementation in Our Emulator

Here's how we've implemented Indirect Indexed addressing:

```rust
impl AddressingMode {
    pub fn get_operand_address(&self, cpu: &Cpu) -> u16 {
        match self {
            // Previous cases...
            AddressingMode::IndirectIndexed => {
                // 1. Get the zero page pointer from the instruction
                let zp_ptr = cpu.read_byte(cpu.pc + 1);
                
                // 2. Read the base address from zero page (wrapping around for high byte)
                let low_byte = cpu.read_byte(zp_ptr) as u16;
                let high_byte = cpu.read_byte(zp_ptr.wrapping_add(1) & 0xFF) as u16;
                let base_addr = (high_byte << 8) | low_byte;
                
                // 3. Add Y register to get the final effective address
                base_addr.wrapping_add(cpu.y as u16)
            }
        }
    }
}
```

Note that we handle the zero page wrap-around by using `zp_ptr.wrapping_add(1) & 0xFF` to ensure that the high byte is always read from the zero page.

### Testing Indirect Indexed Addressing

We test normal operation, page crossing, and zero page wrap-around:

```rust
#[test]
fn test_indirect_indexed_addressing_mode() {
    let memory = MockMemory::new();
    let mut cpu = Cpu::new(Box::new(memory));
    
    // LDA ($80),Y with Y=$10
    // The zero page pointer $80-$81 contains $1234
    // Final effective address is $1234 + $10 = $1244
    cpu.write_byte(0x0200, 0xB1); // LDA (Indirect),Y opcode
    cpu.write_byte(0x0201, 0x80); // Zero page pointer
    
    // At zero page address $80-$81, we store the base address $1234
    cpu.write_word(0x0080, 0x1234); // Base address in zero page
    
    // The actual value we want to read is at $1244 (after adding Y)
    cpu.write_byte(0x1244, 0x42); 
    
    cpu.pc = 0x0200;
    cpu.y = 0x10;
    
    // Test indirect indexed addressing
    let addr = AddressingMode::IndirectIndexed.get_operand_address(&cpu);
    assert_eq!(addr, 0x1244, "Indirect indexed address should be $1244");
    
    let value = cpu.read_byte_using_mode(AddressingMode::IndirectIndexed);
    assert_eq!(value, 0x42, "Value at indirect indexed address $1244 should be $42");
}

#[test]
fn test_indirect_indexed_addressing_mode_page_crossing() {
    let memory = MockMemory::new();
    let mut cpu = Cpu::new(Box::new(memory));
    
    // LDA ($80),Y with Y=$F0
    // The zero page pointer $80-$81 contains $1234
    // Final effective address crosses a page: $1234 + $F0 = $1324
    cpu.write_byte(0x0200, 0xB1); // LDA (Indirect),Y opcode
    cpu.write_byte(0x0201, 0x80); // Zero page pointer
    
    // At zero page address $80-$81, we store the base address $1234
    cpu.write_word(0x0080, 0x1234); // Base address in zero page
    
    // The actual value we want to read is at $1324 (after adding Y, crossing a page)
    cpu.write_byte(0x1324, 0x42); 
    
    cpu.pc = 0x0200;
    cpu.y = 0xF0;
    
    // Test indirect indexed addressing with page crossing
    let addr = AddressingMode::IndirectIndexed.get_operand_address(&cpu);
    assert_eq!(addr, 0x1324, "Indirect indexed with page crossing should be $1324");
    
    let value = cpu.read_byte_using_mode(AddressingMode::IndirectIndexed);
    assert_eq!(value, 0x42, "Value after page crossing should be $42");
}

#[test]
fn test_indirect_indexed_addressing_mode_zero_page_wrap() {
    let memory = MockMemory::new();
    let mut cpu = Cpu::new(Box::new(memory));
    
    // LDA ($FF),Y with Y=$10
    // The zero page pointer wraps from $FF to $00 for the high byte
    cpu.write_byte(0x0200, 0xB1); // LDA (Indirect),Y opcode
    cpu.write_byte(0x0201, 0xFF); // Zero page pointer at $FF (will wrap for high byte)
    
    // Store the base address split between $FF and $00 (wrap-around in zero page)
    cpu.write_byte(0x00FF, 0x34); // Low byte at $FF
    cpu.write_byte(0x0000, 0x12); // High byte at $00 (wrapped around)
    
    // The actual value we want to read is at $1244 (after adding Y)
    cpu.write_byte(0x1244, 0x42); 
    
    cpu.pc = 0x0200;
    cpu.y = 0x10;
    
    // Test indirect indexed addressing with zero page wrap-around
    let addr = AddressingMode::IndirectIndexed.get_operand_address(&cpu);
    assert_eq!(addr, 0x1244, "Indirect indexed with ZP wrap should be $1244");
    
    let value = cpu.read_byte_using_mode(AddressingMode::IndirectIndexed);
    assert_eq!(value, 0x42, "Value with ZP wrap-around should be $42");
}
```

## Conclusion

With all addressing modes now implemented and tested, we have completed a critical component of our 6502 CPU emulation. These addressing modes form the foundation for all 6502 instructions and are essential for accurately emulating how the processor accesses memory.

Let's recap the addressing modes we've implemented:

1. **Immediate**: The operand is the byte following the instruction
2. **Zero Page**: The operand is at a single-byte address in the zero page
3. **Zero Page,X**: Zero page address + X register (with wrap-around)
4. **Zero Page,Y**: Zero page address + Y register (with wrap-around)
5. **Absolute**: The operand is at a full 16-bit address
6. **Absolute,X**: 16-bit address + X register
7. **Absolute,Y**: 16-bit address + Y register
8. **Indirect**: The operand is at the address stored at a 16-bit pointer
9. **Indexed Indirect (X,ind)**: Zero page address + X, then indirection
10. **Indirect Indexed (ind,Y)**: Zero page indirection, then add Y

Each of these modes has unique characteristics and behaviors that we've carefully implemented to ensure our emulator accurately reflects the original hardware.

In the next chapter, we'll build on this foundation to implement the actual instructions that use these addressing modes, bringing our 6502 CPU emulation to life.

## Further Enhancements

There are several ways we could enhance our addressing mode implementation:

1. **Cycle Counting**: Add timing information to calculate the correct number of cycles each addressing mode takes
2. **Page Boundary Detection**: Implement detection of page crossing for timing-sensitive operations
3. **Optimization**: Refine the implementation for performance while maintaining accuracy
4. **Debugging Support**: Add tracing and debugging functionality to visualize addressing mode operations

These enhancements will be addressed in later chapters as we continue to refine our emulator.
