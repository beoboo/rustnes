//! Reading the text a ROM has drawn on screen.
//!
//! Blargg's later ROMs write their result to `$6000` and a message to `$6004`, which is what
//! [`crate::blargg`] reads and why most of the suites need no screen. His earlier ones — the 2005
//! PPU tests, `branch_timing_tests`, `mmc3_irq_tests`, `cpu_timing_test6`, `nmi_sync` — predate
//! that protocol and report on screen only. Seventeen ROMs across five suites, every one of which
//! this runner counted as a failure for want of a way to read the answer rather than because the
//! emulator got it wrong. The first one tried turns out to say PASSED.
//!
//! Reading them is easier than it sounds, because the console code these ROMs share writes ASCII
//! straight into the nametable: the tile index *is* the character code. That was checked before
//! being relied on rather than assumed — the pattern tables were dumped alongside the nametable,
//! and it is the nametable that carries the text.

use std::path::Path;

use anyhow::{Context, Result};
use rn_core::{cartridge::load_rom, system::NesSystem};

/// A nametable is 32 tiles across and 30 down, its tiles preceding its attribute bytes.
const COLUMNS: usize = 32;
const ROWS: usize = 30;

/// The first nametable. A ROM that only prints text has no reason to scroll, and these all draw
/// into this one.
const NAMETABLE: u16 = 0x2000;

/// What a ROM has drawn, as text.
pub struct Screen {
    /// One string per row, trailing blanks trimmed.
    pub rows: Vec<String>,
}

