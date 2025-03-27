.segment "HEADER"
    .byte $4E, $45, $53, $1A   ; NES header
    .byte $02                   ; 2x 16KB PRG ROM
    .byte $01                   ; 1x 8KB CHR ROM
    .byte $01                   ; Vertical mirroring, no battery RAM
    .byte $00                   ; No mapper, plain NROM
    .byte $00, $00, $00, $00   ; Padding bytes
    .byte $00, $00, $00, $00   ; Padding bytes

.segment "STARTUP"
    ; Initialize stack pointer
    LDX #$FF
    TXS

    ; Enable noise channel
    LDA #$08
    STA $4015

    ; Main loop
main_loop:
    ; Play different noise patterns
    LDX #$00
pattern_loop:
    ; Set up noise channel
    LDA #$3F    ; Volume = 15, constant volume
    STA $400C

    ; Set noise period (X)
    LDA #$00    ; Using fixed period for now
    STA $400E

    ; Set length counter
    LDA #$20    ; Medium length
    STA $400F

    ; Wait a bit
    LDX #$40
wait_loop:
    DEX
    BNE wait_loop

    INX
    CPX #$10    ; Try 16 different noise patterns
    BNE pattern_loop

    JMP main_loop

.segment "VECTORS"
    .word $0000, $0000, $0000  ; NMI, Reset, IRQ vectors (not used in this test) 