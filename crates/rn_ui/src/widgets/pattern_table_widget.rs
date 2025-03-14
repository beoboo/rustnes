#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use egui::{Color32, Rect, Sense, Ui, Vec2};
use rn_core::cartridge::Cartridge;

/// Widget for displaying pattern table (CHR ROM/RAM) data
pub struct PatternTableWidget {
    // Visual settings
    tile_size: f32,    // Display size of each 8x8 tile
    zoom: f32,         // Additional zoom factor
    show_grid: bool,   // Whether to show grid lines between tiles
    current_table: u8, // 0 for first pattern table, 1 for second

    // Colors for the 4 possible pixel values (0-3)
    colors: [Color32; 4],
}

impl PatternTableWidget {
    /// Create a new pattern table widget with default settings
    pub fn new() -> Self {
        Self {
            tile_size: 16.0, // Each 8x8 tile is displayed as 16x16 pixels by default
            zoom: 1.0,
            show_grid: true,
            current_table: 0, // Start with first pattern table

            // Default NES-like grayscale palette
            colors: [
                Color32::from_rgb(24, 24, 24),    // 0: Dark gray (background)
                Color32::from_rgb(85, 85, 85),    // 1: Gray
                Color32::from_rgb(170, 170, 170), // 2: Light gray
                Color32::from_rgb(255, 255, 255), // 3: White
            ],
        }
    }

    /// Display the pattern table widget in the given UI
    pub fn ui(&mut self, ui: &mut Ui, cartridge: Option<&Rc<RefCell<Cartridge>>>) -> Result<egui::Response> {
        ui.heading("Pattern Table Viewer");

        // Display controls for the widget
        ui.horizontal(|ui| {
            // Allow user to switch between pattern tables
            ui.label("Pattern Table:");
            if ui.selectable_label(self.current_table == 0, "0").clicked() {
                self.current_table = 0;
            }
            if ui.selectable_label(self.current_table == 1, "1").clicked() {
                self.current_table = 1;
            }

            // Add a small gap
            ui.add_space(20.0);

            // Allow zoom control
            ui.label("Zoom:");
            if ui.button("-").clicked() && self.zoom > 0.5 {
                self.zoom -= 0.25;
            }
            ui.label(format!("{:.2}x", self.zoom));
            if ui.button("+").clicked() && self.zoom < 4.0 {
                self.zoom += 0.25;
            }

            // Add a small gap
            ui.add_space(20.0);

            // Toggle grid lines
            let mut show_grid = self.show_grid;
            ui.checkbox(&mut show_grid, "Show Grid");
            if show_grid != self.show_grid {
                self.show_grid = show_grid;
            }
        });

        // If no cartridge is available, show a message
        if cartridge.is_none() {
            return Ok(ui.label("No cartridge loaded").interact(Sense::click()));
        }

        // Calculate the dimensions of the display area
        // Each pattern table has 256 tiles (16x16 grid)
        let tiles_per_row = 16;
        let num_rows = 16;

        // Calculate display size (add 1 pixel for grid lines if enabled)
        let grid_padding = if self.show_grid { 1.0 } else { 0.0 };
        let display_width = tiles_per_row as f32 * (self.tile_size + grid_padding) * self.zoom;
        let display_height = num_rows as f32 * (self.tile_size + grid_padding) * self.zoom;

        let display_size = Vec2::new(display_width, display_height);

        // Allocate the drawing area
        let (rect, response) = ui.allocate_exact_size(display_size, Sense::click_and_drag());

        // Only draw if the area is visible
        if ui.is_rect_visible(rect) {
            let cartridge_ref = cartridge.unwrap();
            let cart = cartridge_ref.borrow();

            // Calculate base tile index for the selected pattern table
            let base_tile_index = self.current_table as u16 * 256;

            // Draw the pattern table
            let painter = ui.painter_at(rect);

            for row in 0..num_rows {
                for col in 0..tiles_per_row {
                    // Calculate tile index
                    let tile_index = base_tile_index + (row * tiles_per_row + col) as u16;

                    // Get the pixel data for this tile
                    let pixel_data = cart.get_tile_pixels(tile_index);

                    // Calculate top-left corner of the tile
                    let tile_x = rect.min.x + col as f32 * (self.tile_size + grid_padding) * self.zoom;
                    let tile_y = rect.min.y + row as f32 * (self.tile_size + grid_padding) * self.zoom;

                    // Draw each pixel in the tile
                    let pixel_size = self.tile_size * self.zoom / 8.0; // Each tile is 8x8 pixels

                    for py in 0..8 {
                        for px in 0..8 {
                            // Get the pixel value (0-3)
                            let pixel_value = pixel_data[py * 8 + px];

                            // Skip transparent pixels (value 0) if desired
                            // if pixel_value == 0 { continue; }

                            // Calculate pixel position
                            let pixel_x = tile_x + px as f32 * pixel_size;
                            let pixel_y = tile_y + py as f32 * pixel_size;

                            // Draw the pixel as a small rectangle
                            painter.rect_filled(
                                Rect::from_min_size(egui::pos2(pixel_x, pixel_y), egui::vec2(pixel_size, pixel_size)),
                                0.0, // No rounding
                                self.colors[pixel_value as usize],
                            );
                        }
                    }

                    // Draw grid lines if enabled
                    if self.show_grid {
                        // Draw a simple grid by drawing lines directly
                        let x1 = tile_x;
                        let y1 = tile_y;
                        let x2 = tile_x + self.tile_size * self.zoom;
                        let y2 = tile_y + self.tile_size * self.zoom;

                        // Grid color
                        let grid_color = Color32::from_rgba_premultiplied(80, 80, 80, 180);

                        // Draw horizontal lines (top and bottom)
                        painter.line_segment([egui::pos2(x1, y1), egui::pos2(x2, y1)], (1.0, grid_color));
                        painter.line_segment([egui::pos2(x1, y2), egui::pos2(x2, y2)], (1.0, grid_color));

                        // Draw vertical lines (left and right)
                        painter.line_segment([egui::pos2(x1, y1), egui::pos2(x1, y2)], (1.0, grid_color));
                        painter.line_segment([egui::pos2(x2, y1), egui::pos2(x2, y2)], (1.0, grid_color));
                    }
                }
            }
        }

        Ok(response)
    }

