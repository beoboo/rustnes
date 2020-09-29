use env_logger;
use iced::{Color, Column, Container, Element, Length, Row, Sandbox, Settings};
use iced::button::State;
use rustnes_lib::nes::Nes;

use crate::helpers::{button, byte_to_string, text, word_to_string, horizontal_space};
// use log::info;
use crate::side_bar::SideBar;

mod status_bar;
mod side_bar;
mod helpers;
mod style;
mod cycles_counter;
mod cpu_status;
mod instructions;

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
        App {
            nes: Nes::new("../roms/cpu/nestest/nestest.nes"),
            reset_button: State::default(),
            tick_button: State::default(),
            next_button: State::default(),
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
