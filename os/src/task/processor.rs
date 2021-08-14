use super::TaskControlBlock;
use super::__switch;
use super::add_task;
use super::{fetch_task, TaskStatus};
use crate::config::CPU_NUM;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::RefCell;
use lazy_static::*;
lazy_static! {
    pub static ref PROCESSORS: [Processor; CPU_NUM] = Default::default();
}

pub struct Processor {
    inner: RefCell<ProcessorInner>,
}

impl Default for Processor {
    fn default() -> Self {
        Self {
            inner: RefCell::new(ProcessorInner {
                current: None,
                idle_task_cx_ptr: 0,
            }),
        }
    }
}

unsafe impl Sync for Processor {}

struct ProcessorInner {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx_ptr: usize,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(ProcessorInner {
                current: None,
                idle_task_cx_ptr: 0,
            }),
        }
    }
    fn get_idle_task_cx_ptr2(&self) -> *const usize {
        let inner = self.inner.borrow();
        &inner.idle_task_cx_ptr as *const usize
    }

    fn run_next(&self, task: Arc<TaskControlBlock>) {
        let idle_task_cx_ptr2 = self.get_idle_task_cx_ptr2();
        // acquire
        let mut task_inner = task.acquire_inner_lock();
        let next_task_cx_ptr2 = task_inner.get_task_cx_ptr2();
        task_inner.task_status = TaskStatus::Running;
        // unsafe {
        //     use crate::mm::PhysAddr;
        //     // let mut ra: usize;
        //     let ra: usize = (*(task_inner.task_cx_ptr as *const TaskContext)).ra;
        //     // asm!("ld {}, 0({})", out(reg)ra, in(reg)task_inner.task_cx_ptr);
        //     if ra > (8usize << 60)
        //         || ra == 0x80570230
        //         || ra == 0x80371230
        //         || ra == 0x80373230
        //         || ra == 0x80572230
        //     {
        //         let mut sp: usize;
        //         asm!("mv {}, sp", out(reg) sp);
        //         let mut token: usize;
        //         asm!("csrr {}, satp", out(reg) token);
        //         // let idle_task_cx_ptr = *idle_task_cx_ptr2;
        //         warn!(
        //             "wrong ra in scheduler: {:#x}, pid: {}, sp: {:#x}, task_cx addr: {:#x}, trap_cx addr: {:?}",
        //             ra, task.pid.0, sp, task_inner.task_cx_ptr, PhysAddr::from(task_inner.trap_cx_ppn)
        //         );
        //         debug!(
        //             "current satp: {:#x}, task satp: {:#x}",
        //             token,
        //             task_inner.memory_set.token()
        //         );
        //         let mut ra_r: usize;
        //         asm!("mv {}, ra", out(reg) ra_r);
        //         debug!("ra in reg: {:#x}", ra_r);
        //         debug!("*ra: {:#x}", *(ra as *const usize));
        //         debug!("*ra as {:#x?}", *(ra as *const TaskControlBlockInner));
        //         // warn!("task_cx: {:#x?}", task_inner.get_trap_cx());
        //         // asm!("ebreak");
        //     } else {
        //         // debug!(
        //         //     "normal ra before scheduler: {:#x}, task_cx_ptr: {:#x}",
        //         //     ra, task_inner.task_cx_ptr
        //         // );
        //     }
        // }
        // // release
        if let Some(trap_info) = &task_inner.user_trap_info {
            trap_info.enable_user_ext_int();
        }
        drop(task_inner);
        self.inner.borrow_mut().current = Some(task);

        unsafe {
            __switch(idle_task_cx_ptr2, next_task_cx_ptr2);
        }
    }

    fn suspend_current(&self) {
        if let Some(task) = take_current_task() {
            // ---- hold current PCB lock
            let mut task_inner = task.acquire_inner_lock();
            // Change status to Ready
            task_inner.task_status = TaskStatus::Ready;
            if let Some(trap_info) = &task_inner.user_trap_info {
                trap_info.disable_user_ext_int();
            }
            drop(task_inner);
            // ---- release current PCB lock

            // push back to ready queue.
            add_task(task);
        }
    }

    pub fn run(&self) {
        loop {
            if let Some(task) = fetch_task() {
                self.run_next(task);
                // __switch inside run_next
                self.suspend_current();
            }
        }
    }
    pub fn take_current(&self) -> Option<Arc<TaskControlBlock>> {
        self.inner.borrow_mut().current.take()
    }
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.inner
            .borrow()
            .current
            .as_ref()
            .map(|task| Arc::clone(task))
    }
}

// lazy_static! {
//     pub static ref PROCESSOR: Processor = Processor::new();
// }

pub fn hart_id() -> usize {
    let hart_id: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) hart_id);
    }
    hart_id
}

pub fn run_tasks() {
    debug!("run_tasks");
    PROCESSORS[hart_id()].run();
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSORS[hart_id()].take_current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSORS[hart_id()].current()
}

pub fn current_tasks() -> Vec<Option<Arc<TaskControlBlock>>> {
    PROCESSORS
        .iter()
        .map(|processor| processor.current())
        .collect()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.acquire_inner_lock().get_user_token();
    token
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task().unwrap().acquire_inner_lock().get_trap_cx()
}

pub fn schedule(switched_task_cx_ptr2: *const usize) {
    let idle_task_cx_ptr2 = PROCESSORS[hart_id()].get_idle_task_cx_ptr2();
    unsafe {
        __switch(switched_task_cx_ptr2, idle_task_cx_ptr2);
    }
}

pub fn set_current_priority(priority: isize) -> Result<isize, isize> {
    if let Some(current) = current_task() {
        let mut current = current.acquire_inner_lock();
        current.set_priority(priority)
    } else {
        Err(-1)
    }
}

pub fn mmap(start: usize, len: usize, port: usize) -> Result<isize, isize> {
    if let Some(current) = current_task() {
        let mut current = current.acquire_inner_lock();
        current.mmap(start, len, port)
    } else {
        Err(-1)
    }
}

pub fn munmap(start: usize, len: usize) -> Result<isize, isize> {
    if let Some(current) = current_task() {
        let mut current = current.acquire_inner_lock();
        current.munmap(start, len)
    } else {
        Err(-1)
    }
}
