use rustnes::nes::Nes;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;
use std::time::Duration;

struct App {
    sdl: Sdl,
    canvas: WindowCanvas,
    nes: Nes,
}

impl App {
    fn new(nes: Nes) -> App {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        let window = video.window("rustnes", nes.width as u32, nes.height as u32)
            .position_centered()
            .build()
            .expect("Could not initialize video");

        let canvas = window.into_canvas().build()
            .expect("Could not make a canvas");

        App {
            sdl,
            canvas,
            nes,
        }
    }

    fn update(&mut self) {}

    fn render(&mut self) {
        let buffer = self.nes.get_rendered_buffer();

        for i in 0..self.nes.height {
            for j in 0..self.nes.width {
                let pos = ((i * self.nes.width + j) * self.nes.bits_per_pixel) as usize;

                let r = buffer[pos];
                let g = buffer[pos + 1];
                let b = buffer[pos + 2];
                self.canvas.set_draw_color(Color::RGB(r, g, b));
                self.canvas.draw_point(Point::new(j as i32, i as i32)).unwrap();
            }
        }

        self.canvas.present();
    }

    fn run(&mut self) -> Result<(), String> {
        let mut event_pump = self.sdl.event_pump()?;

        'running: loop {
            // Handle events
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running;
                    }
                    _ => {}
                }
            }

            // Update
            self.update();

            // Render
            self.render();

            // Time management!
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }

        Ok(())
    }
}

fn main() -> Result<(), String> {
    let nes = Nes::new();
    let mut app = App::new(nes);
    app.run()?;

    Ok(())
}
