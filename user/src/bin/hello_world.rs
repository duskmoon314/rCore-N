#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use core::sync::atomic::{AtomicBool, Ordering::Relaxed};
use riscv::register::uie;
use user_lib::{get_time, getpid, init_user_trap, set_timer, sleep};
static IS_TIMEOUT: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub fn main() -> i32 {
    println!("[hello world] from pid: {}", getpid());
    sleep(1000);
    let init_res = init_user_trap();
    println!(
        "[hello world] trap init result: {:#x}, now using timer to sleep",
        init_res
    );
    unsafe {
        uie::set_usoft();
        uie::set_utimer();
    }
    let time_us = get_time() * 1000;
    set_timer(time_us + 1000_000);
    while !IS_TIMEOUT.load(Relaxed) {}
    println!("[hello world] timer finished, now exit");

    0
}

#[no_mangle]
pub fn timer_intr_handler(time_us: usize) {
    println!(
        "[user trap default] user timer interrupt, time (us): {}",
        time_us
    );
    IS_TIMEOUT.store(true, Relaxed);
}
