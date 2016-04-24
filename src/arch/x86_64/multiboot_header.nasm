section .multiboot_header
header_start:
    dd 0xE85250D6                ; multiboot magic number
    dd 0                         ; architecture: i386
    dd header_end - header_start ; size of the header
    ; header checksum
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))
    
    dd 0 ; type
    dd 0 ; flags
    dd 8 ; size
header_end:
