use rn_core::{cartridge::load_rom, memory::Addressable, system::NesSystem};
#[test]
fn dmc_fetch_count() {
    let rom = load_rom(std::path::Path::new("../../../nes-roms/super-mario-3-eu.nes")).expect("rom");
    let mut s = NesSystem::new();
    s.load_rom(&rom).expect("load");
    let t = std::fs::read_to_string("../../../nes-roms/super-mario-3-eu.state.json").expect("st");
    let st: rn_core::system::SaveState = serde_json::from_str(&t).expect("p");
    s.load_state(&st).expect("r");

    // Force the DMC on the way the music engine would, to prove the plumbing works at all.
    if std::env::var("FORCE_DMC").is_ok() {
        s.cpu().write_byte(0x4010, 0x0F).ok(); // fastest rate, looping off
        s.cpu().write_byte(0x4012, 0xC0).ok(); // sample at $F000
        s.cpu().write_byte(0x4013, 0xFF).ok(); //长 sample
        s.cpu().write_byte(0x4015, 0x10).ok(); // enable DMC
    }

    let before = s.cpu().cycles();
    let mut frames = 0;
    for _ in 0..10 {
        let b = s.ppu().frame_count();
        while s.ppu().frame_count()==b { if s.step().is_err(){break} }
        frames += 1;
    }
    let cycles = s.cpu().cycles() - before;
    println!("  {frames} frames, {cycles} CPU cycles, {:.2} per frame", cycles as f64 / frames as f64);
    println!("  $4015 = {:02X}", s.cpu().read_byte(0x4015).unwrap_or(0));
}
