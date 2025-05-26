#![allow(dead_code)]
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use egui::{epaint, pos2, Color32, CornerRadius, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use rn_core::audio::SampleConsumer;

/// Widget for visualizing audio waveforms in real-time
pub struct WaveformWidget {
    // Visual settings
    width: usize,    // Width of visualization in pixels
    height: usize,   // Height of visualization
    pixel_size: f32, // Size of each sample point
    zoom: f32,       // Zoom level

    // Sample buffer to store waveform data
    samples: VecDeque<f32>,
    max_samples: usize,

    // Last mixed sample for display
    last_mixed_sample: f32,

    consumer: Box<dyn SampleConsumer<f32>>,
}

impl WaveformWidget {
    /// Create a new waveform visualizer widget for real-time output
    pub fn new(consumer: Box<dyn SampleConsumer<f32>>) -> Self {
        let max_samples = 256;
        
        Self {
            width: max_samples,
            height: 100,
            pixel_size: 2.0,
            zoom: 1.0,
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
            last_mixed_sample: 0.0,
            consumer,
        }
    }

    /// Add new samples to the visualizer directly
    pub fn add_samples(&mut self, new_samples: &[f32]) {
        for &sample in new_samples {
            self.samples.push_back(sample);
            if self.samples.len() > self.max_samples {
                self.samples.pop_front();
            }
        }

        // Update last mixed sample for display
        if let Some(&last) = new_samples.last() {
            self.last_mixed_sample = last;
        }
    }

    /// Show the waveform visualization in the given UI
    pub fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.heading("Audio Output Waveform");

        // Calculate display size
        let display_size = Vec2::new(
            self.width as f32 * self.pixel_size * self.zoom,
            self.height as f32 * self.pixel_size,
        );

        // Allocate the drawing area
        let (rect, response) = ui.allocate_exact_size(display_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Draw background
            painter.rect_filled(rect, 0.0, Color32::from_rgb(20, 20, 40));

            // Draw horizontal center line
            let center_y = rect.min.y + rect.height() / 2.0;
            painter.line_segment(
                [pos2(rect.min.x, center_y), pos2(rect.max.x, center_y)],
                Stroke::new(1.0, Color32::from_gray(100)),
            );

            let mut samples = Vec::new();
            while let Some(sample) = self.consumer.consume() {
                samples.push(sample);
            }
            self.add_samples(&samples);
            let samples = self.samples.iter().cloned().collect::<Vec<_>>();

            // Draw samples as a connected line using epaint::Shape
            if samples.len() >= 2 {
                let mut points = Vec::with_capacity(samples.len());

                // Find max amplitude for normalization if needed
                let max_amplitude = samples.iter()
                    .map(|s| s.abs())
                    .fold(0.0f32, |max, val| max.max(val))
                    .max(1.0); // Ensure we don't divide by zero

                // Only apply scaling if samples exceed [-1.0, 1.0] range
                let scale_factor = if max_amplitude > 1.0 { 1.0 / max_amplitude } else { 1.0 };

                for (i, &sample) in samples.iter().enumerate().take(self.width) {
                    let x = rect.min.x + (i as f32 * self.pixel_size * self.zoom);
                    
                    // Apply normalization to ensure all waveforms fit within the display
                    let normalized_sample = sample * scale_factor;
                    
                    // Invert Y because in egui, Y increases downward
                    let y = center_y - (normalized_sample * rect.height() / 2.0);
                    
                    points.push(pos2(x, y));
                }

                // Draw the waveform as a single shape
                painter.add(epaint::Shape::line(
                    points,
                    Stroke::new(2.0, Color32::from_rgb(100, 255, 100)),
                ));
            }

            // Draw border using lines
            let stroke = Stroke::new(1.0, Color32::from_gray(80));
            // Top line
            painter.line_segment([pos2(rect.min.x, rect.min.y), pos2(rect.max.x, rect.min.y)], stroke);
            // Bottom line
            painter.line_segment([pos2(rect.min.x, rect.max.y), pos2(rect.max.x, rect.max.y)], stroke);
            // Left line
            painter.line_segment([pos2(rect.min.x, rect.min.y), pos2(rect.min.x, rect.max.y)], stroke);
            // Right line
            painter.line_segment([pos2(rect.max.x, rect.min.y), pos2(rect.max.x, rect.max.y)], stroke);
        }

        // Request continuous updates to keep the visualization animated
        ui.ctx().request_repaint();

        response
    }
}
