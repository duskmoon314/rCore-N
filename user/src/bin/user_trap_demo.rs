#![no_std]
#![no_main]
#![feature(asm)]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use lazy_static::*;
use riscv::register::uie;
use spin::Mutex;
use user_lib::{
    exit, get_time, init_user_trap, send_msg, set_timer, spawn, yield_, UserTrapContext,
    UserTrapRecord,
};

lazy_static! {
    static ref PID: Mutex<isize> = Mutex::new(0);
}

#[no_mangle]
pub fn main() -> i32 {
    println!("user trap demo");
    let pid = spawn("uart_ext\0");
    if pid > 0 {
        *PID.lock() = pid;
        init_user_trap();
        let time_us = get_time() * 1000;
        for i in 1..=10 {
            set_timer(time_us + i * 1000_000);
        }
        unsafe {
            uie::set_uext();
            uie::set_usoft();
            uie::set_utimer();
        }
        loop {
            yield_();
        }
    } else {
        println!("[trap demo] spawn failed!");
    }
    0
}

use riscv::register::{ucause, uepc, uip, uscratch, utval};
pub const PAGE_SIZE: usize = 0x1000;
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
pub const USER_TRAP_BUFFER: usize = TRAP_CONTEXT - PAGE_SIZE;
#[no_mangle]
pub fn user_trap_handler(cx: &mut UserTrapContext) -> &mut UserTrapContext {
    let ucause = ucause::read();
    let utval = utval::read();
    match ucause.cause() {
        ucause::Trap::Interrupt(ucause::Interrupt::UserSoft) => {
            let trap_record_num = uscratch::read();
            println!(
                "[user trap demo] user soft interrupt, num: {}",
                trap_record_num
            );
            let mut head_ptr = USER_TRAP_BUFFER as *const UserTrapRecord;
            for _ in 0..trap_record_num {
                unsafe {
                    let trap_record = *head_ptr;
                    let cause = trap_record.cause;
                    println!(
                        "[user trap demo] cause: {}, message {}",
                        cause, trap_record.message,
                    );
                    if ucause::Interrupt::from(cause) == ucause::Interrupt::UserTimer {
                        handle_timer_interrupt();
                    }
                    head_ptr = head_ptr.offset(1);
                }
            }
            unsafe {
                uip::clear_usoft();
            }
        }
        ucause::Trap::Interrupt(ucause::Interrupt::UserTimer) => {
            println!("[user trap demo] user timer interrupt at {} ms", get_time());
            handle_timer_interrupt();
            unsafe {
                uip::clear_utimer();
            }
        }
        _ => {
            println!(
                "Unsupported trap {:?}, utval = {:#x}, uepc = {:#x}!",
                ucause.cause(),
                utval,
                uepc::read()
            );
        }
    }
    cx
}

fn handle_timer_interrupt() {
    lazy_static! {
        static ref TRAP_COUNT: Mutex<usize> = Mutex::new(0);
    }
    let mut trap_count = TRAP_COUNT.lock();
    if *trap_count == 9 {
        println!("[user trap demo] sending SIGTERM");
        send_msg(*PID.lock() as usize, 15);
        drop(trap_count);
        exit(0);
    } else {
        *trap_count += 1;
        let msg = 0xdeadbeef00 + *trap_count as usize;
        println!("[user trap demo] sending msg: {:x?}", msg);
        send_msg(*PID.lock() as usize, msg);
    }
}
