use std::fmt;

use env_logger::{Builder, fmt::{Color, Style}};
use std::sync::atomic::{AtomicUsize, Ordering};
use log::Level;
use std::fmt::Arguments;

pub fn formatted_builder() -> Builder {
    let mut builder = Builder::new();

    builder.format(|f, record| {
        use std::io::Write;

        // let target = record.target();
        // let max_width = max_target_width(target);

        let mut style = f.style();
        map_style(&mut style, record.level());

        // let mut style = f.style();
        // let target = style.set_bold(true).value(Padded {
        //     value: target,
        //     width: max_width,
        // });

        writeln!(
            f,
            "{}",
            style.value(record.args()),
        )
    });

    builder
}

struct Padded<T> {
    value: T,
    width: usize,
}

impl<T: fmt::Display> fmt::Display for Padded<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <width$}", self.value, width = self.width)
    }
}

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

fn max_target_width(target: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

fn map_style(style: &mut Style, level: Level) {
    match level {
        Level::Trace => style.set_color(Color::Rgb(0x6A, 0x8C, 0xAF)),
        Level::Debug => style.set_color(Color::Rgb(0x66, 0x66, 0x66)),
        Level::Info => style.set_color(Color::Rgb(0x8B, 0xCD, 0xCD)),
        Level::Warn => style.set_color(Color::Rgb(0xF6, 0xD1, 0x86)),
        Level::Error => style.set_color(Color::Rgb(0xD8, 0x34, 0x5F)),
    };
}