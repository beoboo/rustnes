#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub enum AddressingMode {
    Accumulator,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Immediate,
    Implied,
    Indirect,
    IndirectIndexedX,
    Relative,
    YIndexedIndirect,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
}
