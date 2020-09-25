use iced::{Align, button, Button, Checkbox, Color, Column, Container, Element, Length, ProgressBar, Radio, Row, Sandbox, scrollable, Scrollable, Settings, slider, Slider, Space, Text, text_input, TextInput};
use sdl2::ttf::Font;

pub fn main() {
    App::run(Settings {
        antialiasing: true,
        default_font: Some(include_bytes!("../assets/CourierNew-Regular.ttf")),
        ..Settings::default()
    })
}

#[derive(Default)]
struct App {
    scroll: scrollable::State,
    input: text_input::State,
    input_value: String,
    button: button::State,
    slider: slider::State,
    slider_value: f32,
    toggle_value: bool,
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    ButtonPressed,
    SliderChanged(f32),
    CheckboxToggled(bool),
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        App::default()
    }

    fn title(&self) -> String {
        String::from("App - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::InputChanged(value) => self.input_value = value,
            Message::ButtonPressed => (),
            Message::SliderChanged(value) => self.slider_value = value,
            Message::CheckboxToggled(value) => self.toggle_value = value,
        }
    }

    fn view(&mut self) -> Element<Message> {
        // let text_input = TextInput::new(
        //     &mut self.input,
        //     "Type something...",
        //     &self.input_value,
        //     Message::InputChanged,
        // )
        //     .padding(10)
        //     .size(20)
        //     .style(style::Theme::Dark);
        //
        // let button = Button::new(&mut self.button, Text::new("Submit"))
        //     .padding(10)
        //     .on_press(Message::ButtonPressed)
        //     .style(style::Theme::Dark);
        //
        // let slider = Slider::new(
        //     &mut self.slider,
        //     0.0..=100.0,
        //     self.slider_value,
        //     Message::SliderChanged,
        // )
        //     .style(style::Theme::Dark);
        //
        // let progress_bar =
        //     ProgressBar::new(0.0..=100.0, self.slider_value).style(style::Theme::Dark);

        let sidebar = App::build_sidebar();

        // let scrollable = Scrollable::new(&mut self.scroll)
        //     .width(Length::Fill)
        //     .height(Length::Units(100))
        //     .style(style::Theme::Dark)
        //     .push(Text::new("Scroll me!"))
        //     .push(Space::with_height(Length::Units(800)))
        //     .push(Text::new("You did it!"));
        //
        // let checkbox = Checkbox::new(
        //     self.toggle_value,
        //     "Toggle me!",
        //     Message::CheckboxToggled,
        // )
        //     .width(Length::Fill)
        //     .style(style::Theme::Dark);

        let content = Column::new()
            .spacing(20)
            .padding(20)
            .max_width(600)
            .push(sidebar);
        // .push(Row::new().spacing(10).push(text_input).push(button))
        // .push(slider)
        // .push(progress_bar)
        // .push(
        //     Row::new()
        //         .spacing(10)
        //         .align_items(Align::Center)
        //         .push(scrollable)
        //         .push(checkbox),
        // );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(style::Theme::Dark)
            .into()
    }
}


