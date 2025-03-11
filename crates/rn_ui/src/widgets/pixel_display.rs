use egui::{Rect, Sense, Ui, Vec2};

use crate::widgets::pixel_provider::PixelDataProvider;

/// Widget for displaying pixel data from various sources (memory or PPU)
pub struct PixelDisplay {
    // Visual settings
    pixel_size: f32, // Size of each pixel
    zoom: f32,       // Zoom level
    show_grid: bool, // Whether to show grid lines
}

impl PixelDisplay {
    /// Create a new pixel display
    pub fn new() -> Self {
        Self {
            pixel_size: 2.0, // Smaller default for more practical display sizes
            zoom: 1.0,
            show_grid: true,
        }
    }

    /// Create a pixel display with custom settings
    pub fn with_settings(pixel_size: f32, zoom: f32, show_grid: bool) -> Self {
        Self {
            pixel_size,
            zoom,
            show_grid,
        }
    }

    /// Show the pixel display in the given UI
    pub fn ui<P: PixelDataProvider>(&mut self, ui: &mut Ui, provider: &P) -> egui::Response {
        ui.heading(provider.title());
        ui.label(provider.description());

        // Get the dimensions
        let width = provider.width();
        let height = provider.height();

        // Calculate display size
        let display_size = Vec2::new(
            width as f32 * self.pixel_size * self.zoom,
            height as f32 * self.pixel_size * self.zoom,
        );

        // Allocate the drawing area
        let (rect, response) = ui.allocate_exact_size(display_size, Sense::click_and_drag());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Draw background
            painter.rect_filled(rect, 0.0, egui::Color32::BLACK);

            // Get pixel data - it's already in RGB format (3 bytes per pixel)
            let pixel_data = provider.get_pixel_data();

            // Draw pixels
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) * 3;
                    if idx + 2 < pixel_data.len() {
                        // Create color from RGB values
                        let color = egui::Color32::from_rgb(pixel_data[idx], pixel_data[idx + 1], pixel_data[idx + 2]);

                        // Calculate pixel rectangle
                        let pixel_rect = Rect::from_min_size(
                            rect.min
                                + Vec2::new(
                                    x as f32 * self.pixel_size * self.zoom,
                                    y as f32 * self.pixel_size * self.zoom,
                                ),
                            Vec2::splat(self.pixel_size * self.zoom),
                        );

                        // Draw the pixel
                        painter.rect_filled(pixel_rect, 0.0, color);
                    }
                }
            }

            // Draw grid lines if enabled and pixels are large enough
            if self.show_grid && self.pixel_size * self.zoom > 4.0 {
                // Vertical grid lines
                for x in 0..=width {
                    let start = rect.min + Vec2::new(x as f32 * self.pixel_size * self.zoom, 0.0);
                    let end = start + Vec2::new(0.0, display_size.y);
                    painter.line_segment([start, end], (1.0, egui::Color32::DARK_GRAY));
                }

                // Horizontal grid lines
                for y in 0..=height {
                    let start = rect.min + Vec2::new(0.0, y as f32 * self.pixel_size * self.zoom);
                    let end = start + Vec2::new(display_size.x, 0.0);
                    painter.line_segment([start, end], (1.0, egui::Color32::DARK_GRAY));
                }
            }
        }

        response
    }

    // Getters and setters

    /// Set the pixel size
    pub fn set_pixel_size(&mut self, size: f32) {
        if size > 1.0 && size < 32.0 {
            self.pixel_size = size;
        }
    }

    /// Set the zoom level
    pub fn set_zoom(&mut self, zoom: f32) {
        if zoom > 0.1 && zoom < 10.0 {
            self.zoom = zoom;
        }
    }

    /// Set whether to show grid lines
    pub fn set_show_grid(&mut self, show: bool) {
        self.show_grid = show;
    }

    /// Set the zoom level and return self for method chaining
    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.set_zoom(zoom);
        self
    }

    /// Set the pixel size and return self for method chaining
    pub fn with_pixel_size(mut self, size: f32) -> Self {
        self.set_pixel_size(size);
        self
    }

    /// Set whether to show grid lines and return self for method chaining
    pub fn with_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    // Getters

    /// Get the current pixel size
    pub fn pixel_size(&self) -> f32 {
        self.pixel_size
    }

    /// Get the current zoom level
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Get whether grid lines are shown
    pub fn show_grid(&self) -> bool {
        self.show_grid
    }
}
