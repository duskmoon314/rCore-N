use alloc::{collections::BTreeSet, sync::Arc};
use lazy_static::*;
use spin::Mutex;

use super::{manager::TaskManager, processor::current_tasks, task::TaskControlBlock};

pub struct TaskPool {
    pub scheduler: TaskManager,
    pub sleeping_tasks: BTreeSet<Arc<TaskControlBlock>>,
}

lazy_static! {
    pub static ref TASK_POOL: Mutex<TaskPool> = Mutex::new(TaskPool::new());
}

impl TaskPool {
    pub fn new() -> Self {
        Self {
            scheduler: TaskManager::new(),
            sleeping_tasks: BTreeSet::new(),
        }
    }

    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.scheduler.add(task);
    }

    pub fn remove(&mut self, task: Arc<TaskControlBlock>) {
        self.scheduler.remove(&task);
    }

    pub fn wake(&mut self, task: Arc<TaskControlBlock>) {
        self.sleeping_tasks.remove(&task);
        self.scheduler.add(task);
    }

    pub fn sleep(&mut self, task: Arc<TaskControlBlock>) {
        self.scheduler.remove(&task);
        self.sleeping_tasks.insert(task);
    }

    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.scheduler.fetch()
    }

    pub fn find(&self, pid: usize) -> Option<Arc<TaskControlBlock>> {
        self.scheduler.find(pid)
    }
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_POOL.lock().add(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_POOL.lock().fetch()
}

pub fn find_task(pid: usize) -> Option<Arc<TaskControlBlock>> {
    for current in current_tasks() {
        let current = current.unwrap();
        if current.pid == pid {
            return Some(current);
        }
    }
    TASK_POOL.lock().find(pid)
}
