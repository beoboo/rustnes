use log::trace;

use crate::ppu::control::Control;
use crate::ppu::frame::Frame;
use crate::ppu::mask::Mask;
use crate::ppu::palette::Palette;
use crate::ppu::status::Status;
use crate::ram::Ram;
use crate::types::{Byte, Word};

pub mod color;
pub mod control;
pub mod mask;
pub mod status;
mod frame;
mod palette;

enum PpuRamType {
    PatternTable,
    NameTable,
    PaletteMap,
    AttributeTable,
}

#[derive(Debug)]
pub struct Ppu {
    pub control: Control,
    pub mask: Mask,
    pub status: Status,
    pub ram: Ram,
    pub frame: Frame,
    pub pattern_tables: Vec<Ram>,
    pub name_tables: Vec<Ram>,
    pub palette_maps: Vec<Ram>,
    pub attribute_tables: Vec<Ram>,
    pub palette: Palette,
    scan_lines: usize,
    cycles: usize,
    address_latch: bool,
    horizontal_scroll: Byte,
    vertical_scroll: Byte,
    address: Word,
    pub frame_complete: bool,
}

impl Default for Ppu {
    fn default() -> Ppu {
        Ppu {
            control: Control::default(),
            mask: Mask::default(),
            status: Status::default(),
            ram: Ram::new(2048),
            frame: Frame::new(Ppu::SCREEN_WIDTH, Ppu::SCREEN_HEIGHT, 4),
            pattern_tables: vec![Ram::new(Ppu::PATTERN_TABLE_SIZE), Ram::new(Ppu::PATTERN_TABLE_SIZE)],
            name_tables: vec![Ram::new(Ppu::NAME_TABLE_SIZE), Ram::new(Ppu::NAME_TABLE_SIZE), Ram::new(Ppu::NAME_TABLE_SIZE), Ram::new(Ppu::NAME_TABLE_SIZE)],
            attribute_tables: vec![Ram::new(Ppu::ATTRIBUTE_TABLE_SIZE), Ram::new(Ppu::ATTRIBUTE_TABLE_SIZE), Ram::new(Ppu::ATTRIBUTE_TABLE_SIZE), Ram::new(Ppu::ATTRIBUTE_TABLE_SIZE)],
            palette_maps: vec![Ram::new(Ppu::PALETTE_MAP_SIZE), Ram::new(Ppu::PALETTE_MAP_SIZE)],
            palette: Palette::default(),
            scan_lines: 0,
            cycles: 0,
            address_latch: false,
            horizontal_scroll: 0,
            vertical_scroll: 0,
            address: 0,
            frame_complete: false,
        }
    }
}

impl Ppu {
    const SCREEN_WIDTH: Word = 256;
    const SCREEN_HEIGHT: Word = 240;
    const CYCLES_PER_SCAN_LINE: usize = 341;
    const MAX_SCAN_LINES: usize = 262;
    const PATTERN_TABLE_SIZE: usize = 0x1000;
    const NAME_TABLE_SIZE: usize = 0x03C0;
    const ATTRIBUTE_TABLE_SIZE: usize = 0x40;
    const PALETTE_MAP_SIZE: usize = 0x10;

    pub fn read_byte(&mut self, address: Word) -> Byte {
        trace!("[Ppu::read_byte] Reading from {:#06X}", address);
        let address = address & 0x0007;

        match address {
            0x0002 => {
                self.address_latch = false;
                self.status.to_byte()
            },
            _ => panic!(format!("[Ppu::read_byte] Not mapped address: {:#6X}", address))
        }

        // trace!("[Ppu::read_byte] Reading from PPU address: {:#06X}", address);
        // self.ram.read_byte(address)
    }

