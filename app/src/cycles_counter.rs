use iced::{Color, Row};

use crate::helpers::text;

#[derive(Debug, Clone, Default)]
pub struct CyclesCounter {}

impl CyclesCounter {
    pub fn view<'a, Message: 'a>(&mut self, cycles: usize) -> Row<'a, Message> {
        let cycles_lbl = text("Cycles: ", Color::WHITE);
        let cycles_txt = text(&cycles.to_string(), Color::WHITE);

        Row::new()
            .spacing(5)
            .push(cycles_lbl)
            .push(cycles_txt)
    }
}


