.globl __push_trace
.attribute arch, "rv64imac"
# event_id in a0
__push_trace:
    # prelogue
    addi sp, sp, -16
    sd t0, 0*8(sp)
    sd t1, 1*8(sp)

    li t0, 0x101000000 # MEMORY_END
    li t1, 2*8
    amoadd.d t1, t1, (t0) # t2 <- queue_tail, queue_tail <- queue_tail + 16
    slli t0, tp, 32
    or a0, a0, t0
    slli t0, gp, 36
    or a0, a0, t0
    sd a0, 0*8(t1)
    csrr a0, cycle
    sd a0, 1*8(t1)

    # epilogue
    ld t1, 1*8(sp)
    ld t0, 0*8(sp)
    addi sp, sp, 16
