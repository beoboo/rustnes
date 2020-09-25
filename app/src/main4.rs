use pushrod::layouts::horizontal_layout::HorizontalLayout;
use pushrod::layouts::vertical_layout::VerticalLayout;
use pushrod::render::{make_points, make_points_origin, make_size};
use pushrod::render::callbacks::widget_id_for_name;
use pushrod::render::engine::Engine;
use pushrod::render::layout::Layout;
use pushrod::render::widget::{BaseWidget, Widget};
use pushrod::render::widget_cache::WidgetContainer;
use pushrod::render::widget_config::{
    CONFIG_BORDER_WIDTH, CONFIG_COLOR_BORDER, CONFIG_COLOR_TEXT, PaddingConstraint,
};
use pushrod::widgets::push_button_widget::PushButtonWidget;
use pushrod::widgets::text_widget::{TextJustify, TextWidget};
use sdl2::pixels::Color;
use sdl2::video::Window;
use sdl2::VideoSubsystem;

#[macro_export]
macro_rules! cast {
    ($a:expr, $b:expr, $c:ident) => {
        $a[$b]
            .widget
            .borrow_mut()
            .as_any()
            .downcast_mut::<$c>()
            .unwrap()
    };
}

const MAX_SPACING: i32 = 20;
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const FRAME_RATE: u8 = 60;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = init_window(video_subsystem);

    let mut engine = Engine::new(SCREEN_WIDTH, SCREEN_HEIGHT, FRAME_RATE);

    build_layout(&mut engine);

    engine.run(sdl_context, window);

    Ok(())
}

fn build_layout(engine: &mut Engine) {
    build_status(engine);

    // let mut layout1 = HorizontalLayout::new(20, 20, 360, 80, PaddingConstraint::new(0, 0, 0, 0, 1));
    // let mut layout2 = VerticalLayout::new(250, 120, 130, 160, PaddingConstraint::new(0, 0, 0, 0, 1));
    //
    // let widget1_id = build_widget(engine, "widget1");
    // let widget2_id = build_widget(engine, "widget2");
    // let widget3_id = build_widget(engine, "widget3");
    // let widget4_id = build_widget(engine, "widget4");
    //
    // layout1.append_widget(widget1_id);
    // layout1.append_widget(widget2_id);
    // layout2.append_widget(widget3_id);
    // layout2.append_widget(widget4_id);
    //
    // engine.add_layout(Box::new(layout1));
    // engine.add_layout(Box::new(layout2));
    //
    // let _ = build_text_widget(engine, "text_widget1", "Spacing:", TextJustify::Right, 20, 116, 70, 22, Color::RGB(0, 0, 0));
    // let _ = build_text_widget(engine, "text_widget2", "1", TextJustify::Left, 100, 116, 40, 22, Color::RGB(0, 0, 255));
    // let _ = build_text_widget(engine, "text_widget3", "Top:", TextJustify::Right, 20, 146, 70, 22, Color::RGB(0, 0, 0));
    // let _ = build_text_widget(engine, "text_widget4", "0", TextJustify::Left, 100, 146, 40, 22, Color::RGB(0, 0, 255));
    // let _ = build_text_widget(engine, "text_widget5", "Bottom:", TextJustify::Right, 20, 176, 70, 22, Color::RGB(0, 0, 0));
    // let _ = build_text_widget(engine, "text_widget6", "0", TextJustify::Left, 100, 176, 40, 22, Color::RGB(0, 0, 255));
    // let _ = build_text_widget(engine, "text_widget7", "Left:", TextJustify::Right, 20, 206, 70, 22, Color::RGB(0, 0, 0));
    // let _ = build_text_widget(engine, "text_widget8", "0", TextJustify::Left, 100, 206, 40, 22, Color::RGB(0, 0, 255));
    // let _ = build_text_widget(engine, "text_widget9", "Right:", TextJustify::Right, 20, 236, 70, 22, Color::RGB(0, 0, 0));
    // let _ = build_text_widget(engine, "text_widget10", "0", TextJustify::Left, 100, 236, 40, 22, Color::RGB(0, 0, 255));
}

