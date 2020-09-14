use crate::addressing_mode::AddressingMode;
use crate::types::Byte;
use std::collections::HashMap;
use crate::error::Error;
use crate::assembler::instruction::Instruction;

type Map = HashMap<String, Instruction>;


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddressingModeMap {
    map: Map
}

pub fn insert(map: &mut Map, op_code: &str, implied: bool, addressing_mode: AddressingMode, value: Byte) {
    let mut instruction = map.entry(op_code.to_string()).or_insert(Instruction::new(op_code, implied));

    instruction.modes.insert(addressing_mode, value);
}

impl AddressingModeMap {
    pub fn new() -> AddressingModeMap {
        let mut map = Map::new();

        insert(&mut map, "BRK", true, AddressingMode::Immediate, 0x00);
        insert(&mut map, "LDA", false, AddressingMode::Immediate, 0xA9);

        AddressingModeMap {
            map
        }
    }

    pub fn find(&self, op_code: &str) -> Result<Instruction, Error> {
        if !self.map.contains_key(op_code) {
            return Err(Error::UnknownOpCode(op_code.to_string()))
        }

        Ok(self.map[op_code].clone())
    }
}


#[cfg(test)]
mod tests {
    use hamcrest2::prelude::*;

    use crate::parser::Parser;

    use super::*;

    #[test]
    fn find_brk() {
        assert_instruction("BRK", true);
        assert_instruction("LDA", false);
    }

    #[test]
    fn find_unknown_opcode() {
        let map = AddressingModeMap::new();

        let error = map.find("UNK").unwrap_err();

        assert_that!(error, equal_to(Error::UnknownOpCode("UNK".to_string())));
    }

    fn assert_instruction(op_code: &str, implied: bool) {
        let map = AddressingModeMap::new();
        let instruction = map.find(op_code).unwrap();

        assert_that!(instruction.op_code, equal_to(op_code.to_string()));
        assert_that!(instruction.implied, equal_to(implied));
    }
}