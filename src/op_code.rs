#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OpCode {
    ADC,
    BRK,
    CLC,
    CLD,
    CLI,
    JMP,
    LDA,
    NOP,
    SBC,
    SEC,
    SED,
    SEI,
}
