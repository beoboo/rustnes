#![allow(dead_code)]
use anyhow::Result;
use egui::{Rect, Sense, Ui, Vec2};

use crate::widgets::pixel_provider::PixelDataProvider;

/// Widget for displaying pixel data from various sources (memory or PPU)
pub struct PixelDisplay {
    // Visual settings
    pixel_size: f32,   // Size of each pixel
    zoom: f32,         // Zoom level
    show_grid: bool,   // Whether to show grid lines
    show_labels: bool, // Whether to draw the provider's title and description above the image

    /// The frame, uploaded once per repaint and drawn as a single image.
    ///
    /// This used to emit one filled rectangle per pixel: 61,440 quads per repaint for a 256x240
    /// screen, and over seven million a second on a 120 Hz display. That is far more than the UI
    /// can sustain, so repaints became irregular and the picture stuttered no matter how precisely
    /// emulation was paced. A texture makes it one quad.
    texture: Option<egui::TextureHandle>,
}

impl PixelDisplay {
    /// Create a new pixel display
    pub fn new() -> Self {
        Self {
            pixel_size: 2.0, // Smaller default for more practical display sizes
            zoom: 1.0,
            show_grid: true,
            show_labels: true,
            texture: None,
        }
    }

    /// Create a pixel display with custom settings
    pub fn with_settings(pixel_size: f32, zoom: f32, show_grid: bool) -> Self {
        Self {
            pixel_size,
            zoom,
            show_grid,
            show_labels: true,
            texture: None,
        }
    }

