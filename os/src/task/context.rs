use crate::trap::trap_return;
use riscv::register::{uie, uip};

#[repr(C)]
pub struct TaskContext {
    ra: usize,
    s: [usize; 12],
    uie: uie::Uie,
    uip: uip::Uip,
    uepc: usize,
    utvec: usize,
    utval: usize,
    ucause: usize,
}

impl TaskContext {
    pub fn goto_trap_return() -> Self {
        Self {
            ra: trap_return as usize,
            s: [0; 12],
            uie: uie::read(),
            uip: uip::read(),
            uepc: 0,
            utvec: 0,
            utval: 0,
            ucause: 0,
        }
    }
}
