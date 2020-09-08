use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
pub enum OpCode {
    BRK = 0x00,
    ADC = 0x69,
    CLC = 0x18,
    JMP = 0x4C,
    LDA = 0xA9,
    SBC = 0xE9,
    SEC = 0x38,
    NOP = 0xEA,
    Unknown,
}
