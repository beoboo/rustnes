use iced::{Color, Row, Text};
use log::info;
use rustnes_lib::cpu::status::Status;

use crate::helpers::{color_from_flag, text, vertical_space};

#[derive(Debug, Clone, Default)]
pub struct StatusBar {}

fn bool_to_string(flag: bool, ch: &str) -> String {
    if flag {ch.to_ascii_uppercase() } else { ch.to_ascii_lowercase() }
}

fn status_flag(flag: bool, ch: &str) -> Text {
    text(bool_to_string(flag, ch).as_str(), color_from_flag(flag))
}

impl StatusBar {
    pub fn view<'a, Message: 'a>(&mut self, status: &'a Status) -> Row<'a, Message> {
        let status_bar = text("Status: ", Color::WHITE);

        Row::new()
            .spacing(5)
            .push(status_bar)
            .push(vertical_space())
            .push(status_flag(status.C, "C"))
            .push(status_flag(status.Z, "Z"))
            .push(status_flag(status.I, "I"))
            .push(status_flag(status.D, "D"))
            .push(status_flag(status.B, "B"))
            .push(status_flag(status.U, "U"))
            .push(status_flag(status.V, "V"))
            .push(status_flag(status.N, "N"))
    }
}


