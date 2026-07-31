#![allow(dead_code)]
use std::cell::RefMut;

use egui::{Grid, RichText, Ui};
use rn_core::ppu::{self, PpuWrapper};

use crate::widgets::{HexEditText, ValueType};

/// Widget for displaying PPU state
#[derive(Default)]
pub struct PpuWidget {
    // Register edit widgets
    ctrl_register: HexEditText,
    mask_register: HexEditText,
    status_register: HexEditText,
    oam_addr_register: HexEditText,
    scroll_x_register: HexEditText,
    scroll_y_register: HexEditText,
    ppu_addr_register: HexEditText,
}


impl PpuWidget {
    /// Create a new PPU widget
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the PPU widget using the given UI and PPU
    pub fn ui(&mut self, ui: &mut Ui, ppu: PpuWrapper) {
        ui.heading("PPU State");

        // Add debug button to show current register values
        if ui.button("Show Current Values").clicked() {
            let ctrl = ppu.ctrl();
            let mask = ppu.mask();
            let status = ppu.status();

            log::info!(
                "PPU Current Values - CTRL: ${:02X}, MASK: ${:02X}, STATUS: ${:02X}",
                ctrl,
                mask,
                status
            );

            // Show in UI too
            ui.label(format!(
                "CTRL: ${:02X}, MASK: ${:02X}, STATUS: ${:02X}",
                ctrl, mask, status
            ));
            ui.label(format!(
                "Show sprites: {}, Show bg: {}",
                (mask & ppu::MASK_SHOW_SPRITES) != 0,
                (mask & ppu::MASK_SHOW_BACKGROUND) != 0
            ));
        }

        // Access PPU registers and internal state through methods
        let frame_count = ppu.frame_count();
        let (scanline, cycle) = ppu.scanline_cycle();

        // Display frame, scanline, and cycle information
        Grid::new("ppu_frame_info_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Frame Count:");
                ui.label(format!("{}", frame_count));
                ui.end_row();

                ui.label("Scanline:");
                ui.label(format!("{}", scanline));
                ui.end_row();

                ui.label("Cycle:");
                ui.label(format!("{}", cycle));
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.heading("PPU Registers");

        // Display register values
        Grid::new("ppu_registers_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                // Control Register ($2000)
                let mut ctrl = ppu.ctrl() as u16;
                if self.ctrl_register.ui(
                    ui,
                    "CTRL ($2000):",
                    &mut ctrl,
                    ValueType::Bit8,
                    Some("PPU Control Register"),
                ) {
                    ppu.set_ctrl(ctrl as u8);
                }
                ui.end_row();

                // Mask Register ($2001)
                let mut mask = ppu.mask() as u16;
                if self.mask_register.ui(
                    ui,
                    "MASK ($2001):",
                    &mut mask,
                    ValueType::Bit8,
                    Some("PPU Mask Register"),
                ) {
                    ppu.set_mask(mask as u8);
                }
                ui.end_row();

                // Status Register ($2002) - Read-only
                ui.label("STATUS ($2002):");
                ui.label(format!("${:02X}", ppu.status()));
                ui.end_row();

                // OAM Address ($2003)
                let mut oam_addr = ppu.oam_addr() as u16;
                if self.oam_addr_register.ui(
                    ui,
                    "OAM ADDR ($2003):",
                    &mut oam_addr,
                    ValueType::Bit8,
                    Some("OAM Address Register"),
                ) {
                    ppu.set_oam_addr(oam_addr as u8);
                }
                ui.end_row();

                // Scroll X/Y ($2005)
                let mut scroll_x = ppu.scroll_x() as u16;
                if self.scroll_x_register.ui(
                    ui,
                    "SCROLL X ($2005.1):",
                    &mut scroll_x,
                    ValueType::Bit8,
                    Some("PPU Scroll X Register"),
                ) {
                    ppu.set_scroll_x(scroll_x as u8);
                }
                ui.end_row();

                let mut scroll_y = ppu.scroll_y() as u16;
                if self.scroll_y_register.ui(
                    ui,
                    "SCROLL Y ($2005.2):",
                    &mut scroll_y,
                    ValueType::Bit8,
                    Some("PPU Scroll Y Register"),
                ) {
                    ppu.set_scroll_y(scroll_y as u8);
                }
                ui.end_row();

                // PPU Address ($2006)
                let mut ppu_addr = ppu.ppu_addr();
                if self.ppu_addr_register.ui(
                    ui,
                    "PPU ADDR ($2006):",
                    &mut ppu_addr,
                    ValueType::Bit16,
                    Some("PPU Address Register"),
                ) {
                    ppu.set_ppu_addr(ppu_addr);
                }
                ui.end_row();
            });

        // Display CTRL register flag details
        ui.add_space(8.0);
        ui.heading("CTRL Register Flags");

