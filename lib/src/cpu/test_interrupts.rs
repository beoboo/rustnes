use hamcrest2::core::*;
use hamcrest2::prelude::*;

use crate::bus::simple_bus::SimpleBus;

use super::*;

#[test]
fn reset() {
    let mut cpu = Cpu::new(0);
    let mut bus = SimpleBus::default();

    bus.write_word(Cpu::RESET_ADDRESS, 0x1234);

    cpu.reset(&mut bus);

    assert_that!(cpu.A, eq(0));
    assert_that!(cpu.X, eq(0));
    assert_that!(cpu.Y, eq(0));
    assert_that!(cpu.PC, eq(0x1234));
    assert_that!(cpu.SP, eq(0xFD));
    assert_that!(cpu.status, eq(Status::from_string("czidbUvn")));
    assert_that!(cpu.left_cycles, eq(7));
}
