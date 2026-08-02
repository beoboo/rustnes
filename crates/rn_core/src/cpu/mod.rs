use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    fmt::Debug,
    rc::Rc,
};

use crate::{errors::NesError, memory::Addressable};
mod addressing_mode;
pub use addressing_mode::AddressingMode;

mod instruction;
pub use instruction::{Instruction, InstructionDecoder, InstructionDecoderError, InstructionMetadata};

mod assembler;
pub use assembler::{AssembleError, AssembleResult, Assembler};

mod disassembler;
pub use disassembler::{DisassembleError, Disassembler};

/// Interrupt vectors, at the very top of the address space.
pub const NMI_VECTOR: u16 = 0xFFFA;
pub const RESET_VECTOR: u16 = 0xFFFC;
pub const IRQ_VECTOR: u16 = 0xFFFE;

/// Cycles taken to push state and jump through a vector.
const INTERRUPT_CYCLES: u8 = 7;

/// CPU status flags
#[derive(Debug, Clone, Copy)]
#[rustfmt::skip]
pub enum CpuFlag {
    Carry            = 0b00000001,
    Zero             = 0b00000010,
    InterruptDisable = 0b00000100,
    DecimalMode      = 0b00001000, // Not used in NES, but part of the 6502 spec
    Break            = 0b00010000, // Not a real flag, used during CPU stack operations
    Unused           = 0b00100000, // Bit 5 is unused, always set to 1
    Overflow         = 0b01000000,
    Negative         = 0b10000000,
}

pub trait CpuInterface: Addressable {}

#[derive(Clone, Debug)]
pub struct CpuWrapper {
    cpu: Rc<RefCell<Cpu>>,
}

impl CpuWrapper {
    pub fn new(cpu: Cpu) -> Self {
        Self {
            cpu: Rc::new(RefCell::new(cpu)),
        }
    }

    pub fn connect_memory(&self, clone: Rc<RefCell<crate::system::Bus>>) {
        self.cpu.borrow_mut().connect_memory(clone);
    }

    pub fn write_bytes(&mut self, addr: u16, data: &[u8]) -> Result<(), NesError> {
        for (i, &byte) in data.iter().enumerate() {
            self.write_byte(addr.wrapping_add(i as u16), byte)?;
        }
        Ok(())
    }

    pub fn step(&self) -> Result<u8, NesError> {
        self.cpu.borrow_mut().step()
    }

    pub fn load_program(&self, program: &[u8], load_address: u16) -> Result<(), NesError> {
        self.cpu.borrow_mut().load_program(program, load_address)
    }

    pub fn pc(&self) -> u16 {
        self.cpu.borrow().registers.pc
    }

    pub fn set_pc(&self, pc: u16) {
        self.cpu.borrow_mut().registers.pc = pc;
    }

    /// Assert the NMI line (edge-triggered; latches until serviced).
    pub fn request_nmi(&self) {
        self.cpu.borrow_mut().request_nmi();
    }

    /// Set the IRQ line's state (level-triggered; held by the asserting device).
    pub fn set_irq_line(&self, asserted: bool) {
        self.cpu.borrow_mut().set_irq_line(asserted);
    }

    /// Whether the IRQ line is currently asserted.
    pub fn irq_line(&self) -> bool {
        self.cpu.borrow().irq_line()
    }

    /// Handles to the interrupt lines, for the devices that assert them.
    pub fn interrupt_lines(&self) -> InterruptLines {
        self.cpu.borrow().interrupt_lines()
    }

    /// Install the callback that advances the rest of the system by one CPU cycle.
    pub fn set_clock(&self, clock: Rc<dyn Fn()>) {
        self.cpu.borrow_mut().set_clock(clock);
    }

    /// Whether bus accesses should drive the clock. Off outside instruction execution.
    pub fn set_executing(&self, executing: bool) {
        self.cpu.borrow().set_executing(executing);
    }

    /// Cycles already run for the last instruction's bus accesses.
    pub fn take_clocked_cycles(&self) -> u8 {
        self.cpu.borrow().take_clocked_cycles()
    }

    /// Every cycle ever run from a bus access.
    pub fn total_clocked_cycles(&self) -> u64 {
        self.cpu.borrow().total_clocked_cycles()
    }

