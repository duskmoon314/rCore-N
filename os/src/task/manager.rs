use core::borrow::Borrow;

use super::{current_task, TaskControlBlock};
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
use spin::Mutex;

pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }

    pub fn find(&self, pid: usize) -> Option<Arc<TaskControlBlock>> {
        for t in self.ready_queue.borrow() {
            if t.pid == pid {
                return Some(t.clone());
            }
        }
        None
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new());
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.lock().add(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.lock().fetch()
}

pub fn find_task(pid: usize) -> Option<Arc<TaskControlBlock>> {
    let current = current_task().unwrap();
    if current.pid == pid {
        return Some(current);
    }
    TASK_MANAGER.lock().find(pid)
}
