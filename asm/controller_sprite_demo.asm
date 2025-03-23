; Controller Sprite Demo ROM for RustNES
; This demonstrates moving a sprite using controller input
; 
; *** FEATURES ***
; 1. A 16x16 sprite (2x2 tiles) controlled by the D-pad
; 2. Real-time response to controller input
; 3. Visual feedback when buttons are pressed
;
; *** INSTRUCTIONS FOR THE NES DEBUGGER ***
; 1. Set the "No cycle limit" checkbox or set cycles to at least 1,000,000
; 2. Click "Run" (not "Step") to execute the program
; 3. Use the Controller 1 UI to move the sprite with the D-pad
; 4. A/B buttons change the sprite's color

.segment "HEADER"
  .byte "NES", $1A      ; iNES header identifier
  .byte 2               ; 2x 16KB PRG-ROM pages
  .byte 1               ; 1x  8KB CHR-ROM pages
  .byte $01, $00        ; mapper 0, vertical mirroring, no battery, no trainer
  .byte $00, $00, $00, $00, $00, $00, $00, $00  ; padding

; Reserve zero page locations for our variables
.segment "ZEROPAGE"
  sprite_x:       .res 1   ; Sprite X position (top-left corner)
  sprite_y:       .res 1   ; Sprite Y position (top-left corner)
  controller1:    .res 1   ; Current controller 1 state
  prev_state:     .res 1   ; Previous controller 1 state
  color_palette:  .res 1   ; Current color palette (0 or 1)
  temp_attr:      .res 1   ; Temporary variable for attribute storage

.segment "STARTUP"
RESET:
  ; Set up the stack
  LDX #$FF
  TXS
  
  ; Initialize variables
  LDA #$80        ; Start in middle of screen X (128)
  STA sprite_x
  LDA #$80        ; Start in middle of screen Y (128)
  STA sprite_y
  LDA #$00        ; Clear controller state
  STA controller1
  STA prev_state
  LDA #$00        ; Initial color palette
  STA color_palette

  ; Set up PPU
  LDA #$00
  STA $2000       ; Disable NMI
  STA $2001       ; Disable rendering

  ; Wait for vblank
  JSR WaitForVBlank

  ; Set up sprite palettes
  LDA #$3F        ; Palette memory high byte
  STA $2006
  LDA #$10        ; Sprite palette 0 address
  STA $2006
  
  ; Set up palette 0 with blue and red
  LDA #$0F        ; Background color (transparent for sprites)
  STA $2007
  LDA #$12        ; Blue (color 1)
  STA $2007
  LDA #$16        ; Red (color 2)
  STA $2007
  LDA #$30        ; White (color 3)
  STA $2007

  ; Set up palette 1 with green and yellow
  LDA #$0F        ; Background color
  STA $2007
  LDA #$2A        ; Green (color 1)
  STA $2007
  LDA #$28        ; Yellow (color 2)
  STA $2007
  LDA #$30        ; White (color 3)
  STA $2007

  ; Set up initial sprite data for a 2x2 tile sprite
  JSR UpdateSpritePosition
  
  ; Initial DMA transfer
  LDA #$00
  STA $2003       ; Set OAM address to 0
  LDA #$02
  STA $4014       ; OAM DMA from $0200
  
  ; Reset PPU address/scroll after palettes
  LDA #$00
  STA $2005       ; X scroll = 0
  STA $2005       ; Y scroll = 0
  STA $2006       ; Reset PPU address high byte
  STA $2006       ; Reset PPU address low byte
  
  ; Enable rendering
  LDA #$18        ; Enable sprites and background
  STA $2001

; Main game loop
MainLoop:
  JSR WaitForVBlank     ; Wait for VBLANK to start
  JSR ReadController    ; Read controller input
  JSR ProcessInput      ; Process controller input to update sprite position
  JSR UpdateSpritePosition  ; Update sprite data based on position
  
  ; OAM DMA transfer (every frame)
  LDA #$00
  STA $2003
  LDA #$02
  STA $4014
  
  ; Reset PPU address registers (important)
  LDA #$00
  STA $2006
  STA $2006
  
  JMP MainLoop          ; Repeat forever

