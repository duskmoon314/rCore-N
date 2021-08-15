#![no_std]
#![no_main]
#![feature(asm)]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use bitflags::bitflags;
use user_lib::{send_msg, sleep, spawn, waitpid};

bitflags! {
    struct UartLoadConfig: u32 {
        const KERNEL_MODE = 0b1;
        const POLLING_MODE = 0b10;
        const INTR_MODE = 0b100;
        const UART3 = 0b1000;
        const UART4 = 0b10000;
        const ALL_MODE = Self::KERNEL_MODE.bits | Self::POLLING_MODE.bits | Self::INTR_MODE.bits;
    }
}

#[no_mangle]
pub fn main() -> i32 {
    println!("[uart benchmark] Kernel mode driver benchmark begins.");
    let pid1 = spawn("uart_load\0") as usize;
    let pid2 = spawn("uart_load\0") as usize;
    sleep(1000);
    let config1 = UartLoadConfig::KERNEL_MODE | UartLoadConfig::UART3;
    let config2 = UartLoadConfig::KERNEL_MODE | UartLoadConfig::UART4;
    send_msg(pid1, config1.bits() as usize);
    send_msg(pid2, config2.bits() as usize);
    let mut exit_code: i32 = 0;
    waitpid(pid1, &mut exit_code);
    waitpid(pid2, &mut exit_code);
    println!("[uart benchmark] Kernel mode driver benchmark finished.");
    sleep(1000);

    println!("[uart benchmark] User mode polling driver benchmark begins.");
    let pid1 = spawn("uart_load\0") as usize;
    let pid2 = spawn("uart_load\0") as usize;
    sleep(1000);
    let config1 = UartLoadConfig::POLLING_MODE | UartLoadConfig::UART3;
    let config2 = UartLoadConfig::POLLING_MODE | UartLoadConfig::UART4;
    send_msg(pid1, config1.bits() as usize);
    send_msg(pid2, config2.bits() as usize);
    waitpid(pid1, &mut exit_code);
    waitpid(pid2, &mut exit_code);
    println!("[uart benchmark] User mode polling driver benchmark finished.");
    sleep(1000);

    println!("[uart benchmark] User mode interrupt driver benchmark begin.");
    let pid1 = spawn("uart_load\0") as usize;
    let pid2 = spawn("uart_load\0") as usize;
    sleep(1000);
    let config1 = UartLoadConfig::INTR_MODE | UartLoadConfig::UART3;
    let config2 = UartLoadConfig::INTR_MODE | UartLoadConfig::UART4;
    send_msg(pid1, config1.bits() as usize);
    send_msg(pid2, config2.bits() as usize);
    waitpid(pid1, &mut exit_code);
    waitpid(pid2, &mut exit_code);
    println!("[uart benchmark] User mode interrupt driver benchmark finished.");
    sleep(1000);
    0
}