        let ctrl = ppu.ctrl();
        ui.horizontal(|ui| {
            // Set spacing to be very compact
            let original_spacing = ui.spacing().clone();
            ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);

            // Define CTRL register flags
            let ctrl_flags = [
                (ppu::CTRL_NMI_ENABLE, "NMI\nEnable", "Generate NMI at start of vblank"),
                (ppu::CTRL_MASTER_SLAVE, "Master/\nSlave", "Not used in NES"),
                (ppu::CTRL_SPRITE_SIZE, "Sprite\nSize", "0: 8x8 sprites, 1: 8x16 sprites"),
                (ppu::CTRL_BACKGROUND_PATTERN, "BG\nPattern", "0: $0000, 1: $1000"),
                (ppu::CTRL_SPRITE_PATTERN, "Sprite\nPattern", "0: $0000, 1: $1000"),
                (ppu::CTRL_INCREMENT_MODE, "Addr\nIncrement", "0: +1, 1: +32"),
                (ppu::CTRL_NAMETABLE_Y, "NT\nY", "0: $2000/$2400, 1: $2800/$2C00"),
                (ppu::CTRL_NAMETABLE_X, "NT\nX", "0: $2000/$2800, 1: $2400/$2C00"),
            ];

            for &(mask, label, tooltip) in &ctrl_flags {
                ui.vertical(|ui| {
                    ui.set_max_width(40.0);

                    // Flag name
                    ui.with_layout(
                        egui::Layout::top_down_justified(egui::Align::Center).with_cross_align(egui::Align::Center),
                        |ui| {
                            ui.label(RichText::new(label).text_style(egui::TextStyle::Small));

                            // Flag status (checkbox)
                            let mut checked = (ctrl & mask) != 0;
                            let response = ui.checkbox(&mut checked, "").on_hover_text(tooltip);

                            if response.changed() {
                                if checked {
                                    ppu.set_ctrl(ctrl | mask);
                                } else {
                                    ppu.set_ctrl(ctrl & !mask);
                                }
                            }
                        },
                    );
                });
            }

            // Restore original spacing
            *ui.spacing_mut() = original_spacing;
        });

        // Display MASK register flag details
        ui.add_space(8.0);
        ui.heading("MASK Register Flags");

        let mask = ppu.mask();
        ui.horizontal(|ui| {
            // Set spacing to be very compact
            let original_spacing = ui.spacing().clone();
            ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);

            // Define MASK register flags
            let mask_flags = [
                (ppu::MASK_EMPHASIZE_BLUE, "\nBlue", "Emphasize blue"),
                (ppu::MASK_EMPHASIZE_GREEN, "\nGreen", "Emphasize green"),
                (ppu::MASK_EMPHASIZE_RED, "\nRed", "Emphasize red"),
                (ppu::MASK_SHOW_SPRITES, "Show\nSprites", "Show sprites"),
                (ppu::MASK_SHOW_BACKGROUND, "Show\nBG", "Show background"),
                (
                    ppu::MASK_SHOW_LEFT_SPRITES,
                    "Left\nSprites",
                    "Show sprites in leftmost 8 pixels",
                ),
                (
                    ppu::MASK_SHOW_LEFT_BACKGROUND,
                    "Left\nBG",
                    "Show background in leftmost 8 pixels",
                ),
                (ppu::MASK_GRAYSCALE, "Gray\nscale", "0: Color, 1: Grayscale"),
            ];

            for &(mask_bit, label, tooltip) in &mask_flags {
                ui.vertical(|ui| {
                    ui.set_max_width(28.0);

                    // Flag name
                    ui.with_layout(
                        egui::Layout::top_down_justified(egui::Align::Center).with_cross_align(egui::Align::Center),
                        |ui| {
                            ui.label(RichText::new(label).text_style(egui::TextStyle::Small));

                            // Flag status (checkbox)
                            let mut checked = (mask & mask_bit) != 0;
                            let response = ui.checkbox(&mut checked, "").on_hover_text(tooltip);

                            if response.changed() {
                                if checked {
                                    ppu.set_mask(mask | mask_bit);
                                } else {
                                    ppu.set_mask(mask & !mask_bit);
                                }
                            }
                        },
                    );
                });
            }

            // Restore original spacing
            *ui.spacing_mut() = original_spacing;
        });

        // Display STATUS register flag details
        ui.add_space(8.0);
        ui.heading("STATUS Register Flags (Read-only)");

        let status = ppu.status();
        ui.horizontal(|ui| {
            // Set spacing to be very compact
            let original_spacing = ui.spacing().clone();
            ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);

            // Define STATUS register flags - note, these are read-only
            let status_flags = [
                (ppu::STATUS_VBLANK, "\nVBlank", "In vblank"),
                (ppu::STATUS_SPRITE_ZERO_HIT, "Sprite 0\nHit", "Sprite 0 hit occurred"),
                (
                    ppu::STATUS_SPRITE_OVERFLOW,
                    "Sprite\nOverflow",
                    "Sprite overflow occurred",
                ),
                (0x10, "\nUnused", "Unused bit 4"),
                (0x08, "\nUnused", "Unused bit 3"),
                (0x04, "\nUnused", "Unused bit 2"),
                (0x02, "\nUnused", "Unused bit 1"),
                (0x01, "\nUnused", "Unused bit 0"),
            ];

            for &(mask_bit, label, tooltip) in &status_flags {
                ui.vertical(|ui| {
                    ui.set_max_width(40.0);

                    // Flag name
                    ui.with_layout(
                        egui::Layout::top_down_justified(egui::Align::Center).with_cross_align(egui::Align::Center),
                        |ui| {
                            ui.label(RichText::new(label).text_style(egui::TextStyle::Small));

                            // Flag status (display only, not editable)
                            let checked = (status & mask_bit) != 0;
                            let mut dummy = checked;
                            ui.add_enabled_ui(false, |ui| {
                                ui.checkbox(&mut dummy, "").on_hover_text(tooltip);
                            });
                        },
                    );
                });
            }

            // Restore original spacing
            *ui.spacing_mut() = original_spacing;
        });
    }
}
