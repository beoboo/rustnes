#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OpCode {
    ADC,
    BRK,
    CLC,
    CLD,
    CLI,
    JMP,
    LDA,
    LDX,
    NOP,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    TXS,
}
