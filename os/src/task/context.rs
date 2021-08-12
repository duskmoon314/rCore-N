use crate::trap::trap_return;
use riscv::register::{uie, uip};

#[repr(C)]
#[derive(Debug)]
pub struct TaskContext {
    pub ra: usize,
    pub s: [usize; 12],
    pub uie: uie::Uie,
    pub uip: uip::Uip,
    pub uepc: usize,
    pub utvec: usize,
    pub utval: usize,
    pub ucause: usize,
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