    pub fn registers(&self) -> CpuRegisters {
        self.cpu.borrow().registers
    }

    pub fn set_registers(&self, registers: CpuRegisters) {
        self.cpu.borrow_mut().registers = registers;
    }

    pub fn reset(&self) -> Result<(), NesError> {
        self.cpu.borrow_mut().reset()
    }

    /// Restore the cycle counter, for a save state.
    pub fn set_cycles(&self, cycles: u64) {
        self.cpu.borrow_mut().cycles = cycles;
    }

    pub fn cycles(&self) -> u64 {
        self.cpu.borrow().cycles
    }
}

impl CpuInterface for CpuWrapper {}

impl Addressable for CpuWrapper {
    fn handles_address(&self, _addr: u16) -> bool {
        true
    }

    fn read_byte(&self, addr: u16) -> Result<u8, NesError> {
        self.cpu.borrow().read_byte(addr)
    }

    fn write_byte(&mut self, addr: u16, value: u8) -> Result<(), NesError> {
        self.cpu.borrow_mut().write_byte(addr, value)
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct CpuRegisters {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub status: u8,
}

impl Default for CpuRegisters {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            pc: 0,
            status: 0x34,
        }
    }
}

/// The two interrupt lines, shared between the CPU and the devices that assert them.
///
/// NMI is edge-triggered and latches until serviced; IRQ is level-triggered and reflects whatever
/// its holders are asserting right now.
#[derive(Clone, Debug)]
pub struct InterruptLines {
    pub nmi: Rc<Cell<bool>>,
    pub irq: Rc<Cell<bool>>,
}

impl InterruptLines {
    /// Latch an NMI. Edge-triggered, so asserting twice before it is serviced is one interrupt.
    pub fn raise_nmi(&self) {
        self.nmi.set(true);
    }

    /// Set the IRQ line to whatever its holders currently assert.
    pub fn set_irq(&self, asserted: bool) {
        self.irq.set(asserted);
    }
}

/// MOS 6502 CPU implementation
pub struct Cpu {
    // Registers
    pub registers: CpuRegisters,

    // CPU cycle count
    pub cycles: u64,

    // Memory connection
    memory: Option<Rc<RefCell<dyn Addressable>>>,

    // Instruction decoder
    decoder: InstructionDecoder,

    /// Advances the rest of the system by one CPU cycle.
    ///
    /// Every 6502 cycle is a bus access, so calling this before each read and write puts the PPU
    /// and APU where they actually stand when the access happens — rather than running the whole
    /// instruction and catching them up afterwards, which makes every access in an instruction
    /// appear simultaneous.
    ///
    /// Deliberately unable to touch the CPU: it is called while the CPU is borrowed. Interrupts
    /// reach it through the shared lines instead.
    #[allow(clippy::type_complexity)]
    clock: Option<Rc<dyn Fn()>>,

    /// Whether an instruction is being executed, so the clock runs only for its accesses.
    ///
    /// Without this, a debugger reading memory to display it would advance the emulator, and
    /// merely looking at the machine would change it.
    executing: Cell<bool>,

    /// Cycles the clock has been run for during the current instruction.
    clocked_cycles: Cell<u8>,

    /// Which cycle of the current instruction samples the interrupt lines.
    ///
    /// Hardware reads them at the end of the second-to-last cycle and acts during the next, not at
    /// the instruction's end. That is what makes CLI, SEI and PLP take effect one instruction late:
    /// they change the I flag *after* the sample, so the flag they set is not the flag it saw.
    ///
    /// Counted in bus accesses, which is meaningful only because every cycle now performs one.
    poll_at: Cell<u8>,

    /// What that sample found: an IRQ eligible at the next instruction boundary.
    irq_sampled: Cell<bool>,

    /// And a latched NMI, which unlike the IRQ line survives until it is serviced.
    nmi_sampled: Cell<bool>,

    /// Every cycle ever run from a bus access, never reset. Diagnostic only.
    total_clocked: Cell<u64>,

    /// Latched NMI request.
    ///
    /// NMI is edge-triggered: the PPU asserts it once when vblank begins, and it stays latched
    /// until the CPU services it. It cannot be masked — that is what "non-maskable" means, and why
    /// this is separate from the IRQ line below.
    /// Shared with whoever asserts it, so the rest of the system can raise an interrupt without
    /// borrowing the CPU. That matters once the system is clocked from inside an instruction: the
    /// CPU is already mutably borrowed then, and reaching back into it would panic.
    nmi_pending: Rc<Cell<bool>>,

