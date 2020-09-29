use std::fmt::{Display, Formatter, Result};

use crate::types::Byte;

#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Status {
    // Carry
    pub C: bool,
    // Zero
    pub Z: bool,
    // Enable/Disable Interrupts
    pub I: bool,
    // Decimal Mode
    pub D: bool,
    // Break
    pub B: bool,
    // Unused
    pub U: bool,
    // Overflow
    pub V: bool,
    // Negative
    pub N: bool,
}

fn _build_status_flag(flags: &str, flag: char) -> bool {
    flags.contains(flag)
}

fn _check_bit(byte: Byte, pos: Byte) -> bool {
    byte & (1 << pos) == (1 << pos)
}

fn _bool_to_bit(flag: bool) -> Byte {
    if flag { 1 } else { 0 }
}

impl Status {
    pub fn reset(&mut self) {
        self.C = false;
        self.Z = false;
        self.I = false;
        self.D = false;
        self.B = false;
        self.U = true;
        self.V = false;
        self.N = false;
    }

    pub fn from_byte(byte: Byte) -> Status {
        Status {
            C: _check_bit(byte, 0),
            Z: _check_bit(byte, 1),
            I: _check_bit(byte, 2),
            D: _check_bit(byte, 3),
            B: _check_bit(byte, 4),
            U: _check_bit(byte, 5),
            V: _check_bit(byte, 6),
            N: _check_bit(byte, 7),
        }
    }

    pub fn to_byte(&self) -> Byte {
        _bool_to_bit(self.C) |
            _bool_to_bit(self.Z) << 1 |
            _bool_to_bit(self.I) << 2 |
            _bool_to_bit(self.D) << 3 |
            _bool_to_bit(self.B) << 4 |
            _bool_to_bit(self.U) << 5 |
            _bool_to_bit(self.V) << 6 |
            _bool_to_bit(self.Z) << 7
    }

    pub fn from_string(flags: &str) -> Status {
        Status {
            C: _build_status_flag(flags, 'C'),
            Z: _build_status_flag(flags, 'Z'),
            I: _build_status_flag(flags, 'I'),
            D: _build_status_flag(flags, 'D'),
            B: _build_status_flag(flags, 'B'),
            U: _build_status_flag(flags, 'U'),
            V: _build_status_flag(flags, 'V'),
            N: _build_status_flag(flags, 'N'),
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}{}{}{}{}{}{}{}",
               if self.C { "C" } else { "c" },
               if self.Z { "Z" } else { "z" },
               if self.I { "I" } else { "i" },
               if self.D { "D" } else { "d" },
               if self.B { "B" } else { "b" },
               if self.U { "U" } else { "u" },
               if self.V { "V" } else { "v" },
               if self.N { "N" } else { "n" },
        )
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn test_from_byte() {
        let status = Status::from_byte(0xFF);

        assert_that!(status.to_string(), eq("CZIDBUVN"))
    }

    #[test]
    fn test_to_byte() {
        let status = Status::from_byte(0xBB);

        assert_that!(status.to_byte(), eq(0xBB))
    }
}