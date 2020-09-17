#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OpCode {
    ADC,
    BNE,
    BPL,
    BRK,
    CLC,
    CLD,
    CLI,
    CPX,
    DEX,
    DEY,
    INX,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    NOP,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TXS,
}
