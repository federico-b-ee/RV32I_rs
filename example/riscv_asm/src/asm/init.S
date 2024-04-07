.section .text.init
.global _start
_start:
    la   sp, __sp
    addi x22, x0, 22
    call _rust_entry

