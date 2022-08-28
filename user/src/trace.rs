// S trap
pub const S_TRAP_VEC_ENTER: usize = 0x57ab_0000;
pub const S_TRAP_VEC_RESTORE: usize = 0x57ab_1000;
pub const S_TRAP_HANDLER: usize = 0x57ab_2000;
pub const S_TRAP_RETURN: usize = 0x57ab_3000;
pub const S_EXT_INTR_ENTER: usize = 0x57ab_4000;
pub const S_EXT_INTR_EXIT: usize = 0x57ab_5000;

// scheduler
pub const SCHEDULE: usize = 0x5ced_0000;
pub const RUN_NEXT: usize = 0x5ced_1000;
pub const SUSPEND_CURRENT: usize = 0x5ced_2000;

// U trap
pub const ENABLE_USER_EXT_INT_ENTER: usize = 0xc7ab_0000;
pub const ENABLE_USER_EXT_INT_EXIT: usize = 0xc7ab_1000;
pub const DISABLE_USER_EXT_INT_ENTER: usize = 0xc7ab_2000;
pub const DISABLE_USER_EXT_INT_EXIT: usize = 0xc7ab_3000;
pub const PUSH_TRAP_RECORD_ENTER: usize = 0xc7ab_4000;
pub const PUSH_TRAP_RECORD_EXIT: usize = 0xc7ab_5000;
pub const TRAP_QUEUE_ENTER: usize = 0xc7ab_6000;
pub const TRAP_QUEUE_EXIT: usize = 0xc7ab_7000;
pub const U_TRAP_HANDLER: usize = 0xc7ab_8000;
pub const U_TRAP_RETURN: usize = 0xc7ab_9000;
pub const U_EXT_HANDLER: usize = 0xc7ab_a000;
pub const U_SOFT_HANDLER: usize = 0xc7ab_b000;
pub const U_TIMER_HANDLER: usize = 0xc7ab_c000;

// syscall
pub const TRACE_SYSCALL_ENTER: usize = 0x575c_0000;
pub const TRACE_SYSCALL_EXIT: usize = 0x575c_1000;

// SBI call
pub const SEND_IPI_ENTER: usize = 0x5b1c_0000;
pub const SEND_IPI_EXIT: usize = 0x5b1c_1000;

// misc
pub const TRACE_TEST: usize = 0x315c_0000;

pub const MEMORY_END: usize = 0x101000000;

core::arch::global_asm!(include_str!("trace.asm"));

extern "C" {
    fn __push_trace(event_id: usize) -> usize;
}

// FIXME: use of possibly-uninitialized `cycle`
pub fn push_trace(event_id: usize) -> usize {
    let mut cycle: usize = 0;
    #[cfg(feature = "board_lrv")]
    unsafe {
        // __push_trace(event_id)
        core::arch::asm!(
            "
        amoadd.d {tail}, {step}, ({mem_end}) # t2 <- queue_tail, queue_tail <- queue_tail + 16
        slli {eid_ext}, tp, 32
        or {eid}, {eid}, {eid_ext}
        slli {eid_ext}, gp, 36
        or {eid}, {eid}, {eid_ext}
        sd {eid}, 0*8({tail})
        csrr {cy}, cycle
        sd {cy}, 1*8({tail})",
        eid = in(reg) event_id,
        step = in(reg) 16,
        mem_end = in(reg) MEMORY_END,
        cy = out(reg) cycle,
        tail = out(reg) _,
        eid_ext = out(reg) _,
        )
    }
    cycle
}
