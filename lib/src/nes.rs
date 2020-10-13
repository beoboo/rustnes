use log::info;

use crate::apu::Apu;
use crate::bus::bus_impl::BusImpl;
use crate::cpu::Cpu;
use crate::ppu::Ppu;
use crate::ram::Ram;
use crate::rom::Rom;
use crate::types::Byte;
use crate::assembler::Assembler;
use crate::parser::Parser;

#[derive(Debug)]
pub struct Nes {
    pub cpu: Cpu,
    pub bus: BusImpl,
    pub width: u32,
    pub height: u32,
    pub bits_per_pixel: u32,
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

impl Default for Nes {
    fn default() -> Self {
        let width= 256;
        let height= 240;
        let bits_per_pixel = 4;

        let ram = Ram::new(0x0800);
        let ppu = Ppu::default();
        let apu = Apu::default();
        let rom = Rom::default();
        let bus = BusImpl::new(ram, ppu, apu, rom);
        let cpu = Cpu::new(0);

        Self {
            cpu,
            bus,
            width,
            height,
            bits_per_pixel,
            cycles: 0,
        }
    }
}

impl Nes {
    pub fn load(&mut self, filename: &str) {
        info!("[Nes::load] {}", filename);
        let rom = Rom::from_file(filename, 16384, 8192);
        self.bus.load_rom(rom);
    }

    pub fn load_mul3(&mut self) {
        let program = build_program("
LDX #10
STX $0000
LDX #3
STX $0001
LDY $0000
LDA #0
CLC
ADC $0001
DEY
BNE $FA
STA $0002
NOP
NOP
NOP
");

        self.bus.rom.load_bytes(program.as_slice(), vec![0u8; 0].as_slice());
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
        if self.cycles % 3 == 0 {
            self.cpu.tick(&mut self.bus);
        }

        self.bus.ppu.tick();
        self.cycles += 1;
    }

    pub fn process_next(&mut self) {
        info!("[Nes::process_next]");
        while self.cpu.tick(&mut self.bus) != 0 {
            self.cycles += 1;
        };
    }

    pub fn is_frame_complete(&self) -> bool {
        self.bus.ppu.frame_complete
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

    pub fn get_rendered_buffer(&self) -> &[u8] {
        self.bus.ppu.frame.data.as_slice()
    }
}