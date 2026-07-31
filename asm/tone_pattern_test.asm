; Tone Pattern Test ROM for RustNES
; This demonstrates how to generate tone patterns using the APU pulse channel 1
; 
; *** SOUND FEATURES ***
; 1. Plays ascending and descending tone patterns
; 2. Demonstrates volume fading effects
; 3. Shows how to control duty cycle
; 4. Uses timing to create musical effects
;
; *** INSTRUCTIONS FOR THE NES DEBUGGER ***
; 1. Set the "No cycle limit" checkbox or set cycles to at least 1,000,000
; 2. Click "Run" to execute the program
; 3. You should hear ascending and descending tone patterns

.segment "HEADER"
  .byte "NES", $1A      ; iNES header identifier
  .byte 1               ; 1x 16KB PRG-ROM pages
  .byte 1               ; 1x  8KB CHR-ROM pages
  .byte $01, $00        ; mapper 0, vertical mirroring, no battery, no trainer
  .byte $00, $00, $00, $00, $00, $00, $00, $00  ; padding

; Reserve zero page locations for variables
.segment "ZEROPAGE"
  note_index:    .res 1   ; Current note index in sequence
  note_timer:    .res 1   ; Timer for note duration
  pattern_type:  .res 1   ; Current pattern type (0=ascending, 1=descending)
  volume_level:  .res 1   ; Current volume level (0-15)
  duty_cycle:    .res 1   ; Current duty cycle (0-3)
  scratch:       .res 1   ; Scratch byte for UpdateDutyAndVolume

.segment "STARTUP"
RESET:
  ; Set up the stack
  LDX #$FF
  TXS
  
  ; Initialize variables
  LDA #$00
  STA note_index
  STA note_timer
  STA pattern_type
  
  LDA #$0F        ; Max volume
  STA volume_level
  
  LDA #$01        ; 25% duty cycle (01xxxxxx)
  STA duty_cycle
  
  ; Set up PPU
  LDA #$00
  STA $2000       ; Disable NMI
  STA $2001       ; Disable rendering

  ; Wait for vblank to ensure we're in a stable state
  JSR WaitForVBlank

  ; Enable pulse channel 1 FIRST.
  ; While a channel is disabled its length counter is forced to 0 and writes to $4003 are
  ; ignored, so programming the channel before enabling it leaves it permanently silent.
  LDA #$01        ; Enable pulse channel 1
  STA $4015       ; APU status/control register

  ; Now initialize the channel registers
  JSR InitializeAPU

; Main loop - play the tone patterns
MainLoop:
  JSR WaitForVBlank     ; Wait for VBLANK for timing
  JSR UpdateNote        ; Update the current note
  JMP MainLoop          ; Repeat forever

; Initialize the APU
InitializeAPU:
  ; Initialize pulse channel 1 ($4000-$4003)
  
  ; $4000 - Duty cycle, envelope, volume
  ; Set initial values with maximum volume
  JSR UpdateDutyAndVolume
  
  ; $4001 - Sweep unit (not used in this example)
  LDA #%00000000  ; Sweep off
  STA $4001
  
  ; $4002/$4003 - Initial frequency
  LDA NoteTable   ; First note frequency (low byte)
  STA $4002
  LDA #$00        ; High byte (0 for our range)
  STA $4003
  
  RTS

; Update the duty cycle and volume
UpdateDutyAndVolume:
  ; Combine duty cycle and volume
  ; Format: DDxx xxxx = duty cycle bits
  ;         xxxx VVVV = volume bits
  LDA duty_cycle   ; Load duty cycle (0-3)
  ASL A            ; Shift left 6 times to position bits 6-7
  ASL A
  ASL A
  ASL A
  ASL A
  ASL A
  STA scratch      ; Store temporarily
                   ; (NOT $00 — that is note_index, the first ZEROPAGE variable)
  
  LDA volume_level ; Load volume (0-15)
  AND #$0F         ; Ensure it's only 4 bits
  ORA scratch      ; Combine with duty cycle
  ORA #%00110000   ; Halt length counter (bit 5) + constant volume (bit 4).
                   ; Without the halt bit each note is cut off after ~83ms, leaving a
                   ; short blip rather than a sustained tone.
  STA $4000        ; Update register
  
  RTS

; Update the current note
UpdateNote:
  ; Increment the note timer
  INC note_timer
  LDA note_timer
  CMP #$10            ; Change note every ~16 frames (about 1/4 second)
  BNE @Done
  
  ; Reset timer
  LDA #$00
  STA note_timer
  
  ; Update which pattern we're playing
  LDX note_index
  CPX #$07            ; If we've reached the end of the sequence
  BNE @UpdateNote
  
  ; Toggle pattern direction and reset index
  LDA pattern_type
  EOR #$01            ; Toggle between 0 and 1
  STA pattern_type
  
  ; Reset note index
  LDA #$00
  STA note_index
  
  ; Cycle through duty cycles when pattern changes
  LDA duty_cycle
  CLC
  ADC #$01
  AND #$03            ; Keep in range 0-3
  STA duty_cycle
  JSR UpdateDutyAndVolume
  
  JMP @LoadNote
  
@UpdateNote:
  ; Increment note index
  INC note_index
  
  ; Update volume - fade out during descending pattern
  LDA pattern_type
  BEQ @VolumeUp       ; If ascending pattern
  
  ; Volume down for descending
  LDA volume_level
  SEC
  SBC #$02            ; Decrease by 2
  BPL @SetVolume      ; If still positive
  LDA #$00            ; Minimum is 0
  JMP @SetVolume
  
@VolumeUp:
  ; Volume up for ascending
  LDA volume_level
  CLC
  ADC #$02            ; Increase by 2
  CMP #$10            ; Check if > 15
  BCC @SetVolume      ; If <= 15
  LDA #$0F            ; Maximum is 15
  
@SetVolume:
  STA volume_level
  JSR UpdateDutyAndVolume
  
@LoadNote:
  ; Load the correct note based on pattern type and index
  LDX note_index
  LDA pattern_type
  BEQ @Ascending
  
  ; Descending pattern - reverse the index
  TXA
  EOR #$07            ; 7-index for descending
  TAX
  
@Ascending:
  ; Load note from table
  LDA NoteTable,X
  STA $4002           ; Low byte of period
  LDA #$00
  STA $4003           ; High byte and restart note
  
@Done:
  RTS

; Frequency table for notes (period values for pulse channel)
; These correspond approximately to a major scale
NoteTable:
  .byte $C9, $B1, $9F, $8D, $7E, $6F, $63, $5A

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
  ; CHR-ROM data (not used in this example, but required)
  ; Just fill with empty pattern
  .res $2000, $00 