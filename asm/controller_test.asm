; Controller Input Test ROM for RustNES
; This demonstrates how to read controller input and display visual feedback
; 
; *** CONTROLLER FEATURES ***
; 1. Reads inputs from the first controller (port 1)
; 2. Displays button states as colored blocks on screen
; 3. Demonstrates strobe and polling behavior
;
; *** INSTRUCTIONS FOR THE NES DEBUGGER ***
; 1. Set the "No cycle limit" checkbox or set cycles to at least 1,000,000
; 2. Click "Run" to execute the program
; 3. Modify Controller 1 state from the UI
; 4. Observe the color blocks changing on screen to reflect button states

.segment "HEADER"
  .byte "NES", $1A      ; iNES header identifier
  .byte 2               ; 2x 16KB PRG-ROM pages
  .byte 1               ; 1x  8KB CHR-ROM pages
  .byte $01, $00        ; mapper 0, vertical mirroring, no battery, no trainer
  .byte $00, $00, $00, $00, $00, $00, $00, $00  ; padding

; Reserve zero page locations for variables
.segment "ZEROPAGE"
  controller1: .res 1   ; Current controller 1 state
  prev_state:  .res 1   ; Previous controller 1 state

.segment "STARTUP"
RESET:
  ; Set up the stack
  LDX #$FF
  TXS
  
  ; Initialize variables
  LDA #$00
  STA controller1      ; Clear controller state
  STA prev_state       ; Clear previous state
  
  ; Set up PPU
  LDA #$00
  STA $2000       ; Disable NMI
  STA $2001       ; Disable rendering

  ; Wait for vblank
  JSR WaitForVBlank

  ; Set up palette for our button indicators
  LDA #$3F        ; Palette memory high byte
  STA $2006
  LDA #$00        ; Palette address
  STA $2006
  
  ; Background palette 0
  LDA #$0F        ; Black (background)
  STA $2007
  LDA #$30        ; White (button released - color 1)
  STA $2007
  LDA #$16        ; Red (button pressed - color 2)
  STA $2007
  LDA #$2A        ; Green (unused - color 3)
  STA $2007
  
  ; Reset PPU address/scroll
  LDA #$00
  STA $2005       ; X scroll = 0
  STA $2005       ; Y scroll = 0
  STA $2006       ; Reset PPU address high byte
  STA $2006       ; Reset PPU address low byte
  
  ; Enable rendering
  LDA #$1E        ; Enable sprites, background, and show leftmost 8 pixels
  STA $2001

; Main game loop
MainLoop:
  JSR WaitForVBlank     ; Wait for VBLANK to start
  JSR ReadController    ; Read controller input
  JSR UpdateDisplay     ; Update the display based on controller state
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
  
  ; Read 8 buttons from controller 1
  LDX #$08          ; 8 buttons to read
  LDY #$00          ; Button state accumulator
  
@ReadLoop:
  LDA $4016         ; Read next button state
  AND #$01          ; Isolate bit 0
  ASL A             ; Shift left because we read buttons in reverse order
  ASL A             ; from A, B, Select, Start, Up, Down, Left, Right
  ASL A             ; But we want to store them in the same order as
  ASL A             ; Button enum: A, B, Select, Start, Up, Down, Left, Right
  STA $00           ; Store in temporary zero page
  TYA               ; Get current button accumulator
  LSR A             ; Shift right to make room for new button
  ORA $00           ; Combine with new button
  TAY               ; Store back in Y
  DEX               ; Count down
  BNE @ReadLoop     ; If not zero, read next button
  
  ; Store the controller state
  STY controller1
  RTS

; Update the display based on controller state
UpdateDisplay:
  ; Prepare to update nametable
  LDA #$20        ; Nametable 0 address high byte
  STA $2006
  LDA #$00        ; Top-left corner
  STA $2006
  
  ; Display 8 button indicators (one per controller button)
  ; Each button state is represented by a different color attribute
  LDX #$00        ; Button counter
  
@ButtonLoop:
  ; Get current button
  LDA controller1
  AND ButtonMasks,X  ; Mask off current button
  BEQ @NotPressed    ; If 0, button is not pressed
  
  ; Button is pressed - use red tile
  LDA #$01        ; Tile for pressed button (solid color)
  STA $2007
  JMP @NextButton
  
@NotPressed:
  ; Button is not pressed - use white tile
  LDA #$02        ; Tile for released button (solid color)
  STA $2007
  
@NextButton:
  INX
  CPX #$08        ; Check if we've processed all 8 buttons
  BNE @ButtonLoop
  
  RTS

; Masks for each button (A, B, Select, Start, Up, Down, Left, Right)
ButtonMasks:
  .byte $01, $02, $04, $08, $10, $20, $40, $80

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
  ; CHR-ROM data
  ; Create simple pattern tiles:
  
  ; Tile 0: Empty black tile
  .byte $00, $00, $00, $00, $00, $00, $00, $00  ; Plane 1
  .byte $00, $00, $00, $00, $00, $00, $00, $00  ; Plane 2
  
  ; Tile 1: Red solid fill (for pressed buttons)
  .byte $FF, $FF, $FF, $FF, $FF, $FF, $FF, $FF  ; Plane 1  
  .byte $00, $00, $00, $00, $00, $00, $00, $00  ; Plane 2
  
  ; Tile 2: White solid fill (for released buttons)
  .byte $FF, $FF, $FF, $FF, $FF, $FF, $FF, $FF  ; Plane 1
  .byte $FF, $FF, $FF, $FF, $FF, $FF, $FF, $FF  ; Plane 2
    
  ; Fill the rest of the pattern table with empty tiles
  .res $1FB0, $00 