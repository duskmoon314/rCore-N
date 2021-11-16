#![no_std]
#![no_main]
#![feature(asm)]

extern crate alloc;
extern crate user_lib;
use core::sync::atomic::{AtomicBool, Ordering::Relaxed};
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;
use riscv::register::uie;
use user_lib::init_user_trap;

static IS_TIMEOUT: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub fn main() -> i32 {
    let mut rng = XorShiftRng::seed_from_u64(0x1020304050607080u64);
    let mut ret: u64 = 0;
    init_user_trap();
    unsafe {
        uie::set_usoft();
    }
    while !IS_TIMEOUT.load(Relaxed) {
        ret = rng.next_u64();
    }
    ret as i32
}
#[no_mangle]
pub fn soft_intr_handler(_pid: usize, _msg: usize) {
    IS_TIMEOUT.store(true, Relaxed);
}
