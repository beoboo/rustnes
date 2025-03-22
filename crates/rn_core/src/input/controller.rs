use std::{
    cell::{Cell, RefCell},
    fmt::Debug,
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use crate::{errors::NesError, memory::Addressable};

/// Strobe latch shared between controllers
/// When set, continuously loads controller state
#[derive(Debug, Clone)]
pub struct StrobeLatch {
    value: Cell<bool>,
}

impl StrobeLatch {
    /// Create a new strobe latch (initially off)
    pub fn new() -> Self {
        Self {
            value: Cell::new(false),
        }
    }

    /// Set the strobe value
    pub fn set(&self, value: bool) {
        self.value.set(value)
    }

    /// Get the current strobe value
    pub fn get(&self) -> bool {
        self.value.get()
    }

    /// Update the strobe based on a write to the controller register
    /// Returns true if the strobe state changed
    pub fn update_from_write(&self, value: u8) -> bool {
        let new_strobe = (value & 0x01) == 0x01;
        let old_strobe = self.value.get();

        // Update the state if needed
        if new_strobe != old_strobe {
            self.value.set(new_strobe);
            true
        } else {
            false
        }
    }
}

/// Controller buttons as bit flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControllerState {
    /// Button states (1 = pressed, 0 = released)
    /// Bit 0: A
    /// Bit 1: B
    /// Bit 2: Select
    /// Bit 3: Start
    /// Bit 4: Up
    /// Bit 5: Down
    /// Bit 6: Left
    /// Bit 7: Right
    button_state: u8,
}

impl ControllerState {
    /// Creates a new controller state with no buttons pressed
    pub fn new() -> Self {
        Self { button_state: 0 }
    }

    /// Sets a button state
    pub fn set_button(&mut self, button: ControllerButton, pressed: bool) {
        let mask = button as u8;
        if pressed {
            self.button_state |= mask;
        } else {
            self.button_state &= !mask;
        }
    }

    /// Gets a button state
    pub fn is_button_pressed(&self, button: ControllerButton) -> bool {
        let mask = button as u8;
        (self.button_state & mask) != 0
    }

    /// Returns the raw button state
    pub fn raw_state(&self) -> u8 {
        self.button_state
    }
}

/// Controller buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ControllerButton {
    A = 0b00000001,
    B = 0b00000010,
    Select = 0b00000100,
    Start = 0b00001000,
    Up = 0b00010000,
    Down = 0b00100000,
    Left = 0b01000000,
    Right = 0b10000000,
}

/// Controller port numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerPort {
    One = 1,
    Two = 2,
}

/// A single NES controller (port 1 or 2)
#[derive(Debug)]
pub struct Controller {
    /// Port number (1 or 2)
    port: ControllerPort,

    /// Current controller state
    controller_state: ControllerState,

    /// Shift register for reading controller state one bit at a time
    shift_register: Cell<u8>,
}

impl Controller {
    /// Creates a new controller for the specified port
    pub fn new(port: ControllerPort) -> Self {
        Self {
            port,
            controller_state: ControllerState::new(),
            shift_register: Cell::new(0),
        }
    }

    /// Updates the controller state
    pub fn set_state(&mut self, state: ControllerState) {
        self.controller_state = state;
    }

    /// Latch the current controller state to the shift register
    pub fn latch_state(&self) {
        self.shift_register.set(self.controller_state.raw_state());
    }

    /// Read the current button state
    pub fn read_button(&self, strobe_active: bool) -> u8 {
        // Extract the low bit of the shift register
        let result = self.shift_register.get() & 0x01;

        // If strobe is not active, shift the register
        if !strobe_active {
            let current = self.shift_register.get();
            let shifted = (current >> 1) | 0x80; // Shift right and set high bit
            self.shift_register.set(shifted);
        } else {
            // If strobe is active, continuously reload shift register
            self.shift_register.set(self.controller_state.raw_state());
        }

        // Return only the low bit, with the rest as 0
        // Real hardware has open bus behavior, but we'll simplify for now
        result
    }

    /// Get the controller port
    pub fn port(&self) -> ControllerPort {
        self.port
    }
}

/// Handler for both NES controllers that implements memory-mapped I/O
#[derive(Debug)]
pub struct ControllerHandler {
    /// Controller for port 1 ($4016)
    controller1: Controller,

    /// Controller for port 2 ($4017)
    controller2: Controller,

