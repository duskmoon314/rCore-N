// Based on https://github.com/sgmarz/osblog

use alloc::{collections::VecDeque, sync::Arc};
use lazy_static::*;
use spin::Mutex;

pub const DEFAULT_OUT_BUFFER_SIZE: usize = 10_000;
pub const DEFAULT_IN_BUFFER_SIZE: usize = 1_000;

pub static mut CONSOLE_QUEUE: Option<VecDeque<u16>> = None;

lazy_static! {
    pub static ref IN_BUFFER: Arc<Mutex<VecDeque<u8>>> =
        Arc::new(Mutex::new(VecDeque::with_capacity(DEFAULT_IN_BUFFER_SIZE)));
    pub static ref OUT_BUFFER: Arc<Mutex<VecDeque<u8>>> =
        Arc::new(Mutex::new(VecDeque::with_capacity(DEFAULT_OUT_BUFFER_SIZE)));
}

pub fn push_stdout(c: u8) {
    unsafe {
        let mut out_buffer = OUT_BUFFER.lock();
        if out_buffer.len() < DEFAULT_OUT_BUFFER_SIZE {
            out_buffer.push_back(c);
        }
    }
}

pub fn pop_stdout() -> u8 {
    unsafe {
        let mut out_buffer = OUT_BUFFER.lock();
        out_buffer.pop_front().unwrap_or(0)
    }
}

pub fn push_stdin(c: u8) {
    unsafe {
        let mut in_buffer = IN_BUFFER.lock();
        if in_buffer.len() < DEFAULT_IN_BUFFER_SIZE {
            in_buffer.push_back(c);
            // if c == 10 || c == 11 {
            //     if let Some(mut q) = CONSOLE_QUEUE.take() {
            //         for i in q.drain(..) {
            //             set_running(i);
            //         }
            //     }
            // }
        }
    }
}

pub fn pop_stdin() -> u8 {
    let mut in_buffer = IN_BUFFER.lock();
    in_buffer.pop_front().unwrap_or(0)
}
