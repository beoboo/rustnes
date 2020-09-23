use crate::types::Byte;
use rand::Rng;

pub struct Buffer {
    pub data: Vec<Byte>,
}

pub struct Nes {
    pub width: u32,
    pub height: u32,
    pub bits_per_pixel: u32,
    // pub buffer: Buffer
}

impl Nes {
    pub fn new() -> Nes {
        Nes {
            width: 256,
            height: 240,
            bits_per_pixel: 4,
            // buffer: Buffer::new(),
        }
    }

    pub fn run() {
        // let rom = Rom::load(filename, 16384, 8192);
        // let mut bus = BusImpl::new(Ram::new(0x0800), Apu::new(), Ppu::new(), rom);
        //
        // let start = bus.read_word(0xFFFC);
        // println!("Starting address: {:#06X}", start);
        //
        // let mut cpu = Cpu::new(start);
        //
        // let mut cycles = cpu.process(&mut bus);
        // let mut total_cycles = cycles;
        //
        // while cycles != 0 {
        //     cycles = cpu.process(&mut bus);
        //     total_cycles += cycles;
        // }
    }

    pub fn get_rendered_buffer(&self) -> Vec<Byte> {
        let buffer_size = (self.width * self.height * self.bits_per_pixel) as usize;
        let mut buffer = vec![0; buffer_size];
        let mut rng = rand::thread_rng();

        for i in 0..self.height {
            // let pos = ((i * self.width) * self.bits_per_pixel) as usize;
            // println!("i: {}, pos: {}", i, pos);
            for j in 0..self.width {
                let pos = ((i * self.width + j) * self.bits_per_pixel) as usize;
                // if pos >= buffer_size {
                //     println!("Pos: {}, i: {}, j: {}", pos, i, j);
                // }
                buffer[pos] = rng.gen_range(0, 255);
                buffer[pos + 1] = rng.gen_range(0, 255);
                buffer[pos + 2] = rng.gen_range(0, 255);
            }
        }

        buffer
    }
}