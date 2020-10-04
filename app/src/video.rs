// use iced_native::{
//     Background, Color, Element, Hasher, layout, Layout, Length,
//     MouseCursor, Point, Size, Widget,
// };
// use iced_wgpu::{Defaults, Primitive, Renderer};
// use rustnes_lib::nes::Nes;
//
// pub struct Video<'a> {
//     radius: u16,
//     nes: &'a Nes,
// }
//
// impl<'a> Video<'a> {
//     pub fn new(radius: u16, nes: &'static Nes) -> Self {
//         Self { radius, nes }
//     }
// }
//
// impl<Message> Widget<Message, Renderer> for Video<'_> {
//     fn width(&self) -> Length {
//         Length::Shrink
//     }
//
//     fn height(&self) -> Length {
//         Length::Shrink
//     }
//
//     fn layout(
//         &self,
//         _renderer: &Renderer,
//         _limits: &layout::Limits,
//     ) -> layout::Node {
//         layout::Node::new(Size::new(
//             f32::from(self.radius) * 2.0,
//             f32::from(self.radius) * 2.0,
//         ))
//     }
//
//     fn draw(
//         &self,
//         _renderer: &mut Renderer,
//         _defaults: &Defaults,
//         layout: Layout<'_>,
//         _cursor_position: Point,
//     ) -> (Primitive, MouseCursor) {
//         (
//             Primitive::Quad {
//                 bounds: layout.bounds(),
//                 background: Background::Color(Color::BLACK),
//                 border_radius: self.radius,
//                 border_width: 0,
//                 border_color: Color::TRANSPARENT,
//             },
//             MouseCursor::OutOfBounds,
//         )
//     }
//
//     fn hash_layout(&self, state: &mut Hasher) {
//         use std::hash::Hash;
//
//         self.radius.hash(state);
//     }
// }
//
// impl<'a, Message> Into<Element<'a, Message, Renderer>> for Video<'a> {
//     fn into(self) -> Element<'a, Message, Renderer> {
//         Element::new(self)
//     }
// }

use iced::{Container, Image, Length};
use iced::image::Handle;
use rustnes_lib::nes::Nes;

#[derive(Default)]
pub struct Video {}

impl Video {
    // pub fn view2<'a, Message>(&'a mut self, nes: &'a Nes) -> Container<'a, Message> {
    //     let buffer = nes.get_rendered_buffer();
    //
    //     let handle = Handle::from_pixels(nes.width, nes.height, buffer);
    //
    //     Container::new(
    //         Image::new(handle)
    //     )
    //         .height(Length::Fill)
    //         .width(Length::Fill)
    // }
    //
    pub fn view<'a>(&'a mut self, nes: &'a Nes) -> Image {
        let buffer = nes.get_rendered_buffer();

        // let handle = Handle::from_memory(buffer);
        let handle = Handle::from_pixels(nes.width, nes.height, buffer.to_vec());

        Image::new(handle)
    }
}


