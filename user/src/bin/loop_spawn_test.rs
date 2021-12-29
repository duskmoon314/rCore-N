#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use user_lib::{spawn, waitpid};

#[no_mangle]
pub fn main() -> i32 {
    for k in 0..1000 {
        let pid: [usize; 2] = array_init::array_init(|_| spawn("hello_world_simple\0") as usize);
        let mut exit_code: i32 = 0;
        println!("[spawn test] loop {}", k);
        for i in pid {
            waitpid(i, &mut exit_code);
            println!("[spawn test] pid {} exited with code {}", i, exit_code);
        }
    }
    0
}
