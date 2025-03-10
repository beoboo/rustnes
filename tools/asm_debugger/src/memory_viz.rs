use egui::{Rect, Sense, Ui, Vec2};

/// Memory visualization that displays a range of memory as pixels
pub struct MemoryVisualizer {
    // Memory range to visualize
    start_addr: u16,  // Start address (inclusive)
    end_addr: u16,    // End address (inclusive)
    width: usize,     // Width of visualization in pixels
    
    // Visual settings
    pixel_size: f32,  // Size of each memory "pixel"
    zoom: f32,        // Zoom level
}

impl MemoryVisualizer {
    /// Create a new memory visualizer
    pub fn new() -> Self {
        Self {
            // Default to video memory range 0x0200-0x05FF
            start_addr: 0x0200,
            end_addr: 0x05FF,
            width: 32, // 32 pixels wide display
            
            // Default visual settings
            pixel_size: 8.0,
            zoom: 1.0,
        }
    }
    
    /// Show the memory visualization in the given UI
    pub fn show(&mut self, ui: &mut Ui, memory: &[u8]) -> egui::Response {
        // Calculate dimensions based on memory range and settings
        let memory_size = (self.end_addr - self.start_addr + 1) as usize;
        let height = (memory_size + self.width - 1) / self.width; // Ceiling division
        
        // Calculate display size
        let display_size = Vec2::new(
            self.width as f32 * self.pixel_size * self.zoom,
            height as f32 * self.pixel_size * self.zoom,
        );
        
        // Allocate the drawing area
        let (rect, response) = ui.allocate_exact_size(display_size, Sense::click_and_drag());
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // Draw background
            painter.rect_filled(rect, 0.0, egui::Color32::BLACK);
            
            // Draw memory pixels
            for i in 0..memory_size {
                if let Some(memory_value) = memory.get((self.start_addr as usize) + i) {
                    // Calculate position in grid
                    let x = (i % self.width) as f32;
                    let y = (i / self.width) as f32;
                    
                    // Calculate pixel position and size
                    let pixel_rect = Rect::from_min_size(
                        rect.min + Vec2::new(x * self.pixel_size * self.zoom, y * self.pixel_size * self.zoom),
                        Vec2::splat(self.pixel_size * self.zoom),
                    );
                    
                    // Convert memory value to color (simple grayscale for now)
                    let color = egui::Color32::from_gray(*memory_value);
                    
                    // Draw the pixel
                    painter.rect_filled(pixel_rect, 0.0, color);
                }
            }
            
            // Optional: Draw grid lines
            if self.pixel_size * self.zoom > 4.0 {
                for x in 0..=self.width {
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
        
        response
    }
    
    // Getters and setters
    pub fn set_range(&mut self, start: u16, end: u16) {
        self.start_addr = start;
        self.end_addr = end;
    }
    
    pub fn set_width(&mut self, width: usize) {
        if width > 0 {
            self.width = width;
        }
    }
    
    pub fn set_zoom(&mut self, zoom: f32) {
        if zoom > 0.1 && zoom < 10.0 {
            self.zoom = zoom;
        }
    }
    
    pub fn set_pixel_size(&mut self, size: f32) {
        if size > 1.0 && size < 32.0 {
            self.pixel_size = size;
        }
    }
} 