impl Screen {
    /// The screen as one string, blank rows dropped.
    pub fn text(&self) -> String {
        self.rows
            .iter()
            .map(|row| row.trim_end())
            .filter(|row| !row.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Decode the first nametable as text.
pub fn read(system: &NesSystem) -> Screen {
    let rows = (0..ROWS)
        .map(|row| {
            (0..COLUMNS)
                .map(|column| {
                    let tile = system.ppu().read_vram(NAMETABLE + (row * COLUMNS + column) as u16);
                    // Anything outside printable ASCII is a graphic rather than a letter, and
                    // reading it as a character would put noise in the middle of the answer.
                    if (0x20..0x7F).contains(&tile) {
                        tile as char
                    } else {
                        ' '
                    }
                })
                .collect::<String>()
        })
        .collect();

    Screen { rows }
}

/// Run a ROM and print what it drew.
pub fn report(rom_path: &Path, frames: usize, raw: bool) -> Result<()> {
    let rom = load_rom(rom_path)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| format!("loading {}", rom_path.display()))?;

    let mut system = NesSystem::new();
    system
        .load_rom(&rom)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("loading the ROM into the system")?;

    let target = system.ppu().frame_count() + frames as u64;
    while system.ppu().frame_count() < target {
        if system.step().is_err() {
            break;
        }
    }

    let screen = read(&system);

    if raw {
        for (row, text) in screen.rows.iter().enumerate() {
            let mut hex = String::new();
            for column in 0..COLUMNS {
                let tile = system.ppu().read_vram(NAMETABLE + (row * COLUMNS + column) as u16);
                hex.push_str(&format!("{tile:02X} "));
            }
            println!("{row:2} {hex}|{text}|");
        }
    } else {
        println!("{}", screen.text());
    }

    Ok(())
}

/// What a screen says about whether the ROM passed, if it says anything.
///
/// A pure function of the text, so it can be tested without a ROM — which matters here, since none
/// can be committed to this repository.
///
/// Two forms appear across these suites, and both are the ROM author's own convention rather than
/// this runner's invention:
///
/// - The console-based ROMs print a title and then `PASSED` or `FAILED #n`.
/// - The 2005 PPU tests print a result code as `$nn`, of which the readme says "a result code of 1
///   always indicates that all tests were passed"; anything else is that test's own error number,
///   listed per ROM in the same file.
/// - `blargg_nes_cpu_test5` says neither. It lists the tests it ran and ends with "All tests
///   complete", or with "Errors: n" and "Failed". Both endings were produced before this was
///   written — the failing one by deliberately breaking `ASL`'s carry flag and running the ROM —
///   rather than inferred from the passing one, because "complete" is not "passed" and a rule that
///   assumed so would turn a broken emulator green.
///
/// Anything else returns `None` rather than a guess. A wrong verdict read off the screen is worse
/// than no verdict at all: it would turn an unmeasured ROM into a green one.
pub fn verdict(text: &str) -> Option<Verdict> {
    let upper = text.to_uppercase();

    if upper.contains("PASSED") {
        return Some(Verdict::Passed);
    }

    // Checked before the completion line below, so a run that somehow printed both is a failure.
    if let Some(rest) = upper.split_once("FAILED").map(|(_, rest)| rest) {
        // "FAILED #3" carries the sub-test number; a bare "FAILED" does not.
        let code = rest
            .trim_start()
            .strip_prefix('#')
            .and_then(|digits| {
                digits
                    .trim_start()
                    .chars()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>()
                    .parse()
                    .ok()
            })
            .unwrap_or(0);
        return Some(Verdict::Failed { code });
    }

    // The instruction-battery form: a list of what ran, and a line saying it got to the end. A
    // failing run does not reach it — it stops and says "Errors: n" and "Failed" instead.
    if upper.contains("ALL TESTS COMPLETE") {
        return Some(Verdict::Passed);
    }

    // A result code on its own, as `$nn` — and *on its own* is the whole of the rule. These ROMs
    // print the code as the only thing on its line, which is what tells it apart from a dollar sign
    // that happens to fall out of a screen this runner cannot read.
    //
    // `MMC1_A12/mmc1_a12.nes` is why the rule is this strict. It draws with its own character set
    // rather than ASCII, so the nametable decodes to noise — "42$ 4;# +;1 3." and the like — and a
    // search for `$` anywhere in the text found one and reported `FAILED #2` for a ROM whose screen
    // says nothing this runner understands. A wrong verdict is worse than no verdict in either
    // direction: a false pass hides a bug, and a false failure sends someone after one that is not
    // there.
    let code = upper.lines().find_map(|line| {
        let rest = line.trim().strip_prefix('$')?;
        let digits: String = rest.chars().take_while(char::is_ascii_hexdigit).collect();
        // Nothing after the digits, or it was not a bare code.
        (!digits.is_empty() && digits.len() == rest.len())
            .then(|| u8::from_str_radix(&digits, 16).ok())?
    })?;

    Some(if code == 1 { Verdict::Passed } else { Verdict::Failed { code } })
}

/// The verdict a screen carries.
#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    Passed,
    Failed { code: u8 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_console_forms() {
        assert_eq!(verdict("BRANCH TIMING BASICS\nPASSED"), Some(Verdict::Passed));
        assert_eq!(
            verdict("MMC3 IRQ COUNTER REVISION A\nFAILED #3"),
            Some(Verdict::Failed { code: 3 })
        );
        // A bare FAILED is still a failure, just without a sub-test number.
        assert_eq!(verdict("SOMETHING\nFAILED"), Some(Verdict::Failed { code: 0 }));
    }

    /// The 2005 PPU tests report a bare code, where one means everything passed.
    #[test]
    fn reads_the_result_code_form() {
        assert_eq!(verdict("$01"), Some(Verdict::Passed));
        assert_eq!(verdict("$06"), Some(Verdict::Failed { code: 6 }));
        assert_eq!(verdict("$0A"), Some(Verdict::Failed { code: 10 }));
        // The ROMs print a title above it, so the code is a line rather than the whole screen.
        assert_eq!(verdict("PALETTE_RAM\n$01"), Some(Verdict::Passed));
    }

    /// The instruction batteries end with a completion line rather than a verdict.
    ///
    /// Both endings were produced from the emulator before this rule was written: the passing one
    /// by running `blargg_nes_cpu_test5/official` as it stood, the failing one by deliberately
    /// breaking `ASL`'s carry flag first. Guessing that "complete" meant "passed" would have made
    /// a broken emulator report green, which is the one outcome worth going out of the way to
    /// prevent.
    #[test]
    fn reads_the_completion_form_and_the_failure_that_replaces_it() {
        assert_eq!(
            verdict("01-implied\n02-immediate\nAll tests complete"),
            Some(Verdict::Passed)
        );
        assert_eq!(
            verdict("01-implied\n02-immediate\nErrors: 5\nFailed"),
            Some(Verdict::Failed { code: 0 })
        );
        // And a failure wins even if both lines somehow appear.
        assert_eq!(
            verdict("All tests complete\nFailed"),
            Some(Verdict::Failed { code: 0 })
        );
    }

    /// A screen this runner cannot read must not be turned into a verdict by a stray dollar sign.
    ///
    /// `MMC1_A12/mmc1_a12.nes` draws with its own character set, so its nametable decodes to noise.
    /// The text below is what this runner actually read off it, and a search for `$` anywhere in
    /// the text found one and called the ROM `FAILED #2`. It is worth a test of its own because the
    /// failure mode is silent: a wrong verdict looks exactly like a right one in a summary line.
    #[test]
    fn a_screen_of_noise_is_not_a_result_code() {
        let noise = "   ,,\"  61 , #(2 !+$ 2\" -+(-$\n         \".4-3$1 3$23\n\
                     42$ 4;# +;1 3.  #)423 #$+ 8";
        assert_eq!(verdict(noise), None);
    }

    /// And the form it has to be distinguished from: the code alone on its line, which is exactly
    /// what `blargg_ppu_tests_2005.09.15b` prints — the whole screen is `$01`.
    #[test]
    fn a_bare_code_on_its_own_line_still_reads() {
        assert_eq!(verdict("  $01"), Some(Verdict::Passed));
        assert_eq!(verdict("VBL_CLEAR_TIME\n$06"), Some(Verdict::Failed { code: 6 }));
        // But not one with anything else on the line with it.
        assert_eq!(verdict("COST $06 EACH"), None);
    }

    /// The important negative: a screen that says nothing must not be read as saying something.
    ///
    /// A ROM whose output this runner cannot interpret has to stay unmeasured. Turning it green
    /// would be worse than the blank it replaces, because nobody re-checks a passing test.
    #[test]
    fn says_nothing_about_a_screen_it_cannot_read() {
        assert_eq!(verdict(""), None);
        assert_eq!(verdict("   \n  \n"), None);
        assert_eq!(verdict("A DEMO WITH NO VERDICT"), None);
        // A stray dollar sign with no digits after it is not a result code.
        assert_eq!(verdict("COST: $ SOMETHING"), None);
    }

    /// "PASSED" wins over a stray code, since the console ROMs draw both a title and a verdict.
    #[test]
    fn prefers_an_explicit_verdict_to_a_number() {
        assert_eq!(verdict("TEST $05 OF $10\nPASSED"), Some(Verdict::Passed));
    }
}
