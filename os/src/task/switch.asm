.altmacro
.macro SAVE_SN n
    sd s\n, (\n+1)*8(sp)
.endm
.macro LOAD_SN n
    ld s\n, (\n+1)*8(sp)
.endm
    .section .text
    .globl __switch
__switch:
    # __switch(
    #     current_task_cx_ptr2: &*const TaskContext,
    #     next_task_cx_ptr2: &*const TaskContext
    # )
    # push TaskContext to current sp and save its address to where a0 points to
    addi sp, sp, -19*8
    sd sp, 0(a0)
    # fill TaskContext with ra & s0-s11
    sd ra, 0(sp)
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr
    csrr s0, uie
    csrr s1, uip
    csrr s2, uepc
    csrr s3, utvec
    csrr s4, utval
    csrr s5, ucause
    sd s0, 13*8(sp)
    sd s1, 14*8(sp)
    sd s2, 15*8(sp)
    sd s3, 16*8(sp)
    sd s4, 17*8(sp)
    sd s5, 18*8(sp)

    # ready for loading TaskContext a1 points to
    ld sp, 0(a1)
    # load registers in the TaskContext
    ld ra, 0(sp)
    sd s0, 13*8(sp)
    sd s1, 14*8(sp)
    sd s2, 15*8(sp)
    sd s3, 16*8(sp)
    sd s4, 17*8(sp)
    sd s5, 18*8(sp)
    csrw uie, s0
    csrw uip, s1
    csrw uepc, s2
    csrw utvec, s3
    csrw utval, s4
    csrw ucause, s5
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
    # pop TaskContext
    addi sp, sp, 19*8
    ret

