use iced::Column;
use rustnes_lib::nes::Nes;

use crate::cpu_status::CpuStatus;
use crate::cycles_counter::CyclesCounter;
use crate::helpers::vertical_space;
use crate::instructions::Instructions;
use crate::status_bar::StatusBar;

#[derive(Debug, Clone, Default)]
pub struct SideBar {
    cycles_counter: CyclesCounter,
    status_bar: StatusBar,
    cpu_status: CpuStatus,
    instructions: Instructions,
}

impl SideBar {
    pub fn view<'a, Message: 'a>(&mut self, nes: &'a Nes) -> Column<'a, Message> {
        Column::new()
            .spacing(5)
            .push(self.cycles_counter.view(nes.cycles))
            .push(vertical_space())
            .push(self.status_bar.view(&nes.cpu.status))
            .push(vertical_space())
            .push(self.cpu_status.view(&nes.cpu))
            .push(vertical_space())
            .push(self.instructions.view(nes.cpu.PC, &nes.bus.rom.prg_rom))
    }
}


