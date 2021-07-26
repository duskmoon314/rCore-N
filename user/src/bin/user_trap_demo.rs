#![no_std]
#![no_main]
#![feature(asm)]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use user_lib::{send_msg, sleep, spawn, UserTrapContext, UserTrapRecord};

#[no_mangle]
pub fn main() -> i32 {
    println!("user trap demo");
    // WORK
    let pid = spawn("uart_ext\0");

    // DOES NOT WORK
    // let pid = spawn("uart_ext");

    if pid > 0 {
        for i in 0..10 {
            sleep(1000);
            let msg = 0xdeadbeef00 + i as usize;
            println!("[trap demo] sending msg: {:x?}", msg);
            send_msg(pid as usize, msg);
        }
    } else {
        println!("[trap demo] spawn failed!");
    }
    0
}

#[no_mangle]
pub fn user_trap_handler(cx: &mut UserTrapContext) -> &mut UserTrapContext {
    cx
}
