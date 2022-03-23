#![no_std]
#![no_main]

extern crate user_lib;
use core::sync::atomic::{fence, AtomicU8, Ordering::SeqCst};
use user_lib::{flush_trace, getpid, println};
pub const MEMORY_END: usize = 0x101000000;
pub const FLUSH_SIZE: usize = 0x400_0000; // 2M

#[no_mangle]
pub fn main() -> i32 {
    let pid = getpid();
    let offset = FLUSH_SIZE * (pid as usize & 3);
    let start = MEMORY_END + offset;
    fence(SeqCst);
    (start..(start + FLUSH_SIZE)).for_each(|a| unsafe {
        let _ = (*(a as *mut AtomicU8)).load(SeqCst);
    });
    flush_trace();
    fence(SeqCst);
    println!("Flushed: 0x{:#x} to 0x{:#x}.", start, start + FLUSH_SIZE);
    0
}
