use crate::types::Byte;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: Byte,
    pub g: Byte,
    pub b: Byte,
    pub a: Byte,
}

impl Color {
    pub fn new(r: Byte, g: Byte, b: Byte) -> Self {
        Self {
            r,
            g,
            b,
            a: 0xFF
        }
    }

    pub fn from_rgba(r: Byte, g: Byte, b: Byte, a: Byte) -> Self {
        Self {
            r,
            g,
            b,
            a
        }
    }
}