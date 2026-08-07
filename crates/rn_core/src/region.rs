//! Which console the cartridge expects: an American NES or a European one.
//!
//! Not a cosmetic difference. The two machines run the *same* CPU against *differently* clocked
//! video, so every piece of timing a game counts in CPU cycles lands somewhere else on the screen:
//!
//! | | NTSC | PAL |
//! |---|---|---|
//! | PPU dots per CPU cycle | 3 | 3.2 |
//! | scanlines in a frame | 262 | 312 |
//! | the odd-frame dot skip | yes | no |
//!
//! The consequence is not subtle, and this project has a scar from it. Super Mario Bros 3's
//! status-bar split wobbled by eight pixels for three separate investigations, each of which went
//! looking for a bug in the interrupt, the mapper or the CPU. There was none: the cartridge was
//! `super-mario-3-eu.nes`, its handler ends in a delay loop the developers tuned for 3.2 dots to a
//! cycle, and running it at 3.0 puts the whole burst 41 dots early — out of hblank and into the
//! visible line, where the CPU's genuine interrupt latency becomes something you can see.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Region {
    #[default]
    Ntsc,
    Pal,
}

impl Region {
    /// PPU dots per CPU cycle, as a fraction.
    ///
    /// PAL's 3.2 is exact — sixteen dots for every five CPU cycles — and has to be carried as a
    /// ratio rather than rounded, because the fifth cycle of every five really does get a fourth
    /// dot and a game counting cycles can see it.
    pub const fn dots_per_cycle(self) -> (u32, u32) {
        match self {
            Region::Ntsc => (3, 1),
            Region::Pal => (16, 5),
        }
    }

    /// The pre-render line, which is also the last line of the frame.
    pub const fn pre_render_scanline(self) -> i16 {
        match self {
            Region::Ntsc => 261,
            Region::Pal => 311,
        }
    }

    /// Whether this console skips a dot on the pre-render line of odd frames.
    ///
    /// NTSC does, to keep the colour subcarrier in step across frames; PAL's timing does not need
    /// it and its PPU does not do it.
    pub const fn skips_a_dot_on_odd_frames(self) -> bool {
        matches!(self, Region::Ntsc)
    }

    /// What the iNES header claims, which is worth reading and worth not trusting.
    ///
    /// Byte 9 bit 0 is the TV system, and plenty of European releases ship with it clear —
    /// `super-mario-3-eu.nes` says NTSC and is a PAL cartridge. Reference emulators keep a
    /// database of cartridge hashes for exactly this reason. So this is a starting point, and
    /// there is an explicit override beside it for when the header lies.
    pub const fn from_ines_header(byte9: u8) -> Self {
        if byte9 & 0x01 != 0 { Region::Pal } else { Region::Ntsc }
    }
}

/// Turns a CPU cycle into PPU dots, carrying the fraction PAL needs.
///
/// NTSC's three-a-cycle needs no state at all; this exists for PAL, where the remainder has to be
/// kept between cycles or the sixteen-dots-per-five-cycles average is lost to rounding.
#[derive(Debug, Clone, Copy)]
pub struct DotClock {
    region: Region,
    remainder: u32,
}

impl DotClock {
    pub const fn new(region: Region) -> Self {
        Self { region, remainder: 0 }
    }

    pub const fn region(&self) -> Region {
        self.region
    }

