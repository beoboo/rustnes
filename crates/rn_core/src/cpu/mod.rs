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

    /// Install the callback that advances the rest of the system across one CPU cycle.
    pub fn set_clock(&self, clock: Rc<dyn Fn(ClockPhase)>) {
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
    /// Drive the /NMI line. A level: the PPU holds it down for as long as it means to.
    ///
    /// The CPU detects the rising edge and remembers it, so releasing the line does not take back
    /// an interrupt already detected — but re-asserting it gives another, which is what a program
    /// toggling $2000 bit 7 during vblank is after.
    pub fn set_nmi(&self, asserted: bool) {
        self.nmi.set(asserted);
    }

    /// Set the IRQ line to whatever its holders currently assert.
    pub fn set_irq(&self, asserted: bool) {
        self.irq.set(asserted);
    }
}

/// Which half of a CPU cycle the clock is being asked to run.
///
/// A 6502 cycle does not end when its bus access does: the access happens partway through, and the
/// cycle runs on past it. Splitting the cycle here is what puts the interrupt poll at the cycle's
/// *end* rather than at the instant of the access — one PPU dot later, and that dot is measurable.
/// `ppu_vbl_nmi/05-nmi_timing` prints which instruction an NMI landed after, one PPU clock later
/// each line; with the whole cycle run before the access, every transition in its table came out
/// one line late.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockPhase {
    /// Before the access: the part of the cycle the access is waiting on.
    BeforeAccess,

    /// After it: the rest of the cycle, ending with the interrupt lines being read.
    AfterAccess,
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

    /// Advances the rest of the system across one CPU cycle, in the two halves either side of the
    /// bus access that cycle performs.
    ///
    /// Every 6502 cycle is a bus access, so calling this around each read and write puts the PPU
    /// and APU where they actually stand when the access happens — rather than running the whole
    /// instruction and catching them up afterwards, which makes every access in an instruction
    /// appear simultaneous.
    ///
    /// Deliberately unable to touch the CPU: it is called while the CPU is borrowed. Interrupts
    /// reach it through the shared lines instead.
    #[allow(clippy::type_complexity)]
    clock: Option<Rc<dyn Fn(ClockPhase)>>,

    /// Whether an instruction is being executed, so the clock runs only for its accesses.
    ///
    /// Without this, a debugger reading memory to display it would advance the emulator, and
    /// merely looking at the machine would change it.
    executing: Cell<bool>,

    /// Cycles the clock has been run for during the current instruction.
    clocked_cycles: Cell<u8>,

    /// The internal NMI signal: an edge has been detected and not yet serviced.
    ///
    /// The wiki: "the internal signal goes high during φ1 of the cycle that follows the one where
    /// the edge is detected, and stays high until the NMI has been handled".
    need_nmi: Cell<bool>,

    /// [`need_nmi`](Self::need_nmi) as it stood one CPU cycle ago — the shadow that is acted on.
    ///
    /// This is the whole of the sampling rule. Hardware reads the lines at the end of the
    /// second-to-last cycle and acts during the next, and a one-cycle-delayed copy checked after
    /// the instruction *is* that, without anything having to know how long the instruction is.
    /// Deriving the polling cycle from the instruction's length was tried twice and failed twice;
    /// see CYCLE_ACCURACY.md.
    prev_need_nmi: Cell<bool>,

    /// The NMI line as it stood at the end of the previous cycle, for detecting its rising edge.
    prev_nmi_line: Cell<bool>,

    /// Whether an IRQ is currently eligible: the line asserted and the I flag clear.
    run_irq: Cell<bool>,

    /// [`run_irq`](Self::run_irq) one cycle ago. Acted on for the same reason as `prev_need_nmi`,
    /// and the reason `CLI`, `SEI` and `PLP` take effect one instruction late: they change the I
    /// flag after the cycle whose value the shadow is still holding.
    prev_run_irq: Cell<bool>,

    /// Every cycle ever run from a bus access, never reset. Diagnostic only.
    total_clocked: Cell<u64>,

    /// State of the /NMI line.
    ///
    /// A level, driven by the PPU, not a latch the CPU consumes: it goes down when the vblank flag
    /// and the enable bit are both set and comes back up when either stops being true. What is
    /// edge-triggered is the CPU's *response* to it — [`end_cpu_cycle`](Self::end_cpu_cycle)
    /// watches for the rising edge and raises [`need_nmi`](Self::need_nmi), which then survives
    /// until the interrupt is serviced. It cannot be masked, which is what "non-maskable" means and
    /// why this is separate from the IRQ line below.
    /// Shared with whoever asserts it, so the rest of the system can raise an interrupt without
    /// borrowing the CPU. That matters once the system is clocked from inside an instruction: the
    /// CPU is already mutably borrowed then, and reaching back into it would panic.
    nmi_line: Rc<Cell<bool>>,

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
            .field("nmi_line", &self.nmi_line.get())
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
            need_nmi: Cell::new(false),
            prev_need_nmi: Cell::new(false),
            prev_nmi_line: Cell::new(false),
            run_irq: Cell::new(false),
            prev_run_irq: Cell::new(false),
            total_clocked: Cell::new(0),
            nmi_line: Rc::new(Cell::new(false)),
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
        self.start_cycle();
        let value = self.memory().and_then(|memory| memory.read_byte(address));
        self.end_cycle();
        value
    }

    /// Write a byte to memory
    pub fn write_byte(&mut self, address: u16, value: u8) -> Result<(), NesError> {
        self.start_cycle();
        let result = self.memory_mut().and_then(|mut memory| memory.write_byte(address, value));
        self.end_cycle();
        result
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
    /// Assert the /NMI line and leave it asserted, as the PPU does through a vblank.
    ///
    /// The CPU takes one interrupt from it, on the edge. Holding the line down does not produce a
    /// second — that needs a release and a fresh edge.
    pub fn request_nmi(&mut self) {
        self.nmi_line.set(true);
    }

    /// Drive the /NMI line to a given level, as the PPU does.
    pub fn set_nmi_line(&mut self, asserted: bool) {
        self.nmi_line.set(asserted);
    }

    /// Set the state of the IRQ line. Level-triggered: the asserting device holds it.
    pub fn set_irq_line(&mut self, asserted: bool) {
        self.irq_line.set(asserted);
    }

    pub fn irq_line(&self) -> bool {
        self.irq_line.get()
    }

    /// Install the callback that advances the rest of the system across one CPU cycle.
    pub fn set_clock(&mut self, clock: Rc<dyn Fn(ClockPhase)>) {
        self.clock = Some(clock);
    }

    /// Open a cycle: run the rest of the system up to the point its bus access happens.
    ///
    /// Counted so the caller can make up whatever the instruction's cycle count exceeds its bus
    /// accesses. Not every 6502 cycle is modelled as an access here — the internal ones are not —
    /// so the remainder still has to be run, just after the accesses rather than instead of them.
    fn start_cycle(&self) {
        if !self.executing.get() {
            return;
        }

        if let Some(clock) = &self.clock {
            let cycle = self.clocked_cycles.get().saturating_add(1);
            self.clocked_cycles.set(cycle);
            self.total_clocked.set(self.total_clocked.get() + 1);
            clock(ClockPhase::BeforeAccess);
        }
    }

    /// Close a cycle: run what is left of it after the access, then read the interrupt lines.
    ///
    /// The access is not the end of the cycle, and the difference is one PPU dot. Reading the lines
    /// at the instant of the access instead put every transition in `05-nmi_timing`'s table one
    /// line late — that test runs one PPU clock later on each line, so a line is a dot.
    fn end_cycle(&self) {
        if !self.executing.get() {
            return;
        }

        if let Some(clock) = &self.clock {
            clock(ClockPhase::AfterAccess);
            self.end_cpu_cycle();
        }
    }

    /// Close out one CPU cycle: shift the interrupt lines into their one-cycle-delayed shadow.
    ///
    /// Run for every cycle, unconditionally. Nothing here knows which cycle of which instruction
    /// this is, and that is the point — "the status of the lines at the end of the second-to-last
    /// cycle" falls out of the delay rather than being computed from an instruction's length.
    fn end_cpu_cycle(&self) {
        // Copied before the line is looked at, so the shadow lags by exactly one cycle.
        self.prev_need_nmi.set(self.need_nmi.get());

        // NMI is edge-triggered: the detector watches for the line going from unasserted to
        // asserted, and the resulting signal stays up until the interrupt is serviced. Reading the
        // line is not servicing it, which is why nothing is consumed here.
        let nmi_line = self.nmi_line.get();
        if !self.prev_nmi_line.get() && nmi_line {
            self.need_nmi.set(true);
        }
        self.prev_nmi_line.set(nmi_line);

        // IRQ is level-triggered, so there is no latch: what matters is the line and the I flag as
        // they stand at the end of this cycle.
        self.prev_run_irq.set(self.run_irq.get());
        self.run_irq
            .set(self.irq_line.get() && !self.get_flag(CpuFlag::InterruptDisable));
    }

    /// Consume the internal NMI signal if it is up, for BRK to take over.
    ///
    /// The signal, not the shadow: an NMI that arrives during BRK's own first cycles is still in
    /// time to redirect the vector, and one that arrived early enough for the shadow to have seen
    /// it would have been serviced instead of BRK ever running.
    pub fn take_nmi_for_hijack(&mut self) -> bool {
        if !self.need_nmi.get() {
            return false;
        }

        // Taking it over *is* servicing it, so the latch is released here and nowhere else.
        self.clear_nmi();
        true
    }

    /// The state of the /NMI line right now.
    pub fn nmi_line(&self) -> bool {
        self.nmi_line.get()
    }

    /// Release the internal NMI signal. Only servicing does this.
    ///
    /// Deliberately does not touch the line: the CPU does not drive it and cannot take it away.
    /// The PPU releases it when the vblank flag goes, and until then the line simply stays down —
    /// harmlessly, because the edge that mattered has already been counted and a level cannot
    /// produce another.
    fn clear_nmi(&self) {
        self.need_nmi.set(false);
        self.prev_need_nmi.set(false);
    }

    /// Drop whatever the shadow is holding, so no interrupt is taken at this instruction's end.
    ///
    /// Only `BRK` needs it: it is an interrupt sequence in an opcode's clothing and, like the
    /// hardware sequences, does not poll.
    pub(crate) fn clear_pending_interrupt_shadow(&self) {
        self.prev_need_nmi.set(false);
        self.prev_run_irq.set(false);
    }

    /// Bring the shadow up to date with the lines as they stand, in one go.
    ///
    /// Only for a CPU with no clock installed, which is every unit test that constructs one
    /// directly: with nothing advancing the cycles there is nothing to shift the shadow along, so
    /// an interrupt raised between steps would never be noticed at all. Two cycles' worth, because
    /// that is what it takes for the present to reach the shadow.
    pub(crate) fn sample_interrupts(&self) {
        self.end_cpu_cycle();
        self.end_cpu_cycle();
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
            nmi: Rc::clone(&self.nmi_line),
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
        // Acted on from the shadow, which holds the lines as they stood one cycle before the
        // previous instruction ended, not from the lines as they stand now.
        //
        // Both shadows are cleared, not only the one being acted on: an interrupt sequence does no
        // polling of its own, so at least one instruction of a handler always runs before another
        // interrupt is taken. Leaving the other standing would service two back to back with no
        // instruction between them, and a handler that never reaches its first instruction never
        // returns. The sequence's own cycles then refill the shadows honestly — servicing sets the
        // InterruptDisable flag, so `run_irq` falls of its own accord.
        if self.prev_need_nmi.get() {
            self.clear_nmi();
            self.prev_run_irq.set(false);
            return Ok(Some(self.service_interrupt(NMI_VECTOR)?));
        }

        if self.prev_run_irq.replace(false) {
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

        // Execute instruction and update cycle count
        let additional_cycles = self.execute(metadata)?;

        // Calculate total cycles: base cycles from metadata + any additional cycles
        let total_cycles = metadata.cycles + additional_cycles;

        // A CPU with no clock installed has run no cycles, so nothing has shifted the shadow along
        // and an interrupt raised between steps would never be seen. Unit tests build such a CPU;
        // the emulator never does. One catch-up per instruction keeps them working and matches
        // what they were written against — the lines looked at exactly once per instruction.
        if self.clocked_cycles.get() == 0 {
            self.sample_interrupts();
        }

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

    /// Where in a CPU cycle the interrupt lines are read, stated as the thing it decides.
    ///
    /// A 6502 cycle does not end when its bus access does. The access happens partway through and
    /// the cycle runs on past it, so an interrupt asserted *after* the access of a cycle is still
    /// caught by that cycle's poll. One PPU dot of the three in a CPU cycle falls after the access,
    /// and that dot is measurable: with the whole cycle run before the access, every transition in
    /// `ppu_vbl_nmi/05-nmi_timing`'s table came out one line late — that ROM runs one PPU clock
    /// later on each line, so a line is a dot.
    ///
    /// Here the same thing is asserted without the ROM and without a PPU: the clock raises the NMI
    /// at a chosen point of a chosen cycle, and the handler records which `LDX #n` had run.
    ///
    /// Returns the X the handler saw — the number of the last `LDX` to complete.
    fn ldx_reached_when_nmi_is_raised(raise_at: (u32, ClockPhase)) -> u8 {
        let mut cpu = setup_cpu();
        cpu.registers.sp = 0xFD;

        // Four two-cycle instructions, so a cycle is half an instruction and the boundary being
        // measured is unambiguous.
        for (i, byte) in [0xA2, 0x01, 0xA2, 0x02, 0xA2, 0x03, 0xA2, 0x04].iter().enumerate() {
            cpu.write_byte(0x8000 + i as u16, *byte).expect("writing the program");
        }

        // The handler: STX $0400, then RTI.
        for (i, byte) in [0x8E, 0x00, 0x04, 0x40].iter().enumerate() {
            cpu.write_byte(0x9000 + i as u16, *byte).expect("writing the handler");
        }
        cpu.write_byte(NMI_VECTOR, 0x00).expect("writing the vector");
        cpu.write_byte(NMI_VECTOR + 1, 0x90).expect("writing the vector");
        cpu.write_byte(0x0400, 0xFF).expect("clearing the result");

        cpu.registers.pc = 0x8000;

        let lines = cpu.interrupt_lines();
        let cycle = Rc::new(Cell::new(0u32));
        {
            let cycle = Rc::clone(&cycle);
            cpu.set_clock(Rc::new(move |phase| {
                // Counted on the way in, so cycle one is the first opcode fetch.
                if phase == ClockPhase::BeforeAccess {
                    cycle.set(cycle.get() + 1);
                }
                if (cycle.get(), phase) == raise_at {
                    lines.set_nmi(true);
                }
            }));
        }

        cpu.set_executing(true);
        for _ in 0..8 {
            cpu.step().expect("stepping");
            if cpu.registers.pc > 0x9000 {
                break;
            }
        }
        cpu.set_executing(false);

        cpu.peek_byte(0x0400).expect("reading the result")
    }

    #[test]
    fn an_nmi_after_the_second_to_last_cycles_access_is_taken_at_that_instructions_end() {
        // Cycle one is the first `LDX`'s opcode fetch, and since the instruction is two cycles long
        // that is its second-to-last cycle. An NMI asserted after that cycle's access is still
        // inside the cycle, so the poll at its end sees it and the interrupt follows the
        // instruction it arrived during. Polling at the instant of the access instead misses it by
        // one dot, defers it to the next cycle, and the handler finds X holding 2.
        assert_eq!(
            ldx_reached_when_nmi_is_raised((1, ClockPhase::AfterAccess)),
            1,
            "the NMI belongs to the cycle it arrived in, so it is taken after the first LDX"
        );
    }

    #[test]
    fn an_nmi_after_the_last_cycles_access_waits_for_the_following_instruction() {
        // The other side of the same boundary, and the reason the assertion above is not merely
        // "interrupts are prompt": cycle two is the first `LDX`'s *last* cycle. What that cycle's
        // poll sees is acted on one cycle later, by which time the instruction is over, so the
        // interrupt falls after the second LDX instead. That is the one-cycle-delayed shadow doing
        // its job — the same rule that makes CLI and SEI take effect one instruction late.
        assert_eq!(
            ldx_reached_when_nmi_is_raised((2, ClockPhase::AfterAccess)),
            2,
            "sampled at the last cycle, so it cannot be acted on until the next instruction ends"
        );
    }

    /// The IRQ line obeys the same one-cycle delay, and it is a separate path: level-triggered,
    /// masked by the I flag, and with no latch of its own.
    #[test]
    fn an_irq_raised_during_an_instruction_waits_a_cycle_to_be_noticed() {
        let mut cpu = setup_cpu();
        cpu.registers.sp = 0xFD;
        cpu.set_flag(CpuFlag::InterruptDisable, false);

        for (i, byte) in [0xA2, 0x01, 0xA2, 0x02, 0xA2, 0x03].iter().enumerate() {
            cpu.write_byte(0x8000 + i as u16, *byte).expect("writing the program");
        }
        for (i, byte) in [0x8E, 0x00, 0x04, 0x40].iter().enumerate() {
            cpu.write_byte(0x9000 + i as u16, *byte).expect("writing the handler");
        }
        cpu.write_byte(IRQ_VECTOR, 0x00).expect("writing the vector");
        cpu.write_byte(IRQ_VECTOR + 1, 0x90).expect("writing the vector");
        cpu.write_byte(0x0400, 0xFF).expect("clearing the result");
        cpu.registers.pc = 0x8000;

        let lines = cpu.interrupt_lines();
        let cycle = Rc::new(Cell::new(0u32));
        {
            let cycle = Rc::clone(&cycle);
            cpu.set_clock(Rc::new(move |phase| {
                if phase == ClockPhase::BeforeAccess {
                    cycle.set(cycle.get() + 1);
                }
                // Held from the end of the first instruction's last cycle onwards, as a device
                // holds it: the line is level-triggered and nothing here releases it.
                if cycle.get() >= 2 && phase == ClockPhase::AfterAccess {
                    lines.set_irq(true);
                }
            }));
        }

        cpu.set_executing(true);
        for _ in 0..8 {
            cpu.step().expect("stepping");
            if cpu.registers.pc > 0x9000 {
                break;
            }
        }
        cpu.set_executing(false);

        assert_eq!(
            cpu.peek_byte(0x0400).expect("reading the result"),
            2,
            "asserted at the first LDX's last cycle, so the second one runs before it is taken"
        );
    }

    /// An NMI the CPU has already detected survives the line being released.
    ///
    /// The other half of `reading_status_well_into_vblank_releases_the_line_with_the_flag`: the PPU
    /// lets /NMI go when a program reads $2002, and that must not cancel an interrupt whose edge
    /// has already been counted. Detection is edge-triggered and the resulting signal persists
    /// until it is serviced — the line going back up is not servicing it.
    #[test]
    fn an_nmi_already_detected_survives_the_line_being_released() {
        let mut cpu = setup_cpu();
        cpu.registers.sp = 0xFD;

        for (i, byte) in [0xA2, 0x01, 0xA2, 0x02].iter().enumerate() {
            cpu.write_byte(0x8000 + i as u16, *byte).expect("writing the program");
        }
        for (i, byte) in [0x8E, 0x00, 0x04, 0x40].iter().enumerate() {
            cpu.write_byte(0x9000 + i as u16, *byte).expect("writing the handler");
        }
        cpu.write_byte(NMI_VECTOR, 0x00).expect("writing the vector");
        cpu.write_byte(NMI_VECTOR + 1, 0x90).expect("writing the vector");
        cpu.write_byte(0x0400, 0xFF).expect("clearing the result");
        cpu.registers.pc = 0x8000;

        let lines = cpu.interrupt_lines();
        let cycle = Rc::new(Cell::new(0u32));
        {
            let cycle = Rc::clone(&cycle);
            cpu.set_clock(Rc::new(move |phase| {
                if phase == ClockPhase::BeforeAccess {
                    cycle.set(cycle.get() + 1);
                }
                if phase == ClockPhase::AfterAccess {
                    // Down for one cycle only, then straight back up — a $2002 read landing
                    // immediately after the edge was detected.
                    lines.set_nmi(cycle.get() == 1);
                }
            }));
        }

        cpu.set_executing(true);
        for _ in 0..8 {
            cpu.step().expect("stepping");
            if cpu.registers.pc > 0x9000 {
                break;
            }
        }
        cpu.set_executing(false);

        assert_eq!(
            cpu.peek_byte(0x0400).expect("reading the result"),
            1,
            "the edge was counted, so releasing the line cannot take the interrupt back"
        );
    }

    /// And a line held down does not produce a second interrupt — only a fresh edge does.
    ///
    /// The PPU holds /NMI down for the whole of vblank, twenty scanlines of it. If the level rather
    /// than its edge were what counted, the handler would be re-entered every cycle and the machine
    /// would never leave it.
    #[test]
    fn a_line_held_down_gives_exactly_one_interrupt() {
        let mut cpu = setup_cpu();
        cpu.registers.sp = 0xFD;

        // The handler counts its own entries by incrementing $0400, then returns.
        for (i, byte) in [0xEE, 0x00, 0x04, 0x40].iter().enumerate() {
            cpu.write_byte(0x9000 + i as u16, *byte).expect("writing the handler");
        }
        // NOPs to return to.
        for i in 0..8 {
            cpu.write_byte(0x8000 + i, 0xEA).expect("writing the program");
        }
        cpu.write_byte(NMI_VECTOR, 0x00).expect("writing the vector");
        cpu.write_byte(NMI_VECTOR + 1, 0x90).expect("writing the vector");
        cpu.write_byte(0x0400, 0x00).expect("clearing the count");
        cpu.registers.pc = 0x8000;

        {
            let lines = cpu.interrupt_lines();
            cpu.set_clock(Rc::new(move |phase| {
                if phase == ClockPhase::AfterAccess {
                    lines.set_nmi(true);
                }
            }));
        }

        cpu.set_executing(true);
        for _ in 0..12 {
            cpu.step().expect("stepping");
        }
        cpu.set_executing(false);

        assert_eq!(
            cpu.peek_byte(0x0400).expect("reading the count"),
            1,
            "one rising edge, one interrupt, however long the line stays down"
        );
    }
}
