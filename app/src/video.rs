use iced::Image;
use iced::image::Handle;
use rustnes_lib::nes::Nes;

#[derive(Default)]
pub struct Video {}

impl Video {
    pub fn view<'a>(&'a mut self, nes: &'a Nes) -> Image {
        let buffer = nes.get_rendered_buffer();

        let handle = Handle::from_pixels(nes.width, nes.height, buffer.to_vec());

        Image::new(handle)
    }
}


