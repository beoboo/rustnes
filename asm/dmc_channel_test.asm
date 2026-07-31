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

    ; Enable DMC channel
    LDA #$10
    STA $4015

    ; Set up DMC channel
    ; $4010: bit 6 = loop, bits 0-3 = rate index. $4F = loop + fastest rate.
    LDA #$4F
    STA $4010

    ; $4012 - Sample address (the byte is the address, as $C000 + value * 64)
    LDA #$00
    STA $4012

    ; $4013 - Sample length (in 16-byte units, so $01 = 17 bytes)
    LDA #$01
    STA $4013

    ; $4011 - Direct load: sets the 7-bit DAC level straight away.
    ; Must be non-zero to be audible; $40 is mid-scale.
    ; NOTE: the DMC's memory reader is not implemented yet, so this demo exercises the DAC and
    ; the delta decoder, not real DPCM sample playback.
    LDA #$40
    STA $4011

    ; Main loop
main_loop:
    ; Wait a bit
    LDX #$FF
wait_loop:
    DEX
    BNE wait_loop

    ; Restart DMC sample
    LDA #$10
    STA $4015

    JMP main_loop

.segment "VECTORS"
    .word $0000, $0000, $0000  ; NMI, Reset, IRQ vectors (not used in this test)

.segment "CHARS"
    ; Sample data for DMC (256 bytes of simple waveform)
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20
    .byte $00, $20, $40, $60, $80, $A0, $C0, $E0, $FF, $E0, $C0, $A0, $80, $60, $40, $20 