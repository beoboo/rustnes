; Simple Animation Test ROM for RustNES
; This demonstrates a basic bouncing ball using a 2x2 tile sprite
; 
; *** ANIMATION FEATURES ***
; 1. A 16x16 sprite (2x2 tiles) bounces around the screen
; 2. Simple bouncing motion with edge detection
;
; *** NEW CPU INSTRUCTIONS USED ***
; - CLC, SEC: Used for addition and subtraction operations
; - BEQ, BNE: Branch on equal/not equal (for direction changes)
; - ADC, SBC: Addition and subtraction with carry (for position updates)
; - CMP: Compare (for boundary checking)
;
; *** INSTRUCTIONS FOR THE NES DEBUGGER ***
; 1. Set the "No cycle limit" checkbox or set cycles to at least 1,000,000
; 2. Click "Run" (not "Step") to execute the program
; 3. The animation should start immediately - a ball will bounce around the screen
; 4. For best results, use the "Run to Next Frame" feature repeatedly

.segment "HEADER"
  .byte "NES", $1A      ; iNES header identifier
  .byte 2               ; 2x 16KB PRG-ROM pages
  .byte 1               ; 1x  8KB CHR-ROM pages
  .byte $01, $00        ; mapper 0, vertical mirroring, no battery, no trainer
  .byte $00, $00, $00, $00, $00, $00, $00, $00  ; padding

; Reserve zero page locations for our variables
.segment "ZEROPAGE"
  ball_x:      .res 1   ; Ball X position (top-left corner)
  ball_y:      .res 1   ; Ball Y position (top-left corner)
  x_vel:       .res 1   ; X velocity (0=left, 1=right)
  y_vel:       .res 1   ; Y velocity (0=up, 1=down)

.segment "STARTUP"
RESET:
  ; Set up the stack
  LDX #$FF
  TXS
  
  ; Initialize variables
  LDA #$80        ; Start in middle of screen X (128)
  STA ball_x
  LDA #$80        ; Start in middle of screen Y (128)
  STA ball_y
  LDA #$01        ; Start moving right
  STA x_vel
  LDA #$01        ; Start moving down
  STA y_vel

  ; Set up PPU
  LDA #$00
  STA $2000       ; Disable NMI
  STA $2001       ; Disable rendering

  ; Wait for vblank (simplified)
  JSR WaitForVBlank

  ; Set up sprite palettes
  LDA #$3F        ; Palette memory high byte
  STA $2006
  LDA #$10        ; Sprite palette 0 address
  STA $2006
  
  ; Set up palette 0 with blue
  LDA #$0F        ; Background color (transparent for sprites)
  STA $2007
  LDA #$12        ; Blue
  STA $2007
  LDA #$12        ; Blue again
  STA $2007
  LDA #$12        ; Blue again
  STA $2007

  ; Set up palette 1 with red
  LDA #$0F        ; Background color
  STA $2007
  LDA #$16        ; Red
  STA $2007
  LDA #$16        ; Red again
  STA $2007
  LDA #$16        ; Red again
  STA $2007
  
  ; Set up palette 2 with green
  LDA #$0F        ; Background color
  STA $2007
  LDA #$2A        ; Green
  STA $2007
  LDA #$2A        ; Green again
  STA $2007
  LDA #$2A        ; Green again
  STA $2007
  
  ; Set up palette 3 with yellow
  LDA #$0F        ; Background color
  STA $2007
  LDA #$28        ; Yellow
  STA $2007
  LDA #$28        ; Yellow again
  STA $2007
  LDA #$28        ; Yellow again
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

  ; Update ball position
  JSR UpdateBallPosition
  
  ; Update sprite data based on ball position
  JSR UpdateSpritePosition
  
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

; Update the ball's position based on velocity
UpdateBallPosition:
  ; Check X velocity
  LDA x_vel
  BEQ MoveLeft
  ; Moving right
  LDA ball_x
  CMP #$E8             ; Right edge (240 - 16 = 224)
  BNE ContinueRight    ; Not at right edge yet
  
  ; At right edge, switch to moving left
  LDA #$00
  STA x_vel
  JMP CheckYVelocity
  
