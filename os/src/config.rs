pub const USER_STACK_SIZE: usize = 0x4000;
pub const KERNEL_STACK_SIZE: usize = 0x4000;
pub const KERNEL_HEAP_SIZE: usize = 0x20_0000;

#[cfg(feature = "board_qemu")]
pub const MEMORY_END: usize = 0x80800000;

#[cfg(feature = "board_lrv")]
pub const MEMORY_END: usize = 0x100800000;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
pub const USER_TRAP_BUFFER: usize = TRAP_CONTEXT - PAGE_SIZE;

#[cfg(feature = "board_qemu")]
pub const CLOCK_FREQ: usize = 12500000;

#[cfg(feature = "board_lrv")]
pub const CLOCK_FREQ: usize = 10_000_000;