    /// State of the IRQ line.
    ///
    /// IRQ is level-triggered: a device holds the line low for as long as its condition persists,
    /// so this is set and cleared by whoever asserts it (the APU frame counter, today) rather than
    /// consumed by the CPU. It only takes effect while the InterruptDisable flag is clear.
    irq_line: Rc<Cell<bool>>,
}

impl Debug for Cpu {
    /// Hand-written because the clock callback is a closure, which cannot be derived. Nothing is
    /// lost: what matters when inspecting a CPU is its registers and how far it has run.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cpu")
            .field("registers", &self.registers)
            .field("cycles", &self.cycles)
            .field("nmi_pending", &self.nmi_pending.get())
            .field("irq_line", &self.irq_line.get())
            .finish_non_exhaustive()
    }
}

impl Cpu {
    /// Create a new CPU instance initialized to power-up state with the provided memory
    pub fn new() -> Self {
        // Initial state according to NES specs
        // See: https://www.nesdev.org/wiki/CPU_power_up_state
        Self {
            registers: CpuRegisters::default(),
            cycles: 0,
            memory: None,
            decoder: InstructionDecoder::new(),
            clock: None,
            executing: Cell::new(false),
            clocked_cycles: Cell::new(0),
            poll_at: Cell::new(0),
            irq_sampled: Cell::new(false),
            nmi_sampled: Cell::new(false),
            total_clocked: Cell::new(0),
            nmi_pending: Rc::new(Cell::new(false)),
            irq_line: Rc::new(Cell::new(false)),
        }
    }

    /// Get a copy of the current registers
    pub fn registers(&self) -> CpuRegisters {
        self.registers
    }

    pub fn connect_memory(&mut self, memory: Rc<RefCell<dyn Addressable>>) {
        self.memory = Some(memory);
    }

    /// Get the value of a specific CPU flag
    pub fn get_flag(&self, flag: CpuFlag) -> bool {
        (self.registers.status & flag as u8) != 0
    }

    /// Set a specific CPU flag to the given value
    pub fn set_flag(&mut self, flag: CpuFlag, value: bool) {
        if value {
            self.registers.status |= flag as u8;
        } else {
            self.registers.status &= !(flag as u8);
        }
    }

    /// Checks if a specific flag is set
    pub fn is_flag_set(&self, flag: CpuFlag) -> bool {
        (self.registers.status & (flag as u8)) != 0
    }

    /// Clears a status flag
    pub fn clear_flag(&mut self, flag: CpuFlag) {
        self.registers.status &= !(flag as u8);
    }

    /// Read a byte from memory
    pub fn read_byte(&self, address: u16) -> Result<u8, NesError> {
        self.tick_bus();
        self.memory()?.read_byte(address)
    }

    /// Write a byte to memory
    pub fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError> {
        self.tick_bus();
        self.memory_mut()?.write_byte(address, value)
    }

    /// Read a word (16-bits) from memory
    /// Read without driving the clock, for inspecting memory rather than executing.
    ///
    /// Needed wherever the emulator looks at an operand it has already fetched — deciding whether
    /// an index crossed a page, say. Hardware reads that operand once; reading it a second time to
    /// answer a question about it would invent a bus cycle that does not exist.
    pub fn peek_byte(&self, address: u16) -> Result<u8, NesError> {
        self.memory()?.read_byte(address)
    }

    /// Read a word without driving the clock. See [`peek_byte`](Self::peek_byte).
    pub fn peek_word(&self, address: u16) -> Result<u16, NesError> {
        let low = self.peek_byte(address)? as u16;
        let high = self.peek_byte(address.wrapping_add(1))? as u16;
        Ok((high << 8) | low)
    }

    pub fn read_word(&self, address: u16) -> Result<u16, NesError> {
        // Two separate byte reads, because that is what the processor does: an address operand is
        // fetched low byte then high byte, on consecutive cycles. Reading it as one word in a
        // single memory operation skipped a bus access, and so a cycle, for every instruction with
        // a two-byte operand.
        let low = self.read_byte(address)? as u16;
        let high = self.read_byte(address.wrapping_add(1))? as u16;
        Ok((high << 8) | low)
    }

