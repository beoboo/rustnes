; PPU Pixel Test Program
; This program draws a single red pixel at coordinates (128, 120) using the PPU

    ; Initialize the PPU
    LDA #$00    ; Set PPUCTRL to 0 (NMI disabled)
    STA $2000
    LDA #$00    ; Set PPUMASK to 0 (rendering disabled)
    STA $2001

    ; Set palette entry 0 to red color (color $21)
    LDA #$3F    ; Set high byte of PPU address to $3F (palette memory)
    STA $2006
    LDA #$00    ; Set low byte of PPU address to $00 (first palette entry)
    STA $2006
    LDA #$21    ; Load red color
    STA $2007   ; Store to PPU data port

    ; Set a pattern table entry to have a single pixel on
    ; First, set PPU address to pattern table entry 1
    LDA #$00    ; High byte of pattern table address
    STA $2006
    LDA #$10    ; Pattern #1, low byte
    STA $2006
    
    ; Write pattern with a single pixel in the middle
    LDA #$00    ; First 7 rows are blank
    STA $2007
    STA $2007
    STA $2007
    STA $2007
    STA $2007
    STA $2007
    STA $2007
    LDA #$08    ; Middle row has a single pixel on (bit 4)
    STA $2007
    LDA #$00    ; Last 8 rows are blank
    STA $2007
    STA $2007
    STA $2007
    STA $2007
    STA $2007
    STA $2007
    STA $2007
    STA $2007

    ; Place the tile in the middle of the screen
    ; Set PPU address to nametable position in the middle
    LDA #$20    ; High byte of nametable base address
    STA $2006
    LDA #$ED    ; Middle of screen (row 15, column 13)
    STA $2006
    LDA #$01    ; Tile #1 (the one we defined)
    STA $2007

    ; Enable rendering
    LDA #$00    ; Make sure PPUCTRL is still 0 (NMI disabled)
    STA $2000
    LDA #$1E    ; Enable background, sprites, and show leftmost 8 pixels
    STA $2001

    ; Loop forever (no need for interrupts in this simple test)
; loop:
;     JMP loop 