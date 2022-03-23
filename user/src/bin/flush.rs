#![no_std]
#![no_main]
use user_lib::spawn;
#[no_mangle]
pub fn main() -> i32 {
    for _ in 0..4 {
        spawn("flush_trace\0");
    }
    0
}
