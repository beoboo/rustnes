use super::Cpu;

/// Addressing modes for the 6502 CPU
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
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
        // At $0200: Some opcode that uses zero page addressing
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
}
