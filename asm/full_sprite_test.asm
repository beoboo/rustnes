; Single Sprite Test ROM for RustNES
; Displays one 8x8 sprite at the center of the screen

.segment "HEADER"
  .byte "NES", $1A      ; NES header identifier
  .byte $01             ; 1 x 16KB PRG ROM
  .byte $01             ; 1 x 8KB CHR ROM
  .byte $00             ; Mapper 0 (NROM), vertical mirroring
  .byte $00, $00, $00, $00, $00, $00, $00, $00, $00 ; Padding

.segment "ZEROPAGE"
  ; Variables
  ; No variables needed for this simple example

.segment "STARTUP"
Reset:
  SEI                   ; Disable interrupts
  CLD                   ; Clear decimal mode (not used on NES)
  LDX #$40
  STX $4017            ; Disable APU frame IRQ
  LDX #$FF
  TXS                   ; Set up stack
  INX                   ; X = 0
  STX $2000            ; Disable NMI
  STX $2001            ; Disable rendering
  STX $4010            ; Disable DMC IRQs

  ; Wait for first vblank
  JSR WaitVBlank

  ; Clear RAM
  LDA #$00
  LDX #$00
ClearRAM:
  STA $0000, X         ; Clear zero page
  STA $0100, X         ; Clear stack
  STA $0200, X         ; Clear OAM (sprite memory)
  STA $0300, X         ; Clear extra RAM
  STA $0400, X
  STA $0500, X
  STA $0600, X
  STA $0700, X
  INX
  BNE ClearRAM

  ; Wait for second vblank
  JSR WaitVBlank

  ; Set up sprite in OAM (located at $0200)
  LDA #$80              ; Y position = 128 (middle of screen)
  STA $0200
  LDA #$00              ; Tile number = 0 (first tile)
  STA $0201
  LDA #$00              ; Attributes: no flip, palette 0
  STA $0202
  LDA #$80              ; X position = 128 (middle of screen)
  STA $0203

  ; Set up the palette
  LDA #$3F
  STA $2006             ; PPUADDR high byte = $3F
  LDA #$10
  STA $2006             ; PPUADDR low byte = $10 (sprite palette 0)
  
  ; Write 4 colors for sprite palette 0
  LDA #$0F              ; Black
  STA $2007
  LDA #$15              ; Light green
  STA $2007
  LDA #$27              ; Red
  STA $2007
  LDA #$30              ; White
  STA $2007

  ; Enable sprites
  LDA #%10000000        ; Enable NMI
  STA $2000
  LDA #%00010000        ; Enable sprites
  STA $2001

MainLoop:
  ; Infinite loop
  JMP MainLoop

WaitVBlank:
  BIT $2002             ; Check VBlank flag in PPU status
  BPL WaitVBlank        ; Branch if not in VBlank (bit 7 = 0)
  RTS

NMI:
  ; Transfer OAM data to PPU during VBlank
  LDA #$00
  STA $2003             ; Set OAM address to 0
  LDA #$02
  STA $4014             ; OAM DMA transfer from $0200

  RTI                   ; Return from interrupt

; Fill unused space
.segment "CODE"
  .res $C000-*, $FF

; Interrupt vectors
.segment "VECTORS"
  .word NMI             ; NMI vector
  .word Reset           ; Reset vector
  .word Reset           ; IRQ/BRK vector (unused)

; CHR ROM data (8KB) - Simple 8x8 box sprite
.segment "CHARS"
  ; First tile (8x8 sprite) - Bit plane 0
  .byte %00000000
  .byte %00111100
  .byte %00111100
  .byte %00111100
  .byte %00111100
  .byte %00111100
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
  
  ; Filling rest of CHR-ROM with zeros
  .res $1FE0, $00 