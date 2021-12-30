#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use bitflags::bitflags;
use user_lib::{send_msg, sleep, spawn, waitpid};

const CPU_LOAD_NUM: usize = 1;

bitflags! {
    struct IpcLoadConfig: u32 {
        const MSG_MODE = 0b1;
        const MAIL_MODE = 0b10;
        const UINTC_MODE = 0b100;
        const ALL_MODE = Self::MSG_MODE.bits | Self::MAIL_MODE.bits | Self::UINTC_MODE.bits;
    }
}

#[no_mangle]
pub fn main() -> i32 {
    let cpu_load_pid: [usize; CPU_LOAD_NUM] =
        array_init::array_init(|_| spawn("cpu_load\0") as usize);
    let mut exit_code: i32 = 0;
    println!("[ipc benchmark] Sendmsg benchmark begins.");
    let pid1 = spawn("ipc_load\0") as usize;
    let pid2 = spawn("ipc_load\0") as usize;
    sleep(1000);
    let config1 = IpcLoadConfig::MSG_MODE;
    let config2 = IpcLoadConfig::MSG_MODE;
    send_msg(pid1, config1.bits() as usize | pid2 << 8);
    send_msg(pid2, config2.bits() as usize | pid1 << 8);
    waitpid(pid1, &mut exit_code);
    waitpid(pid2, &mut exit_code);
    println!("[ipc benchmark] Sendmsg benchmark finished.");
    sleep(1000);

    println!("[ipc benchmark] Mailbox benchmark begins.");
    let pid1 = spawn("ipc_load\0") as usize;
    let pid2 = spawn("ipc_load\0") as usize;
    sleep(1000);
    let config1 = IpcLoadConfig::MAIL_MODE;
    let config2 = IpcLoadConfig::MAIL_MODE;
    send_msg(pid1, config1.bits() as usize | pid2 << 8);
    send_msg(pid2, config2.bits() as usize | pid1 << 8);
    waitpid(pid1, &mut exit_code);
    waitpid(pid2, &mut exit_code);
    println!("[ipc benchmark] Mailbox benchmark finished.");
    sleep(1000);

    for i in cpu_load_pid {
        send_msg(i, 15);
        waitpid(i, &mut exit_code);
    }
    0
}
