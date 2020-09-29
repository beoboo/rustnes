use iced::{Row, Color, Column, Text};
use crate::helpers::{text, word_to_string, byte_to_string};
use rustnes_lib::cpu::Cpu;

#[derive(Debug, Clone, Default)]
pub struct CpuStatus {
}


fn word_register(register: u16) -> Text {
    text(format!("{} [{}]", word_to_string(register), register).as_str(), Color::WHITE)
}

fn byte_register(register: u8) -> Text {
    text(format!("{} [{}]", byte_to_string(register), register).as_str(), Color::WHITE)
}

impl CpuStatus {
    pub fn view<'a, Message: 'a>(&mut self, cpu: &'a Cpu) -> Row<'a, Message> {
        let pc_lbl = text("PC: ", Color::WHITE);
        let pc_txt = word_register(cpu.PC);
        let a_lbl = text("A: ", Color::WHITE);
        let a_txt = byte_register(cpu.A);
        let x_lbl = text("X: ", Color::WHITE);
        let x_txt = byte_register(cpu.X);
        let y_lbl = text("Y: ", Color::WHITE);
        let y_txt = byte_register(cpu.Y);
        let sp_lbl = text("SP: ", Color::WHITE);
        let sp_txt = byte_register(cpu.SP);

        let left_column = Column::new()
            .spacing(5)
            .push(pc_lbl)
            .push(a_lbl)
            .push(x_lbl)
            .push(y_lbl)
            .push(sp_lbl);
        let right_column = Column::new()
            .spacing(5)
            .push(pc_txt)
            .push(a_txt)
            .push(x_txt)
            .push(y_txt)
            .push(sp_txt);

        Row::new()
            .spacing(5)
            .push(left_column)
            .push(right_column)
    }
}


