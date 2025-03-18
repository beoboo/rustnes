/// System-level components for coordinating between different subsystems
///
/// This module contains components that aren't specific to any one subsystem
/// but instead coordinate between multiple systems.
pub mod bus;
pub mod dma;
pub mod nes_system;

pub use bus::Bus;
pub use dma::DmaController;
pub use nes_system::{NesSystem, SystemState};
