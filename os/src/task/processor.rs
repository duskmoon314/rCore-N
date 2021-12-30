use super::TaskContext;
use super::TaskControlBlock;
use super::__switch2;
use super::add_task;
use super::{fetch_task, TaskStatus};
use crate::config::CPU_NUM;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::arch::asm;
use core::cell::RefCell;
use riscv::register::cycle;

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
                idle_task_cx: Default::default(),
                idle_task_cx_ptr: 0,
            }),
        }
    }
}

unsafe impl Sync for Processor {}

struct ProcessorInner {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx: TaskContext,
    idle_task_cx_ptr: usize,
}

impl Processor {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(ProcessorInner {
                current: None,
                idle_task_cx: Default::default(),
                idle_task_cx_ptr: 0,
            }),
        }
    }
    fn get_idle_task_cx_ptr(&self) -> *mut TaskContext {
        let mut inner = self.inner.borrow_mut();
        &mut inner.idle_task_cx as *mut TaskContext
    }
    #[allow(unused)]
    #[deprecated]
    fn get_idle_task_cx_ptr2(&self) -> *const usize {
        let inner = self.inner.borrow();
        &inner.idle_task_cx_ptr as *const usize
    }

    fn run_next(&self, task: Arc<TaskControlBlock>) {
        let idle_task_cx_ptr = self.get_idle_task_cx_ptr();
        trace!(
            "[run next] idle task cx ptr: {:x?}, task cx: {:#x?}",
            idle_task_cx_ptr,
            unsafe { &*idle_task_cx_ptr }
        );
        // acquire
        let mut task_inner = task.acquire_inner_lock();
        let next_task_cx_ptr = task_inner.get_task_cx_ptr();
        task_inner.task_status = TaskStatus::Running(hart_id());
        if let Some(trap_info) = &task_inner.user_trap_info {
            trap_info.enable_user_ext_int();
        }
        let task_cx = unsafe { &*next_task_cx_ptr };
        trace!(
            "next task cx ptr: {:#x?}, task cx: {:#x?}",
            next_task_cx_ptr,
            task_cx
        );
        task_inner.last_cpu_cycle = cycle::read();
        // release
        drop(task_inner);
        self.inner.borrow_mut().current = Some(task);

        unsafe {
            __switch2(idle_task_cx_ptr, next_task_cx_ptr);
        }
    }

    fn suspend_current(&self) {
        trace!("[suspend current]");
        if let Some(task) = take_current_task() {
            // ---- hold current PCB lock
            let mut task_inner = task.acquire_inner_lock();
            // Change status to Ready
            task_inner.task_status = TaskStatus::Ready;
            if let Some(trap_info) = &task_inner.user_trap_info {
                trap_info.disable_user_ext_int();
            }
            task_inner.total_cpu_cycle_count += cycle::read() - task_inner.last_cpu_cycle;
            drop(task_inner);
            // ---- release current PCB lock

            // push back to ready queue.
            add_task(task);
        }
    }

    pub fn run(&self) {
        loop {
            if let Some(task) = fetch_task() {
                // unsafe { riscv::asm::sfence_vma_all() }
                self.run_next(task);
                // __switch inside run_next
                // debug!("idle");
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

#[allow(unused)]
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

pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let idle_task_cx_ptr = PROCESSORS[hart_id()].get_idle_task_cx_ptr();
    trace!(
        "[schedule] switched task cx ptr: {:x?}, task cx: {:x?}",
        switched_task_cx_ptr,
        unsafe { &*switched_task_cx_ptr }
    );
    trace!(
        "[schedule] idle task cx ptr: {:x?}, task cx: {:x?}",
        idle_task_cx_ptr,
        unsafe { &*idle_task_cx_ptr }
    );
    unsafe {
        __switch2(switched_task_cx_ptr, idle_task_cx_ptr);
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
