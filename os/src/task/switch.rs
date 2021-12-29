use super::TaskContext;
use core::arch::global_asm;

global_asm!(include_str!("switch.asm"));

extern "C" {
    #[deprecated]
    pub fn __switch(current_task_cx_ptr2: *const usize, next_task_cx_ptr2: *const usize);
    pub fn __switch2(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *mut TaskContext);
}
