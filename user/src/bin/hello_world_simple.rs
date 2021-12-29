#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use user_lib::{getpid, sleep};

#[no_mangle]
pub fn main() -> i32 {
    println!("[hello world] from pid: {}", getpid());
    sleep(100);
    0
}
