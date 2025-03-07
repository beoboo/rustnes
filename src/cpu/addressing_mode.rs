use super::Cpu;

/// Addressing modes for the 6502 CPU
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect, // (Indirect,X) - Pre-indexed indirect
    IndirectIndexed, // (Indirect),Y - Post-indexed indirect
    // We'll add more modes later
}

impl AddressingMode {
    /// Returns the operand address for the given addressing mode
    /// For Immediate mode, this is simply the byte following the opcode (PC+1)
    pub fn get_operand_address(&self, cpu: &Cpu) -> u16 {
        match self {
            AddressingMode::Immediate => cpu.pc + 1,
            AddressingMode::ZeroPage => {
                // Zero page addressing uses only a single byte for the address
                // We read that byte and use it as an address in the range $0000-$00FF
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
            },
            AddressingMode::ZeroPageY => {
                // Get the zero page address from the byte after the opcode
                let zero_page_addr = cpu.read_byte(cpu.pc + 1);
                
                // Add the Y register to it (with wrap-around in the zero page)
                let effective_addr = (zero_page_addr.wrapping_add(cpu.y)) as u16;
                
                // The high byte is always 0 since we stay in the zero page
                effective_addr
            },
            AddressingMode::Absolute => {
                // Read a full 16-bit address (little-endian)
                cpu.read_word(cpu.pc + 1)
            },
            AddressingMode::AbsoluteX => {
                // Read the base address and add X register
                let base_addr = cpu.read_word(cpu.pc + 1);
                base_addr.wrapping_add(cpu.x as u16)
            },
            AddressingMode::AbsoluteY => {
                // Read the base address and add Y register
                let base_addr = cpu.read_word(cpu.pc + 1);
                base_addr.wrapping_add(cpu.y as u16)
            },
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
            },
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
            },
            AddressingMode::IndirectIndexed => {
                // 1. Get the zero page pointer from the instruction
                let zp_ptr = cpu.read_byte(cpu.pc + 1) as u16;
                
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

#[cfg(test)]
mod tests {
    use crate::{cpu::Cpu, memory::MockMemory};

    use super::*;
    
    #[test]
    fn test_immediate_addressing_mode() {
        // Create a simple setup with a CPU and mock memory
        let mut cpu = Cpu::new(Box::new(MockMemory::new()));
        
        // Setup the memory with an instruction and immediate value
        cpu.write_byte(0x0200, 0xA9); // LDA Immediate opcode
        cpu.write_byte(0x0201, 0x42); // The immediate value $42
        
        // Set CPU state
        cpu.pc = 0x0200;
        
        // Test immediate addressing mode
        let value = cpu.read_byte_using_mode(AddressingMode::Immediate);
        assert_eq!(value, 0x42, "Immediate addressing mode should read the value after PC");
    }

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
    
    #[test]
    fn test_zero_page_x_addressing_mode_wrap_around() {
        // Create a CPU with mock memory
        let memory = MockMemory::new();
        let mut cpu = Cpu::new(Box::new(memory));
        
        // Setup memory for wrap-around test:
        // At $0200: Opcode using Zero Page,X addressing
        // At $0201: Zero page address $FE
        // CPU X register: $05
        // Effective address: $03 (after wrap-around)
        // At $0003: The value $42 we want to read
        cpu.write_byte(0x0200, 0xB5); // LDA Zero Page,X opcode
        cpu.write_byte(0x0201, 0xFE); // Zero page address $FE
        cpu.write_byte(0x0003, 0x42); // Value at wrapped address $03
        
        // Set CPU state
        cpu.pc = 0x0200;
        cpu.x = 0x05;  // X register is $05
        // Effective address should be $03 (0xFE + 0x05 = 0x103, which wraps to 0x03)
        
        // Test zero page,X wrap-around behavior
        let addr = AddressingMode::ZeroPageX.get_operand_address(&cpu);
        assert_eq!(addr, 0x0003, "Zero page,X address should wrap to $0003");
        
        let value = cpu.read_byte_using_mode(AddressingMode::ZeroPageX);
        assert_eq!(value, 0x42, "Value at wrapped address $03 should be $42");
    }
    
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
    }
    
    #[test]
    fn test_zero_page_y_addressing_mode_wrap_around() {
        // Create a CPU with mock memory
        let memory = MockMemory::new();
        let mut cpu = Cpu::new(Box::new(memory));
        
        // Setup memory for wrap-around test:
        // At $0200: Opcode using Zero Page,Y addressing
        // At $0201: Zero page address $FB
        // CPU Y register: $07
        // Effective address: $02 (after wrap-around)
        // At $0002: The value $42 we want to read
        cpu.write_byte(0x0200, 0xB6); // LDX Zero Page,Y opcode
        cpu.write_byte(0x0201, 0xFB); // Zero page address $FB
        cpu.write_byte(0x0002, 0x42); // Value at wrapped address $02
        
        // Set CPU state
        cpu.pc = 0x0200;
        cpu.y = 0x07;  // Y register is $07
        // Effective address should be $02 (0xFB + 0x07 = 0x102, which wraps to 0x02)
        
        // Test zero page,Y wrap-around behavior
        let addr = AddressingMode::ZeroPageY.get_operand_address(&cpu);
        assert_eq!(addr, 0x0002, "Zero page,Y address should wrap to $0002");
        
        let value = cpu.read_byte_using_mode(AddressingMode::ZeroPageY);
        assert_eq!(value, 0x42, "Value at wrapped address $02 should be $42");
    }
    
    #[test]
    fn test_absolute_addressing_mode() {
        // Create a CPU with mock memory
        let memory = MockMemory::new();
        let mut cpu = Cpu::new(Box::new(memory));
        
        // Setup memory with Absolute addressing
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
    
    #[test]
    fn test_absolute_x_addressing_mode() {
        let memory = MockMemory::new();
        let mut cpu = Cpu::new(Box::new(memory));
        
        // Base address $1234, X=$10, effective address=$1244
        cpu.write_byte(0x0200, 0xBD); // LDA Absolute,X opcode
        cpu.write_byte(0x0201, 0x34); // Low byte of address
        cpu.write_byte(0x0202, 0x12); // High byte of address
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
        cpu.write_byte(0x0201, 0xF0); // Low byte of address
        cpu.write_byte(0x0202, 0x12); // High byte of address
        cpu.write_byte(0x1310, 0x42); // Value at effective address $1310
        
        cpu.pc = 0x0200;
        cpu.x = 0x20;
        
        let addr = AddressingMode::AbsoluteX.get_operand_address(&cpu);
        assert_eq!(addr, 0x1310);
        
        let value = cpu.read_byte_using_mode(AddressingMode::AbsoluteX);
        assert_eq!(value, 0x42);
    }
    
    #[test]
    fn test_absolute_y_addressing_mode() {
        let memory = MockMemory::new();
        let mut cpu = Cpu::new(Box::new(memory));
        
        // Base address $1234, Y=$15, effective address=$1249
        cpu.write_byte(0x0200, 0xB9); // LDA Absolute,Y opcode
        cpu.write_byte(0x0201, 0x34); // Low byte of address
        cpu.write_byte(0x0202, 0x12); // High byte of address
        cpu.write_byte(0x1249, 0x42); // Value at effective address $1249
        
        cpu.pc = 0x0200;
        cpu.y = 0x15;
        
        let addr = AddressingMode::AbsoluteY.get_operand_address(&cpu);
        assert_eq!(addr, 0x1249);
        
        let value = cpu.read_byte_using_mode(AddressingMode::AbsoluteY);
        assert_eq!(value, 0x42);
    }
    
    #[test]
    fn test_absolute_y_addressing_mode_wrap_around() {
        let memory = MockMemory::new();
        let mut cpu = Cpu::new(Box::new(memory));
        
        // Base address $FFFA, Y=$10, effective address=$000A (wrap around)
        cpu.write_byte(0x0200, 0xB9); // LDA Absolute,Y opcode
        cpu.write_byte(0x0201, 0xFA); // Low byte of address
        cpu.write_byte(0x0202, 0xFF); // High byte of address
        cpu.write_byte(0x000A, 0x42); // Value at wrapped address $000A
        
        cpu.pc = 0x0200;
        cpu.y = 0x10;
        
        let addr = AddressingMode::AbsoluteY.get_operand_address(&cpu);
        assert_eq!(addr, 0x000A);
        
        let value = cpu.read_byte_using_mode(AddressingMode::AbsoluteY);
        assert_eq!(value, 0x42);
    }
    
    #[test]
    fn test_indirect_addressing_mode() {
        let memory = MockMemory::new();
        let mut cpu = Cpu::new(Box::new(memory));
        
        // JMP ($1234) - Jump to the address stored at $1234
        cpu.write_byte(0x0200, 0x6C); // JMP Indirect opcode
        cpu.write_byte(0x0201, 0x34); // Low byte of indirect pointer
        cpu.write_byte(0x0202, 0x12); // High byte of indirect pointer
        
        // At $1234-$1235, store the target address $ABCD
        cpu.write_byte(0x1234, 0xCD); // Low byte of target address
        cpu.write_byte(0x1235, 0xAB); // High byte of target address
        
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
        cpu.write_byte(0x0201, 0xFF); // Low byte of indirect pointer
        cpu.write_byte(0x0202, 0x12); // High byte of indirect pointer
        
        // The pointer straddles a page boundary:
        cpu.write_byte(0x12FF, 0xCD); // Low byte comes from $12FF
        cpu.write_byte(0x1200, 0xAB); // High byte comes from $1200 (same page, not $1300)
        // For comparison, what would be expected without the bug:
        cpu.write_byte(0x1300, 0xEF); // This should NOT be used
        
        cpu.pc = 0x0200;
        
        // Test the JMP indirect bug
        let addr = AddressingMode::Indirect.get_operand_address(&cpu);
        assert_eq!(addr, 0xABCD, "Indirect addressing with page boundary bug should return $ABCD");
    }
    
    #[test]
    fn test_indexed_indirect_addressing_mode() {
        let memory = MockMemory::new();
        let mut cpu = Cpu::new(Box::new(memory));
        
        // LDA ($80,X) with X=$04
        // So the pointer address is at zero page address $84
        cpu.write_byte(0x0200, 0xA1); // LDA (Indirect,X) opcode
        cpu.write_byte(0x0201, 0x80); // Zero page pointer base
        
        // At zero page address $84-$85 (after adding X), we store the target address $1234
        cpu.write_byte(0x0084, 0x34); // Low byte of target address
        cpu.write_byte(0x0085, 0x12); // High byte of target address
        
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
        cpu.write_byte(0x0001, 0xCD); // Low byte of target address
        cpu.write_byte(0x0002, 0xAB); // High byte of target address
        
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
        cpu.write_byte(0x0080, 0x34); // Low byte of base address
        cpu.write_byte(0x0081, 0x12); // High byte of base address
        
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
        cpu.write_byte(0x0080, 0x34); // Low byte of base address
        cpu.write_byte(0x0081, 0x12); // High byte of base address
        
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
}
