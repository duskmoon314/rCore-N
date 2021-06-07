use crate::trap::trap_return;

#[repr(C)]
pub struct TaskContext {
    ra: usize,
    s: [usize; 12],
    utvec: usize,
    // uie: usize,
}

impl TaskContext {
    pub fn goto_trap_return() -> Self {
        Self {
            ra: trap_return as usize,
            s: [0; 12],
            utvec: 0,
        }
    }
}
