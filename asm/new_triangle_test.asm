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

    ; Enable both pulse channel 1 and triangle channel
    LDA #$05    ; Bit 0 = pulse 1, Bit 2 = triangle channel
    STA $4015

    ; Set up pulse channel with full volume
    ; Pulse 1 duty cycle and volume - Using 50% duty cycle
    LDA #$BF    ; 10111111 - 50% duty, constant volume, halt length counter, max volume (15)
    STA $4000
    
    ; Turn off sweep for pulse 1
    LDA #$00    ; 00000000 - No sweep, period=0, negate=0, shift=0
    STA $4001
    
    ; Set pulse 1 frequency - using a high value for clear tone
    LDA #$AF    ; Timer low byte - lower value = higher frequency
    STA $4002
    
    ; Set pulse timer high and load length counter
    ; High 3 bits = timer high bits
    ; Low 5 bits = length counter load index
    LDA #$08    ; 00001000 - Higher timer bits=1, length counter index=8
    STA $4003

    ; Set up triangle channel
    ; Bit 7 = 1 (control flag/halt counter)
    ; Bits 0-6 = Linear counter value (maximum = 127)
    LDA #$FF    ; 11111111 - Halt flag on, maximum linear counter value
    STA $4008

    ; Set triangle timer period (frequency)
    ; Lower value = higher frequency
    ; Using a higher frequency that should be clearly audible
    ; but different from the pulse channel for harmonization
    LDA #$42    ; Low byte of timer period (relatively high frequency)
    STA $400A
    
    ; Set timer high byte and length counter
    ; High 3 bits = timer high bits
    ; Low 5 bits = length counter load index
    LDA #%00011111    ; 00011111 - High timer bits = 0, length counter index = 31
    STA $400B

    ; Initial delay to ensure registers are loaded
    LDX #$FF
init_delay:
    DEX
    BNE init_delay
    
main_loop:
    ; Re-enable channels periodically
    LDA #$05    ; Pulse 1 and triangle
    STA $4015
    
    ; Keep the pulse channel active
    LDA #$BF    ; 10111111 - 50% duty, constant volume, halt length counter, max volume
    STA $4000
    
    ; Ensure pulse frequency parameters are set
    LDA #$AF    ; Timer low byte - lower value = higher frequency  
    STA $4002
    
    ; Refresh pulse length counter to ensure it keeps playing
    LDA #$08    ; 00001000 - Higher timer bits=1, length counter index=8
    STA $4003
    
    ; Keep the triangle linear counter refreshed
    LDA #$FF    ; Keep halt flag on, maximum linear counter
    STA $4008
    
    ; Refresh the triangle length counter periodically
    LDA #%00011111    ; High timer bits = 0, length counter index = 31
    STA $400B
    
    ; Short delay between refreshes
    LDX #$40    ; Shorter delay for more frequent refreshes
delay_loop:
    NOP        ; No operation, just waste time
    DEX
    BNE delay_loop
    
    JMP main_loop

.segment "VECTORS"
    .word $0000, $0000, $0000  ; NMI, Reset, IRQ vectors (not used in this test) 