    /// Show the pixel display in the given UI
    pub fn ui<P: PixelDataProvider>(&mut self, ui: &mut Ui, provider: &P) -> Result<egui::Response> {
        if self.show_labels {
            ui.heading(provider.title());
            ui.label(provider.description());
        }

        // Get the dimensions
        let width = provider.width();
        let height = provider.height();

        // Calculate display size
        let display_size = self.scaled_size(width, height);

        // Allocate the drawing area
        let (rect, response) = ui.allocate_exact_size(display_size, Sense::click_and_drag());

        if ui.is_rect_visible(rect) {
            // Get pixel data - it's already in RGB format (3 bytes per pixel)
            let pixel_data = provider.get_pixel_data()?;

            let expected = width * height * 3;
            if pixel_data.len() >= expected {
                let image = egui::ColorImage::from_rgb([width, height], &pixel_data[..expected]);

                // Nearest-neighbour, so scaling keeps the hard pixel edges the NES actually has
                // rather than blurring them.
                let options = egui::TextureOptions::NEAREST;

                match &mut self.texture {
                    Some(texture) => texture.set(image, options),
                    None => {
                        self.texture = Some(ui.ctx().load_texture("pixel_display", image, options));
                    },
                }
            }

            let painter = ui.painter();

            // Draw background
            painter.rect_filled(rect, 0.0, egui::Color32::BLACK);

            if let Some(texture) = &self.texture {
                painter.image(
                    texture.id(),
                    rect,
                    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }

            // Grid lines are a debugging aid, and only legible when pixels are large. They are
            // drawn over the image rather than between pixels.
            if self.show_grid && self.pixel_size * self.zoom > 4.0 {
                for x in 0..=width {
                    let start = rect.min + Vec2::new(x as f32 * self.pixel_size * self.zoom, 0.0);
                    let end = start + Vec2::new(0.0, display_size.y);
                    painter.line_segment([start, end], (1.0, egui::Color32::DARK_GRAY));
                }

                for y in 0..=height {
                    let start = rect.min + Vec2::new(0.0, y as f32 * self.pixel_size * self.zoom);
                    let end = start + Vec2::new(display_size.x, 0.0);
                    painter.line_segment([start, end], (1.0, egui::Color32::DARK_GRAY));
                }
            }
        }

        Ok(response)
    }

    // Getters and setters

    /// Largest zoom at which a `width` x `height` image fits the space available, with a margin.
    ///
    /// The margin is the point of this. Sizing an image to exactly the available width puts it on
    /// the scrollbar threshold: filling the width makes a scrollbar appear, which shrinks the
    /// available width, which hides the scrollbar again — so the picture resizes by a few pixels
    /// on every repaint. That oscillation reads as a persistent flicker that no amount of work on
    /// emulation timing can fix, because the emulator is not involved in it.
    ///
    /// Quantising to whole steps matters for the same reason: a zoom that drifts by a fraction of
    /// a percent each repaint shimmers under nearest-neighbour sampling, as pixel boundaries land
    /// on different sides of a screen pixel from one frame to the next.
    pub fn fit_zoom(&self, ui: &Ui, width: usize, height: usize) -> f32 {
        let vertical = Self::quantise(self.fit(ui.available_height(), height));
        self.fit_zoom_width(ui, width).min(vertical)
    }

    /// As [`fit_zoom`](Self::fit_zoom), but constrained only by width.
    ///
    /// For displays that are meant to scroll vertically, where fitting the height would shrink
    /// them to nothing.
    pub fn fit_zoom_width(&self, ui: &Ui, width: usize) -> f32 {
        Self::quantise(self.fit(ui.available_width(), width))
    }

    /// On-screen size of a `width` x `height` image at the current settings.
    ///
    /// A source pixel occupies `pixel_size * zoom` screen points, not `zoom`: callers that need to
    /// reason about the drawn size — to centre it, say — must ask rather than assume, because
    /// forgetting `pixel_size` scales the answer by a factor of two at the default settings.
    pub fn scaled_size(&self, width: usize, height: usize) -> Vec2 {
        Vec2::new(
            width as f32 * self.pixel_size * self.zoom,
            height as f32 * self.pixel_size * self.zoom,
        )
    }

    /// Space to insert before a `pixels`-tall image so it sits centred in `available` points.
    ///
    /// Never negative: an image larger than the space is left flush with the top, where at least
    /// its first scanlines are visible, rather than pulled up so that both ends are cut off.
    pub fn centering_offset(&self, available: f32, pixels: usize) -> f32 {
        ((available - pixels as f32 * self.pixel_size * self.zoom) / 2.0).max(0.0)
    }

    fn fit(&self, space: f32, pixels: usize) -> f32 {
        /// Enough clearance that filling the space cannot itself summon a scrollbar.
        const MARGIN: f32 = 16.0;
        (space - MARGIN).max(0.0) / (pixels as f32 * self.pixel_size)
    }

    fn quantise(zoom: f32) -> f32 {
        const STEP: f32 = 0.25;
        ((zoom / STEP).floor() * STEP).max(STEP)
    }

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

    /// Set whether the provider's title and description are drawn above the image.
    ///
    /// Turning them off is what fullscreen wants, and it is also a sizing matter: the labels are
    /// drawn after the zoom has been measured against the available height, so their height comes
    /// straight off the bottom of the picture.
    pub fn set_show_labels(&mut self, show: bool) {
        self.show_labels = show;
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

impl Default for PixelDisplay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bug this guards against: an image sized to exactly the space available fills its
    /// container, which makes a scrollbar appear, which shrinks the space, which hides the
    /// scrollbar — resizing the picture on every repaint. It looks exactly like a flickering
    /// emulator, and no measurement of the emulator can find it.
    #[test]
    fn a_fitted_image_never_fills_the_space_it_was_measured_against() {
        let display = PixelDisplay::new();

        for space in [200.0, 512.0, 513.0, 777.0, 1024.0, 1920.0] {
            let zoom = PixelDisplay::quantise(display.fit(space, 256));
            let width = 256.0 * display.pixel_size * zoom;
            assert!(
                width < space,
                "at {space}px available the image is {width}px, which can summon a scrollbar"
            );
        }
    }

    /// Sub-pixel drift shimmers under nearest-neighbour sampling, as pixel boundaries land on
    /// different sides of a screen pixel between repaints.
    #[test]
    fn zoom_moves_in_whole_steps() {
        let display = PixelDisplay::new();
        for space in [600.0, 601.0, 602.0, 603.0] {
            let zoom = PixelDisplay::quantise(display.fit(space, 256));
            assert_eq!(zoom % 0.25, 0.0, "{zoom} is not a whole step");
        }
    }

    #[test]
    fn a_space_too_small_to_fit_still_yields_a_usable_zoom() {
        let display = PixelDisplay::new();
        assert!(PixelDisplay::quantise(display.fit(0.0, 256)) > 0.0);
    }

    /// The bug this guards against: fullscreen centred the picture with `240.0 * zoom`, leaving
    /// `pixel_size` out. At the default pixel size of 2 that is half the height the image actually
    /// occupies, so the offset was twice what centring called for and the picture was pushed down
    /// until its bottom half fell off the screen — cut in half and not centred at once.
    #[test]
    fn centring_leaves_as_much_space_below_the_image_as_above_it() {
        let mut display = PixelDisplay::new();

        // A 1080p screen with the picture fitted to it, overscan on and off.
        for lines in [240, 224] {
            let space = 1080.0;
            display.set_zoom(PixelDisplay::quantise(display.fit(space, lines)));

            let drawn = display.scaled_size(256, lines).y;
            let above = display.centering_offset(space, lines);
            let below = space - above - drawn;

            assert!(below >= 0.0, "{lines} lines: the image overruns the space by {}px", -below);
            assert!(
                (above - below).abs() < 1.0,
                "{lines} lines: {above}px above but {below}px below the picture"
            );
        }
    }

    /// An image too big for the space is left flush with the top, where its first scanlines are
    /// still visible, rather than pulled up so that both ends are cut off.
    #[test]
    fn centring_never_moves_an_oversized_image_off_the_top() {
        let display = PixelDisplay::new();
        assert_eq!(display.centering_offset(100.0, 240), 0.0);
    }
}
