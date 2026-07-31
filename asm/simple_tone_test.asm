; Basic Tone Test ROM for RustNES
; This demonstrates how to generate a simple continuous tone using the APU pulse channel 1
; Uses only the implemented instructions (no INC)

.segment "HEADER"
  .byte "NES", $1A      ; iNES header identifier
  .byte 1               ; 1x 16KB PRG-ROM pages
  .byte 1               ; 1x  8KB CHR-ROM pages
  .byte $01, $00        ; mapper 0, vertical mirroring, no battery, no trainer
  .byte $00, $00, $00, $00, $00, $00, $00, $00  ; padding

.segment "STARTUP"
RESET:
  ; Set up the stack
  LDX #$FF
  TXS
  
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

; Main loop - just maintain the tone
MainLoop:
  JSR WaitForVBlank     ; Wait for VBLANK for timing
  JMP MainLoop          ; Repeat forever

; Initialize the APU
InitializeAPU:
  ; Initialize pulse channel 1 ($4000-$4003)
  
  ; $4000 - Duty cycle, envelope, volume
  ; 01xx xxxx = 25% duty cycle (square wave)
  ; xx1x xxxx = halt the length counter, so the tone sustains instead of
  ;             fading out after ~80ms
  ; xxx1 xxxx = use constant volume (disable envelope)
  ; xxxx 1111 = maximum volume (15)
  LDA #%01111111
  STA $4000
  
  ; $4001 - Sweep unit
  ; 0xxx xxxx = sweep off
  LDA #%00000000
  STA $4001
  
  ; $4002/$4003 - Frequency (period)
  ; Set to play a middle A note (440Hz)
  ; Period = CPU_FREQ / (16 * note_freq) - 1
  ; For 440Hz: Period = 1789773 / (16 * 440) - 1 = 254
  LDA #%11111110  ; Low 8 bits of period (254)
  STA $4002
  LDA #%00000000  ; High 3 bits of period
  STA $4003       ; Writing to $4003 restarts the note
  
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
  ; CHR-ROM data (not used in this example, but required)
  ; Just fill with empty pattern
  .res $2000, $00 