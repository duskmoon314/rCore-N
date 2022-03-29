#![no_std]
#![no_main]
use user_lib::{println, spawn, waitpid};
#[no_mangle]
pub fn main() -> i32 {
    const WORKER_NUM: usize = 4;
    println!(
        "Start flushing trace from cache with {} worker processes...",
        WORKER_NUM
    );
    let worker_pid: [usize; WORKER_NUM] =
        array_init::array_init(|_| spawn("flush_trace\0") as usize);
    let mut exit_code: i32 = 0;
    for i in 0..WORKER_NUM {
        waitpid(worker_pid[i], &mut exit_code);
    }
    0
}
