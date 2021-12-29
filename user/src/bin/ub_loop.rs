#![no_std]
#![no_main]
#![feature(asm)]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use user_lib::{getpid, spawn, waitpid};

#[no_mangle]
pub fn main() -> i32 {
    println!("[ub loop] from pid: {}", getpid());
    let mut exit_code: i32 = 0;
    for i in 0..100 {
        println!("[ub loop] loop: {}", i);
        let pid = spawn("uart_benchmark\0") as usize;
        waitpid(pid, &mut exit_code);
    }
    0
}
