.segment "HEADER"
.byte "NES", 26, 2, 1, 0, 0

.segment "CODE"
.proc irq_handler
  RTI
.endproc

.proc nmi_handler
  RTI
.endproc

.proc reset_handler
  SEI
  CLD
  LDX #$00
  STX $2000
  STX $2001
vblankwait:
  BIT $2002
  BPL vblankwait
  JMP main
.endproc

.proc main
    LDX #10
    STX $0000
    LDX #3
    STX $0001
    LDY $0000
    LDA #0
    CLC
    loop:
    ADC $0001
    DEY
    BNE loop
    STA $0002
    NOP
    NOP
    NOP
.endproc

.segment "VECTORS"
.addr nmi_handler, reset_handler, irq_handler

.segment "CHARS"
.res 8192
.segment "STARTUP"