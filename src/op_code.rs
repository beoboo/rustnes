use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
pub enum OpCode {
    BRK = 0x00,
    ADC = 0x69,
    CLC = 0x18,
    LDA = 0xA9,
    SEC = 0x38,
    Unknown
}
