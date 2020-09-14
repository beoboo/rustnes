#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub enum AddressingMode {
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Immediate,
    Implied,
    Indirect,
    IndirectX,
    IndirectY,
    XIndirect,
    YIndirect,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
}
