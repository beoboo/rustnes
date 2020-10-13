use std::fmt::{Display, Formatter, Result};

use crate::types::Byte;

#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mask {
    // Grayscale
    pub G: bool,
    // background left column enable
    pub m: bool,
    // sprite left column enable
    pub M: bool,
    // background enable
    pub b: bool,
    // sprite enable
    pub s: bool,
    // R emphasis
    pub Rem: bool,
    // G emphasis
    pub Gem: bool,
    // B emphasis
    pub Bem: bool,
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

impl Mask {
    pub fn reset(&mut self) {
        self.G = false;
        self.m = false;
        self.M = false;
        self.b = false;
        self.s = false;
        self.Rem = false;
        self.Gem = false;
        self.Bem = false;
    }

    pub fn from_byte(byte: Byte) -> Mask {
        Mask {
            G: _check_bit(byte, 0),
            m: _check_bit(byte, 1),
            M: _check_bit(byte, 2),
            b: _check_bit(byte, 3),
            s: _check_bit(byte, 4),
            Rem: _check_bit(byte, 5),
            Gem: _check_bit(byte, 6),
            Bem: _check_bit(byte, 7),
        }
    }

    pub fn to_byte(&self) -> Byte {
        _bool_to_bit(self.G) |
        _bool_to_bit(self.m) << 1 |
        _bool_to_bit(self.M) << 2 |
        _bool_to_bit(self.b) << 3 |
        _bool_to_bit(self.s) << 4 |
        _bool_to_bit(self.Rem) << 5 |
        _bool_to_bit(self.Gem) << 6 |
        _bool_to_bit(self.Bem) << 7
    }

    pub fn from_string(flags: &str) -> Mask {
        let flags = flags.as_bytes();

        Mask {
            G: flags[0] as char == 'G',
            m: flags[1] as char == 'M',
            M: flags[2] as char == 'M',
            b: flags[3] as char == 'B',
            s: flags[4] as char == 'S',
            Rem: flags[5] as char == 'R',
            Gem: flags[6] as char == 'G',
            Bem: flags[7] as char == 'B',
        }
    }
}

impl Display for Mask {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}{}{}{}{}{}{}{}",
               if self.Bem { "B" } else { "b" },
               if self.Gem { "G" } else { "g" },
               if self.Rem { "R" } else { "r" },
               if self.s { "S" } else { "s" },
               if self.b { "B" } else { "b" },
               if self.M { "M" } else { "m" },
               if self.m { "M" } else { "m" },
               if self.G { "G" } else { "g" },
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
        let status = Mask::from_byte(0xFF);

        assert_that!(status.to_string(), eq("BGRSBMMG"))
    }

    #[test]
    fn test_to_byte() {
        let status = Mask::from_byte(0xBB);

        assert_that!(status.to_byte(), eq(0xBB))
    }
}