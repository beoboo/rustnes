; Sample NES ROM with segments
.segment "HEADER"
; This would be the iNES header bytes in a real ROM

.segment "STARTUP"
    ; Main program code
    LDA #$01
    STA $0200
    
.segment "VECTORS"
    ; NMI, Reset, and IRQ vectors
    .word $0000  ; NMI vector
    .word $8000  ; Reset vector (points to STARTUP)
    .word $0000  ; IRQ vector
    
.segment "CHARS"
    ; Character data would go here 