use rn_core::{cartridge::load_rom, system::NesSystem};

/// The nametable map should show real content and mark the viewport.
#[test]
fn map_renders_content_and_marks_the_viewport() {
    // Commercial ROMs cannot live in this repository, so the test skips cleanly without one rather
    // than failing on a machine that has none. `RN_REQUIRE_ROMS` turns the skip into a failure, for
    // a machine that does have them: a skipped test and a passing one look identical in `cargo
    // test`, and one that quietly measures nothing is worse than one that is not there.
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../nes-roms/donkey-kong.nes");
    if !path.exists() {
        assert!(
            std::env::var_os("RN_REQUIRE_ROMS").is_none(),
            "RN_REQUIRE_ROMS is set but there is no ROM at {}",
            path.display()
        );
        eprintln!("SKIP: no ROM at {} (set RN_REQUIRE_ROMS to make this a failure)", path.display());
        return;
    }
    let path = path.as_path();

    let rom = load_rom(path).expect("loading");
    let mut system = NesSystem::new();
    system.load_rom(&rom).expect("loading into system");
    for _ in 0..400_000 {
        if system.step().is_err() { break }
    }

    let map = system.ppu().render_nametable_map();
    assert_eq!(map.len(), 512 * 480 * 3, "map should be 512x480 RGB");

    let marker = [255u8, 32, 32];
    let markers = map.chunks_exact(3).filter(|p| *p == marker).count();
    // The outline is a 256x240 rectangle: 2*256 + 2*240 pixels, minus overlap at the corners.
    assert!(markers > 900, "viewport outline should be drawn, found {markers} marker pixels");

    let distinct: std::collections::HashSet<_> = map.chunks_exact(3).map(|p| [p[0],p[1],p[2]]).collect();
    println!("MAP distinct colours: {}", distinct.len());
    assert!(distinct.len() >= 3, "map should show real content, got {} colours", distinct.len());
}
