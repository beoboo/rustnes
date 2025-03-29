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

    ; SIMPLE AUDIO TEST
    ; Enable pulse channel 1 AND triangle channel
    LDA #$05    ; Enable pulse 1 (bit 0) and triangle (bit 2)
    STA $4015
    
    ; ---- Setup Pulse Channel 1 ----
    ; Set pulse 1 parameters - continuous high-volume tone
    ; %10xx xxxx = 50% duty cycle 
    ; %xx1x xxxx = Length counter halt (sustain note)
    ; %xxx1 xxxx = Constant volume
    ; %xxxx 1111 = Maximum volume (15)
    LDA #$BF    ; 10111111
    STA $4000

    ; Disable sweep
    LDA #$00
    STA $4001

    ; Set a medium frequency
    LDA #$A0    ; Timer low (try different values for different pitches)
    STA $4002
    LDA #$00    ; Timer high = 0, length counter = 0 (but halted)
    STA $4003
    
    ; Make sure the sound keeps playing by writing to $4003 again
    LDA #$00
    STA $4003
    
    ; ---- Setup Triangle Channel ----
    ; Linear counter halt flag must be set (bit 7)
    ; Linear counter load value in lower 7 bits
    LDA #$FF    ; 11111111 - Halt flag set, max counter value
    STA $4008
    
    ; Set a medium frequency for continuous tone
    LDA #$72    ; Timer low (different from pulse for harmony)
    STA $400A
    
    ; Timer high and length counter load
    ; %xxx0 0000 = Timer high bits are 0
    ; %000x xxxx = Length counter load index = 31 (longest value)
    LDA #%00011111    
    STA $400B
    
    ; Make sure to immediately write to $400B again to ensure length counter is loaded
    LDA #%00011111
    STA $400B

loop:
    ; Make sure the sound keeps playing by refreshing registers
    ; Approximately every 1/2 second
    LDX #$FF    ; Outer loop counter
outer_loop:
    LDY #$FF    ; Inner loop counter
inner_loop:
    DEY
    BNE inner_loop
    
    DEX
    BNE outer_loop
    
    ; Re-enable the pulse and triangle channels
    LDA #$05    ; Pulse 1 and triangle
    STA $4015
    
    ; Refresh pulse parameters
    LDA #$BF    ; 10111111 - 50% duty, volume=15, halt=1, constant=1
    STA $4000
    
    ; Refresh triangle parameters  
    LDA #$FF    ; Linear counter max, halt flag set
    STA $4008
    
    ; Refresh the triangle length counter
    LDA #%00011111
    STA $400B
    
    JMP loop

.segment "VECTORS"
    .word $0000, $0000, $0000  ; NMI, Reset, IRQ vectors (not used in this test) 