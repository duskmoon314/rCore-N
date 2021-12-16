use crate::trap::trap_return;
use riscv::register::{uie, uip};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TaskContext {
    pub ra: usize,
    pub s: [usize; 12],
    pub uie: usize,
    pub uip: usize,
    pub uepc: usize,
    pub utvec: usize,
    pub utval: usize,
    pub ucause: usize,
    pub sp: usize,
}

impl TaskContext {
    pub fn goto_trap_return(kernel_stack_top: usize) -> Self {
        Self {
            ra: trap_return as usize,
            s: [0; 12],
            uie: uie::read().bits(),
            uip: uip::read().bits(),
            uepc: 0,
            utvec: 0,
            utval: 0,
            ucause: 0,
            sp: kernel_stack_top,
        }
    }
}

impl Default for TaskContext {
    fn default() -> Self {
        Self {
            ra: 0xDEDEDEDE,
            s: [0x23232323; 12],
            uie: 0,
            uip: 0,
            uepc: 0,
            utvec: 0,
            utval: 0,
            ucause: 0,
            sp: 0xABABABAB,
        }
    }
}