fn build_status(engine: &mut Engine) {
    let x = SCREEN_WIDTH as i32 * 70 / 100;
    let y = SCREEN_HEIGHT as i32 * 10 / 100;
    let height = 22;

    let mut layout = HorizontalLayout::new(x, y, 70, height, PaddingConstraint::new(0, 0, 0, 0, 1));
    let mut flags = HorizontalLayout::new(x + 90, y, 120, height, PaddingConstraint::new(0, 0, 0, 0, 1));

    let title_id = build_text_widget(engine, "status_txt", "Status:", TextJustify::Right, x, y, 80, height, Color::RGB(0, 0, 0));
    layout.append_widget(title_id);
    engine.add_layout(Box::new(layout));

    let flag_c = build_text_widget(engine, "status_c", "C", TextJustify::Center, x + 0, y, 0, height, Color::RGB(255, 0, 0));
    let flag_z = build_text_widget(engine, "status_z", "Z", TextJustify::Center, x + 80, y, 0, height, Color::RGB(255, 0, 0));
    let flag_i = build_text_widget(engine, "status_z", "I", TextJustify::Center, x + 90, y, 0, height, Color::RGB(255, 0, 0));
    let flag_d = build_text_widget(engine, "status_d", "-", TextJustify::Center, x + 100, y, 0, height, Color::RGB(255, 0, 0));
    let flag_b = build_text_widget(engine, "status_b", "B", TextJustify::Center, x + 110, y, 0, height, Color::RGB(255, 0, 0));
    let flag_u = build_text_widget(engine, "status_u", "U", TextJustify::Center, x + 120, y, 0, height, Color::RGB(255, 0, 0));
    let flag_v = build_text_widget(engine, "status_v", "V", TextJustify::Center, x + 130, y, 0, height, Color::RGB(255, 0, 0));
    let flag_n = build_text_widget(engine, "status_n", "N", TextJustify::Center, x + 140, y, 0, height, Color::RGB(255, 0, 0));

    flags.append_widget(flag_c);
    flags.append_widget(flag_z);
    flags.append_widget(flag_i);
    flags.append_widget(flag_d);
    flags.append_widget(flag_b);
    flags.append_widget(flag_u);
    flags.append_widget(flag_v);
    flags.append_widget(flag_n);

    engine.add_layout(Box::new(flags));
}

fn build_text_widget(engine: &mut Engine, widget_id: &str, text: &str, justify: TextJustify, x: i32, y: i32, width: u32, height: u32, color: Color) -> i32 {
    let mut text_widget = TextWidget::new(
        String::from("assets/CourierNew-Regular.ttf"),
        sdl2::ttf::FontStyle::NORMAL,
        16,
        justify,
        String::from(text),
        make_points(0, 0),
        make_size(0, 0),
    );

    text_widget
        .get_config()
        .set_color(CONFIG_COLOR_TEXT, color);

    text_widget
        .get_config()
        .set_color(CONFIG_COLOR_BORDER, Color::RGB(0, 0, 0));

    text_widget
        .get_config()
        .set_numeric(CONFIG_BORDER_WIDTH, 2);

    engine.add_widget(Box::new(text_widget), String::from(widget_id))
}

fn build_widget(engine: &mut Engine, title: &str) -> i32 {
    let mut widget = BaseWidget::new(make_points_origin(), make_size(0, 0));

    widget
        .get_config()
        .set_color(CONFIG_COLOR_BORDER, Color::RGB(0, 0, 0));
    widget.get_config().set_numeric(CONFIG_BORDER_WIDTH, 2);

    engine.add_widget(Box::new(widget), String::from(title))
}

fn init_window(video_subsystem: VideoSubsystem) -> Window {
    video_subsystem
        .window("rustnes", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap()
}
//
// fn refresh_widgets(_widgets: &[WidgetContainer]) {
//     _widgets[0]
//         .widget
//         .borrow_mut()
//         .get_config()
//         .set_invalidated(true);
//     _widgets[5]
//         .widget
//         .borrow_mut()
//         .get_config()
//         .set_invalidated(true);
//     _widgets[6]
//         .widget
//         .borrow_mut()
//         .get_config()
//         .set_invalidated(true);
//     _widgets[9]
//         .widget
//         .borrow_mut()
//         .get_config()
//         .set_invalidated(true);
//     _widgets[10]
//         .widget
//         .borrow_mut()
//         .get_config()
//         .set_invalidated(true);
//     _widgets[13]
//         .widget
//         .borrow_mut()
//         .get_config()
//         .set_invalidated(true);
//     _widgets[14]
//         .widget
//         .borrow_mut()
//         .get_config()
//         .set_invalidated(true);
//     _widgets[17]
//         .widget
//         .borrow_mut()
//         .get_config()
//         .set_invalidated(true);
//     _widgets[18]
//         .widget
//         .borrow_mut()
//         .get_config()
//         .set_invalidated(true);
//     _widgets[21]
//         .widget
//         .borrow_mut()
//         .get_config()
//         .set_invalidated(true);
//     _widgets[22]
//         .widget
//         .borrow_mut()
//         .get_config()
//         .set_invalidated(true);
// }