ContinueRight:
  ; Continue moving right
  LDA ball_x
  CLC                  ; Clear carry before add
  ADC #$01             ; Add 1 to X position
  STA ball_x
  JMP CheckYVelocity
  
MoveLeft:
  ; Moving left
  LDA ball_x
  CMP #$08             ; Left edge (8)
  BNE ContinueLeft     ; Not at left edge yet
  
  ; At left edge, switch to moving right
  LDA #$01
  STA x_vel
  JMP CheckYVelocity
  
ContinueLeft:
  ; Continue moving left
  LDA ball_x
  SEC                  ; Set carry before subtract
  SBC #$01             ; Subtract 1 from X position
  STA ball_x
  
CheckYVelocity:
  ; Check Y velocity
  LDA y_vel
  BEQ MoveUp
  
  ; Moving down
  LDA ball_y
  CMP #$D8             ; Bottom edge (224 - 16 = 208)
  BNE ContinueDown     ; Not at bottom edge yet
  
  ; At bottom edge, switch to moving up
  LDA #$00
  STA y_vel
  RTS
  
ContinueDown:
  ; Continue moving down
  LDA ball_y
  CLC                  ; Clear carry before add
  ADC #$01             ; Add 1 to Y position
  STA ball_y
  RTS
  
MoveUp:
  ; Moving up
  LDA ball_y
  CMP #$08             ; Top edge (8)
  BNE ContinueUp       ; Not at top edge yet
  
  ; At top edge, switch to moving down
  LDA #$01
  STA y_vel
  RTS
  
ContinueUp:
  ; Continue moving up
  LDA ball_y
  SEC                  ; Set carry before subtract
  SBC #$01             ; Subtract 1 from Y position
  STA ball_y
  RTS

