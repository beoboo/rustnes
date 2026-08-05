//! Reference verdicts: run a blargg-protocol ROM in tetanes and report what it says.
//!
//! Exists to answer one question this project cannot answer alone — whether a known-good emulator
//! passes the ROMs we still fail — rather than to be part of the emulator. A ROM we both fail is
//! obscure; one that tetanes passes is a bug of ours with a readable reference beside it.

use tetanes_core::control_deck::ControlDeck;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let rom = args.next().expect("usage: nesref <rom> [frames]");
    let frames: usize = args.next().unwrap_or_else(|| "1800".into()).parse()?;

    let mut deck = ControlDeck::new();
    deck.load_rom_path(&rom)?;

    for _ in 0..frames {
        let _ = deck.clock_frame()?;
    }

    // Blargg's ROMs put their status at $6000 and a message at $6004. Read straight off the bus
    // rather than through `sram`, which only answers for battery-backed carts.
    let bus = deck.bus_mut();
    let status = bus.peek(0x6000);
    let signature = [bus.peek(0x6001), bus.peek(0x6002), bus.peek(0x6003)];
    let mut message = String::new();
    for offset in 0..512u16 {
        match bus.peek(0x6004 + offset) {
            0 => break,
            byte => message.push(byte as char),
        }
    }

    let verdict = match status {
        _ if signature != [0xDE, 0xB0, 0x61] => "NO PROTOCOL".to_string(),
        0x00 => "PASS".to_string(),
        0x80 => "still running".to_string(),
        0x81 => "wants a reset".to_string(),
        code => format!("FAIL({code:02X})"),
    };

    println!("{verdict}  {}", message.trim().replace('\n', " | "));
    Ok(())
}
