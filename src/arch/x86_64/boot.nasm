global start
extern long_mode_start

section .rodata
gdt64:
    dq 0 ; zero entry
.code: equ $ - gdt64 ; new
    dq (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53) ; code segment
.data: equ $ - gdt64 ; new
    dq (1<<44) | (1<<47) | (1<<41) ; data segment
.pointer:
  dw $ - gdt64 - 1
  dq gdt64


section .bss
align 4096
; Page tables
page_map_table:
  resb 4096
page_directory_pointer_table:
  resb 4096
page_directory_table:
  resb 4096  
  
stack_bottom:
  ; set up a small stack.
  resb 64
stack_top:

section .text
bits 32
start:
  ; initalize the stack pointer to the stack we allocated
  ; now we can call functions.
  mov esp, stack_top
  
  ; check that the CPU can do what we're about to ask it to do:
  ;   1. that the multiboot system worked correctly
  ;   2. check that cpuid works
  ;   3. check that we have the ability to enter long mode
  call check_multiboot
  call check_cpuid
  call check_long_mode
  
  ; set up paging
  call initialize_page_tables
  call enable_paging
  
  ; enable SSE, since the rust compiler will use
  ; SEE registers
  call enable_sse
  
  ; load the global descriptor table
  lgdt [gdt64.pointer]
  
  ; set up our segment registers
  mov ax, 16
  mov ss, ax  ; stack selector
  mov ds, ax  ; data selector
  mov es, ax  ; extra selector
  
  ; jump to long mode!
  jmp gdt64.code:long_mode_start

; According to the multiboot2 spec, the bootloader
; must write a magic number to eax. We check it in this
; function.
check_multiboot:
  cmp eax, 0x36d76289
  jne .no_multiboot
  ret
.no_multiboot:
  mov al, "0"
  jmp boot_error
  
; Check kif CPUID is supported. Thanks to OSDEV wiki for this code,
; since it's not very obvious.
; http://wiki.osdev.org/Setting_Up_Long_Mode#Detection_of_CPUID
check_cpuid:
  ; Check if CPUID is supported by attempting to flip the ID bit (bit 21) in
  ; the FLAGS register. If we can flip it, CPUID is available.
  ; Copy FLAGS in to EAX via stack
  pushfd
  pop eax
  ; Copy to ECX as well for comparing later on
  mov ecx, eax
  ; Flip the ID bit
  xor eax, 1 << 21
  ; Copy EAX to FLAGS via the stack
  push eax
  popfd
  ; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
  pushfd
  pop eax
  ; Restore FLAGS from the old version stored in ECX (i.e. flipping the ID bit
  ; back if it was ever flipped).
  push ecx
  popfd
  ; Compare EAX and ECX. If they are equal then that means the bit wasn't
  ; flipped, and CPUID isn't supported.
  cmp eax, ecx
  je .no_cpuid
  ret
.no_cpuid:
  mov al, "1"
  jmp boot_error

; Check if we can enter long mode.
; Once again, inspired by 
; http://wiki.osdev.org/Setting_Up_Long_Mode#x86_or_x86-64
check_long_mode:
  mov eax, 0x80000000
  cpuid
  cmp eax, 0x80000001
  jb .no_long_mode
  mov eax, 0x80000001
  cpuid
  ; the long-mode bit is bit 29
  test edx, 1 << 29
  jz .no_long_mode
  ret
.no_long_mode:
  mov al, "2"
  jmp boot_error
  
; In order to make the transition to long mode,
; we have to set up page tables so we can map some
; memory for the kernel.
initialize_page_tables:
  ; the page tables we are setting up here are multi-level.
  ; our ultimate goal is to create a 2MB page that we will
  ; map to our kernel.
  
  ; set up a directory pointer table entry
  ; and set the first entry in the highest table to point to it
  mov eax, page_directory_pointer_table
  or eax, 0b11 ; readable + writable
  mov [page_map_table], eax
  
  ; set up the second layer of the page table
  mov eax, page_directory_table
  or eax, 0b11 ; readable + writable
  mov [page_directory_pointer_table], eax
  
  mov eax, page_map_table
  or eax, 0b11 ; readable + writable
  mov [page_directory_table], eax
  
  ; create a 512 huge (2MB) pages for the kernel (total of 1GB)
  mov ecx, 0
.loop_header:
  ; 2mb
  mov eax, 0x200000
  ; eax - offset to the i-th page
  mul ecx
  ; huge page, present, writable
  or eax, 0b10000011
  ; map the page
  mov [page_directory_table + ecx * 8], eax
  ; continue until we have mapped 512 pages
  inc ecx
  cmp ecx, 512
  jne .loop_header
  
  ret
  
; Enable paging by writing the page tables we
; set up into the cr3 register
enable_paging:
  ; move our highest-level page table to cr3
  mov eax, page_map_table
  mov cr3, eax
  
  ; set the physical address extension flag in cr4
  mov eax, cr4
  or eax, 1 << 5
  mov cr4, eax
  
  ; set the long mode flag in the model specific register
  mov ecx, 0xC0000080
  rdmsr
  or eax, 1 << 8
  wrmsr
  
  ; set the paging flag in cr0
  mov eax, cr0
  or eax, 1 << 31
  mov cr0, eax
  
  ret
  
  ; Check for SSE and enable it.
  ; thanks again to OS Dev
  enable_sse:
    ; check for SSE
    mov eax, 0x1
    cpuid
    test edx, 1<<25
    jz .no_SSE
    ; enable SSE
    mov eax, cr0
    and ax, 0xFFFB      ; clear coprocessor emulation CR0.EM
    or ax, 0x2          ; set coprocessor monitoring  CR0.MP
    mov cr0, eax
    mov eax, cr4
    or ax, 3 << 9       ; set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
    mov cr4, eax
    ret
  .no_SSE:
    mov al, "3"
    jmp boot_error

; Outputs an error message to the console
; and halts. Used for unrecoverable boot errors.
; The error code (in ascii) is in al.
; The error codes are:
;   0 - multiboot magic number fail
;   1 - cpuid not supported
;   2 - long mode is not available
;   3 - SSE not available
boot_error:
  mov dword [0xb8000], 0x4f524f45 ; ER
  mov dword [0xb8004], 0x4f3a4f52 ; R:
  mov dword [0xb8008], 0x4f204f20 ; __
  mov byte [0xb800a], al         ; error code
  hlt
