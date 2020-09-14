#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub enum AddressingMode {
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Immediate,
    Indirect,
    IndirectX,
    IndirectY,
    XIndirect,
    YIndirect,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
}
