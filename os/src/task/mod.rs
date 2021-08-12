mod context;
mod manager;
mod pid;
mod pool;
mod processor;
mod switch;
mod task;

use crate::{loader::get_app_data_by_name, task::task::TaskControlBlockInner};
use alloc::sync::Arc;
use lazy_static::*;

use switch::__switch;
use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;
pub use pid::{pid_alloc, KernelStack, PidHandle};
pub use pool::{add_task, fetch_task, find_task};
pub use processor::{
    current_task, current_trap_cx, current_user_token, hart_id, mmap, munmap, run_tasks, schedule,
    set_current_priority, take_current_task,
};

pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- hold current PCB lock
    let mut task_inner = task.acquire_inner_lock();
    let task_cx_ptr2 = task_inner.get_task_cx_ptr2();
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    unsafe {
        use crate::mm::PhysAddr;
        let ra: usize = (*(task_inner.task_cx_ptr as *const TaskContext)).ra;
        if ra > (8usize << 60)
            || ra == 0x80570230
            || ra == 0x80371230
            || ra == 0x80373230
            || ra == 0x80572230
        {
            let mut sp: usize;
            asm!("mv {}, sp", out(reg) sp);
            let mut token: usize;
            asm!("csrr {}, satp", out(reg) token);
            warn!(
                "wrong ra before scheduler: {:#x}, pid: {}, sp: {:#x}, task_cx addr: {:#x}, trap_cx addr: {:?}",
                ra, task.pid.0, sp, task_inner.task_cx_ptr, PhysAddr::from( task_inner.trap_cx_ppn)
            );
            debug!(
                "current satp: {:#x}, task satp: {:#x}",
                token,
                task_inner.memory_set.token()
            );
            debug!("*ra: {:#x}", *(ra as *const usize));
            debug!("*ra as {:#x?}", *(ra as *const TaskControlBlockInner));
            trace!(
                "task_cx: {:#x?}",
                *(task_inner.task_cx_ptr as *const TaskContext)
            );
            trace!("trap_cx: {:#x?}", task_inner.get_trap_cx());
            // asm!("ebreak");
        } else {
            // debug!(
            //     "normal ra before scheduler: {:#x}, task_cx_ptr: {:#x}",
            //     ra, task_inner.task_cx_ptr
            // );
        }
    }
    if let Some(trap_info) = &task_inner.user_trap_info {
        trap_info.disable_user_ext_int();
    }
    drop(task_inner);
    // ---- release current PCB lock

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr2);

    let task = current_task().unwrap();
    // ---- hold current PCB lock
    let inner = task.acquire_inner_lock();
    if let Some(trap_info) = &inner.user_trap_info {
        trap_info.enable_user_ext_int();
    }
}

pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();
    debug!("pid: {} exited with code {}", task.pid.0, exit_code);
    // **** hold current PCB lock
    let mut inner = task.acquire_inner_lock();
    if let Some(trap_info) = &inner.user_trap_info {
        trap_info.remove_user_ext_int_map();
        use riscv::register::sie;
        unsafe {
            sie::clear_uext();
            sie::clear_usoft();
            sie::clear_utimer();
        }
    }

    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // ++++++ hold initproc PCB lock here
    {
        let mut initproc_inner = INITPROC.acquire_inner_lock();
        for child in inner.children.iter() {
            child.acquire_inner_lock().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ release parent PCB lock here

    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();
    drop(inner);
    // **** release current PCB lock
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let _unused: usize = 0;
    schedule(&_unused as *const _);

    let task = current_task().unwrap();
    let task_inner = task.acquire_inner_lock();
    if let Some(trap_info) = &task_inner.user_trap_info {
        trap_info.enable_user_ext_int();
    }
}

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(
        get_app_data_by_name("initproc").unwrap()
    ));
}

pub fn add_initproc() {
    debug!("add_initproc");
    add_task(INITPROC.clone());
}