    // Getter and setter methods for the widget properties

    /// Set the tile size (display size of each 8x8 tile)
    pub fn set_tile_size(&mut self, size: f32) {
        self.tile_size = size.max(4.0).min(64.0); // Limit to reasonable range
    }

    /// Set the zoom factor
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.max(0.25).min(4.0); // Limit to reasonable range
    }

    /// Get the current zoom factor
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Set whether to show grid lines
    pub fn set_show_grid(&mut self, show: bool) {
        self.show_grid = show;
    }

    /// Get whether grid lines are shown
    pub fn show_grid(&self) -> bool {
        self.show_grid
    }

    /// Set the current pattern table (0 or 1)
    pub fn set_current_table(&mut self, table: u8) {
        self.current_table = table.min(1); // Only allow 0 or 1
    }

    /// Get the current pattern table (0 or 1)
    pub fn current_table(&self) -> u8 {
        self.current_table
    }

    /// Set custom colors for the pattern table display
    pub fn set_colors(&mut self, colors: [Color32; 4]) {
        self.colors = colors;
    }

    /// Get the current colors used for the pattern table display
    pub fn colors(&self) -> &[Color32; 4] {
        &self.colors
    }

    /// Get the current tile size
    pub fn tile_size(&self) -> f32 {
        self.tile_size
    }

    // Builder-style methods

    /// Set the tile size and return self for chaining
    pub fn with_tile_size(mut self, size: f32) -> Self {
        self.set_tile_size(size);
        self
    }

    /// Set the zoom factor and return self for chaining
    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.set_zoom(zoom);
        self
    }

    /// Set whether to show grid lines and return self for chaining
    pub fn with_grid(mut self, show: bool) -> Self {
        self.set_show_grid(show);
        self
    }

    /// Set the current pattern table and return self for chaining
    pub fn with_table(mut self, table: u8) -> Self {
        self.set_current_table(table);
        self
    }

    /// Set custom colors and return self for chaining
    pub fn with_colors(mut self, colors: [Color32; 4]) -> Self {
        self.set_colors(colors);
        self
    }
}

impl Default for PatternTableWidget {
    fn default() -> Self {
        Self::new()
    }
}
