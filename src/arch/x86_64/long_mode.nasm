global long_mode_start
extern kernel_main

section .text
bits 64
long_mode_start:
  ; call into rust!
  call kernel_main
  hlt
