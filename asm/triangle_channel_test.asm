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

    ; SIMPLE TRIANGLE CHANNEL TEST
    ; Enable triangle channel only
    LDA #$04    ; Bit 2 = triangle
    STA $4015

    ; Set up triangle channel - continuous sound
    ; Linear counter halt flag must be set (bit 7)
    ; Linear counter load value in lower 7 bits
    LDA #$FF    ; 11111111 - Halt flag set, max counter value
    STA $4008
    
    ; No need to write to $4009 (unused)

    ; Set a medium frequency for continuous tone
    LDA #$A0    ; Timer low (medium frequency)
    STA $400A
    
    ; Timer high and length counter load
    ; The triangle channel is enabled whenever the length counter is non-zero
    ; %xxx0 0000 = Timer high bits are 0
    ; %000x xxxx = Length counter load index = 31 (longest value)
    LDA #%00011111    
    STA $400B
    
    ; Make sure to immediately write to $400B again to ensure length counter is loaded
    LDA #%00011111
    STA $400B

loop:
    ; Make sure the sound keeps playing by refreshing the counter values
    ; Approximately every 1/2 second
    LDX #$FF    ; Outer loop counter
outer_loop:
    LDY #$FF    ; Inner loop counter
inner_loop:
    DEY
    BNE inner_loop
    
    DEX
    BNE outer_loop
    
    ; Refresh the linear counter to keep triangle going
    LDA #$FF    ; Linear counter max, halt flag set
    STA $4008
    
    ; Refresh the length counter too
    LDA #%00011111
    STA $400B
    
    JMP loop

.segment "VECTORS"
    .word $0000, $0000, $0000  ; NMI, Reset, IRQ vectors (not used in this test) 