    /// Write a word (16-bits) to memory
    pub fn write_word(&mut self, address: u16, value: u16) -> Result<(), NesError> {
        self.memory_mut()?.write_word(address, value)
    }

    pub fn memory(&self) -> Result<Ref<'_, dyn Addressable>, NesError> {
        Ok(self.memory.as_ref().ok_or(NesError::MemoryNotConnected)?.borrow())
    }

    pub fn memory_mut(&mut self) -> Result<RefMut<'_, dyn Addressable>, NesError> {
        Ok(self.memory.as_ref().ok_or(NesError::MemoryNotConnected)?.borrow_mut())
    }

    /// Push a byte onto the stack
    pub fn push_byte(&mut self, value: u8) -> Result<(), NesError> {
        let stack_addr = 0x0100 | (self.registers.sp as u16);
        self.write_byte(stack_addr, value)?;
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        Ok(())
    }

    /// Pop a byte from the stack
    pub fn pop_byte(&mut self) -> Result<u8, NesError> {
        // The stack is read at the current pointer and discarded before the pointer moves. That
        // cycle exists to increment the pointer, and like every other cycle it drives the bus.
        let _ = self.read_byte(0x0100 | (self.registers.sp as u16));

        self.registers.sp = self.registers.sp.wrapping_add(1);
        let stack_addr = 0x0100 | (self.registers.sp as u16);
        self.read_byte(stack_addr)
    }

    /// Push a word onto the stack (high byte first, then low byte)
    pub fn push_word(&mut self, value: u16) -> Result<(), NesError> {
        let high = (value >> 8) as u8;
        let low = (value & 0xFF) as u8;
        self.push_byte(high)?;
        self.push_byte(low)?;
        Ok(())
    }

    /// Pop a word from the stack (low byte first, then high byte)
    pub fn pop_word(&mut self) -> Result<u16, NesError> {
        let low = self.pop_byte()? as u16;
        let high = self.pop_byte()? as u16;
        Ok((high << 8) | low)
    }

    /// Reset the CPU
    pub fn reset(&mut self) -> Result<(), NesError> {
        // Set registers to their initial values
        self.registers = CpuRegisters::default();

        // Read the reset vector from 0xFFFC-0xFFFD
        self.registers.pc = self.read_word(0xFFFC)?;

        // Reset takes 7 cycles
        self.cycles = 7;

        Ok(())
    }

    /// Load a program into memory and set up the reset vector
    pub fn load_program(&mut self, program: &[u8], load_address: u16) -> Result<(), NesError> {
        // Load the program into memory
        for (i, &byte) in program.iter().enumerate() {
            self.write_byte(load_address.wrapping_add(i as u16), byte)?;
        }

        // Set the reset vector to point to our program
        self.write_word(0xFFFC, load_address)?;

        // Reset the CPU to prepare it for execution
        self.reset()
    }

    /// Read a byte using the specified addressing mode - simplified for tests
    pub fn read_byte_using_mode(&self, mode: AddressingMode) -> Result<u8, NesError> {
        let addr = mode.get_operand_address(self)?;
        self.read_byte(addr)
    }

    /// Execute a single CPU instruction and return the number of cycles used
    /// Assert the NMI line. Edge-triggered, so this latches until serviced.
    pub fn request_nmi(&mut self) {
        self.nmi_pending.set(true);
    }

    /// Set the state of the IRQ line. Level-triggered: the asserting device holds it.
    pub fn set_irq_line(&mut self, asserted: bool) {
        self.irq_line.set(asserted);
    }

    pub fn irq_line(&self) -> bool {
        self.irq_line.get()
    }

    /// Install the callback that advances the rest of the system by one CPU cycle.
    pub fn set_clock(&mut self, clock: Rc<dyn Fn()>) {
        self.clock = Some(clock);
    }

    /// Advance the rest of the system by one cycle, for one bus access.
    ///
    /// Counted so the caller can make up whatever the instruction's cycle count exceeds its bus
    /// accesses. Not every 6502 cycle is modelled as an access here — the internal ones are not —
    /// so the remainder still has to be run, just after the accesses rather than instead of them.
    fn tick_bus(&self) {
        if !self.executing.get() {
            return;
        }

        if let Some(clock) = &self.clock {
            let cycle = self.clocked_cycles.get().saturating_add(1);
            self.clocked_cycles.set(cycle);
            self.total_clocked.set(self.total_clocked.get() + 1);
            clock();

            // Sampled after the cycle has run, so the lines are read as they stand at its end.
            if cycle == self.poll_at.get() {
                self.sample_interrupts();
            }
        }
    }

    /// Consume a pending NMI if one is asserted, for BRK to take over.
    ///
    /// Both the latched sample and the raw line are checked: an NMI raised during BRK itself has
    /// not been sampled, because an interrupt sequence does no polling, but it still arrives in
    /// time to redirect the vector.
    pub fn take_nmi_for_hijack(&mut self, arrived_during: bool) -> bool {
        if !arrived_during {
            return false;
        }

        // Taking it over *is* servicing it, so the latch is released here and nowhere else.
        self.clear_nmi();
        true
    }

    /// Whether an NMI is latched right now, without disturbing it.
    pub fn nmi_latched(&self) -> bool {
        self.nmi_pending.get()
    }

    /// Release the NMI edge latch. Only servicing does this.
    fn clear_nmi(&self) {
        self.nmi_pending.set(false);
        self.nmi_sampled.set(false);
    }

    /// Read the interrupt lines, as hardware does before an instruction's last cycle.
    pub(crate) fn sample_interrupts(&self) {
        // Read, not taken. NMI is edge-triggered: the edge sets a latch that persists until the
        // interrupt is serviced, and looking at it is not servicing it. Consuming it here made the
        // latch single-shot with more than one consumer, so whichever code looked at it first
        // silently destroyed it for everyone else — which is how an NMI went missing entirely
        // around BRK, leaving a test spinning forever on an interrupt that had already happened.
        if self.nmi_pending.get() {
            self.nmi_sampled.set(true);
        }

        // The flag as it stands at this cycle. An instruction that changes it later in its own
        // execution cannot affect this sample, which is the entire point.
        self.irq_sampled
            .set(self.irq_line.get() && !self.get_flag(CpuFlag::InterruptDisable));
    }

    /// Every cycle ever run from a bus access.
    pub fn total_clocked_cycles(&self) -> u64 {
        self.total_clocked.get()
    }

    /// Cycles already run for the current instruction's bus accesses.
    pub fn take_clocked_cycles(&self) -> u8 {
        self.clocked_cycles.replace(0)
    }

    /// Whether the clock runs on bus accesses. Off outside instruction execution, so inspecting
    /// memory does not advance the machine.
    pub fn set_executing(&self, executing: bool) {
        self.executing.set(executing);
    }

    /// Handles to the interrupt lines, for the parts of the system that assert them.
    ///
    /// Returned as shared cells rather than served through methods on the CPU so that a device can
    /// raise an interrupt while the CPU is mid-instruction, which is when they actually happen.
    pub fn interrupt_lines(&self) -> InterruptLines {
        InterruptLines {
            nmi: Rc::clone(&self.nmi_pending),
            irq: Rc::clone(&self.irq_line),
        }
    }

    /// Enter an interrupt handler: push the return address and status, then jump through `vector`.
    ///
    /// The pushed status has Break *clear* and Unused set. That bit is how a handler distinguishes
    /// a hardware interrupt from a `BRK`, which pushes it set — the 6502 has no real Break flag,
    /// only these pushed copies.
    fn service_interrupt(&mut self, vector: u16) -> Result<u8, NesError> {
        self.push_word(self.registers.pc)?;

        let status = (self.registers.status & !(CpuFlag::Break as u8)) | CpuFlag::Unused as u8;
        self.push_byte(status)?;

        // Mask further IRQs while the handler runs. NMI is unaffected, being non-maskable.
        self.set_flag(CpuFlag::InterruptDisable, true);
        self.registers.pc = self.read_word(vector)?;

        Ok(INTERRUPT_CYCLES)
    }

    /// Service a pending interrupt, if any, returning the cycles it took.
    ///
    /// Checked before each instruction rather than mid-instruction: the 6502 finishes the current
    /// instruction before honouring an interrupt, and this emulator steps whole instructions.
    fn poll_interrupts(&mut self) -> Result<Option<u8>, NesError> {
        // Acted on from the sample taken during the previous instruction, not from the lines as
        // they stand now. Each is cleared as it is consumed: servicing sets the InterruptDisable
        // flag, so a sample left standing would send the CPU back into the handler it just entered.
        // An interrupt sequence performs no polling of its own, so at least one instruction of a
        // handler always runs before another interrupt is taken. Both samples are therefore
        // discarded, not just the one being acted on: leaving the other standing would service two
        // interrupts back to back with no instruction between them, and a handler that never
        // reaches its first instruction never returns.
        if self.nmi_sampled.get() {
            // Servicing releases the latch, and discards the IRQ sample with it: an interrupt
            // sequence does no polling of its own, so one instruction of the handler must run
            // before another interrupt is taken.
            self.clear_nmi();
            self.irq_sampled.set(false);
            return Ok(Some(self.service_interrupt(NMI_VECTOR)?));
        }

        if self.irq_sampled.replace(false) {
            return Ok(Some(self.service_interrupt(IRQ_VECTOR)?));
        }

        Ok(None)
    }

    pub fn step(&mut self) -> Result<u8, NesError> {
        // An interrupt takes the place of this step's instruction.
        if let Some(cycles) = self.poll_interrupts()? {
            self.cycles += cycles as u64;
            return Ok(cycles);
        }

        // Fetch opcode
        let opcode = self.fetch()?;

        // Decode instruction
        let metadata = self.decoder.decode(opcode)?;

        // The lines are read before the last cycle. The count is known here — after decoding,
        // before executing — which is the only point at which it can be turned into a cycle to
        // watch for. A page-crossing penalty lengthens the instruction without moving the sample,
        // which is why the base count is used rather than the total.
        // BRK is an interrupt sequence wearing an opcode, and like the others it does not poll.
        self.poll_at.set(if matches!(metadata.instruction, Instruction::BRK) {
            0
        } else {
            metadata.cycles.saturating_sub(1)
        });

        // The opcode fetch has already happened — it is cycle one, and it ran before the opcode
        // was known and so before this target could be set. A two-cycle instruction samples at the
        // end of cycle one, which is therefore already behind us, so it is taken now rather than
        // waited for.
        if self.clocked_cycles.get() >= self.poll_at.get() {
            self.sample_interrupts();
            self.poll_at.set(0);
        }

        // Execute instruction and update cycle count
        let additional_cycles = self.execute(metadata)?;

        // Calculate total cycles: base cycles from metadata + any additional cycles
        let total_cycles = metadata.cycles + additional_cycles;

        // An instruction shorter than expected — or one running without a clock at all, as the
        // unit tests do — would never reach the chosen cycle. Sampling on the way out means the
        // lines are always looked at exactly once.
        if self.poll_at.get() != 0 && self.clocked_cycles.get() < self.poll_at.get() {
            self.sample_interrupts();
        }
        self.poll_at.set(0);

        // Update the CPU's cycle counter
        self.cycles += total_cycles as u64;

        Ok(total_cycles)
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;
    use crate::memory::Ram;

    /// Helper function to set up a CPU with memory for testing
    fn setup_cpu() -> Cpu {
        setup_cpu_with_memory(Ram::default())
    }

    fn setup_cpu_with_memory(memory: Ram) -> Cpu {
        let mut cpu = Cpu::new();
        cpu.connect_memory(Rc::new(RefCell::new(memory)));
        cpu
    }

    #[test]
    fn test_cpu_flags() {
        let mut cpu = setup_cpu();

        // Test flag is initially not set
        assert!(!cpu.get_flag(CpuFlag::Zero));

        // Test setting a flag
        cpu.set_flag(CpuFlag::Zero, true);
        assert!(cpu.get_flag(CpuFlag::Zero));

        // Test clearing a flag
        cpu.set_flag(CpuFlag::Zero, false);
        assert!(!cpu.get_flag(CpuFlag::Zero));
    }

    #[test]
    fn test_cpu_memory_interaction() -> Result<()> {
        let mut cpu = setup_cpu();

        // Test writing and reading bytes
        cpu.write_byte(0x1000, 0x42)?;
        assert_eq!(cpu.read_byte(0x1000)?, 0x42);

        // Test writing and reading words
        cpu.write_word(0x2000, 0x1234)?;
        assert_eq!(cpu.read_word(0x2000)?, 0x1234);

        Ok(())
    }

    #[test]
    fn test_stack_operations() -> Result<()> {
        let mut cpu = setup_cpu();

        // Test push and pop byte
        cpu.push_byte(0x42)?;
        assert_eq!(cpu.registers.sp, 0xFC);
        assert_eq!(cpu.pop_byte()?, 0x42);
        assert_eq!(cpu.registers.sp, 0xFD);

        // Test push and pop word
        cpu.push_word(0x1234)?;
        assert_eq!(cpu.registers.sp, 0xFB);
        assert_eq!(cpu.pop_word()?, 0x1234);
        assert_eq!(cpu.registers.sp, 0xFD);

        Ok(())
    }

    #[test]
    fn test_reset() -> Result<()> {
        // Use RAM with full address space (0x0000-0xFFFF) for testing
        let mut ram = Ram::default();

        // Set reset vector
        ram.write_byte(0xFFFC, 0x34)?;
        ram.write_byte(0xFFFD, 0x12)?;

        let mut cpu = setup_cpu_with_memory(ram);
        cpu.reset()?;

        // Check if PC was set to the reset vector
        assert_eq!(cpu.registers.pc, 0x1234);
        // Check if SP was set to 0xFD
        assert_eq!(cpu.registers.sp, 0xFD);
        // Check if cycles were set to 7
        assert_eq!(cpu.cycles, 7);

        Ok(())
    }

    #[test]
    fn test_step_lda_immediate() -> Result<()> {
        let mut ram = Ram::default();

        // Set up a simple program: LDA #$42
        ram.write_byte(0x0000, 0xA9)?; // LDA immediate
        ram.write_byte(0x0001, 0x42)?; // Value to load

        let mut cpu = setup_cpu_with_memory(ram);
        cpu.registers.pc = 0x0000; // Set PC to our program

        // Execute one instruction
        let cycles = cpu.step()?;

        // Verify results
        assert_eq!(cpu.registers.a, 0x42);
        assert_eq!(cpu.registers.pc, 0x0002);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.cycles, 2);

        Ok(())
    }

    #[test]
    fn test_unknown_opcode() -> Result<()> {
        let mut ram = Ram::default();

        // $02 is a JAM/KIL opcode: it locks the real CPU up and has no behaviour worth
        // emulating, so it stays undecodable. ($FF was used here until the unofficial opcodes
        // were implemented and it became a valid ISB Absolute,X.)
        ram.write_byte(0x0000, 0x02)?;

        let mut cpu = setup_cpu_with_memory(ram);
        cpu.registers.pc = 0x0000;

        // Execute one instruction - this should now return an error for the invalid opcode
        let result = cpu.step();

        // Verify the error is the expected one
        assert!(result.is_err(), "Expected an error for invalid opcode");
        if let Err(NesError::InstructionDecoderError(InstructionDecoderError::InvalidOpcode(op))) = result {
            assert_eq!(op, 0x02);
        } else {
            anyhow::bail!("Expected InvalidOpcode error, got: {:?}", result);
        }

        // PC should still be incremented because fetch still happened
        assert_eq!(cpu.registers.pc, 0x0001);

        Ok(())
    }

    #[test]
    fn test_load_program() -> Result<()> {
        let mut cpu = setup_cpu();

        // Simple program: LDA #$42, STA $0200, BRK
        let program = [0xA9, 0x42, 0x8D, 0x00, 0x02, 0x00];
        let load_address = 0x8000;

        // Load the program
        cpu.load_program(&program, load_address)?;

        // Verify the program was loaded correctly
        for (i, &byte) in program.iter().enumerate() {
            assert_eq!(cpu.read_byte(load_address + i as u16)?, byte);
        }

        // Verify the reset vector was set correctly
        assert_eq!(cpu.read_word(0xFFFC)?, load_address);

        // Verify the CPU was reset and PC points to the program
        assert_eq!(cpu.registers.pc, load_address);

        // Execute the first instruction (LDA #$42)
        cpu.step()?;
        assert_eq!(cpu.registers.a, 0x42);

        // Execute the second instruction (STA $0200)
        cpu.step()?;
        assert_eq!(cpu.read_byte(0x0200)?, 0x42);

        Ok(())
    }
}
