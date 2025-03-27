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

    ; Enable triangle channel
    LDA #$04
    STA $4015

    ; Set up triangle channel
    LDA #$80    ; Linear counter = 128, length counter halt flag set
    STA $4008

    ; Main loop
main_loop:
    ; Play ascending tones
    LDX #$00
ascending:
    ; Set frequency (period = 0x40 - X)
    LDA #$00
    STA $400A
    LDA #$01    ; High byte of period (0x40)
    STA $400B   ; This also loads length counter with value 1

    ; Wait a bit
    LDX #$20
wait_loop:
    DEX
    BNE wait_loop

    INX
    CPX #$40
    BNE ascending

    ; Play descending tones
    LDX #$40
descending:
    DEX
    BEQ main_loop

    ; Set frequency (period = 0x40 - X)
    LDA #$00
    STA $400A
    LDA #$01    ; High byte of period (0x40)
    STA $400B   ; This also loads length counter with value 1

    ; Wait a bit
    LDX #$20
wait_loop2:
    DEX
    BNE wait_loop2

    JMP descending

.segment "VECTORS"
    .word $0000, $0000, $0000  ; NMI, Reset, IRQ vectors (not used in this test) 