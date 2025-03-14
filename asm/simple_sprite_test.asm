; Simple Sprite Test ROM for RustNES
; Minimal code to display a single sprite

.segment "HEADER"
  .byte "NES", $1A      ; NES header identifier
  .byte $01             ; 1 x 16KB PRG ROM
  .byte $01             ; 1 x 8KB CHR ROM
  .byte $00             ; Mapper 0 (NROM)
  .byte $00, $00, $00, $00, $00, $00, $00, $00, $00 ; Padding

.segment "STARTUP"
Reset:
  ; Minimal initialization - wait for PPU to warm up
  LDX #$00
  STX $2000            ; Disable NMI
  STX $2001            ; Disable rendering
  
  ; Wait for first vblank
  Wait1:
    BIT $2002
    BPL Wait1
  
  ; Wait for second vblank to ensure PPU is ready
  Wait2:
    BIT $2002
    BPL Wait2

  ; ESSENTIAL #1: Set up sprite in OAM (located at $0200)
  ; Sprite 1: A box in the center of the screen
  LDA #$80              ; Y position = 128 (middle of screen)
  STA $0200
  LDA #$00              ; Tile number = 0 (first tile)
  STA $0201
  LDA #$00              ; Attributes: no flip, palette 0 (green colors)
  STA $0202
  LDA #$80              ; X position = 128 (middle of screen)
  STA $0203
  
  ; Sprite 2: A box at top-left of screen
  LDA #$20              ; Y position = 32 (top area)
  STA $0204
  LDA #$00              ; Tile number = 0 (first tile)
  STA $0205  
  LDA #$01              ; Attributes: no flip, palette 1 (red colors)
  STA $0206
  LDA #$20              ; X position = 32 (left area)
  STA $0207
  
  ; Sprite 3: A box at bottom-right of screen
  LDA #$E0              ; Y position = 224 (bottom area)
  STA $0208
  LDA #$00              ; Tile number = 0 (first tile)
  STA $0209
  LDA #$02              ; Attributes: no flip, palette 2 (blue colors)
  STA $020A
  LDA #$E0              ; X position = 224 (right area)
  STA $020B

  ; ESSENTIAL #2: Set up the palettes
  LDA #$3F
  STA $2006             ; PPUADDR high byte = $3F
  LDA #$10
  STA $2006             ; PPUADDR low byte = $10 (sprite palette 0)
  
  ; Write 4 colors for sprite palette 0 (green)
  LDA #$0F              ; Black (transparent)
  STA $2007
  LDA #$2A              ; Light green
  STA $2007
  LDA #$1A              ; Dark green
  STA $2007
  LDA #$30              ; White
  STA $2007
  
  ; Write 4 colors for sprite palette 1 (red)
  LDA #$0F              ; Black (transparent)
  STA $2007
  LDA #$16              ; Light red
  STA $2007
  LDA #$06              ; Dark red
  STA $2007
  LDA #$30              ; White
  STA $2007
  
  ; Write 4 colors for sprite palette 2 (blue)
  LDA #$0F              ; Black (transparent)
  STA $2007
  LDA #$22              ; Light blue
  STA $2007
  LDA #$12              ; Dark blue
  STA $2007
  LDA #$30              ; White
  STA $2007
  
  ; Reset PPU address latch
  LDA $2002
  
  ; ESSENTIAL NEW STEP: Set up OAM DMA transfer
  LDA #$00
  STA $2003             ; Set OAM address to 0
  LDA #$02
  STA $4014             ; Start OAM DMA from $0200
  
  ; ESSENTIAL NEW STEP: Configure PPUCTRL
  LDA #%10000000        ; Enable NMI + Use pattern table 0 for sprites
  STA $2000

  ; ESSENTIAL #3: Enable sprites
  LDA #%00010000        ; Enable sprites only
  STA $2001

  ; Simple infinite loop
  JMP *                 ; Jump to current address (infinite loop)

; Vectors
.segment "VECTORS"
  .word $0000           ; NMI vector (unused)
  .word Reset           ; Reset vector
  .word $0000           ; IRQ/BRK vector (unused)

; ESSENTIAL #4: CHR ROM data - Simple 8x8 box sprite
.segment "CHARS"
  ; First tile (8x8 sprite) - Bit plane 0
  .byte %00000000
  .byte %00111100
  .byte %01111110
  .byte %01111110
  .byte %01111110
  .byte %01111110
  .byte %00111100
  .byte %00000000
  
  ; First tile - Bit plane 1
  .byte %00000000
  .byte %00000000
  .byte %00111100
  .byte %00111100
  .byte %00111100
  .byte %00111100
  .byte %00000000
  .byte %00000000
  
  ; Fill the rest with zeros
  .res $1FE0, $00 