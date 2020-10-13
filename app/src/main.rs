use std::time::Instant;

use env_logger;
use iced::{Application, Column, Command, Container, Element, executor, Length, Row, Settings, Subscription};
use iced::button::State;
use rustnes_lib::nes::Nes;

use crate::helpers::{button, horizontal_space};
use crate::ram::Ram;
// use log::info;
use crate::side_bar::SideBar;
use crate::video::Video;

mod status_bar;
mod side_bar;
mod helpers;
mod style;
mod cycles_counter;
mod cpu_status;
mod instructions;
mod ram;
mod video;

pub fn main() {
    env_logger::init();

    App::run(iced::Settings {
        default_font: Some(include_bytes!("../assets/cour.ttf")),
        ..Settings::default()
    })
}

struct App {
    nes: Nes,
    reset_button: State,
    next_button: State,
    pause_button: State,
    play_button: State,
    ram: Ram,
    side_bar: SideBar,
    video: Video,
    is_playing: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Pause,
    Play,
    Reset,
    Tick,
    NextInstruction,
    NextFrame(Instant),
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut nes = Nes::default();
        nes.load("../roms/cpu/nestest/nestest.nes");
        // let mut nes = Nes::new("../roms/cpu/instr_test-v5/official_only.nes");
        // let mut nes = Nes::new("../roms/mul3.nes");
        nes.reset();

        println!("ROM banks: {:?}", nes.bus.rom.header);
        println!("ROM length: {:?}", nes.bus.rom.prg_rom.len());

        (
            App {
                nes,
                pause_button: State::default(),
                play_button: State::default(),
                reset_button: State::default(),
                next_button: State::default(),
                ram: Ram::default(),
                side_bar: SideBar::default(),
                video: Video::default(),
                is_playing: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("rustnes")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Pause => self.pause(),
            Message::Play => self.play(),
            Message::Reset => self.reset(),
            Message::Tick => self.tick(),
            Message::NextInstruction => self.process_next(),
            Message::NextFrame(_now) => if self.is_playing {
                self.next_frame()
            },
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let millis = 1000 / 60;
        time::every(std::time::Duration::from_millis(millis)).map(Message::NextFrame)
    }

    fn view(&mut self) -> Element<Message> {
        let App {
            pause_button,
            play_button,
            reset_button,
            next_button,
            nes,
            ram,
            side_bar,
            video,
            ..
        } = self;

        let mut controls = Row::new()
            .push(button(reset_button, "Reset").on_press(Message::Reset))
            .push(horizontal_space())
            .push(button(next_button, "Next").on_press(Message::NextInstruction))
            .push(horizontal_space());

        if self.is_playing {
            controls = controls.push(button(pause_button, "Pause").on_press(Message::Pause));
        } else {
            controls = controls.push(button(play_button, "Play").on_press(Message::Play));
        }

        let main_content = Column::new()
            .spacing(20)
            .push(ram.view(nes))
            .push(controls);

        let content = Row::new()
            .spacing(20)
            .push(main_content)
            .push(side_bar.view(nes))
            .push(video.view(nes));

        Container::new(content)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(style::Theme::Dark)
            .into()
    }
}

impl App {
    fn pause(&mut self) {
        self.is_playing = false;
    }

    fn play(&mut self) {
        self.is_playing = true;
    }

    fn reset(&mut self) {
        self.nes.reset();
    }

    fn next_frame(&mut self) {
        self.nes.tick();

        while !self.nes.is_frame_complete() {
            self.nes.tick();
        }
    }

    fn tick(&mut self) {
        self.nes.tick();
    }

    fn process_next(&mut self) {
        self.nes.process_next();
    }
}

mod time {
    use iced::futures;

    pub fn every(
        duration: std::time::Duration,
    ) -> iced::Subscription<std::time::Instant> {
        iced::Subscription::from_recipe(Every(duration))
    }

    struct Every(std::time::Duration);

    impl<H, I> iced_native::subscription::Recipe<H, I> for Every
        where
            H: std::hash::Hasher,
    {
        type Output = std::time::Instant;

        fn hash(&self, state: &mut H) {
            use std::hash::Hash;

            std::any::TypeId::of::<Self>().hash(state);
            self.0.hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: futures::stream::BoxStream<'static, I>,
        ) -> futures::stream::BoxStream<'static, Self::Output> {
            use futures::stream::StreamExt;

            async_std::stream::interval(self.0)
                .map(|_| std::time::Instant::now())
                .boxed()
        }
    }
}