    /// Shared strobe latch used by both controllers
    strobe: StrobeLatch,
}

impl ControllerHandler {
    /// Creates a new controller handler
    pub fn new() -> Self {
        Self {
            controller1: Controller::new(ControllerPort::One),
            controller2: Controller::new(ControllerPort::Two),
            strobe: StrobeLatch::new(),
        }
    }

    /// Set the state of controller 1
    pub fn set_controller1_state(&mut self, state: ControllerState) {
        self.controller1.set_state(state);
        // If strobe is active, latch state immediately
        if self.strobe.get() {
            self.controller1.latch_state();
        }
    }

    /// Set the state of controller 2
    pub fn set_controller2_state(&mut self, state: ControllerState) {
        self.controller2.set_state(state);
        // If strobe is active, latch state immediately
        if self.strobe.get() {
            self.controller2.latch_state();
        }
    }

    /// Process a write to update the strobe
    fn process_strobe_write(&mut self, value: u8) {
        let strobe_changed = self.strobe.update_from_write(value);

        // If strobe state changed, latch both controllers
        if strobe_changed {
            self.controller1.latch_state();
            self.controller2.latch_state();
        }
    }
}

impl Addressable for ControllerHandler {
    fn handles_address(&self, address: u16) -> bool {
        // Handle controller registers at $4016 and $4017
        address == 0x4016 || address == 0x4017
    }

