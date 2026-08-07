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
