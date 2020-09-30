use iced::{Color, Column, Row, Text};
use rustnes_lib::cpu::Cpu;

use crate::helpers::{byte_to_string, text, word_to_string};

#[derive(Debug, Clone, Default)]
pub struct CpuStatus {}


fn word_register(register: u16) -> Text {
    text(format!("{} [{}]", word_to_string(register), register).as_str(), Color::WHITE)
}

fn byte_register(register: u8) -> Text {
    text(format!("{} [{}]", byte_to_string(register), register).as_str(), Color::WHITE)
}

impl CpuStatus {
    pub fn view<'a, Message: 'a>(&mut self, cpu: &'a Cpu) -> Row<'a, Message> {
        let left_column = Column::new()
            .spacing(5)
            .push(text("PC: ", Color::WHITE))
            .push(text("A: ", Color::WHITE))
            .push(text("X: ", Color::WHITE))
            .push(text("Y: ", Color::WHITE))
            .push(text("SP: ", Color::WHITE));
        let right_column = Column::new()
            .spacing(5)
            .push(word_register(cpu.PC))
            .push(byte_register(cpu.A))
            .push(byte_register(cpu.X))
            .push(byte_register(cpu.Y))
            .push(byte_register(cpu.SP));

        Row::new()
            .spacing(5)
            .push(left_column)
            .push(right_column)
    }
}


