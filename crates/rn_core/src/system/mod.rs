/// System-level components for coordinating between different subsystems
///
/// This module contains components that aren't specific to any one subsystem
/// but instead coordinate between multiple systems.
mod bus;

pub use bus::Bus;
