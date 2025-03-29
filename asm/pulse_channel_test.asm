.segment "HEADER"
    .byte $4E, $45, $53, $1A   ; NES header
    .byte $02                   ; 2x 16KB PRG ROM
    .byte $01                   ; 1x 8KB CHR ROM
    .byte $01                   ; Vertical mirroring, no battery RAM
    .byte $00                   ; No mapper, plain NROM
    .byte $00, $00, $00, $00   ; Padding bytes
    .byte $00, $00, $00, $00   ; Padding bytes

.segment "STARTUP"
    ; Initialize the stack pointer
    LDX #$FF
    TXS

    ; Enable both pulse channels
    LDA #$03    ; Enable pulse 1 and pulse 2
    STA $4015

    ; Set up pulse channel 1 with 50% duty cycle and maximum volume
    ; Bits 6-7 = 10 (50% duty cycle)
    ; Bit 5 = 1 (halt length counter)
    ; Bit 4 = 1 (constant volume)
    ; Bits 0-3 = F (max volume)
    LDA #$BF    ; 10111111 - 50% duty, constant volume=15, halt length counter
    STA $4000

    ; Disable sweep unit entirely (bit 7 = 0)
    LDA #$00
    STA $4001

    ; Set very low timer value (high frequency) for clear audible tone
    LDA #$50    ; Low byte of timer period
    STA $4002
    
    ; High byte of timer + length counter
    ; Set to a long length counter value
    LDA #$F8    ; 11111000 - Set timer high to 7, length = 0
    STA $4003

    ; Set up pulse channel 2 with 25% duty cycle and 3/4 volume
    ; Bits 6-7 = 01 (25% duty cycle)
    ; Bit 5 = 1 (halt length counter)
    ; Bit 4 = 1 (constant volume)
    ; Bits 0-3 = B (medium-high volume)
    LDA #$7B    ; 01111011 - 25% duty, constant volume=11, halt length counter
    STA $4004

    ; Since we're not using loop below to update registers,
    ; set them again to be sure (sometimes first write is ignored)
    LDA #$BF    ; 10111111 - 50% duty, constant volume=15, halt length counter
    STA $4000
    
    ; Refresh the $4003 register to be sure length counter is started
    LDA #$F8    ; 11111000 - Set timer high to 7, length = 0
    STA $4003

main_loop:
    ; Just loop forever
    JMP main_loop

.segment "VECTORS"
    .word $0000, $0000, $0000  ; NMI, Reset, IRQ vectors (not used in this test) 