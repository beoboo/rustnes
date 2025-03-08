use super::{AddressingMode, Cpu, CpuFlag};

/// 6502 CPU instruction opcodes
/// Starting with just LDA immediate as a first step
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    LDA, // Load Accumulator
}

/// Instruction metadata containing the opcode, instruction type, addressing mode,
/// and cycle count information
#[derive(Debug, Clone, Copy)]
pub struct InstructionMetadata {
    pub opcode: u8,
    pub instruction: Instruction,
    pub addressing_mode: AddressingMode,
    pub bytes: u8,
    pub cycles: u8,
}

/// Instruction decoder for the 6502 CPU
pub struct InstructionDecoder {
    /// Lookup table for CPU instructions
    instruction_table: [Option<InstructionMetadata>; 256],
}

impl InstructionDecoder {
    /// Create a new instruction decoder with a populated lookup table
    pub fn new() -> Self {
        let mut decoder = Self {
            instruction_table: [None; 256],
        };
        decoder.populate_instruction_table();
        decoder
    }

    /// Populate the instruction lookup table with only LDA immediate
    fn populate_instruction_table(&mut self) {
        // Just LDA immediate (0xA9) for now
        self.add_instruction(0xA9, Instruction::LDA, AddressingMode::Immediate, 2, 2);
    }

    /// Add an instruction to the lookup table
    fn add_instruction(
        &mut self,
        opcode: u8,
        instruction: Instruction,
        addressing_mode: AddressingMode,
        bytes: u8,
        cycles: u8,
    ) {
        self.instruction_table[opcode as usize] = Some(InstructionMetadata {
            opcode,
            instruction,
            addressing_mode,
            bytes,
            cycles,
        });
    }

    /// Decode an opcode into instruction metadata
    pub fn decode(&self, opcode: u8) -> Option<InstructionMetadata> {
        self.instruction_table[opcode as usize]
    }
}

impl Cpu {
    /// Fetch the next instruction
    pub fn fetch(&mut self) -> u8 {
        let opcode = self.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        opcode
    }

    /// Execute a single instruction
    pub fn execute(&mut self, instruction_metadata: InstructionMetadata) -> u8 {
        let cycles = instruction_metadata.cycles;
        
        // Execute the instruction based on addressing mode
        match instruction_metadata.instruction {
            Instruction::LDA => self.lda(instruction_metadata.addressing_mode),
        }

        // Increment PC (already incremented by 1 in fetch)
        self.pc = self.pc.wrapping_add((instruction_metadata.bytes - 1) as u16);

        cycles
    }

    /// LDA - Load Accumulator (immediate mode only for now)
    fn lda(&mut self, addressing_mode: AddressingMode) {
        if addressing_mode == AddressingMode::Immediate {
            // In immediate mode, the value is the byte after the opcode
            let value = self.read_byte(self.pc);
            self.a = value;
            
            // Set flags
            self.set_flag(CpuFlag::Zero, value == 0);
            self.set_flag(CpuFlag::Negative, (value & 0x80) != 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Ram;

    #[test]
    fn test_lda_immediate() {
        let mut cpu = Cpu::new(Box::new(Ram::new()));
        
        // Set up test
        cpu.pc = 0x0100;
        cpu.write_byte(0x0100, 0xA9); // LDA immediate opcode
        cpu.write_byte(0x0101, 0x42); // Value to load
        
        // Execute
        let opcode = cpu.fetch();
        let decoder = InstructionDecoder::new();
        let metadata = decoder.decode(opcode).unwrap();
        let cycles = cpu.execute(metadata);
        
        // Verify results
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.pc, 0x0102);
        assert_eq!(cycles, 2);
        assert!(!cpu.get_flag(CpuFlag::Zero));
        assert!(!cpu.get_flag(CpuFlag::Negative));
    }
} 