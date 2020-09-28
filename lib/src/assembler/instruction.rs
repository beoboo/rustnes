use std::collections::HashMap;

use crate::addressing_mode::AddressingMode;
use crate::error::Error;
use crate::types::Byte;

type AddressingModes = HashMap<AddressingMode, Byte>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    pub op_code: String,
    pub implied: bool,
    pub relative: bool,
    pub modes: AddressingModes,
}

impl Instruction {
    pub fn new(op_code: &str, implied: bool, relative: bool) -> Instruction {
        Instruction {
            op_code: op_code.to_string(),
            implied,
            relative,
            modes: AddressingModes::new(),
        }
    }
    //
    // pub fn add_mode(&mut self, addressing_mode: AddressingMode, value: Byte) {
    //     self.modes.insert(addressing_mode, value);
    // }

    pub fn find(&self, addressing_mode: AddressingMode) -> Result<Byte, Error> {
        if !self.modes.contains_key(&addressing_mode) {
            return Err(Error::UnknownAddressingMode(self.op_code.to_string(), addressing_mode));
        }

        Ok(self.modes[&addressing_mode])
    }

    pub fn contains(&self, addressing_mode: AddressingMode) -> bool {
        self.modes.contains_key(&addressing_mode)
    }
}


#[cfg(test)]
mod tests {
    use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use super::*;
    //
    // #[test]
    // fn find_brk() {
    //     let mut instruction = Instruction::new("BRK", true, false);
    //     instruction.add_mode(AddressingMode::Immediate, 0x00);
    //
    //     assert_that!(instruction.implied, is(true));
    //     assert_that!(instruction.relative, is(false));
    //     assert_that!(instruction.find(AddressingMode::Immediate).unwrap(), equal_to(0x00));
    // }

    #[test]
    fn find_unknown_addressing_mode() {
        let instruction = Instruction::new("BRK", false, true);

        let error = instruction.find(AddressingMode::Absolute).unwrap_err();

        assert_that!(error, equal_to(Error::UnknownAddressingMode("BRK".to_string(), AddressingMode::Absolute)));
    }
}