    fn read_byte(&self, address: u16) -> Result<u8, NesError> {
        match address {
            0x4016 => Ok(self.controller1.read_button(self.strobe.get())),
            0x4017 => Ok(self.controller2.read_button(self.strobe.get())),
            _ => Err(NesError::MemoryAccessError(address)),
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError> {
        match address {
            0x4016 => {
                // Write to controller 1 register updates the strobe
                self.process_strobe_write(value);
                Ok(())
            },
            0x4017 => {
                // Writing to $4017 is used for APU frame counter control
                // We'll ignore it for now as it's not relevant to the controller
                Ok(())
            },
            _ => Err(NesError::MemoryAccessError(address)),
        }
    }
}

/// Wrapper for the ControllerHandler to make it easier to use with the bus
#[derive(Debug, Clone)]
pub struct ControllerHandlerWrapper {
    handler: Rc<RefCell<ControllerHandler>>,
}

impl ControllerHandlerWrapper {
    /// Create a new controller handler wrapper
    pub fn new() -> Self {
        Self {
            handler: Rc::new(RefCell::new(ControllerHandler::new())),
        }
    }

    /// Set the state of controller 1
    pub fn set_controller1_state(&self, state: ControllerState) {
        self.handler.borrow_mut().set_controller1_state(state);
    }

    /// Set the state of controller 2
    pub fn set_controller2_state(&self, state: ControllerState) {
        self.handler.borrow_mut().set_controller2_state(state);
    }
}

impl Addressable for ControllerHandlerWrapper {
    fn handles_address(&self, address: u16) -> bool {
        self.handler.borrow().handles_address(address)
    }

    fn read_byte(&self, address: u16) -> Result<u8, NesError> {
        self.handler.borrow().read_byte(address)
    }

    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError> {
        self.handler.borrow_mut().write_byte(address, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_button_state() {
        let mut state = ControllerState::new();

        // Initially no buttons are pressed
        assert_eq!(state.raw_state(), 0);

        // Press A button
        state.set_button(ControllerButton::A, true);
        assert_eq!(state.raw_state(), 0b00000001);
        assert!(state.is_button_pressed(ControllerButton::A));

        // Press B button
        state.set_button(ControllerButton::B, true);
        assert_eq!(state.raw_state(), 0b00000011);
        assert!(state.is_button_pressed(ControllerButton::A));
        assert!(state.is_button_pressed(ControllerButton::B));

        // Release A button
        state.set_button(ControllerButton::A, false);
        assert_eq!(state.raw_state(), 0b00000010);
        assert!(!state.is_button_pressed(ControllerButton::A));
        assert!(state.is_button_pressed(ControllerButton::B));
    }

    #[test]
    fn test_controller_input_sequence() -> Result<(), NesError> {
        // Create a controller handler
        let mut handler = ControllerHandler::new();

        // First test - read while nothing is pressed
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // A should be 0
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // B should be 0

        // Set up A and B pressed for controller 1
        let mut state = ControllerState::new();
        state.set_button(ControllerButton::A, true);
        state.set_button(ControllerButton::B, true);
        handler.set_controller1_state(state);

        // Standard controller reading sequence
        // First, strobe on
        handler.write_byte(0x4016, 0x01)?;

        // With strobe on, should get A button state repeatedly
        assert_eq!(handler.read_byte(0x4016)?, 0x01); // A is pressed (1)
        assert_eq!(handler.read_byte(0x4016)?, 0x01); // Still A with strobe on

        // Now strobe off to latch the state
        handler.write_byte(0x4016, 0x00)?;

        // Read all 8 button states one by one
        assert_eq!(handler.read_byte(0x4016)?, 0x01); // A button (pressed)
        assert_eq!(handler.read_byte(0x4016)?, 0x01); // B button (pressed)
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // Select (not pressed)
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // Start (not pressed)
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // Up (not pressed)
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // Down (not pressed)
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // Left (not pressed)
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // Right (not pressed)

        // The 9th read and beyond should return 1 (open bus behavior)
        assert_eq!(handler.read_byte(0x4016)?, 0x01);
        assert_eq!(handler.read_byte(0x4016)?, 0x01);

        // Now test that changing state mid-reading doesn't affect latched values
        // Strobe to latch the state again
        handler.write_byte(0x4016, 0x01)?;
        handler.write_byte(0x4016, 0x00)?;

        // Read the first button (should be A pressed)
        assert_eq!(handler.read_byte(0x4016)?, 0x01);

        // Now change the controller state while reading is in progress
        let mut new_state = ControllerState::new();
        new_state.set_button(ControllerButton::Start, true); // Only Start is pressed
        handler.set_controller1_state(new_state);

        // Continue reading - should still see old latched values
        assert_eq!(handler.read_byte(0x4016)?, 0x01); // B was pressed in latched state
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // Select was not pressed

        // Start a new reading sequence
        handler.write_byte(0x4016, 0x01)?;
        handler.write_byte(0x4016, 0x00)?;

        // Now we should see the new state
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // A not pressed
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // B not pressed
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // Select not pressed
        assert_eq!(handler.read_byte(0x4016)?, 0x01); // Start pressed

        Ok(())
    }

    #[test]
    fn test_controller2_independence() -> Result<(), NesError> {
        // Create a controller handler
        let mut handler = ControllerHandler::new();

        // Set up different states for controller 1 and 2
        let mut state1 = ControllerState::new();
        state1.set_button(ControllerButton::A, true);

        let mut state2 = ControllerState::new();
        state2.set_button(ControllerButton::B, true);

        handler.set_controller1_state(state1);
        handler.set_controller2_state(state2);

        // Strobe to latch both controllers
        handler.write_byte(0x4016, 0x01)?;
        handler.write_byte(0x4016, 0x00)?;

        // Read from controller 1
        assert_eq!(handler.read_byte(0x4016)?, 0x01); // A pressed
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // B not pressed

        // Read from controller 2
        assert_eq!(handler.read_byte(0x4017)?, 0x00); // A not pressed
        assert_eq!(handler.read_byte(0x4017)?, 0x01); // B pressed

        // Make sure controllers maintain independent button states
        for _ in 0..6 {
            // Skip remaining buttons
            handler.read_byte(0x4016)?;
            handler.read_byte(0x4017)?;
        }

        // Set new states
        let mut new_state1 = ControllerState::new();
        new_state1.set_button(ControllerButton::Start, true);

        let mut new_state2 = ControllerState::new();
        new_state2.set_button(ControllerButton::Select, true);

        handler.set_controller1_state(new_state1);
        handler.set_controller2_state(new_state2);

        // Strobe again
        handler.write_byte(0x4016, 0x01)?;
        handler.write_byte(0x4016, 0x00)?;

        // Skip A and B
        handler.read_byte(0x4016)?;
        handler.read_byte(0x4016)?;
        handler.read_byte(0x4017)?;
        handler.read_byte(0x4017)?;

        // Verify Select and Start
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // Select not pressed on controller 1
        assert_eq!(handler.read_byte(0x4016)?, 0x01); // Start pressed on controller 1

        assert_eq!(handler.read_byte(0x4017)?, 0x01); // Select pressed on controller 2
        assert_eq!(handler.read_byte(0x4017)?, 0x00); // Start not pressed on controller 2

        Ok(())
    }

    #[test]
    fn test_controller_handler() -> Result<(), NesError> {
        let mut handler = ControllerHandler::new();

        // Test address handling
        assert!(handler.handles_address(0x4016));
        assert!(handler.handles_address(0x4017));
        assert!(!handler.handles_address(0x4018));

        // Set up state for controller 1
        let mut state1 = ControllerState::new();
        state1.set_button(ControllerButton::A, true);
        state1.set_button(ControllerButton::B, true);
        handler.set_controller1_state(state1);

        // Set up state for controller 2
        let mut state2 = ControllerState::new();
        state2.set_button(ControllerButton::Start, true);
        handler.set_controller2_state(state2);

        // Write to strobe to latch controller state
        handler.write_byte(0x4016, 0x01)?; // Strobe on
        handler.write_byte(0x4016, 0x00)?; // Strobe off

        // Read controller 1 state bit by bit
        assert_eq!(handler.read_byte(0x4016)?, 0x01); // A button
        assert_eq!(handler.read_byte(0x4016)?, 0x01); // B button
                                                      // Rest of buttons are not pressed (0)
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // Select
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // Start

        // Read controller 2 state bit by bit
        assert_eq!(handler.read_byte(0x4017)?, 0x00); // A button
        assert_eq!(handler.read_byte(0x4017)?, 0x00); // B button
        assert_eq!(handler.read_byte(0x4017)?, 0x00); // Select
        assert_eq!(handler.read_byte(0x4017)?, 0x01); // Start

        Ok(())
    }

    #[test]
    fn test_strobe_behavior() -> Result<(), NesError> {
        let mut handler = ControllerHandler::new();

        // Setup controller 1 with A pressed
        let mut state = ControllerState::new();
        state.set_button(ControllerButton::A, true);
        handler.set_controller1_state(state);

        // Turn on strobe
        handler.write_byte(0x4016, 0x01)?;

        // With strobe on, should repeatedly read the same value
        assert_eq!(handler.read_byte(0x4016)?, 0x01); // A button
        assert_eq!(handler.read_byte(0x4016)?, 0x01); // Still A button

        // Update controller state while strobe is active
        let mut new_state = ControllerState::new();
        new_state.set_button(ControllerButton::B, true);
        handler.set_controller1_state(new_state);

        // Should immediately see the new state since strobe is active
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // No A button
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // No A button

        // Turn off strobe to "freeze" the state
        handler.write_byte(0x4016, 0x00)?;

        // Should read frozen state one bit at a time
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // A (not pressed)
        assert_eq!(handler.read_byte(0x4016)?, 0x01); // B (pressed)
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // Select

        // Change state after strobe is off - should not affect readings
        let mut newer_state = ControllerState::new();
        newer_state.set_button(ControllerButton::Start, true);
        handler.set_controller1_state(newer_state);

        // Should continue reading old frozen state
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // Start (was not pressed in frozen state)

        Ok(())
    }

    #[test]
    fn test_controller_handler_wrapper() -> Result<(), NesError> {
        let handler = ControllerHandlerWrapper::new();

        // Test address handling
        assert!(handler.handles_address(0x4016));
        assert!(handler.handles_address(0x4017));
        assert!(!handler.handles_address(0x4018));

        // Set up state for controller 1
        let mut state1 = ControllerState::new();
        state1.set_button(ControllerButton::A, true);
        state1.set_button(ControllerButton::B, true);
        handler.set_controller1_state(state1);

        // Set up state for controller 2
        let mut state2 = ControllerState::new();
        state2.set_button(ControllerButton::Start, true);
        handler.set_controller2_state(state2);

        // Write to strobe to latch controller state
        let mut handler_clone = handler.clone();
        handler_clone.write_byte(0x4016, 0x01)?; // Strobe on
        handler_clone.write_byte(0x4016, 0x00)?; // Strobe off

        // Read controller 1 state bit by bit
        assert_eq!(handler.read_byte(0x4016)?, 0x01); // A button
        assert_eq!(handler.read_byte(0x4016)?, 0x01); // B button
                                                      // Rest of buttons are not pressed (0)
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // Select
        assert_eq!(handler.read_byte(0x4016)?, 0x00); // Start

        // Read controller 2 state bit by bit
        assert_eq!(handler.read_byte(0x4017)?, 0x00); // A button
        assert_eq!(handler.read_byte(0x4017)?, 0x00); // B button
        assert_eq!(handler.read_byte(0x4017)?, 0x00); // Select
        assert_eq!(handler.read_byte(0x4017)?, 0x01); // Start

        Ok(())
    }
}