mod style {
    use iced::{
        button, checkbox, container, progress_bar, radio, scrollable, slider,
        text_input,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Theme {
        Light,
        Dark,
    }

    impl Theme {
        pub const ALL: [Theme; 2] = [Theme::Light, Theme::Dark];
    }

    impl Default for Theme {
        fn default() -> Theme {
            Theme::Light
        }
    }

    impl From<Theme> for Box<dyn container::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Container.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn radio::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Radio.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn text_input::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::TextInput.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn button::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Button.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn scrollable::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Scrollable.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn slider::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Slider.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn progress_bar::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::ProgressBar.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn checkbox::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Checkbox.into(),
            }
        }
    }

    mod dark {
        use iced::{
            Background, button, checkbox, Color, container, progress_bar,
            radio, scrollable, slider, text_input,
        };

        const SURFACE: Color = Color::from_rgb(
            0x40 as f32 / 255.0,
            0x44 as f32 / 255.0,
            0x4B as f32 / 255.0,
        );

        const ACCENT: Color = Color::from_rgb(
            0x6F as f32 / 255.0,
            0xFF as f32 / 255.0,
            0xE9 as f32 / 255.0,
        );

        const ACTIVE: Color = Color::from_rgb(
            0x72 as f32 / 255.0,
            0x89 as f32 / 255.0,
            0xDA as f32 / 255.0,
        );

        const HOVERED: Color = Color::from_rgb(
            0x67 as f32 / 255.0,
            0x7B as f32 / 255.0,
            0xC4 as f32 / 255.0,
        );

        pub struct Container;

        impl container::StyleSheet for Container {
            fn style(&self) -> container::Style {
                container::Style {
                    background: Some(Background::Color(Color::from_rgb8(
                        0x36, 0x39, 0x3F,
                    ))),
                    text_color: Some(Color::WHITE),
                    ..container::Style::default()
                }
            }
        }

        pub struct Radio;

        impl radio::StyleSheet for Radio {
            fn active(&self) -> radio::Style {
                radio::Style {
                    background: Background::Color(SURFACE),
                    dot_color: ACTIVE,
                    border_width: 1,
                    border_color: ACTIVE,
                }
            }

            fn hovered(&self) -> radio::Style {
                radio::Style {
                    background: Background::Color(Color { a: 0.5, ..SURFACE }),
                    ..self.active()
                }
            }
        }

        pub struct TextInput;

        impl text_input::StyleSheet for TextInput {
            fn active(&self) -> text_input::Style {
                text_input::Style {
                    background: Background::Color(SURFACE),
                    border_radius: 2,
                    border_width: 0,
                    border_color: Color::TRANSPARENT,
                }
            }

            fn focused(&self) -> text_input::Style {
                text_input::Style {
                    border_width: 1,
                    border_color: ACCENT,
                    ..self.active()
                }
            }

            fn hovered(&self) -> text_input::Style {
                text_input::Style {
                    border_width: 1,
                    border_color: Color { a: 0.3, ..ACCENT },
                    ..self.focused()
                }
            }

            fn placeholder_color(&self) -> Color {
                Color::from_rgb(0.4, 0.4, 0.4)
            }

            fn value_color(&self) -> Color {
                Color::WHITE
            }

            fn selection_color(&self) -> Color {
                ACTIVE
            }
        }

        pub struct Button;

        impl button::StyleSheet for Button {
            fn active(&self) -> button::Style {
                button::Style {
                    background: Some(Background::Color(ACTIVE)),
                    border_radius: 3,
                    text_color: Color::WHITE,
                    ..button::Style::default()
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    background: Some(Background::Color(HOVERED)),
                    text_color: Color::WHITE,
                    ..self.active()
                }
            }

            fn pressed(&self) -> button::Style {
                button::Style {
                    border_width: 1,
                    border_color: Color::WHITE,
                    ..self.hovered()
                }
            }
        }

        pub struct Scrollable;

        impl scrollable::StyleSheet for Scrollable {
            fn active(&self) -> scrollable::Scrollbar {
                scrollable::Scrollbar {
                    background: Some(Background::Color(SURFACE)),
                    border_radius: 2,
                    border_width: 0,
                    border_color: Color::TRANSPARENT,
                    scroller: scrollable::Scroller {
                        color: ACTIVE,
                        border_radius: 2,
                        border_width: 0,
                        border_color: Color::TRANSPARENT,
                    },
                }
            }

            fn hovered(&self) -> scrollable::Scrollbar {
                let active = self.active();

                scrollable::Scrollbar {
                    background: Some(Background::Color(Color {
                        a: 0.5,
                        ..SURFACE
                    })),
                    scroller: scrollable::Scroller {
                        color: HOVERED,
                        ..active.scroller
                    },
                    ..active
                }
            }

            fn dragging(&self) -> scrollable::Scrollbar {
                let hovered = self.hovered();

                scrollable::Scrollbar {
                    scroller: scrollable::Scroller {
                        color: Color::from_rgb(0.85, 0.85, 0.85),
                        ..hovered.scroller
                    },
                    ..hovered
                }
            }
        }

        pub struct Slider;

        impl slider::StyleSheet for Slider {
            fn active(&self) -> slider::Style {
                slider::Style {
                    rail_colors: (ACTIVE, Color { a: 0.1, ..ACTIVE }),
                    handle: slider::Handle {
                        shape: slider::HandleShape::Circle { radius: 9 },
                        color: ACTIVE,
                        border_width: 0,
                        border_color: Color::TRANSPARENT,
                    },
                }
            }

            fn hovered(&self) -> slider::Style {
                let active = self.active();

                slider::Style {
                    handle: slider::Handle {
                        color: HOVERED,
                        ..active.handle
                    },
                    ..active
                }
            }

            fn dragging(&self) -> slider::Style {
                let active = self.active();

                slider::Style {
                    handle: slider::Handle {
                        color: Color::from_rgb(0.85, 0.85, 0.85),
                        ..active.handle
                    },
                    ..active
                }
            }
        }

        pub struct ProgressBar;

        impl progress_bar::StyleSheet for ProgressBar {
            fn style(&self) -> progress_bar::Style {
                progress_bar::Style {
                    background: Background::Color(SURFACE),
                    bar: Background::Color(ACTIVE),
                    border_radius: 10,
                }
            }
        }

        pub struct Checkbox;

        impl checkbox::StyleSheet for Checkbox {
            fn active(&self, is_checked: bool) -> checkbox::Style {
                checkbox::Style {
                    background: Background::Color(if is_checked {
                        ACTIVE
                    } else {
                        SURFACE
                    }),
                    checkmark_color: Color::WHITE,
                    border_radius: 2,
                    border_width: 1,
                    border_color: ACTIVE,
                }
            }

            fn hovered(&self, is_checked: bool) -> checkbox::Style {
                checkbox::Style {
                    background: Background::Color(Color {
                        a: 0.8,
                        ..if is_checked { ACTIVE } else { SURFACE }
                    }),
                    ..self.active(is_checked)
                }
            }
        }
    }
}

