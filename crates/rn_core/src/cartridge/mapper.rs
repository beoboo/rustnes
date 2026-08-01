//! Cartridge mappers.
//!
//! The NES CPU can address 32 KB of program space and the PPU 8 KB of graphics, but games shipped
//! far more than that. Cartridges bridged the gap with hardware that switches which part of the
//! ROM is currently visible — and every manufacturer did it differently. A mapper is that
//! hardware, and an emulator needs one per scheme.
//!
//! Writes into cartridge space are how a game talks to its mapper. That is why this cannot be
//! modelled as plain ROM: `STA $8000` is not a discarded write to read-only memory, it is a bank
//! switch, and treating cartridge space as RAM silently corrupts the program instead.

use super::Mirroring;

/// A cartridge's bank-switching hardware.
///
/// Address arguments are the raw CPU or PPU addresses, so implementations can decode the register
/// layout their real hardware used.
pub trait Mapper: std::fmt::Debug {
    /// Read from CPU address space, `$4020..=$FFFF`.
    fn read_prg(&self, address: u16) -> u8;

    /// Write to CPU address space. Usually a mapper register rather than storage.
    fn write_prg(&mut self, address: u16, value: u8);

    /// Read from PPU pattern-table space, `$0000..=$1FFF`.
    fn read_chr(&self, address: u16) -> u8;

    /// Write to pattern-table space. Only meaningful for cartridges with CHR RAM.
    fn write_chr(&mut self, address: u16, value: u8);

    /// Nametable mirroring, which some mappers let the game change at runtime.
    fn mirroring(&self) -> Mirroring;

    /// Whether the mapper is asserting an IRQ.
    ///
    /// Only scanline-counting mappers such as MMC3 ever do; the default suits the rest.
    fn irq_pending(&self) -> bool {
        false
    }

    /// Acknowledge the mapper's IRQ.
    fn acknowledge_irq(&mut self) {}

    /// Notify the mapper that a scanline has been rendered, for those that count them.
    fn on_scanline(&mut self) {}
}

const PRG_BANK: usize = 8 * 1024;
const CHR_BANK: usize = 1024;

/// Read a byte from `data` as if it were `bank`-sized windows, wrapping if the bank is out of
/// range — which is what a real cartridge's address lines do.
fn banked(data: &[u8], bank: usize, bank_size: usize, offset: usize) -> u8 {
    if data.is_empty() {
        return 0;
    }
    let banks = (data.len() / bank_size).max(1);
    data[(bank % banks) * bank_size + offset]
}

/// NROM (mapper 0): no banking at all.
///
/// The whole ROM is visible at once. A 16 KB image is mirrored into both halves of `$8000..=$FFFF`,
/// which is what makes its reset vector at `$FFFC` resolve.
#[derive(Debug)]
pub struct Nrom {
    prg: Vec<u8>,
    chr: Vec<u8>,
    mirroring: Mirroring,
}

impl Nrom {
    pub fn new(prg: Vec<u8>, chr: Vec<u8>, mirroring: Mirroring) -> Self {
        Self { prg, chr, mirroring }
    }
}

impl Mapper for Nrom {
    fn read_prg(&self, address: u16) -> u8 {
        if self.prg.is_empty() {
            return 0;
        }
        let offset = (address as usize).saturating_sub(0x8000);
        self.prg[offset % self.prg.len()]
    }

    fn write_prg(&mut self, _address: u16, _value: u8) {
        // No registers: the ROM is fixed in place.
    }

    fn read_chr(&self, address: u16) -> u8 {
        self.chr.get(address as usize & 0x1FFF).copied().unwrap_or(0)
    }

