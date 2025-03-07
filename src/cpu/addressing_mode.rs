use super::Cpu;

/// Addressing modes for the 6502 CPU
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
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
}
