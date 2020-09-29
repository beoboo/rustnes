mod status_bar;
mod side_bar;
mod helpers;
mod style;
mod cycles_counter;
mod cpu_status;

use iced::{Color, Column, Container, Element, Length, Row, Sandbox, Settings};
use rustnes_lib::nes::Nes;
use env_logger;
// use log::info;
use crate::side_bar::SideBar;
use crate::helpers::{word_to_string, byte_to_string, text, button};
use iced::button::State;

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
    tick_button: State,
    side_bar: SideBar,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        App {
            nes: Nes::new("../roms/cpu/nestest/nestest.nes"),
            tick_button: State::default(),
            side_bar: SideBar::default(),
        }
    }

    fn title(&self) -> String {
        String::from("rustnes")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => self.tick(),
        }
    }

    fn view(&mut self) -> Element<Message> {
        let App {
            tick_button,
            nes,
            side_bar,
            ..
        } = self;

        // info!("[App::view] {}", nes.status);
        let ram = App::build_ram();
        // let side_bar = self.build_sidebar(nes.status);

        let content = Row::new()
            .spacing(20)
            .push(ram)
            .push(side_bar.view(nes));

        let controls = Row::new().push(
            button(tick_button, "Tick")
                .on_press(Message::Tick));

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
    fn tick(&mut self) {
        self.nes.tick();
    }

    // fn build_ram(scroll_state: &'a mut State) -> Scrollable<'a, Message> {
    //     let mut text = String::new();
    //
    //     for i in 0..0xFF00 {
    //         text = text + App::build_bytes(i, vec![0; 16].as_slice()).as_str() + "\n";
    //     }
    //
    //     Scrollable::new(scroll_state)
    //         .push(text(text.as_str(), Color::WHITE))
    // }

    fn build_ram() -> Column<'a, Message> {
        Column::new().spacing(10)
            .push(App::build_ram_page(0))
            .push(App::build_ram_page(0x80))
    }

    fn build_ram_page(page: u16) -> Row<'a, Message> {
        let mut line = String::new();

        for i in 0..16 {
            line = line + App::build_addresses(page * 256 + i * 16, vec![0; 16].as_slice()).as_str() + "\n";
        }

        Row::new()
            .push(text(line.as_str(), Color::WHITE))
    }

    fn build_addresses(address: u16, vals: &[u8]) -> String {
        let mut bytes = String::from(format!("{}:", word_to_string(address)));

        for i in 0..vals.len() {
            bytes = bytes + " " + byte_to_string(vals[i]).as_str();
        }

        bytes
    }
}
