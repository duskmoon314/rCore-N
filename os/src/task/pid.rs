use crate::config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE};
use crate::mm::{MapPermission, VirtAddr, KERNEL_SPACE};
use alloc::collections::BTreeMap;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use lazy_static::*;
use spin::Mutex;

use super::task::TaskControlBlock;

struct PidAllocator {
    current: usize,
    recycled: Vec<usize>,
    task_table: BTreeMap<usize, Weak<TaskControlBlock>>,
}

impl PidAllocator {
    pub fn new() -> Self {
        PidAllocator {
            current: 0,
            recycled: Vec::new(),
            task_table: BTreeMap::new(),
        }
    }
    pub fn alloc(&mut self) -> PidHandle {
        let pid = match self.recycled.pop() {
            Some(pid) => pid,
            None => {
                self.current += 1;
                self.current - 1
            }
        };
        PidHandle(pid)
    }
    pub fn add_task(&mut self, pid: usize, task: Arc<TaskControlBlock>) -> Result<(), usize> {
        match self.task_table.try_insert(pid, Arc::downgrade(&task)) {
            Ok(_) => Ok(()),
            Err(err) => Err(*err.entry.key()),
        }
    }
    pub fn dealloc(&mut self, pid: usize) {
        assert!(pid < self.current);
        // assert!(
        //     self.recycled.iter().find(|ppid| **ppid == pid).is_none(),
        //     "pid {} has been deallocated!",
        //     pid
        // );
        assert!(
            self.task_table.remove(&pid).is_some(),
            "pid {} has been deallocated!",
            pid
        );
        // self.recycled.push(pid);
    }
}

lazy_static! {
    static ref PID_ALLOCATOR: Mutex<PidAllocator> = Mutex::new(PidAllocator::new());
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PidHandle(pub usize);

impl Drop for PidHandle {
    fn drop(&mut self) {
        //println!("drop pid {}", self.0);
        PID_ALLOCATOR.lock().dealloc(self.0);
    }
}

impl PartialEq<usize> for PidHandle {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

pub fn pid_alloc() -> PidHandle {
    PID_ALLOCATOR.lock().alloc()
}

pub fn add_task_2_map(pid: usize, task: Arc<TaskControlBlock>) {
    PID_ALLOCATOR.lock().add_task(pid, task).unwrap();
}

pub fn find_task(pid: usize) -> Option<Arc<TaskControlBlock>> {
    PID_ALLOCATOR
        .lock()
        .task_table
        .get(&pid)
        .and_then(|weak| weak.upgrade())
        .and_then(|strong| {
            if strong.acquire_inner_lock().is_zombie() {
                None
            } else {
                Some(strong)
            }
        })
}

/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

#[derive(Debug)]
pub struct KernelStack {
    pid: usize,
}

impl KernelStack {
    pub fn new(pid_handle: &PidHandle) -> Self {
        let pid = pid_handle.0;
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(pid);
        KERNEL_SPACE.lock().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        KernelStack { pid: pid_handle.0 }
    }
    pub fn push_on_top<T>(&self, value: T) -> *mut T
    where
        T: Sized,
    {
        let kernel_stack_top = self.get_top();
        let ptr_mut = (kernel_stack_top - core::mem::size_of::<T>()) as *mut T;
        unsafe {
            *ptr_mut = value;
        }
        ptr_mut
    }
    pub fn get_top(&self) -> usize {
        let (_, kernel_stack_top) = kernel_stack_position(self.pid);
        kernel_stack_top
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let (kernel_stack_bottom, _) = kernel_stack_position(self.pid);
        let kernel_stack_bottom_va: VirtAddr = kernel_stack_bottom.into();
        KERNEL_SPACE
            .lock()
            .remove_area_with_start_vpn(kernel_stack_bottom_va.into());
    }
}