; Update sprite position based on ball_x and ball_y
; This creates a 2x2 tile sprite (16x16 pixels total)
UpdateSpritePosition:
  ; Sprite 0: Top-left tile of ball
  LDA ball_y          ; Y position 
  STA $0200
  LDA #$00            ; Tile 0 (top-left of ball)
  STA $0201
  LDA #$00            ; Attributes (palette 0 - blue)
  STA $0202
  LDA ball_x          ; X position
  STA $0203
  
  ; Sprite 1: Top-right tile of ball
  LDA ball_y          ; Y position
  STA $0204
  LDA #$01            ; Tile 1 (top-right of ball)
  STA $0205
  LDA #$01            ; Attributes (palette 1 - red)
  STA $0206
  LDA ball_x          ; X position
  CLC
  ADC #$08            ; Add 8 pixels to X for the right half
  STA $0207
  
  ; Sprite 2: Bottom-left tile of ball
  LDA ball_y          ; Y position
  CLC
  ADC #$08            ; Add 8 pixels to Y for the bottom half
  STA $0208
  LDA #$10            ; Tile 16 (bottom-left of ball)
  STA $0209
  LDA #$02            ; Attributes (palette 2 - green)
  STA $020A
  LDA ball_x          ; X position
  STA $020B
  
  ; Sprite 3: Bottom-right tile of ball
  LDA ball_y          ; Y position
  CLC
  ADC #$08            ; Add 8 pixels to Y for the bottom half
  STA $020C
  LDA #$11            ; Tile 17 (bottom-right of ball)
  STA $020D
  LDA #$03            ; Attributes (palette 3 - yellow)
  STA $020E
  LDA ball_x          ; X position
  CLC
  ADC #$08            ; Add 8 pixels to X for the right half
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
  ; Top-left part of the ball (Tile 0) - Bit plane 0 - Shape mask
  .byte %00000111  ; Row 1: ·····███  
  .byte %00011111  ; Row 2: ···█████  
  .byte %00111111  ; Row 3: ··██████  
  .byte %01111111  ; Row 4: ·███████  
  .byte %01111111  ; Row 5: ·███████  
  .byte %11111111  ; Row 6: ████████  
  .byte %11111111  ; Row 7: ████████  
  .byte %11111111  ; Row 8: ████████  
  
  ; Top-left part of the ball - Bit plane 1 - Blue quadrant
  .byte %00000111  ; Row 1: ·····███
  .byte %00011111  ; Row 2: ···█████
  .byte %00111111  ; Row 3: ··██████
  .byte %01111111  ; Row 4: ·███████
  .byte %01111111  ; Row 5: ·███████
  .byte %11111111  ; Row 6: ████████
  .byte %11111111  ; Row 7: ████████
  .byte %11111111  ; Row 8: ████████
  
  ; Top-right part of the ball (Tile 1) - Bit plane 0 - Shape mask
  .byte %11100000  ; Row 1: ███·····  
  .byte %11111000  ; Row 2: █████···  
  .byte %11111100  ; Row 3: ██████··  
  .byte %11111110  ; Row 4: ███████·  
  .byte %11111110  ; Row 5: ███████·  
  .byte %11111111  ; Row 6: ████████  
  .byte %11111111  ; Row 7: ████████  
  .byte %11111111  ; Row 8: ████████  
  
  ; Top-right part of the ball - Bit plane 1 - Red quadrant
  .byte %11100000  ; Row 1: ███·····  
  .byte %11111000  ; Row 2: █████···  
  .byte %11111100  ; Row 3: ██████··  
  .byte %11111110  ; Row 4: ███████·  
  .byte %11111110  ; Row 5: ███████·  
  .byte %11111111  ; Row 6: ████████  
  .byte %11111111  ; Row 7: ████████  
  .byte %11111111  ; Row 8: ████████  

  ; Fill the space between tiles 1 and 16 with empty tiles (14 tiles = 224 bytes)
  .res 224, $00
  
  ; Bottom-left part of the ball (Tile 16) - Bit plane 0 - Shape mask
  .byte %11111111  ; Row 1: ████████  
  .byte %11111111  ; Row 2: ████████  
  .byte %11111111  ; Row 3: ████████  
  .byte %01111111  ; Row 4: ·███████  
  .byte %01111111  ; Row 5: ·███████  
  .byte %00111111  ; Row 6: ··██████  
  .byte %00011111  ; Row 7: ···█████  
  .byte %00000111  ; Row 8: ·····███  
  
  ; Bottom-left part of the ball - Bit plane 1 - Green quadrant
  .byte %11111111  ; Row 1: ████████  
  .byte %11111111  ; Row 2: ████████  
  .byte %11111111  ; Row 3: ████████  
  .byte %01111111  ; Row 4: ·███████  
  .byte %01111111  ; Row 5: ·███████  
  .byte %00111111  ; Row 6: ··██████  
  .byte %00011111  ; Row 7: ···█████  
  .byte %00000111  ; Row 8: ·····███  
  
  ; Bottom-right part of the ball (Tile 17) - Bit plane 0 - Shape mask
  .byte %11111111  ; Row 1: ████████  
  .byte %11111111  ; Row 2: ████████  
  .byte %11111111  ; Row 3: ████████  
  .byte %11111110  ; Row 4: ███████·  
  .byte %11111110  ; Row 5: ███████·  
  .byte %11111100  ; Row 6: ██████··  
  .byte %11111000  ; Row 7: █████···  
  .byte %11100000  ; Row 8: ███·····  
  
  ; Bottom-right part of the ball - Bit plane 1 - Yellow quadrant (using palette 1)
  .byte %11111111  ; Row 1: ████████  
  .byte %11111111  ; Row 2: ████████  
  .byte %11111111  ; Row 3: ████████  
  .byte %11111110  ; Row 4: ███████·  
  .byte %11111110  ; Row 5: ███████·  
  .byte %11111100  ; Row 6: ██████··  
  .byte %11111000  ; Row 7: █████···  
  .byte %11100000  ; Row 8: ███·····  
  
  ; Fill the rest of the pattern table with empty tiles
  .res $1E00, $00 