    pub fn write_byte(&mut self, address: Word, data: Byte) {
        let address = address & 0x0007;

        println!("[Ppu::write_byte] Writing {:#04X} to PPU address {:#06X}", data, address);
        trace!("[Ppu::write_byte] Writing {:#04X} to PPU address {:#06X}", data, address);

        match address {
            0x0000 => self.control = Control::from_byte(data),
            0x0001 => self.mask = Mask::from_byte(data),
            0x0005 => {
                if self.address_latch {
                    self.address_latch = false;
                    self.vertical_scroll = data;
                } else {
                    self.address_latch = true;
                    self.horizontal_scroll = data;
                }
            },
            0x0006 => {
                let data = data as Word;
                println!("Data: {:#06X}", data << 8);

                if self.address_latch {
                    self.address_latch = false;
                    self.address |= data;
                } else {
                    self.address_latch = true;
                    self.address = data << 8;
                }
                trace!("[Ppu::write_byte] PPU address is {:#06X}", self.address);
            },
            0x0007 => {
                self.write_data(data);
            },
            _ => panic!(format!("[Ppu::write_byte] Not mapped address: {:#6X}", address))
        }

        self.ram.write_byte(address, data);
    }

    fn write_data(&mut self, data:Byte) {
        match self.address {
            0x0000..=0x0FFF => self.pattern_tables[0].write_byte(self.address, data),
            0x1000..=0x1FFF => self.pattern_tables[1].write_byte(self.address - 0x1000, data),
            0x2000..=0x23BF => self.name_tables[0].write_byte(self.address - 0x2000, data),
            0x23C0..=0x23BF => self.name_tables[0].write_byte(self.address - 0x23C0, data),
            0x3F00..=0x3F0F => self.palette_maps[0].write_byte(self.address - 0x3F00, data),
            _ => panic!(format!("[Ppu::write_byte] Not mapped address: {:#6X}", self.address))
        }
    }
    //
    // pub fn load_palette(&mut self, palette: Palette) {
    //     self.palettes.push(palette);
    // }

