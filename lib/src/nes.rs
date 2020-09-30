use log::info;
use rand::Rng;

use crate::apu::Apu;
use crate::bus::bus_impl::BusImpl;
use crate::cpu::Cpu;
use crate::ppu::Ppu;
use crate::ram::Ram;
use crate::rom::Rom;
use crate::types::Byte;
use crate::assembler::Assembler;
use crate::parser::Parser;

pub struct Buffer {
    pub data: Vec<Byte>,
}

pub struct Nes {
    pub cpu: Cpu,
    pub bus: BusImpl,
    pub width: u32,
    pub height: u32,
    pub bits_per_pixel: u32,
    // pub buffer: Buffer
    pub cycles: usize,
}

fn build_program(source: &str) -> Vec<Byte> {
    println!("Processing:\n {}", source);
    let assembler = Assembler::default();
    let parser = Parser::default();
    let tokens = parser.parse(source).unwrap();

    let program = match assembler.assemble(tokens) {
        Ok(program) => program,
        Err(e) => panic!("Assembler error: {}", e)
    };
    println!("Program: {:x?}", program);

    program.data
}


impl Nes {
    pub fn new(filename: &str) -> Nes {
        let rom = Rom::load(filename, 16384, 8192);
//         let program = build_program("
// LDX #10
// STX $0000
// LDX #3
// STX $0001
// LDY $0000
// LDA #0
// CLC
// ADC $0001
// DEY
// BNE $FA
// STA $0002
// NOP
// NOP
// NOP
// ");

        // let rom = Rom::new(program.as_slice(), vec![0u8; 0].as_slice());
        let bus = BusImpl::new(Ram::new(0x0800), Apu::default(), Ppu::default(), rom.clone());
        let cpu = Cpu::new(0);

        Nes {
            cpu,
            bus,
            width: 256,
            height: 240,
            bits_per_pixel: 4,
            // buffer: Buffer::new(),
            cycles: 0,
        }
    }

    pub fn reset(&mut self) {
        info!("[Nes::reset]");
        self.cpu.reset(&mut self.bus);
        self.cycles += 1;

        while self.cpu.tick(&mut self.bus) != 0 {
            self.cycles += 1;
        }

        self.cycles += 1;
    }

    pub fn tick(&mut self) {
        info!("[Nes::tick]");
        self.cpu.tick(&mut self.bus);
        self.cycles += 1;
    }

    pub fn process_next(&mut self) {
        info!("[Nes::process_next]");
        while self.cpu.tick(&mut self.bus) != 0 {
            self.cycles += 1;
        };
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