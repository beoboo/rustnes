use std::collections::BTreeMap;

use crate::disassembler::assembly_line::AssemblyLine;
use crate::instructions::addressing_mode::AddressingMode;
use crate::types::Word;

#[derive(Default, Clone, Debug)]
pub struct Assembly {
    map: BTreeMap<Word, AssemblyLine>,
}

impl Assembly {
    pub fn add(&mut self, address: Word, source_code: &str, addressing_mode: AddressingMode, operand: Word) {
        self.map.insert(address, AssemblyLine::new(address, source_code, addressing_mode, operand));
    }

    pub fn before(&self, address: Word, count: usize) -> Vec<&AssemblyLine> {
        self.map
            .iter()
            .filter(|(k, _)| *k < &address)
            .rev()
            .take(count)
            .map(|(_, v)| v)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    pub fn at(&self, address: Word) -> &AssemblyLine {
        self.map.get(&address).unwrap_or_else(|| panic!("[Assembly::at] Cannot find address {:#06X}", address))
    }

    pub fn after(&self, address: Word, count: usize) -> Vec<&AssemblyLine> {
        self.map
            .iter()
            .filter(|(k, _)| *k > &address)
            .take(count)
            .map(|(_, v)| v)
            .collect::<Vec<_>>()
    }
}

