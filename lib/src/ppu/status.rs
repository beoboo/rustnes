use std::fmt::{Display, Formatter, Result};

use crate::types::Byte;

#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Status {
    // sprite overflow
    pub O: bool,
    // sprite 0 hit
    pub S: bool,
    // vblank
    pub V: bool,
}

fn _build_status_flag(flags: &str, flag: &str) -> bool {
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
        self.O = false;
        self.S = false;
        self.V = false;
    }

    pub fn from_byte(byte: Byte) -> Status {
        Status {
            O: _check_bit(byte, 5),
            S: _check_bit(byte, 6),
            V: _check_bit(byte, 7),
        }
    }

    pub fn to_byte(&self) -> Byte {
        _bool_to_bit(self.O) << 5 |
            _bool_to_bit(self.S) << 6 |
            _bool_to_bit(self.V) << 7
    }

    pub fn from_string(flags: &str) -> Status {
        let flags = flags.as_bytes();

        Status {
            O: flags[5] as char == 'O',
            S: flags[6] as char == 'S',
            V: flags[7] as char == 'V',
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}{}{}-----",
               if self.V { "V" } else { "v" },
               if self.S { "S" } else { "s" },
               if self.O { "O" } else { "o" },
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

        assert_that!(status.to_string(), eq("VSO-----"))
    }

    #[test]
    fn test_to_byte() {
        let status = Status::from_byte(0xE0);

        assert_that!(status.to_byte(), eq(0xE0))
    }
}