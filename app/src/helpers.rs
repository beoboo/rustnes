use iced::{button, Button, Color, HorizontalAlignment, Length, Space, Text};

use crate::style;

pub fn byte_to_string(val: u8) -> String {
    format!("${:02X}", val)
}

pub fn word_to_string(val: u16) -> String {
    format!("${:04X}", val)
}

pub fn color_from_flag(flag: bool) -> Color {
    if flag { Color::from_rgb8(0, 255, 0) } else { Color::from_rgb8(255, 0, 0) }
}

pub fn text<S: Into<String>>(text: S, color: Color) -> Text {
    Text::new(text.into()).color(color).size(15)
}

pub fn button<'a, Message: 'a>(state: &'a mut button::State, label: &str) -> Button<'a, Message> {
    Button::new(
        state,
        Text::new(label).horizontal_alignment(HorizontalAlignment::Center),
    ).style(style::Theme::Dark)
}

pub fn vertical_space() -> Space {
    Space::with_width(Length::Units(10))
}

pub fn horizontal_space() -> Space {
    Space::with_width(Length::Units(10))
}
