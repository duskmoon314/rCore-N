#![no_std]
#![no_main]
#![feature(asm)]

#[macro_use]
extern crate user_lib;

use riscv::register::{mtvec::TrapMode, uie, uip};
use riscv::register::{ustatus, utvec};
use user_lib::{exec, fork, sleep, wait, yield_};

pub fn init_u() {
    extern "C" {
        fn __alltraps_u();
    }
    unsafe {
        utvec::write(__alltraps_u as usize, TrapMode::Direct);
    }
}

#[no_mangle]
fn main() -> i32 {
    println!("hello initproc");
    // let pid = fork();
    // unsafe {
    //     asm!("nop");
    //     if pid != 0 {
    //         utvec::write(0xaaa0, TrapMode::Direct);
    //     } else {
    //         utvec::write(0xfff0, TrapMode::Direct);
    //     }
    // }

    // for _ in 0..5 {
    //     sleep(200);
    //     println!(
    //         "pid {} utvec {:?} ustatus {:?}",
    //         pid,
    //         utvec::read(),
    //         ustatus::read()
    //     );
    // }

    // -------------------

    init_u();
    println!("utvec {:?}", utvec::read());

    unsafe {
        uie::set_usoft();
        uip::set_usoft();
    }

    0
}