    /// How many dots this CPU cycle is worth. For NTSC always three; for PAL three, with a fourth
    /// on every fifth cycle.
    pub fn dots_for_this_cycle(&mut self) -> u32 {
        let (numerator, denominator) = self.region.dots_per_cycle();
        let total = self.remainder + numerator;
        self.remainder = total % denominator;
        total / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// NTSC is exactly three dots a cycle, with nothing accumulated between them.
    #[test]
    fn ntsc_is_three_dots_every_cycle() {
        let mut clock = DotClock::new(Region::Ntsc);
        for _ in 0..100 {
            assert_eq!(clock.dots_for_this_cycle(), 3);
        }
    }

    /// PAL averages 3.2 by giving every fifth cycle a fourth dot — and it must be *every* fifth,
    /// not a drifting one, or a game's cycle-counted delay lands on a different dot each frame.
    #[test]
    fn pal_is_sixteen_dots_every_five_cycles() {
        let mut clock = DotClock::new(Region::Pal);
        let pattern: Vec<u32> = (0..10).map(|_| clock.dots_for_this_cycle()).collect();
        assert_eq!(pattern, vec![3, 3, 3, 3, 4, 3, 3, 3, 3, 4]);
    }

    /// Over a whole frame the two rates differ by exactly the amount that moved Super Mario Bros
    /// 3's split: a 206-cycle interrupt handler spans 618 dots on NTSC and 659 on PAL.
    #[test]
    fn the_two_rates_diverge_by_the_amount_the_smb3_handler_did() {
        let span = |region: Region| {
            let mut clock = DotClock::new(region);
            (0..206).map(|_| clock.dots_for_this_cycle()).sum::<u32>()
        };
        assert_eq!(span(Region::Ntsc), 618);
        assert_eq!(span(Region::Pal), 659, "measured against tetanes at the same scene");
    }

    /// A PAL frame is fifty scanlines longer, and does not skip a dot.
    #[test]
    fn pal_frames_are_longer_and_do_not_skip() {
        assert_eq!(Region::Ntsc.pre_render_scanline(), 261);
        assert_eq!(Region::Pal.pre_render_scanline(), 311);
        assert!(Region::Ntsc.skips_a_dot_on_odd_frames());
        assert!(!Region::Pal.skips_a_dot_on_odd_frames());
    }

    /// The header is read, and the default when it says nothing is NTSC.
    #[test]
    fn the_header_is_a_starting_point() {
        assert_eq!(Region::from_ines_header(0x00), Region::Ntsc);
        assert_eq!(Region::from_ines_header(0x01), Region::Pal);
        assert_eq!(Region::default(), Region::Ntsc);
    }
}

/// The APU's region-dependent tables, checked where they are used rather than only where they are
/// declared — a table that exists but is never selected is the failure mode worth testing for.
#[cfg(test)]
mod apu_tables {
    use crate::apu::Apu;
    use crate::memory::Addressable;
    use crate::region::Region;

    /// Writing `$4010` picks a DMC rate from the table for the console the APU is set to. Rate 0
    /// is 428 CPU cycles on NTSC and 398 on PAL — a PAL CPU is slower, so the same audible pitch
    /// needs fewer of its cycles, and the table is its own list rather than the NTSC one scaled.
    #[test]
    fn the_dmc_rate_table_follows_the_region() {
        let period = |region: Region, rate: u8| {
            let mut apu = Apu::new();
            apu.set_region(region);
            apu.write_byte(0x4010, rate).unwrap();
            apu.dmc_period()
        };

        assert_eq!(period(Region::Ntsc, 0x00), 428);
        assert_eq!(period(Region::Pal, 0x00), 398);
        assert_eq!(period(Region::Ntsc, 0x0F), 54);
        assert_eq!(period(Region::Pal, 0x0F), 50);
    }

    /// And the noise channel's, which is a different list again.
    #[test]
    fn the_noise_period_table_follows_the_region() {
        let period = |region: Region, index: u8| {
            let mut apu = Apu::new();
            apu.set_region(region);
            apu.write_byte(0x400E, index).unwrap();
            apu.noise_period()
        };

        assert_eq!(period(Region::Ntsc, 0x0F), 4068);
        assert_eq!(period(Region::Pal, 0x0F), 3778);
        assert_eq!(period(Region::Ntsc, 0x02), 16);
        assert_eq!(period(Region::Pal, 0x02), 14);
    }

    /// The frame sequencer's IRQ arrives later on PAL, because its sequence is longer: 33254 CPU
    /// cycles against 29830. Driven through `tick` rather than read off a constant, so this fails
    /// if the table exists but nothing selects it.
    #[test]
    fn the_frame_sequencer_is_longer_on_pal() {
        let cycles_to_irq = |region: Region| {
            let mut apu = Apu::new();
            apu.set_region(region);
            // 4-step mode with the IRQ allowed.
            apu.write_byte(0x4017, 0x00).unwrap();
            for cycle in 1..60_000u64 {
                apu.tick();
                if apu.irq_pending() {
                    return cycle;
                }
            }
            0
        };

        let (ntsc, pal) = (cycles_to_irq(Region::Ntsc), cycles_to_irq(Region::Pal));
        assert!(ntsc > 0 && pal > 0, "the frame IRQ never arrived: NTSC {ntsc}, PAL {pal}");
        assert!(
            pal > ntsc,
            "PAL's sequence is ~11% longer, so its frame IRQ must come later: NTSC {ntsc}, PAL {pal}"
        );
        assert!(
            (pal as f64 / ntsc as f64 - 33254.0 / 29830.0).abs() < 0.01,
            "the two should differ by the ratio of their sequence lengths: NTSC {ntsc}, PAL {pal}"
        );
    }
}
