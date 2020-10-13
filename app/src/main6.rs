use fltk::*;
use fltk::{app::*, frame::*, group::*, image, window::*};
use rustnes_lib::nes::Nes;

fn main() {
    let app = App::default().with_scheme(Scheme::Gtk);

    let mut nes = Nes::default();
    nes.load("../roms/cpu/nestest/nestest.nes");
    // let mut nes = Nes::new("../roms/cpu/instr_test-v5/official_only.nes");
    // let mut nes = Nes::new("../roms/mul3.nes");
    nes.reset();

    println!("ROM banks: {:?}", nes.bus.rom.header);
    println!("ROM length: {:?}", nes.bus.rom.prg_rom.len());

    let screen_width = 720;
    let screen_height = 486;
    let mut wind = DoubleWindow::default()
        .with_label("rustnes")
        .with_size(screen_width, screen_height)
        .center_screen();

    let mut h_pack = group::Pack::new(0, 0, screen_width, screen_height, "");
    h_pack.set_spacing(10);
    h_pack.set_align(Align::Center);
    // h_pack.set_border(1);
    let mut v_pack = group::Pack::new(0, 0,  nes.width as i32, nes.height as i32, "");
    let mut frame = Frame::new(0, 0, nes.width as i32, nes.height as i32, "");
    v_pack.end();

    let mut v_pack = group::Pack::new(0, 0, 200, 350, "");
    v_pack.set_frame(FrameType::BorderFrame);
    Frame::new(0, 0, 50, 50, "Status: ").set_align(Align::Left).set_align();
    Frame::new(0, 0, 50, 50, "PC: ");
    Frame::new(0, 0, 50, 50, "A: ");
    Frame::new(0, 0, 50, 50, "X: ");
    Frame::new(0, 0, 50, 50, "Y: ");
    v_pack.end();

    // Frame::new(0, 0, 50, 50, nes.cpu.status.to_string().as_str()).set_align(Align::Right);
    // Frame::new(0, 0, 50, 50, nes.cpu.A.to_string().as_str()).set_align(Align::Right);
    // Frame::new(0, 0, 50, 50, "A: ").set_align(Align::Right);
    // Frame::new(0, 0, 50, 50, "X: ").set_align(Align::Right);
    // Frame::new(0, 0, 50, 50, "Y: ").set_align(Align::Right);

    h_pack.end();
    h_pack.set_type(PackType::Horizontal);


    wind.set_color(Color::White);
    wind.end();
    wind.show();

    while app.wait().unwrap() {
        nes.tick();

        while !nes.is_frame_complete() {
            nes.tick();
        }

        let buffer = nes.get_rendered_buffer();
        let image = image::RgbImage::new(&buffer, nes.width, nes.height, 4).unwrap();

        frame.set_image(Some(image));
        frame.redraw();
        wind.redraw();

        // std::thread::sleep(std::time::Duration::from_millis(1));
        // let millis = 1/60;
    }
}
