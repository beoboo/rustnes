#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use egui::Color32;
use rn_core::{cpu::Cpu, errors::NesError, ppu::Ppu};
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

pub type ProviderFn = Box<dyn Fn(u16) -> Result<u8, NesError>>;

/// Memory pixel adapter that converts memory bytes to RGB format matching the PPU
pub struct MemoryPixelAdapter {
    start_addr: u16,
    end_addr: u16,
    display_width: usize,
    title: String,
    description: String,
    read_fn: Box<dyn Fn(u16) -> Result<u8, NesError>>,
}

impl MemoryPixelAdapter {
    /// Create a new memory pixel adapter using a custom read function
    pub fn new<F>(read_fn: F, start_addr: u16, end_addr: u16, width: usize) -> Self
    where
        F: Fn(u16) -> Result<u8, NesError> + 'static,
    {
        Self {
            start_addr,
            end_addr,
            display_width: width,
            title: format!("Memory Visualization ({:#06X}-{:#06X})", start_addr, end_addr),
            description: "Each byte is represented as a pixel with color value".to_string(),
            read_fn: Box::new(read_fn),
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
        // Use the custom read function if provided
        for addr in self.start_addr..=self.end_addr {
            let memory_value = (self.read_fn)(addr)?;
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
        memory_size.div_ceil(self.display_width) // Ceiling division
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// PPU pixel adapter for displaying the PPU frame buffer
/// The PPU's full output, before any overscan is hidden.
const NES_WIDTH: usize = 256;
const NES_HEIGHT: usize = 240;

pub struct PpuPixelAdapter {
    title: String,
    description: String,
    frame_buffer_fn: Box<dyn Fn() -> Vec<u8>>,
    /// Scanlines hidden at the top and bottom, as a television's overscan hid them.
    ///
    /// A CRT never showed the whole 240 lines: the outermost rows fell behind the bezel, so games
    /// treated them as a margin and put nothing there that mattered. They are not always blank
    /// though — a vertical scroll of 240 or more makes the PPU read attribute data as tiles, and
    /// the resulting garbage rows land exactly in that hidden margin. Showing every line renders
    /// them faithfully and makes the picture look broken in a way no player ever saw.
    overscan: usize,
}

impl PpuPixelAdapter {
    /// Create a new PPU pixel adapter using a custom frame buffer provider
    pub fn new<F>(frame_buffer_fn: F) -> Self
    where
        F: Fn() -> Vec<u8> + 'static,
    {
        Self {
            title: "PPU Display".to_string(),
            description: "NES screen output (256x240)".to_string(),
            frame_buffer_fn: Box::new(frame_buffer_fn),
            overscan: 0,
        }
    }

    /// Hide `lines` scanlines at the top and bottom. Eight is what a television typically hid.
    pub fn with_overscan(mut self, lines: usize) -> Self {
        self.overscan = lines.min(NES_HEIGHT / 2);
        self.description = format!("NES screen output ({}x{})", NES_WIDTH, self.height());
        self
    }
}

impl PixelDataProvider for PpuPixelAdapter {
    fn get_pixel_data(&self) -> Result<Vec<u8>> {
        let frame = (self.frame_buffer_fn)();
        if self.overscan == 0 {
            return Ok(frame);
        }

        let row = NES_WIDTH * 3;
        let start = self.overscan * row;
        let end = (NES_HEIGHT - self.overscan) * row;
        Ok(frame.get(start..end).map(<[u8]>::to_vec).unwrap_or(frame))
    }

    fn width(&self) -> usize {
        NES_WIDTH
    }

    fn height(&self) -> usize {
        NES_HEIGHT - self.overscan * 2
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Displays all four nametables at once as a 512x480 image.
///
/// The NES has four logical nametables but only enough VRAM for two, so the other two are
/// mirrors. Seeing all four together makes it obvious which hold content, how mirroring has
/// aliased them, and where the visible viewport sits — which is the part that matters when a
/// scroll goes wrong, and is invisible from the 256x240 output alone.
pub struct NametableMapAdapter {
    title: String,
    description: String,
    map_fn: Box<dyn Fn() -> Vec<u8>>,
}

impl NametableMapAdapter {
    pub fn new<F>(map_fn: F) -> Self
    where
        F: Fn() -> Vec<u8> + 'static,
    {
        Self {
            title: "Nametables".to_string(),
            description: "All four nametables (512x480); the viewport is outlined in red".to_string(),
            map_fn: Box::new(map_fn),
        }
    }
}

impl PixelDataProvider for NametableMapAdapter {
    fn get_pixel_data(&self) -> Result<Vec<u8>> {
        Ok((self.map_fn)())
    }

    fn width(&self) -> usize {
        512
    }

    fn height(&self) -> usize {
        480
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[cfg(test)]
mod overscan_tests {
    use super::*;

    fn numbered_frame() -> Vec<u8> {
        // Every pixel of row y holds y, so a cropped frame says which rows survived.
        (0..NES_HEIGHT).flat_map(|y| std::iter::repeat_n(y as u8, NES_WIDTH * 3)).collect()
    }

    #[test]
    fn without_overscan_the_whole_picture_is_shown() {
        let adapter = PpuPixelAdapter::new(numbered_frame);
        assert_eq!(adapter.height(), 240);
        assert_eq!(adapter.get_pixel_data().unwrap().len(), NES_WIDTH * NES_HEIGHT * 3);
    }

    /// A television hid the outermost scanlines, and games relied on that: an out-of-range
    /// vertical scroll puts attribute data on screen as tiles, and those garbage rows land in
    /// exactly the margin no player could see.
    #[test]
    fn overscan_removes_equal_numbers_of_rows_from_each_end() {
        let adapter = PpuPixelAdapter::new(numbered_frame).with_overscan(8);

        assert_eq!(adapter.height(), 224, "eight hidden at the top and eight at the bottom");
        assert_eq!(adapter.width(), 256, "overscan here does not touch the sides");

        let data = adapter.get_pixel_data().unwrap();
        assert_eq!(data.len(), NES_WIDTH * 224 * 3);
        assert_eq!(data[0], 8, "the first visible row should be row 8");
        assert_eq!(*data.last().unwrap(), 231, "the last should be row 231");
    }

    #[test]
    fn overscan_cannot_crop_away_the_entire_picture() {
        let adapter = PpuPixelAdapter::new(numbered_frame).with_overscan(1000);
        assert_eq!(adapter.height(), 0);
        assert!(adapter.get_pixel_data().is_ok(), "an absurd request must not panic");
    }
}