; Read controller input
ReadController:
  ; First, set strobe bit to 1 to begin controller reading
  LDA #$01
  STA $4016
  
  ; Then set strobe bit to 0 to latch button states
  LDA #$00
  STA $4016
  
  ; Save previous state to detect changes
  LDA controller1
  STA prev_state
  
  ; Clear controller state
  LDA #$00
  STA controller1
  
  ; Read button A (bit 0)
  LDA $4016
  AND #$01          ; Isolate bit 0
  STA controller1   ; Store in bit 0
  
  ; Read button B (bit 1)
  LDA $4016
  AND #$01
  ASL A             ; Shift to bit 1
  ORA controller1
  STA controller1
  
  ; Read Select (bit 2)
  LDA $4016
  AND #$01
  ASL A
  ASL A             ; Shift to bit 2
  ORA controller1
  STA controller1
  
  ; Read Start (bit 3)
  LDA $4016
  AND #$01
  ASL A
  ASL A
  ASL A             ; Shift to bit 3
  ORA controller1
  STA controller1
  
  ; Read Up (bit 4)
  LDA $4016
  AND #$01
  ASL A
  ASL A
  ASL A
  ASL A             ; Shift to bit 4
  ORA controller1
  STA controller1
  
  ; Read Down (bit 5)
  LDA $4016
  AND #$01
  ASL A
  ASL A
  ASL A
  ASL A
  ASL A             ; Shift to bit 5
  ORA controller1
  STA controller1
  
  ; Read Left (bit 6)
  LDA $4016
  AND #$01
  ASL A
  ASL A
  ASL A
  ASL A
  ASL A
  ASL A             ; Shift to bit 6
  ORA controller1
  STA controller1
  
  ; Read Right (bit 7)
  LDA $4016
  AND #$01
  ASL A
  ASL A
  ASL A
  ASL A
  ASL A
  ASL A
  ASL A             ; Shift to bit 7
  ORA controller1
  STA controller1
  
  RTS

; Process controller input to update sprite position
ProcessInput:
  ; Check D-pad buttons to move sprite
  
  ; Check Up button (bit 4)
  LDA controller1
  AND #$10          ; Mask for Up button (bit 4)
  BEQ @NotUp        ; Skip if not pressed
  
  ; Move sprite up
  LDA sprite_y
  CMP #$08          ; Check top boundary
  BEQ @NotUp        ; Don't move if at boundary
  SEC               ; Set carry before subtract
  SBC #$02          ; Move up by 2 pixels
  STA sprite_y
  
@NotUp:
  ; Check Down button (bit 5)
  LDA controller1
  AND #$20          ; Mask for Down button (bit 5)
  BEQ @NotDown      ; Skip if not pressed
  
  ; Move sprite down
  LDA sprite_y
  CMP #$D8          ; Check bottom boundary (240-16-8=216)
  BEQ @NotDown      ; Don't move if at boundary
  CLC               ; Clear carry before add
  ADC #$02          ; Move down by 2 pixels
  STA sprite_y
  
@NotDown:
  ; Check Left button (bit 6)
  LDA controller1
  AND #$40          ; Mask for Left button (bit 6)
  BEQ @NotLeft      ; Skip if not pressed
  
  ; Move sprite left
  LDA sprite_x
  CMP #$08          ; Check left boundary
  BEQ @NotLeft      ; Don't move if at boundary
  SEC               ; Set carry before subtract
  SBC #$02          ; Move left by 2 pixels
  STA sprite_x
  
@NotLeft:
  ; Check Right button (bit 7)
  LDA controller1
  AND #$80          ; Mask for Right button (bit 7)
  BEQ @NotRight     ; Skip if not pressed
  
  ; Move sprite right
  LDA sprite_x
  CMP #$E8          ; Check right boundary (256-16-8=232)
  BEQ @NotRight     ; Don't move if at boundary
  CLC               ; Clear carry before add
  ADC #$02          ; Move right by 2 pixels
  STA sprite_x
  
@NotRight:
  ; Check A button to change color palette (bit 0)
  LDA controller1
  AND #$01          ; Mask for A button (bit 0)
  BEQ @NotA         ; Skip if not pressed
  
  ; Check if button was just pressed
  LDA prev_state
  AND #$01          ; Mask for A button (bit 0)
  BNE @NotA         ; Skip if already pressed before
  
  ; Toggle color palette
  LDA color_palette
  CMP #$00          ; Compare with 0
  BEQ @SetToOne     ; If it's 0, set to 1
  
  ; Otherwise set to 0
  LDA #$00
  STA color_palette
  JMP @NotA
  
@SetToOne:
  LDA #$01
  STA color_palette
  
@NotA:
  RTS

