; Sample NES ROM with segments and directives
.segment "HEADER"
    ; iNES header (16 bytes)
    .byte $4E, $45, $53, $1A  ; Magic header (NES followed by MS-DOS EOF)
    .byte $02                 ; 2 * 16KB PRG ROM
    .byte $01                 ; 1 * 8KB CHR ROM
    .byte $01                 ; Mapper 0, vertical mirroring
    .byte $00                 ; Mapper 0, playchoice, VS unisystem
    .res 8, $00               ; Padding

.segment "STARTUP"
    ; Main program code
    LDA #$01
    STA $0200
    BRK

.segment "VECTORS"
    ; NMI, Reset, and IRQ vectors (6 bytes)
    .word $0000  ; NMI vector
    .word $8000  ; Reset vector (points to STARTUP)
    .word $0000  ; IRQ vector
    
.segment "CHARS"
    ; Character data (example of a simple sprite)
    .byte $00, $3C, $7E, $7E, $7E, $7E, $3C, $00  ; Simple circle pattern
    .res 8, $00                                    ; Empty second row 