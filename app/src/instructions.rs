use rustnes_lib::cpu::status::Status;
use iced::{Column, Color};
use log::info;
use crate::helpers::{color_from_flag, text, vertical_space};

#[derive(Debug, Clone, Default)]
pub struct Instructions {
    code: Vec<Instruction>
}

impl Instructions {
    pub fn view<'a, Message: 'a>(&mut self, status: &'a Status) -> Column<'a, Message> {
        info!("[Instructions::view] {}", status);
        let status_bar = text("Status: ", Color::WHITE);
        let c_flag = text("C", color_from_flag(status.C));
        let z_flag = text("Z", color_from_flag(status.Z));
        let i_flag = text("I", color_from_flag(status.I));
        let d_flag = text("-", Color::from_rgb8(0, 255, 0));
        let b_flag = text("B", color_from_flag(status.B));
        let u_flag = text("U", color_from_flag(status.U));
        let v_flag = text("V", color_from_flag(status.V));
        let n_flag = text("N", color_from_flag(status.N));

        Column::new()
            .spacing(5)
            .push(status_bar)
            .push(vertical_space())
            .push(c_flag)
            .push(z_flag)
            .push(i_flag)
            .push(d_flag)
            .push(b_flag)
            .push(u_flag)
            .push(v_flag)
            .push(n_flag)
    }
}


