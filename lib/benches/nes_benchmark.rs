use criterion::{criterion_group, criterion_main, Criterion};
use rustnes_lib::nes::Nes;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut nes = Nes::default();
    nes.load("../roms/cpu/nestest/nestest.nes");
    // let mut nes = Nes::new("../roms/cpu/instr_test-v5/official_only.nes");
    // let mut nes = Nes::new("../roms/mul3.nes");
    nes.reset();

    c.bench_function("nes 1", |b| b.iter(|| nes.tick()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);