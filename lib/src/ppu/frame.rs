use crate::types::{Byte, Word};
use crate::ppu::color::Color;

#[derive(Debug)]
pub struct Frame {
    pub data: Vec<Byte>,
    width: Word,
    height: Word,
    bytes_per_pixel: Word,
}

impl Frame {
    pub fn new(width: Word, height: Word, bytes_per_pixel: Word) -> Self {
        Self {
            data: vec![0; width as usize * height as usize * bytes_per_pixel as usize],
            width,
            height,
            bytes_per_pixel,
        }
    }

    pub fn at(&self, x: Word, y: Word) -> Option<Color> {
        let pos = self.pos(x, y);

        match pos {
            Some(pos) => Some(Color::from_rgba(self.data[pos], self.data[pos + 1], self.data[pos + 2], self.data[pos + 3])),
            None => None,
        }
    }

    pub fn draw(&mut self, x: Word, y: Word, color: Color) {
        let pos = self.pos(x, y);

        match pos {
            Some(pos) => {
                self.data[pos] = color.r;
                self.data[pos + 1] = color.g;
                self.data[pos + 2] = color.b;
                self.data[pos + 3] = color.a;
            },
            None => {}
        }

    }

    fn pos(&self, x: Word, y: Word) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None
        }

        let x = x as usize;
        let y = y as usize;
        let width = self.width as usize;
        let bytes_per_pixel = self.bytes_per_pixel as usize;

        let pos = (x + (y * width)) * bytes_per_pixel;
        Some(pos)
    }
}


#[cfg(test)]
mod tests {
    use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn test_draw() {
        let mut frame = Frame::new(100, 100, 4);
        frame.draw(0, 0, Color::from_rgba(0x12, 0x34, 0x56, 0x78));

        assert_that!(frame.data[0], eq(0x12));
        assert_that!(frame.data[1], eq(0x34));
        assert_that!(frame.data[2], eq(0x56));
        assert_that!(frame.data[3], eq(0x78));
    }

    #[test]
    fn test_bounds() {
        let mut frame = Frame::new(100, 100, 4);
        frame.draw(-1i16 as Word, -1i16 as Word, Color::from_rgba(0x12, 0x34, 0x56, 0x78));
        frame.draw(100, 1000, Color::from_rgba(0x12, 0x34, 0x56, 0x78));
    }
}