    pub fn tick(&mut self) {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        let color = self.palette.colors[rng.gen_range(0, self.palette.colors.len())];
        self.frame.draw(self.cycles as Word, self.scan_lines as Word, color);

        self.cycles += 1;

        if self.cycles >= Ppu::CYCLES_PER_SCAN_LINE {
            self.cycles = 0;
            self.scan_lines += 1;
        }

        if self.scan_lines == Ppu::SCREEN_HEIGHT as usize {
            self.status.V = true;
        }

        if self.scan_lines == Ppu::MAX_SCAN_LINES {
            self.frame_complete = true;
            self.status.V = false;
        }

        if self.scan_lines > Ppu::MAX_SCAN_LINES {
            self.scan_lines = 0;
            self.frame_complete = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use crate::ppu::color::Color;

    use super::*;

    #[test]
    fn test_read_byte() {
        let mut ppu = Ppu::default();

        ppu.status = Status::from_byte(0x60);
        assert_that!(ppu.read_byte(0x0002), eq(0x60));
    }

    #[test]
    fn test_write_bytes() {
        let mut ppu = Ppu::default();

        ppu.write_byte(0x0, 0xFF);
        assert_that!(ppu.control.to_byte(), eq(0xFF));
        ppu.write_byte(0x1, 0xFF);
        assert_that!(ppu.mask.to_byte(), eq(0xFF));
    }

    #[test]
    fn test_tick() {
        let color = Color::from_rgba(0x12, 0x34, 0x56, 0x78);
        let mut ppu = build_ppu(&[color]);

        ppu.tick();
        ppu.tick();

        assert_that!(ppu.scan_lines, eq(0));
        assert_that!(ppu.cycles, eq(2));
        assert_that!(ppu.frame.at(0, 0), eq(Some(color)));
        assert_that!(ppu.frame.at(1, 0), eq(Some(color)));
        assert_that!(ppu.frame_complete, eq(false));
    }

    #[test]
    fn test_scanline() {
        let color = Color::from_rgba(0x12, 0x34, 0x56, 0x78);
        let mut ppu = build_ppu(&[color]);

        for _ in 0..Ppu::CYCLES_PER_SCAN_LINE {
            ppu.tick();
        }

        assert_that!(ppu.scan_lines, eq(1));
        assert_that!(ppu.cycles, eq(0));
        assert_that!(ppu.frame.at(0, 0), eq(Some(color)));
        assert_that!(ppu.frame.at(Ppu::SCREEN_WIDTH - 1, 0), eq(Some(color)));
        assert_that!(ppu.status.V, eq(false));
    }

    #[test]
    fn test_frame() {
        let color = Color::from_rgba(0x12, 0x34, 0x56, 0x78);
        let mut ppu = build_ppu(&[color]);

        // Visible frame
        for _ in 0..Ppu::SCREEN_HEIGHT as usize {
            for _ in 0..Ppu::CYCLES_PER_SCAN_LINE {
                ppu.tick();
            }
        }

        assert_that!(ppu.scan_lines, eq(Ppu::SCREEN_HEIGHT as usize));
        assert_that!(ppu.cycles, eq(0));
        assert_that!(ppu.frame.at(0, 0), eq(Some(color)));
        assert_that!(ppu.frame.at(Ppu::SCREEN_WIDTH - 1, 0), eq(Some(color)));
        assert_that!(ppu.frame.at(0, Ppu::SCREEN_HEIGHT - 1), eq(Some(color)));
        assert_that!(ppu.frame.at(Ppu::SCREEN_WIDTH - 1, Ppu::SCREEN_HEIGHT - 1), eq(Some(color)));
        assert_that!(ppu.status.V, eq(true));

        // Unvisible section of the frame
        for _ in Ppu::SCREEN_HEIGHT as usize..Ppu::MAX_SCAN_LINES {
            for _ in 0..Ppu::CYCLES_PER_SCAN_LINE {
                ppu.tick();
            }
        }

        assert_that!(ppu.scan_lines, eq(Ppu::MAX_SCAN_LINES));
        assert_that!(ppu.frame_complete, eq(true));
        assert_that!(ppu.status.V, eq(false));

        // Pre-render line
        ppu.tick();
        assert_that!(ppu.status.V, eq(false));

        for _ in 0..Ppu::CYCLES_PER_SCAN_LINE - 1 {
            ppu.tick();
        }

        assert_that!(ppu.scan_lines, eq(0));
        assert_that!(ppu.frame_complete, eq(false));
    }

    #[test]
    fn test_control() {
        let mut ppu = Ppu::default();
        ppu.address_latch = true;

        let _ = ppu.read_byte(0x0002);

        assert_that!(ppu.address_latch, is(false));
    }

    #[test]
    fn test_scroll() {
        let mut ppu = Ppu::default();

        assert_scroll(&ppu, false, 0x00, 0x00);

        ppu.write_byte(0x5, 0xFF);
        assert_scroll(&ppu, true, 0xFF, 0x00);

        ppu.write_byte(0x5, 0xFF);
        assert_scroll(&ppu, false, 0xFF, 0xFF);
    }

    #[test]
    fn test_address() {
        let mut ppu = Ppu::default();

        assert_address(&ppu, false, 0x0000);

        ppu.write_byte(0x6, 0x12);
        assert_address(&ppu, true, 0x1200);

        ppu.write_byte(0x6, 0x34);
        assert_address(&ppu, false, 0x1234);
    }

    #[test]
    fn test_write_pattern_table() {
        let mut ppu = Ppu::default();
        let address = 0x0000;

        assert_that!(ppu.pattern_tables[0].read_byte(address), eq(0x00));

        ppu.address = address;
        ppu.write_byte(0x0007, 0xAB);

        assert_that!(ppu.pattern_tables[0].read_byte(address), eq(0xAB));
    }

    #[test]
    fn test_write_name_table() {
        let mut ppu = Ppu::default();
        let address = 0x2000;

        assert_that!(ppu.name_tables[0].read_byte(address - 0x2000), eq(0x00));

        ppu.address = address;
        ppu.write_byte(0x0007, 0xAB);

        assert_that!(ppu.name_tables[0].read_byte(address - 0x2000), eq(0xAB));
    }

    #[test]
    fn test_write_palette_map() {
        let mut ppu = Ppu::default();
        let address = 0x3F00;

        assert_that!(ppu.palette_maps[0].read_byte(address - 0x3F00), eq(0x00));

        ppu.address = address;
        ppu.write_byte(0x0007, 0xAB);

        assert_that!(ppu.palette_maps[0].read_byte(address - 0x3F00), eq(0xAB));
    }

    #[test]
    fn test_write2() {
        let mut ppu = Ppu::default();

        // Pattern tables
        assert_write(&mut ppu, PpuRamType::PatternTable, 0, 0x0000, 0x0000);
        assert_write(&mut ppu, PpuRamType::PatternTable, 1, 0x1000, 0x1000);

        // Name and attribute tables
        assert_write(&mut ppu, PpuRamType::NameTable, 0, 0x2000, 0x2000);
        assert_write(&mut ppu, PpuRamType::AttributeTable, 0, 0x23C0, 0x23C0);
        assert_write(&mut ppu, PpuRamType::NameTable, 1, 0x2400, 0x2400);
        assert_write(&mut ppu, PpuRamType::AttributeTable, 1, 0x27C0, 0x27C0);
        assert_write(&mut ppu, PpuRamType::NameTable, 2, 0x2800, 0x2800);
        assert_write(&mut ppu, PpuRamType::AttributeTable, 2, 0x2BC0, 0x2BC0);
        assert_write(&mut ppu, PpuRamType::NameTable, 3, 0x2C00, 0x2C00);
        assert_write(&mut ppu, PpuRamType::AttributeTable, 3, 0x2FC0, 0x2FC0);

        // Name and attribute tables mirrors 0x3000 - 0x3EFF
        assert_write(&mut ppu, PpuRamType::NameTable, 0, 0x3000, 0x3000);
        assert_write(&mut ppu, PpuRamType::AttributeTable, 0, 0x33C0, 0x33C0);
        assert_write(&mut ppu, PpuRamType::NameTable, 1, 0x3400, 0x3400);
        assert_write(&mut ppu, PpuRamType::AttributeTable, 1, 0x37C0, 0x37C0);
        assert_write(&mut ppu, PpuRamType::NameTable, 2, 0x3800, 0x3800);
        assert_write(&mut ppu, PpuRamType::AttributeTable, 2, 0x3BC0, 0x3BC0);
        assert_write(&mut ppu, PpuRamType::NameTable, 3, 0x3C00, 0x3C00);
        assert_write(&mut ppu, PpuRamType::AttributeTable, 3, 0x3FC0, 0x3FC0);

        // Palette maps
        assert_write(&mut ppu, PpuRamType::PaletteMap, 0, 0x3F00, 0x3F00);
        assert_write(&mut ppu, PpuRamType::PaletteMap, 1, 0x3F00, 0x3F00);

        // Palette mirrors 0x3F20 - 0x3FFF
        assert_write(&mut ppu, PpuRamType::PaletteMap, 0, 0x3F20, 0x3F20);
        assert_write(&mut ppu, PpuRamType::PaletteMap, 1, 0x3FFF, 0x3FFF);
        // assert_write(&mut ppu, PpuRamType::PaletteMap, 1, 0x3F00, 0x3F00);
    }

    fn assert_write(ppu: &mut Ppu, ram_type: PpuRamType, index: usize, address: Word, offset: Word) {
        assert_ram(ppu, &ram_type, index, address, offset, 0x00);

        ppu.address = address;
        ppu.write_byte(0x0007, 0xAB);

        assert_ram(ppu, &ram_type, index, address, offset, 0xAB);
    }

    fn assert_ram(ppu: &mut Ppu, ram_type: &PpuRamType, index: usize, address: Word, offset: Word, data: Byte) {
        let ram = match ram_type {
            PpuRamType::PatternTable => &ppu.pattern_tables[index],
            PpuRamType::NameTable => &ppu.name_tables[index],
            PpuRamType::AttributeTable => &ppu.attribute_tables[index],
            PpuRamType::PaletteMap => &ppu.palette_maps[index],
        };

        assert_that!(ram.read_byte(address - offset), eq(data));
    }



    fn assert_scroll(ppu: &Ppu, address_latch: bool, horizontal_scroll: Byte, vertical_scroll: Byte) {
        assert_that!(ppu.address_latch, eq(address_latch));
        assert_that!(ppu.horizontal_scroll, eq(horizontal_scroll));
        assert_that!(ppu.vertical_scroll, eq(vertical_scroll));
    }

    fn assert_address(ppu: &Ppu, address_latch: bool, address: Word) {
        assert_that!(ppu.address_latch, eq(address_latch));
        assert_that!(ppu.address, eq(address));
    }

    fn build_ppu(colors: &[Color]) -> Ppu {
        let mut ppu = Ppu::default();

        ppu.palette = Palette::new(colors);

        ppu
    }
}