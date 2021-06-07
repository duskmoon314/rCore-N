global_asm!(include_str!("switch.asm"));

extern "C" {
    pub fn __switch(current_task_cx_ptr2: *const usize, next_task_cx_ptr2: *const usize);
}
