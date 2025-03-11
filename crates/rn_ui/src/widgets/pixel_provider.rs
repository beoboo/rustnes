#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc};

use egui::Color32;
use rn_core::{cpu::Cpu, ppu::Ppu};
use anyhow::Result;
/// Trait for providing pixel data for display
pub trait PixelDataProvider {
    /// Get the pixel data to be displayed in RGB format (3 bytes per pixel)
    fn get_pixel_data(&self) -> Result<Vec<u8>>;

    /// Get the width of the display in pixels
    fn width(&self) -> usize;

    /// Get the height of the display in pixels
    fn height(&self) -> usize;

    /// Get a title for the display
    fn title(&self) -> &str;

    /// Get a description of the display
    fn description(&self) -> &str;
}

/// Memory pixel adapter that converts memory bytes to RGB format matching the PPU
pub struct MemoryPixelAdapter {
    cpu: Rc<RefCell<Cpu>>,
    start_addr: u16,
    end_addr: u16,
    display_width: usize,
    title: String,
    description: String,
}

impl MemoryPixelAdapter {
    /// Create a new memory pixel adapter
    pub fn new(cpu: Rc<RefCell<Cpu>>, start_addr: u16, end_addr: u16, width: usize) -> Self {
        Self {
            cpu,
            start_addr,
            end_addr,
            display_width: width,
            title: format!("Memory Visualization ({:#06X}-{:#06X})", start_addr, end_addr),
            description: "Each byte is represented as a pixel with color value".to_string(),
        }
    }

    /// Convert a memory byte to an RGB color
    fn byte_to_rgb(&self, value: u8) -> [u8; 3] {
        // Define a color palette based on common NES colors
        match value {
            0x00 => [0, 0, 0],       // Black
            0x01 => [255, 255, 255], // White
            0x02 => [124, 124, 124], // Dark Gray
            0x03 => [188, 188, 188], // Light Gray
            0x04 => [248, 56, 0],    // Red
            0x05 => [252, 160, 68],  // Orange
            0x06 => [236, 200, 76],  // Yellow
            0x07 => [116, 208, 0],   // Green
            0x08 => [0, 120, 248],   // Blue
            0x09 => [104, 68, 252],  // Purple
            0x0A => [168, 0, 32],    // Dark Red
            0x0B => [0, 168, 0],     // Dark Green
            0x0C => [0, 0, 168],     // Dark Blue
            0x0D => [0, 168, 168],   // Cyan
            0x0E => [168, 0, 168],   // Magenta
            0x0F => [168, 168, 0],   // Yellow-Green
            // For values outside the defined palette, use a gradient based on the value
            _ => {
                let brightness = (value as f32 / 255.0 * 0.7 + 0.3) * 255.0;
                [brightness as u8, brightness as u8, brightness as u8]
            },
        }
    }
}

impl PixelDataProvider for MemoryPixelAdapter {
    fn get_pixel_data(&self) -> Result<Vec<u8>> {
        let memory_size = (self.end_addr - self.start_addr + 1) as usize;
        // Allocate vector with 3 bytes per pixel (RGB)
        let mut rgb_data = Vec::with_capacity(memory_size * 3);

        // Get memory values and convert to RGB
        let cpu = self.cpu.borrow();
        for addr in self.start_addr..=self.end_addr {
            let memory_value = cpu.read_byte(addr)?;
            let rgb = self.byte_to_rgb(memory_value);
            rgb_data.push(rgb[0]);
            rgb_data.push(rgb[1]);
            rgb_data.push(rgb[2]);
        }

        Ok(rgb_data)
    }

    fn width(&self) -> usize {
        self.display_width
    }

    fn height(&self) -> usize {
        let memory_size = (self.end_addr - self.start_addr + 1) as usize;
        (memory_size + self.display_width - 1) / self.display_width // Ceiling division
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// PPU pixel adapter for displaying the PPU frame buffer
pub struct PpuPixelAdapter {
    ppu: Rc<RefCell<Ppu>>,
    title: String,
    description: String,
}

impl PpuPixelAdapter {
    /// Create a new PPU pixel adapter
    pub fn new(ppu: Rc<RefCell<Ppu>>) -> Self {
        Self {
            ppu,
            title: "PPU Display".to_string(),
            description: "NES screen output (256x240)".to_string(),
        }
    }
}

impl PixelDataProvider for PpuPixelAdapter {
    fn get_pixel_data(&self) -> Result<Vec<u8>> {
        // Make a copy of the PPU frame buffer
        let ppu = self.ppu.borrow();
        let frame_buffer = ppu.frame_buffer();
        Ok(frame_buffer.to_vec())
    }

    fn width(&self) -> usize {
        256 // NES native resolution width
    }

    fn height(&self) -> usize {
        240 // NES native resolution height
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn description(&self) -> &str {
        &self.description
    }
}
