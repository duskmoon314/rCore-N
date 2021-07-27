use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use lazy_static::*;
use riscv::register::time;
use spin::Mutex;

const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;
pub const USEC_PER_SEC: usize = 1000_000;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[allow(dead_code)]
impl TimeVal {
    pub fn new() -> Self {
        TimeVal { sec: 0, usec: 0 }
    }
}

#[allow(unused_variables)]
pub fn get_time(mut ts: Vec<*mut usize>, tz: usize) -> isize {
    let t = time::read();
    unsafe {
        *ts[0] = t / CLOCK_FREQ;
        *ts[1] = (t % CLOCK_FREQ) * 1000000 / CLOCK_FREQ;
        trace!("t {} sec {} usec {}", t, *ts[0], *ts[1]);
    }

    0
}

#[allow(dead_code)]
pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MSEC_PER_SEC)
}

#[allow(dead_code)]
pub fn get_time_us() -> usize {
    time::read() * USEC_PER_SEC / CLOCK_FREQ
}

pub fn set_next_trigger() {
    // set_timer(time::read() + CLOCK_FREQ / TICKS_PER_SEC);
    set_virtual_timer(time::read() + CLOCK_FREQ / TICKS_PER_SEC, 0);
}

lazy_static! {
    pub static ref TIMER_MAP: Arc<Mutex<BTreeMap<usize, usize>>> =
        Arc::new(Mutex::new(BTreeMap::new()));
}

pub fn set_virtual_timer(time: usize, pid: usize) {
    if time < time::read() {
        warn!("Time travel unallowed!");
        return;
    }
    let mut timer_map = TIMER_MAP.lock();
    timer_map.insert(time, pid);
    if let Some((timer_min, _)) = timer_map.first_key_value() {
        if time == *timer_min {
            set_timer(time);
        }
    }
}
