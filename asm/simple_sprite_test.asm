; Extremely simplified sprite test program
; Uses only the most basic instructions confirmed to be implemented

; *** IMPORTANT INSTRUCTIONS FOR THE NES DEBUGGER ***
;
; 1. Set the "No cycle limit" checkbox or set cycles to at least 1,000,000
; 2. Click "Run" (not "Step") to execute the program
; 3. Wait for sprites to appear - they should show up after enough cycles
; 4. If no sprites appear, try the "Write Test Sprite" option under System menu
;
; The PPU rendering happens every frame (89,342 PPU cycles), so you need to
; run enough cycles to trigger at least one frame render.

.segment "HEADER"
  .byte "NES", $1A      ; iNES header identifier
  .byte 2               ; 2x 16KB PRG-ROM pages
  .byte 1               ; 1x  8KB CHR-ROM pages
  .byte $01, $00        ; mapper 0, vertical mirroring, no battery, no trainer
  .byte $00, $00, $00, $00, $00, $00, $00, $00  ; padding

.segment "STARTUP"
RESET:
  ; Skip CPU setup - use only implemented instructions
  LDA #$00
  STA $2000             ; Disable NMI
  STA $2001             ; Disable rendering

  ; Wait for vblank (simplified)
  JSR WaitForVBlank

  ; VERY IMPORTANT: Set up sprites in the center of the screen
  ; with maximum size and visibility
  
  ; Create a 2x2 sprite block (4 sprites in a square configuration)
  ; This will be much more visible than small sprites
  
  ; Top-left sprite at (120, 100)
  LDA #100              ; Y position
  STA $0200
  LDA #0                ; Tile index 0 (solid square)
  STA $0201
  LDA #$00              ; Attributes - palette 0 (white)
  STA $0202
  LDA #120              ; X position
  STA $0203
  
  ; Top-right sprite at (128, 100)
  LDA #100              ; Y position
  STA $0204
  LDA #0                ; Tile index 0
  STA $0205
  LDA #$01              ; Attributes - palette 1 (red)
  STA $0206
  LDA #128              ; X position
  STA $0207
  
  ; Bottom-left sprite at (120, 108)
  LDA #108              ; Y position
  STA $0208
  LDA #0                ; Tile index 0
  STA $0209
  LDA #$02              ; Attributes - palette 2 (green)
  STA $020A
  LDA #120              ; X position
  STA $020B
  
  ; Bottom-right sprite at (128, 108)
  LDA #108              ; Y position
  STA $020C
  LDA #0                ; Tile index 0
  STA $020D
  LDA #$03              ; Attributes - palette 3 (blue)
  STA $020E
  LDA #128              ; X position
  STA $020F

  ; Set up sprite palette with extremely bright colors
  LDA #$3F              ; Palette memory high byte
  STA $2006
  LDA #$10              ; Sprite palette 0 address
  STA $2006
  
  ; Palette 0 - White
  LDA #$30              ; White
  STA $2007             ; Universal background color (unused)
  LDA #$30              ; White - brightest white possible
  STA $2007
  STA $2007
  STA $2007
  
  ; Palette 1 - Red
  LDA #$30              ; White
  STA $2007             ; Universal background color (unused)
  LDA #$16              ; Red - bright red
  STA $2007
  STA $2007
  STA $2007
  
  ; Palette 2 - Green
  LDA #$30              ; White
  STA $2007             ; Universal background color (unused)
  LDA #$1A              ; Green - bright green
  STA $2007
  STA $2007
  STA $2007
  
  ; Palette 3 - Blue
  LDA #$30              ; White
  STA $2007             ; Universal background color (unused)
  LDA #$12              ; Blue - bright blue
  STA $2007
  STA $2007
  STA $2007

  ; OAM DMA transfer (extremely important part)
  LDA #$00
  STA $2003             ; Set OAM address to 0
  LDA #$02
  STA $4014             ; Start DMA transfer from $0200 to OAM

  ; IMPORTANT: Reset scroll/address registers after palette update
  LDA #$00
  STA $2005             ; Reset scroll X
  STA $2005             ; Reset scroll Y
  STA $2006             ; Reset PPU address high byte
  STA $2006             ; Reset PPU address low byte

  ; Enable sprite rendering IMMEDIATELY (crucial)
  LDA #$10              ; Enable sprites only (no emphasis bits)
  STA $2001

  ; *** IMPORTANT: We need to run the main loop for MANY cycles! ***
  ; This is key to letting the PPU render a frame (it needs at least 89,342 PPU cycles)
MainLoop:
  JSR WaitForVBlank     ; Wait for VBLANK
  
  ; Re-do OAM DMA transfer EVERY frame
  LDA #$00
  STA $2003
  LDA #$02
  STA $4014
  
  ; Reset the PPU address register (important after any PPU access)
  LDA #$00
  STA $2006
  STA $2006
  
  ; Try different rendering options on different frames
  ; For simple testing, just keep toggling off/on to make sprites more visible
  LDA #$00              ; Disable rendering
  STA $2001
  
  LDA #$10              ; Re-enable sprites (ONLY sprites, no emphasis)
  STA $2001
  
  ; IMPORTANT: Loop must run many times to allow frame rendering
  JMP MainLoop
  
; Wait for VBLANK
WaitForVBlank:
  BIT $2002             ; Clear VBLANK flag
WaitVBlankStart:
  BIT $2002             ; Test VBLANK flag
  BPL WaitVBlankStart   ; Loop until VBLANK flag is set
  RTS

.segment "VECTORS"
  .word 0, 0, 0         ; Unused
  .word RESET           ; Reset vector
  .word 0               ; Unused

.segment "CHARS"
  ; Solid 8x8 square for maximum visibility
  ; Using solid white tiles (both bit planes set)
  ; First bit plane - all pixels on
  .byte %11111111
  .byte %11111111
  .byte %11111111
  .byte %11111111
  .byte %11111111
  .byte %11111111
  .byte %11111111
  .byte %11111111
  ; Second bit plane - all pixels on (important for maximum visibility!)
  .byte %11111111
  .byte %11111111
  .byte %11111111
  .byte %11111111
  .byte %11111111
  .byte %11111111
  .byte %11111111
  .byte %11111111 