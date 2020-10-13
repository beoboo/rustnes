use iced::{Color, Column, Text};
// use log::{info, trace};
// use log::trace;
use rustnes_lib::bus::Bus;
use rustnes_lib::disassembler::assembly::Assembly;
use rustnes_lib::disassembler::assembly_line::AssemblyLine;
use rustnes_lib::disassembler::Disassembler;
use rustnes_lib::nes::Nes;
use rustnes_lib::types::Word;

use crate::helpers::{text, vertical_space};

#[derive(Debug, Clone, Default)]
pub struct Instructions {
    assembly: Option<Assembly>
}

fn instruction_text(line: &AssemblyLine, color: Color) -> Text {
    text(line.to_string(), color)
}

impl Instructions {
    pub fn view<'a, Message: 'a>(&mut self, pc: Word, nes: &Nes) -> Column<'a, Message> {
        // trace!("[Instructions::view] {:#06X}", pc);
        if self.assembly.is_none() {
            let disassembler = Disassembler::default();
            self.assembly = Some(disassembler.disassemble(&nes.bus.read(0x8000, 0xFFFF - 0x0006), 0x8000));
        }

        let assembly = self.assembly.as_ref().unwrap();

        let mut column = Column::new()
            .push(text("Instructions", Color::WHITE))
            .push(vertical_space());

        let range = 10;

        let pc = if pc > 0x7FFF && nes.bus.rom.header.prg_rom_size == 1 {
            pc - 0x4000
        } else {
            pc
        };

        let previous = assembly.before(pc, range);
        let current = assembly.at(pc);
        let next = assembly.after(pc, range);

        for _ in 0..(range - previous.len()) {
            column = column.push(text("---", Color::from_rgb8(120, 120, 120)));
        }
        for i in previous {
            column = column.push(instruction_text(i, Color::from_rgb8(120, 120, 120)));
        }

        column = column.push(instruction_text(current, Color::WHITE));

        for v in &next {
            column = column.push(instruction_text(v, Color::from_rgb8(120, 120, 120)));
        }
        for _ in 0..(range - next.len()) {
            column = column.push(text("---", Color::from_rgb8(120, 120, 120)));
        }

        column
    }
}


