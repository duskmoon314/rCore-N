    .section .text.entry
    .globl _start
_start:
    # a0: hart id
    mv tp, a0
    la sp, boot_stack
    # li t1, 4096 * 16 # t1 = 4096 * 16 64KB
    addi t0, a0, 1  # t0 = a0 + 1 hartid+1
    slli t0, t0, 16 # 64K * (hartid + 1)
    add sp, sp, t0  # sp = sp + t0
    call rust_main

    .section .bss.stack
    .globl boot_stack
boot_stack:
    .space 4096 * 16 * 4
    .globl boot_stack_top
boot_stack_top: