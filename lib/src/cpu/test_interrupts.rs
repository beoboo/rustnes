use hamcrest2::core::*;
use hamcrest2::prelude::*;

use super::*;
use crate::bus::simple_bus::SimpleBus;

#[test]
fn reset() {
    let mut cpu = Cpu::new(0);
    let mut bus = SimpleBus::default();

    bus.write_word(0xFFFC, 0x1234);

    cpu.reset(&mut bus);

    assert_that!(cpu.A, eq(0));
    assert_that!(cpu.X, eq(0));
    assert_that!(cpu.Y, eq(0));
    assert_that!(cpu.PC, eq(0x1234));
    assert_that!(cpu.SP, eq(0xFD));
    assert_that!(cpu.status, eq(Status::from_string("czidbUvn")));
}
