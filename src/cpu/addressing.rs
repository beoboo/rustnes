/// Addressing modes for the 6502 CPU
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddressingMode {
    Immediate,
    // We'll add more modes later
}

impl AddressingMode {
    /// Returns the operand address for the given addressing mode
    /// For Immediate mode, this is simply the byte following the opcode (PC+1)
    pub fn get_operand_address(&self, program_counter: u16) -> u16 {
        match self {
            AddressingMode::Immediate => program_counter + 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{cpu::Cpu, memory::{Memory, MockMemory}};

    use super::*;
    
    #[test]
    fn test_immediate_addressing_mode() {
        // Create a simple setup with a CPU and mock memory
        let mut cpu = Cpu::new(Box::new(MockMemory::new()));
        let mut memory = MockMemory::new(); // Assuming you have a mock memory implementation
        
        // Setup the memory with an instruction and immediate value
        memory.write_byte(0x0200, 0xA9); // LDA Immediate opcode
        memory.write_byte(0x0201, 0x42); // The immediate value $42
        
        // Set CPU state
        cpu.pc = 0x0200;
        cpu.memory = Box::new(memory); // Assuming you have a method to connect memory
        
        // Test immediate addressing mode
        let value = cpu.read_byte_using_mode(AddressingMode::Immediate);
        assert_eq!(value, 0x42, "Immediate addressing mode should read the value after PC");
    }
}
