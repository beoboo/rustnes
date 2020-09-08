use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
pub enum OpCode {
    ADC,
    BRK,
    CLC,
    JMP,
    LDA,
    NOP,
    SBC,
    SEC,
    Unknown,
}