; Update sprite position based on sprite_x and sprite_y
; This creates a 2x2 tile sprite (16x16 pixels total)
UpdateSpritePosition:
  ; Get current palette for attributes
  LDA color_palette     ; 0 or 1
  ASL A                 ; Shift to bits 0-1 for palette select in attributes
  STA temp_attr         ; Store in temp variable instead of X register
  
  ; Sprite 0: Top-left tile
  LDA sprite_y          ; Y position 
  STA $0200
  LDA #$00              ; Tile 0 (top-left)
  STA $0201
  LDA temp_attr         ; Get attributes (palette) from temp
  STA $0202
  LDA sprite_x          ; X position
  STA $0203
  
  ; Sprite 1: Top-right tile
  LDA sprite_y          ; Y position
  STA $0204
  LDA #$01              ; Tile 1 (top-right)
  STA $0205
  LDA temp_attr         ; Get attributes (palette) from temp
  STA $0206
  LDA sprite_x          ; X position
  CLC
  ADC #$08              ; Add 8 pixels to X for the right half
  STA $0207
  
  ; Sprite 2: Bottom-left tile
  LDA sprite_y          ; Y position
  CLC
  ADC #$08              ; Add 8 pixels to Y for the bottom half
  STA $0208
  LDA #$10              ; Tile 16 (bottom-left)
  STA $0209
  LDA temp_attr         ; Get attributes (palette) from temp
  STA $020A
  LDA sprite_x          ; X position
  STA $020B
  
  ; Sprite 3: Bottom-right tile
  LDA sprite_y          ; Y position
  CLC
  ADC #$08              ; Add 8 pixels to Y for the bottom half
  STA $020C
  LDA #$11              ; Tile 17 (bottom-right)
  STA $020D
  LDA temp_attr         ; Get attributes (palette) from temp
  STA $020E
  LDA sprite_x          ; X position
  CLC
  ADC #$08              ; Add 8 pixels to X for the right half
  STA $020F
  
  RTS

; Wait for VBLANK to start
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
  ; Top-left part of the sprite (Tile 0) - Bit plane 0
  .byte %00011111  ; Row 1: ···█████
  .byte %00111111  ; Row 2: ··██████
  .byte %01111111  ; Row 3: ·███████
  .byte %11111111  ; Row 4: ████████
  .byte %11111111  ; Row 5: ████████
  .byte %11111111  ; Row 6: ████████
  .byte %11111111  ; Row 7: ████████
  .byte %11111111  ; Row 8: ████████
  
  ; Bit plane 1
  .byte %00000000  ; Row 1: ········
  .byte %00000000  ; Row 2: ········
  .byte %00000000  ; Row 3: ········
  .byte %00000000  ; Row 4: ········
  .byte %00000000  ; Row 5: ········
  .byte %00000000  ; Row 6: ········
  .byte %00000000  ; Row 7: ········
  .byte %00000000  ; Row 8: ········
  
  ; Top-right part of the sprite (Tile 1) - Bit plane 0
  .byte %11111000  ; Row 1: █████···
  .byte %11111100  ; Row 2: ██████··
  .byte %11111110  ; Row 3: ███████·
  .byte %11111111  ; Row 4: ████████
  .byte %11111111  ; Row 5: ████████
  .byte %11111111  ; Row 6: ████████
  .byte %11111111  ; Row 7: ████████
  .byte %11111111  ; Row 8: ████████
  
  ; Bit plane 1
  .byte %00000000  ; Row 1: ········
  .byte %00000000  ; Row 2: ········
  .byte %00000000  ; Row 3: ········
  .byte %00000000  ; Row 4: ········
  .byte %00000000  ; Row 5: ········
  .byte %00000000  ; Row 6: ········
  .byte %00000000  ; Row 7: ········
  .byte %00000000  ; Row 8: ········

  ; Fill the space between tiles 1 and 16 with empty tiles (14 tiles = 224 bytes)
  .res 224, $00
  
  ; Bottom-left part of the sprite (Tile 16) - Bit plane 0
  .byte %11111111  ; Row 1: ████████
  .byte %11111111  ; Row 2: ████████
  .byte %11111111  ; Row 3: ████████
  .byte %11111111  ; Row 4: ████████
  .byte %11111111  ; Row 5: ████████
  .byte %01111111  ; Row 6: ·███████
  .byte %00111111  ; Row 7: ··██████
  .byte %00011111  ; Row 8: ···█████
  
  ; Bit plane 1
  .byte %00000000  ; Row 1: ········
  .byte %00000000  ; Row 2: ········
  .byte %00000000  ; Row 3: ········
  .byte %00000000  ; Row 4: ········
  .byte %00000000  ; Row 5: ········
  .byte %00000000  ; Row 6: ········
  .byte %00000000  ; Row 7: ········
  .byte %00000000  ; Row 8: ········
  
  ; Bottom-right part of the sprite (Tile 17) - Bit plane 0
  .byte %11111111  ; Row 1: ████████
  .byte %11111111  ; Row 2: ████████
  .byte %11111111  ; Row 3: ████████
  .byte %11111111  ; Row 4: ████████
  .byte %11111111  ; Row 5: ████████
  .byte %11111110  ; Row 6: ███████·
  .byte %11111100  ; Row 7: ██████··
  .byte %11111000  ; Row 8: █████···
  
  ; Bit plane 1
  .byte %00000000  ; Row 1: ········
  .byte %00000000  ; Row 2: ········
  .byte %00000000  ; Row 3: ········
  .byte %00000000  ; Row 4: ········
  .byte %00000000  ; Row 5: ········
  .byte %00000000  ; Row 6: ········
  .byte %00000000  ; Row 7: ········
  .byte %00000000  ; Row 8: ········
  
  ; Fill the rest of the pattern table with empty tiles
  .res $1E00, $00
