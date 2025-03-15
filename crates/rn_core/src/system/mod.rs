/// System-level components for coordinating between different subsystems
///
/// This module contains components that aren't specific to any one subsystem
/// but instead coordinate between multiple systems.
mod bus;
mod nes_system;
mod dma;

pub use bus::Bus;
pub use nes_system::{NesSystem, SystemState};
pub use dma::DmaController;
