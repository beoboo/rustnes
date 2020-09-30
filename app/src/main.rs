use env_logger;
use iced::{Column, Container, Element, Length, Row, Sandbox, Settings};
use iced::button::State;
use rustnes_lib::nes::Nes;

use crate::helpers::{button, horizontal_space};
// use log::info;
use crate::side_bar::SideBar;
use crate::ram::Ram;

mod status_bar;
mod side_bar;
mod helpers;
mod style;
mod cycles_counter;
mod cpu_status;
mod instructions;
mod ram;

pub fn main() {
    env_logger::init();

    App::run(iced::Settings {
        // default_font: Some(include_bytes!("../assets/CourierNew-Regular.ttf")),
        default_font: Some(include_bytes!("../assets/cour.ttf")),
        ..Settings::default()
    })
}

struct App {
    nes: Nes,
    reset_button: State,
    tick_button: State,
    next_button: State,
    ram: Ram,
    side_bar: SideBar,
}

#[derive(Debug, Clone)]
enum Message {
    Reset,
    Tick,
    ProcessNext,
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        // let mut nes = Nes::new("../roms/cpu/nestest/nestest.nes");
        let mut nes = Nes::new("../roms/mul3.nes");
        nes.reset();

        App {
            nes,
            reset_button: State::default(),
            tick_button: State::default(),
            next_button: State::default(),
            ram: Ram::default(),
            side_bar: SideBar::default(),
        }
    }

    fn title(&self) -> String {
        String::from("rustnes")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Reset => self.reset(),
            Message::Tick => self.tick(),
            Message::ProcessNext => self.process_next(),
        }
    }

    fn view(&mut self) -> Element<Message> {
        let App {
            reset_button,
            tick_button,
            next_button,
            nes,
            ram,
            side_bar,
            ..
        } = self;

        let content = Row::new()
            .spacing(20)
            .push(ram.view(nes))
            .push(side_bar.view(nes));

        let controls = Row::new()
            .push(
                button(reset_button, "Reset")
                    .on_press(Message::Reset))
            .push(horizontal_space())
            .push(
                button(tick_button, "Tick")
                    .on_press(Message::Tick))
            .push(horizontal_space())
            .push(
                button(next_button, "Next")
                    .on_press(Message::ProcessNext));

        let main = Column::new()
            .spacing(20)
            .padding(20)
            .push(content)
            .push(controls);

        Container::new(main)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(style::Theme::Dark)
            .into()
    }
}

impl<'a> App {
    fn reset(&mut self) {
        self.nes.reset();
    }

    fn tick(&mut self) {
        self.nes.tick();
    }

    fn process_next(&mut self) {
        self.nes.process_next();
    }
}
