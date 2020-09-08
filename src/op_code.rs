#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OpCode {
    ADC,
    BRK,
    CLC,
    CLI,
    JMP,
    LDA,
    NOP,
    SBC,
    SEC,
    SEI,
}
