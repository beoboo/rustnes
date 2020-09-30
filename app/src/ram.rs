use iced::{Color, Column, Row};
use rustnes_lib::nes::Nes;
use rustnes_lib::types::{Byte, Word};
// use log::trace;
use crate::helpers::{byte_to_string, text, word_to_string};

#[derive(Debug, Clone, Default)]
pub struct Ram {}

impl Ram {
    pub fn view<'a, Message: 'a>(&mut self, nes: &'a Nes) -> Column<'a, Message> {
        Column::new()
            .spacing(10)
            .push(Ram::build_ram_page(0, &nes.bus.ram.data[0..=0xFF]))
        // .push(Ram::build_ram_page(0x80))
    }

    fn build_ram_page<'a, Message: 'a>(page: Word, data: &[Byte]) -> Row<'a, Message> {
        let mut line = String::new();

        for i in 0..16 as Word {
            let start = i as usize * 16;
            let end = (i as usize + 1) * 16;

            line = line + Ram::build_addresses(page * 256 + i * 16, &data[start..end]).as_str() + "\n";
        }

        Row::new()
            .push(text(line.as_str(), Color::WHITE))
    }

    fn build_addresses(address: Word, vals: &[Byte]) -> String {
        let mut bytes = String::from(format!("{}:", word_to_string(address)));

        for i in 0..vals.len() {
            bytes = bytes + " " + byte_to_string(vals[i]).as_str();
        }

        bytes
    }
}


