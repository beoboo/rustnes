use std::fmt::{Display, Formatter, Result};

use crate::types::Byte;

#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Control {
    // Nametable select 1
    pub N1: bool,
    // Nametable select 2
    pub N2: bool,
    // increment mode
    pub I: bool,
    // sprite tile select
    pub S: bool,
    // background tile select
    pub B: bool,
    // sprite height
    pub H: bool,
    // master/slave
    pub P: bool,
    // NMI enable
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

impl Control {
    pub fn reset(&mut self) {
        self.N1 = false;
        self.N2 = false;
        self.I = false;
        self.S = false;
        self.B = false;
        self.P = false;
        self.H = false;
        self.V = false;
    }

    pub fn from_byte(byte: Byte) -> Control {
        Control {
            N1: _check_bit(byte, 0),
            N2: _check_bit(byte, 1),
            I: _check_bit(byte, 2),
            S: _check_bit(byte, 3),
            B: _check_bit(byte, 4),
            P: _check_bit(byte, 5),
            H: _check_bit(byte, 6),
            V: _check_bit(byte, 7),
        }
    }

    pub fn to_byte(&self) -> Byte {
        _bool_to_bit(self.N1) |
            _bool_to_bit(self.N2) << 1 |
            _bool_to_bit(self.I) << 2 |
            _bool_to_bit(self.S) << 3 |
            _bool_to_bit(self.B) << 4 |
            _bool_to_bit(self.P) << 5 |
            _bool_to_bit(self.H) << 6 |
            _bool_to_bit(self.V) << 7
    }

    pub fn from_string(flags: &str) -> Control {
        let flags = flags.as_bytes();

        Control {
            N1: flags[0] as char == 'N',
            N2: flags[1] as char == 'N',
            I: flags[2] as char == 'I',
            S: flags[3] as char == 'S',
            B: flags[4] as char == 'B',
            P: flags[5] as char == 'P',
            H: flags[6] as char == 'H',
            V: flags[7] as char == 'V',
        }
    }
}

impl Display for Control {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}{}{}{}{}{}{}{}",
               if self.V { "V" } else { "v" },
               if self.H { "H" } else { "h" },
               if self.P { "P" } else { "p" },
               if self.B { "B" } else { "b" },
               if self.S { "S" } else { "s" },
               if self.I { "I" } else { "i" },
               if self.N2 { "N" } else { "n" },
               if self.N1 { "N" } else { "n" },
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
        let status = Control::from_byte(0xFF);

        assert_that!(status.to_string(), eq("VHPBSINN"))
    }

    #[test]
    fn test_to_byte() {
        let status = Control::from_byte(0xBB);

        assert_that!(status.to_byte(), eq(0xBB))
    }
}