impl<'a> App {
    fn build_ram() -> Column<'a, Message> {
        let status_row = App::build_status();
        let registers_row = App::build_registers();

        Column::new()
            .spacing(5)
            .push(status_row)
            .push(registers_row)
    }

    fn build_sidebar() -> Column<'a, Message> {
        let status_row = App::build_status();
        let registers_row = App::build_registers();

        Column::new()
            .spacing(5)
            .push(status_row)
            .push(registers_row)
    }


    fn build_status() -> Row<'a, Message> {
        let status = Text::new("Status: ");
        let c_flag = Text::new("C").color(Color::from_rgb8(255, 0, 0));
        let z_flag = Text::new("Z").color(Color::from_rgb8(255, 0, 0));
        let i_flag = Text::new("I").color(Color::from_rgb8(255, 0, 0));
        let d_flag = Text::new("-").color(Color::from_rgb8(255, 0, 0));
        let b_flag = Text::new("B").color(Color::from_rgb8(255, 0, 0));
        let u_flag = Text::new("U").color(Color::from_rgb8(255, 0, 0));
        let v_flag = Text::new("V").color(Color::from_rgb8(255, 0, 0));
        let n_flag = Text::new("N").color(Color::from_rgb8(255, 0, 0));

        Row::new()
            .spacing(5)
            .push(status)
            .push(Space::with_width(Length::Units(10)))
            .push(c_flag)
            .push(z_flag)
            .push(i_flag)
            .push(d_flag)
            .push(b_flag)
            .push(u_flag)
            .push(v_flag)
            .push(n_flag)
    }

    fn build_registers() -> Row<'a, Message> {
        let pc_lbl = Text::new("PC: ");
        let pc_txt = Text::new("$0000 [0]");
        let a_lbl = Text::new("A: ");
        let a_txt = Text::new("$00 [0]");
        let x_lbl = Text::new("X: ");
        let x_txt = Text::new("$00 [0]");
        let y_lbl = Text::new("Y: ");
        let y_txt = Text::new("$00 [0]");
        let sp_lbl = Text::new("SP: ");
        let sp_txt = Text::new("$00 [0]");

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