    fn write_chr(&mut self, address: u16, value: u8) {
        // Only cartridges with CHR RAM accept this; those with CHR ROM ignore it.
        let index = address as usize & 0x1FFF;
        if index < self.chr.len() {
            self.chr[index] = value;
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

/// UxROM (mapper 2): one switchable 16 KB PRG bank, plus a fixed last bank.
///
/// `$8000..=$BFFF` is selectable and `$C000..=$FFFF` is always the final bank, so the reset vector
/// and interrupt handlers stay reachable no matter which bank is switched in. Used by Mega Man,
/// Castlevania and many others.
#[derive(Debug)]
pub struct UxRom {
    prg: Vec<u8>,
    chr: Vec<u8>,
    bank: usize,
    mirroring: Mirroring,
}

impl UxRom {
    pub fn new(prg: Vec<u8>, chr: Vec<u8>, mirroring: Mirroring) -> Self {
        // CHR is usually RAM on these boards.
        let chr = if chr.is_empty() { vec![0; 8 * 1024] } else { chr };
        Self {
            prg,
            chr,
            bank: 0,
            mirroring,
        }
    }

    fn last_bank(&self) -> usize {
        (self.prg.len() / (16 * 1024)).saturating_sub(1)
    }
}

impl Mapper for UxRom {
    fn read_prg(&self, address: u16) -> u8 {
        let bank = if address < 0xC000 { self.bank } else { self.last_bank() };
        banked(&self.prg, bank, 16 * 1024, address as usize & 0x3FFF)
    }

    fn write_prg(&mut self, _address: u16, value: u8) {
        // Any write into cartridge space selects the low bank.
        self.bank = value as usize & 0x0F;
    }

    fn read_chr(&self, address: u16) -> u8 {
        self.chr.get(address as usize & 0x1FFF).copied().unwrap_or(0)
    }

    fn write_chr(&mut self, address: u16, value: u8) {
        let index = address as usize & 0x1FFF;
        if index < self.chr.len() {
            self.chr[index] = value;
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

/// MMC1 (mapper 1): the most common mapper, configured through a serial shift register.
///
/// Used by Zelda, Metroid, Mega Man 2 and hundreds of others. Its registers are not written
/// directly: a game writes one bit at a time to cartridge space, five writes to load one register,
/// with the destination chosen by the address of the *last* write. Writing a value with bit 7 set
/// resets the sequence — which is how a game recovers from an interrupted write.
#[derive(Debug)]
pub struct Mmc1 {
    prg: Vec<u8>,
    chr: Vec<u8>,

    /// Bits accumulated so far, and how many. Five writes complete a transfer.
    shift: u8,
    shift_count: u8,

    control: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,
}

impl Mmc1 {
    pub fn new(prg: Vec<u8>, chr: Vec<u8>) -> Self {
        let chr = if chr.is_empty() { vec![0; 8 * 1024] } else { chr };
        Self {
            prg,
            chr,
            shift: 0,
            shift_count: 0,
            // Power-on state fixes the last bank at $C000, so the reset vector is reachable
            // before the game has configured anything.
            control: 0x0C,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
        }
    }

    fn prg_banks_16k(&self) -> usize {
        (self.prg.len() / (16 * 1024)).max(1)
    }

    /// Which 16 KB PRG bank appears at `address`, per the control register's mode bits.
    fn prg_bank_for(&self, address: u16) -> usize {
        let mode = (self.control >> 2) & 0x03;
        let bank = (self.prg_bank & 0x0F) as usize;
        let last = self.prg_banks_16k() - 1;

        match mode {
            // 32 KB switching: the low bit of the bank number is ignored.
            0 | 1 => (bank & !1) + usize::from(address >= 0xC000),
            // Fix the first bank at $8000, switch $C000.
            2 => {
                if address < 0xC000 {
                    0
                } else {
                    bank
                }
            },
            // Switch $8000, fix the last bank at $C000. The power-on default.
            _ => {
                if address < 0xC000 {
                    bank
                } else {
                    last
                }
            },
        }
    }

    /// Which 4 KB CHR bank appears at `address`.
    fn chr_bank_for(&self, address: u16) -> usize {
        let eight_kb_mode = (self.control & 0x10) == 0;
        if eight_kb_mode {
            // One 8 KB bank, so the low bit of the register is ignored.
            (self.chr_bank_0 & !1) as usize + usize::from(address >= 0x1000)
        } else if address < 0x1000 {
            self.chr_bank_0 as usize
        } else {
            self.chr_bank_1 as usize
        }
    }
}

impl Mapper for Mmc1 {
    fn read_prg(&self, address: u16) -> u8 {
        banked(&self.prg, self.prg_bank_for(address), 16 * 1024, address as usize & 0x3FFF)
    }

    fn write_prg(&mut self, address: u16, value: u8) {
        // Bit 7 set aborts the sequence and restores the power-on PRG mode.
        if value & 0x80 != 0 {
            self.shift = 0;
            self.shift_count = 0;
            self.control |= 0x0C;
            return;
        }

        // Bits arrive least-significant first.
        self.shift |= (value & 0x01) << self.shift_count;
        self.shift_count += 1;

        if self.shift_count < 5 {
            return;
        }

        // The fifth write commits, to whichever register its address selects.
        let loaded = self.shift;
        self.shift = 0;
        self.shift_count = 0;

        match address {
            0x8000..=0x9FFF => self.control = loaded,
            0xA000..=0xBFFF => self.chr_bank_0 = loaded,
            0xC000..=0xDFFF => self.chr_bank_1 = loaded,
            _ => self.prg_bank = loaded,
        }
    }

    fn read_chr(&self, address: u16) -> u8 {
        banked(&self.chr, self.chr_bank_for(address), 4 * 1024, address as usize & 0x0FFF)
    }

    fn write_chr(&mut self, address: u16, value: u8) {
        let bank = self.chr_bank_for(address);
        let banks = (self.chr.len() / (4 * 1024)).max(1);
        let index = (bank % banks) * 4 * 1024 + (address as usize & 0x0FFF);
        if index < self.chr.len() {
            self.chr[index] = value;
        }
    }

    fn mirroring(&self) -> Mirroring {
        match self.control & 0x03 {
            0 => Mirroring::SingleScreenLower,
            1 => Mirroring::SingleScreenUpper,
            2 => Mirroring::Vertical,
            _ => Mirroring::Horizontal,
        }
    }
}

/// AxROM (mapper 7): 32 KB PRG banking with single-screen mirroring.
///
/// Used by Battletoads and Rare's other titles. Simple by design: one write selects both the
/// program bank and which nametable the whole screen uses.
#[derive(Debug)]
pub struct AxRom {
    prg: Vec<u8>,
    chr: Vec<u8>,
    bank: usize,
    upper_screen: bool,
}

impl AxRom {
    pub fn new(prg: Vec<u8>, chr: Vec<u8>) -> Self {
        let chr = if chr.is_empty() { vec![0; 8 * 1024] } else { chr };
        Self {
            prg,
            chr,
            bank: 0,
            upper_screen: false,
        }
    }
}

impl Mapper for AxRom {
    fn read_prg(&self, address: u16) -> u8 {
        banked(&self.prg, self.bank, 32 * 1024, address as usize & 0x7FFF)
    }

    fn write_prg(&mut self, _address: u16, value: u8) {
        self.bank = (value & 0x07) as usize;
        // Bit 4 picks which single screen the whole background uses.
        self.upper_screen = (value & 0x10) != 0;
    }

    fn read_chr(&self, address: u16) -> u8 {
        self.chr.get(address as usize & 0x1FFF).copied().unwrap_or(0)
    }

    fn write_chr(&mut self, address: u16, value: u8) {
        let index = address as usize & 0x1FFF;
        if index < self.chr.len() {
            self.chr[index] = value;
        }
    }

    fn mirroring(&self) -> Mirroring {
        if self.upper_screen {
            Mirroring::SingleScreenUpper
        } else {
            Mirroring::SingleScreenLower
        }
    }
}

/// MMC3 (mapper 4): eight banks plus a scanline counter.
///
/// Used by Super Mario Bros 3 among many others. Two PRG banks are switchable and two fixed; CHR
/// is split into six windows. Its scanline IRQ is what games use to split the screen — SMB3's
/// status bar is exactly that.
#[derive(Debug)]
pub struct Mmc3 {
    prg: Vec<u8>,
    chr: Vec<u8>,

    /// Which of the eight bank registers the next `$8001` write targets, plus the mode bits.
    bank_select: u8,
    banks: [u8; 8],

    mirroring: Mirroring,

    irq_latch: u8,
    irq_counter: u8,
    irq_enabled: bool,
    irq_pending: bool,
    irq_reload: bool,
}

impl Mmc3 {
    pub fn new(prg: Vec<u8>, chr: Vec<u8>, mirroring: Mirroring) -> Self {
        let chr = if chr.is_empty() { vec![0; 8 * 1024] } else { chr };
        Self {
            prg,
            chr,
            bank_select: 0,
            banks: [0; 8],
            mirroring,
            irq_latch: 0,
            irq_counter: 0,
            irq_enabled: false,
            irq_pending: false,
            irq_reload: false,
        }
    }

    fn prg_banks(&self) -> usize {
        (self.prg.len() / PRG_BANK).max(1)
    }

    /// Which 8 KB PRG bank appears in each of the four windows.
    ///
    /// Bit 6 of the bank-select register swaps which of the two switchable windows is which; the
    /// last bank is always fixed at `$E000`, so the vectors never move.
    fn prg_bank_for(&self, address: u16) -> usize {
        let last = self.prg_banks() - 1;
        let second_last = last.saturating_sub(1);
        let swapped = (self.bank_select & 0x40) != 0;

        match address {
            0x8000..=0x9FFF => {
                if swapped {
                    second_last
                } else {
                    self.banks[6] as usize
                }
            },
            0xA000..=0xBFFF => self.banks[7] as usize,
            0xC000..=0xDFFF => {
                if swapped {
                    self.banks[6] as usize
                } else {
                    second_last
                }
            },
            _ => last,
        }
    }

    /// Which 1 KB CHR bank appears at `address`.
    ///
    /// Bit 7 of the bank-select register swaps the 2 KB and 1 KB halves of pattern space.
    fn chr_bank_for(&self, address: u16) -> usize {
        let address = address & 0x1FFF;
        let inverted = (self.bank_select & 0x80) != 0;
        let slot = if inverted { (address ^ 0x1000) >> 10 } else { address >> 10 };

        match slot {
            0 => (self.banks[0] & 0xFE) as usize,
            1 => (self.banks[0] | 0x01) as usize,
            2 => (self.banks[1] & 0xFE) as usize,
            3 => (self.banks[1] | 0x01) as usize,
            4 => self.banks[2] as usize,
            5 => self.banks[3] as usize,
            6 => self.banks[4] as usize,
            _ => self.banks[5] as usize,
        }
    }
}

impl Mapper for Mmc3 {
    fn read_prg(&self, address: u16) -> u8 {
        banked(&self.prg, self.prg_bank_for(address), PRG_BANK, address as usize & 0x1FFF)
    }

    fn write_prg(&mut self, address: u16, value: u8) {
        // Registers are selected by the address's high bits and whether it is even or odd.
        match (address & 0xE001, address) {
            (0x8000, _) => self.bank_select = value,
            (0x8001, _) => {
                let index = (self.bank_select & 0x07) as usize;
                self.banks[index] = value;
            },
            (0xA000, _) => {
                self.mirroring = if value & 0x01 == 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                };
            },
            (0xA001, _) => { /* PRG-RAM protect; not modelled */ },
            (0xC000, _) => self.irq_latch = value,
            (0xC001, _) => self.irq_reload = true,
            (0xE000, _) => {
                self.irq_enabled = false;
                self.irq_pending = false;
            },
            (0xE001, _) => self.irq_enabled = true,
            _ => {},
        }
    }

    fn read_chr(&self, address: u16) -> u8 {
        banked(&self.chr, self.chr_bank_for(address), CHR_BANK, address as usize & 0x03FF)
    }

    fn write_chr(&mut self, address: u16, value: u8) {
        let bank = self.chr_bank_for(address);
        let banks = (self.chr.len() / CHR_BANK).max(1);
        let index = (bank % banks) * CHR_BANK + (address as usize & 0x03FF);
        if index < self.chr.len() {
            self.chr[index] = value;
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    fn acknowledge_irq(&mut self) {
        self.irq_pending = false;
    }

    /// Count down one scanline, raising the IRQ when the counter reaches zero.
    ///
    /// This is what a game uses to know it has reached a particular line — the mechanism behind
    /// a status bar that stays put while the playfield scrolls.
    fn on_scanline(&mut self) {
        if self.irq_counter == 0 || self.irq_reload {
            self.irq_counter = self.irq_latch;
            self.irq_reload = false;
        } else {
            self.irq_counter -= 1;
        }

        if self.irq_counter == 0 && self.irq_enabled {
            self.irq_pending = true;
        }
    }
}

/// Build the mapper a ROM's header asks for.
///
/// Returns `None` for schemes that are not implemented, so the caller can say so plainly rather
/// than running the game with silently wrong banking.
pub fn create(number: u8, prg: Vec<u8>, chr: Vec<u8>, mirroring: Mirroring) -> Option<Box<dyn Mapper>> {
    match number {
        0 => Some(Box::new(Nrom::new(prg, chr, mirroring))),
        // MMC1 and AxROM control mirroring themselves, so the header's value is only a starting
        // hint and is deliberately not passed along.
        1 => Some(Box::new(Mmc1::new(prg, chr))),
        2 => Some(Box::new(UxRom::new(prg, chr, mirroring))),
        4 => Some(Box::new(Mmc3::new(prg, chr, mirroring))),
        7 => Some(Box::new(AxRom::new(prg, chr))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PRG image where each 8 KB bank is filled with its own index, so a read identifies its bank.
    fn banked_prg(banks: usize) -> Vec<u8> {
        (0..banks).flat_map(|bank| std::iter::repeat_n(bank as u8, PRG_BANK)).collect()
    }

    /// The same idea in 16 KB banks, for mappers that switch at that granularity.
    fn prg_16k_banks(banks: usize) -> Vec<u8> {
        (0..banks).flat_map(|bank| std::iter::repeat_n(bank as u8, 16 * 1024)).collect()
    }

    #[test]
    fn nrom_mirrors_a_16k_image_into_both_halves() {
        let mut prg = vec![0u8; 16 * 1024];
        prg[0] = 0xAA;
        let mapper = Nrom::new(prg, vec![], Mirroring::Horizontal);

        assert_eq!(mapper.read_prg(0x8000), 0xAA);
        assert_eq!(mapper.read_prg(0xC000), 0xAA, "a 16 KB image appears at both $8000 and $C000");
    }

    #[test]
    fn nrom_ignores_writes_to_cartridge_space() {
        let mut mapper = Nrom::new(vec![0x11; 16 * 1024], vec![], Mirroring::Horizontal);
        mapper.write_prg(0x8000, 0xFF);
        assert_eq!(mapper.read_prg(0x8000), 0x11, "cartridge ROM must not be writable");
    }

    #[test]
    fn uxrom_switches_the_low_bank_and_fixes_the_last() {
        let prg: Vec<u8> = (0..4).flat_map(|bank| std::iter::repeat_n(bank as u8, 16 * 1024)).collect();
        let mut mapper = UxRom::new(prg, vec![], Mirroring::Vertical);

        // The last bank is always visible at $C000, which is what keeps the vectors reachable.
        assert_eq!(mapper.read_prg(0xC000), 3);

        assert_eq!(mapper.read_prg(0x8000), 0, "bank 0 selected at reset");
        mapper.write_prg(0x8000, 2);
        assert_eq!(mapper.read_prg(0x8000), 2, "a write should switch the low bank");
        assert_eq!(mapper.read_prg(0xC000), 3, "the fixed bank must not move");
    }

    #[test]
    fn mmc3_fixes_the_last_bank_at_e000() {
        let mapper = Mmc3::new(banked_prg(8), vec![], Mirroring::Horizontal);

        // This is what SMB3 needs to boot at all: the reset vector lives in the last bank.
        assert_eq!(mapper.read_prg(0xE000), 7, "the last bank is hardwired at $E000");
    }

    #[test]
    fn mmc3_switches_prg_banks_through_its_registers() {
        let mut mapper = Mmc3::new(banked_prg(8), vec![], Mirroring::Horizontal);

        // Select register 6 ($8000 window), then write the bank number to $8001.
        mapper.write_prg(0x8000, 6);
        mapper.write_prg(0x8001, 3);
        assert_eq!(mapper.read_prg(0x8000), 3);

        mapper.write_prg(0x8000, 7);
        mapper.write_prg(0x8001, 5);
        assert_eq!(mapper.read_prg(0xA000), 5);
    }

    #[test]
    fn mmc3_bit_6_swaps_the_switchable_windows() {
        let mut mapper = Mmc3::new(banked_prg(8), vec![], Mirroring::Horizontal);
        mapper.write_prg(0x8000, 6);
        mapper.write_prg(0x8001, 3);
        assert_eq!(mapper.read_prg(0x8000), 3);

        // With bit 6 set, the same bank appears at $C000 and $8000 becomes fixed.
        mapper.write_prg(0x8000, 0x40 | 6);
        assert_eq!(mapper.read_prg(0xC000), 3);
        assert_eq!(mapper.read_prg(0x8000), 6, "$8000 becomes the second-to-last bank");
    }

    #[test]
    fn mmc3_lets_the_game_change_mirroring() {
        let mut mapper = Mmc3::new(banked_prg(2), vec![], Mirroring::Horizontal);

        mapper.write_prg(0xA000, 0);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
        mapper.write_prg(0xA000, 1);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn mmc3_scanline_counter_raises_an_irq() {
        let mut mapper = Mmc3::new(banked_prg(2), vec![], Mirroring::Horizontal);

        mapper.write_prg(0xC000, 3); // latch: fire after 3 scanlines
        mapper.write_prg(0xC001, 0); // request reload
        mapper.write_prg(0xE001, 0); // enable

        mapper.on_scanline(); // reload to 3
        assert!(!mapper.irq_pending());
        mapper.on_scanline(); // 2
        mapper.on_scanline(); // 1
        assert!(!mapper.irq_pending(), "should not fire early");
        mapper.on_scanline(); // 0 -> fire
        assert!(mapper.irq_pending(), "counter reaching zero should raise the IRQ");

        mapper.acknowledge_irq();
        assert!(!mapper.irq_pending());
    }

    #[test]
    fn mmc3_irq_can_be_disabled() {
        let mut mapper = Mmc3::new(banked_prg(2), vec![], Mirroring::Horizontal);
        mapper.write_prg(0xC000, 1);
        mapper.write_prg(0xC001, 0);
        mapper.write_prg(0xE000, 0); // disable, and acknowledge anything pending

        for _ in 0..8 {
            mapper.on_scanline();
        }
        assert!(!mapper.irq_pending(), "a disabled counter must not raise an IRQ");
    }


    /// Load one MMC1 register the way a game does: five writes, one bit at a time, low bit first.
    fn mmc1_write(mapper: &mut Mmc1, address: u16, value: u8) {
        for bit in 0..5 {
            mapper.write_prg(address, (value >> bit) & 0x01);
        }
    }

    #[test]
    fn mmc1_defaults_to_the_last_bank_at_c000() {
        // Before a game configures anything it must still be able to read its own reset vector.
        let mapper = Mmc1::new(prg_16k_banks(4), vec![]);
        assert_eq!(mapper.read_prg(0xC000), 3, "power-on state must expose the last bank");
    }

    #[test]
    fn mmc1_loads_a_register_over_five_writes() {
        let mut mapper = Mmc1::new(prg_16k_banks(8), vec![]);

        // Mode 3 (switch $8000, fix last at $C000) is the power-on default.
        mmc1_write(&mut mapper, 0xE000, 2);
        assert_eq!(mapper.read_prg(0x8000), 2, "the PRG bank register should have taken effect");
        assert_eq!(mapper.read_prg(0xC000), 7, "the last bank stays fixed");
    }

    #[test]
    fn mmc1_ignores_an_incomplete_transfer() {
        let mut mapper = Mmc1::new(prg_16k_banks(8), vec![]);

        // Four writes are not enough; nothing should change.
        for bit in 0..4 {
            mapper.write_prg(0xE000, (3 >> bit) & 0x01);
        }
        assert_eq!(mapper.read_prg(0x8000), 0, "a partial transfer must not commit");
    }

    #[test]
    fn mmc1_reset_bit_aborts_a_transfer_and_restores_the_default_mode() {
        let mut mapper = Mmc1::new(prg_16k_banks(8), vec![]);

        // Start a transfer, then abort it midway — which is how a game recovers if an interrupt
        // lands between writes.
        mapper.write_prg(0xE000, 1);
        mapper.write_prg(0xE000, 1);
        mapper.write_prg(0xE000, 0x80);

        // The aborted bits must not leak into the next transfer.
        mmc1_write(&mut mapper, 0xE000, 1);
        assert_eq!(mapper.read_prg(0x8000), 1);
        assert_eq!(mapper.read_prg(0xC000), 7, "reset restores the fixed-last-bank mode");
    }

    #[test]
    fn mmc1_prg_modes_place_banks_where_the_control_register_says() {
        let mut mapper = Mmc1::new(prg_16k_banks(8), vec![]);

        // Mode 2: first bank fixed at $8000, switchable at $C000.
        mmc1_write(&mut mapper, 0x8000, 0b0_1000);
        mmc1_write(&mut mapper, 0xE000, 5);
        assert_eq!(mapper.read_prg(0x8000), 0, "mode 2 fixes bank 0 at $8000");
        assert_eq!(mapper.read_prg(0xC000), 5);

        // Mode 0/1: 32 KB switching, so the low bit of the bank number is ignored.
        mmc1_write(&mut mapper, 0x8000, 0b0_0000);
        mmc1_write(&mut mapper, 0xE000, 5);
        assert_eq!(mapper.read_prg(0x8000), 4, "32 KB mode ignores the low bit");
        assert_eq!(mapper.read_prg(0xC000), 5);
    }

    #[test]
    fn mmc1_controls_its_own_mirroring() {
        let mut mapper = Mmc1::new(prg_16k_banks(2), vec![]);

        for (bits, expected) in [
            (0, Mirroring::SingleScreenLower),
            (1, Mirroring::SingleScreenUpper),
            (2, Mirroring::Vertical),
            (3, Mirroring::Horizontal),
        ] {
            mmc1_write(&mut mapper, 0x8000, 0b0_1100 | bits);
            assert_eq!(mapper.mirroring(), expected, "control bits {bits} should select {expected:?}");
        }
    }

    #[test]
    fn axrom_switches_32k_banks_and_picks_a_single_screen() {
        let prg: Vec<u8> = (0..4).flat_map(|bank| std::iter::repeat_n(bank as u8, 32 * 1024)).collect();
        let mut mapper = AxRom::new(prg, vec![]);

        assert_eq!(mapper.read_prg(0x8000), 0);
        assert_eq!(mapper.mirroring(), Mirroring::SingleScreenLower);

        // One write sets both the bank and which nametable the whole screen uses.
        mapper.write_prg(0x8000, 0x02 | 0x10);
        assert_eq!(mapper.read_prg(0x8000), 2);
        assert_eq!(mapper.read_prg(0xFFFF), 2, "a 32 KB bank covers the whole window");
        assert_eq!(mapper.mirroring(), Mirroring::SingleScreenUpper);
    }

    #[test]
    fn unimplemented_mappers_are_reported_rather_than_guessed() {
        assert!(create(0, vec![0; 16 * 1024], vec![], Mirroring::Horizontal).is_some());
        assert!(create(1, vec![0; 16 * 1024], vec![], Mirroring::Horizontal).is_some());
        assert!(create(4, vec![0; 16 * 1024], vec![], Mirroring::Horizontal).is_some());
        assert!(create(7, vec![0; 16 * 1024], vec![], Mirroring::Horizontal).is_some());
        assert!(
            create(99, vec![0; 16 * 1024], vec![], Mirroring::Horizontal).is_none(),
            "an unknown mapper must not silently fall back to NROM"
        );
    }
}
