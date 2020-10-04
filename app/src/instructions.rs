use iced::{Color, Column, Text};
// use log::{info, trace};
use log::trace;
use rustnes_lib::disassembler::Disassembler;
use rustnes_lib::disassembler::line::Line;
use rustnes_lib::types::Word;
use crate::helpers::{text, vertical_space};
use rustnes_lib::instructions::addressing_mode::AddressingMode;
use rustnes_lib::nes::Nes;

#[derive(Debug, Clone, Default)]
pub struct Instructions {
    disassembler: Disassembler
}

fn relative_address(pos: Word, relative: Word) -> String {
    let pos = if relative > 0x80 {
        pos + relative - 0xFF
    } else {
        pos + relative
    };

    format!(" [{:#06X}]", pos + 1)
}

fn instruction_to_string(pos: Word, line: &Line) -> String {
    let address = if line.addressing_mode == AddressingMode::Relative { relative_address(pos, line.address) } else { String::from("") };

    format!("{:#06X} {}{}", pos, line.instruction, address)
}

fn instruction_text(pos: Word, line: &Line, color: Color) -> Text {
    text(instruction_to_string(pos, line).as_str(), color)
}

impl Instructions {
    pub fn view<'a, Message: 'a>(&mut self, pc: Word, nes: &Nes) -> Column<'a, Message> {
        trace!("[Instructions::view] {:#06X}", pc);
        // let pc = pc - 0x8000;
        // let pc = pc - 0xC000;
        trace!("[Instructions::view] {:#06X}", pc);
        let instructions = &self.disassembler.disassemble(&nes.bus.rom.prg_rom[0..=0xFFFF]);

        let mut column = Column::new()
            .push(text("Instructions", Color::WHITE))
            .push(vertical_space());

        let previous = instructions.iter().filter(|(k, _)| *k < &pc).rev().take(10).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>();
        let current = instructions.get(&pc).unwrap();
        let next =     instructions.iter().filter(|(k, _)| *k > &pc).take(10).collect::<Vec<_>>();

        for _ in 0..(10 - previous.len()) {
            column = column.push(text("---", Color::from_rgb8(120, 120, 120)));
        }
        for (k, v) in previous {
            column = column.push(instruction_text(*k, v, Color::from_rgb8(120, 120, 120)));
        }

        column = column.push(instruction_text(pc, &current, Color::WHITE));

        for (k, v) in &next {
            column = column.push(instruction_text(**k, v, Color::from_rgb8(120, 120, 120)));
        }
        for _ in 0..(10 - next.len()) {
            column = column.push(text("---", Color::from_rgb8(120, 120, 120)));
        }

        column
    }
}


