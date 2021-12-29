#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use user_lib::{sleep, spawn, waitpid};

#[no_mangle]
pub fn main() -> i32 {
    println!("[spawn test]");
    let pid: [usize; 2] = array_init::array_init(|_| spawn("hello_world\0") as usize);
    let mut exit_code: i32 = 0;
    for i in pid {
        waitpid(i, &mut exit_code);
        println!("[spawn test] pid {} exited with code {}", i, exit_code);
    }
    println!("[uart benchmark] User mode interrupt driver benchmark finished.");
    sleep(1000);
    0
}
