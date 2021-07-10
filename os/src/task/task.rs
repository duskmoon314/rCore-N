use super::TaskContext;
use super::{pid_alloc, KernelStack, PidHandle};
use crate::fs::{File, MailBox, Socket, Stdin, Stdout};
use crate::mm::{MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::trap::{trap_handler, TrapContext};
use crate::{config::TRAP_CONTEXT, loader::get_app_data_by_name, mm::translated_str};
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use riscv::register::{uie, uip, utval};
use spin::{Mutex, MutexGuard};

const USER_TRAP_BUFFER_SIZE: usize = 20;

pub struct UserTrapQueue {
    inner: Mutex<UserTrapBuffer>,
}

pub struct UserTrapRecord {
    pub cause: usize,
    pub source: usize,
}

pub struct UserTrapBuffer {
    arr: [UserTrapRecord; USER_TRAP_BUFFER_SIZE],
    tail: usize,
}

pub struct UserTrapInfo {
    pub uip: usize,
    pub uie: usize,
    pub trap_queue: Arc<UserTrapQueue>,
}

pub unsafe fn restore_user_trap_info(user_trap_info: &Arc<UserTrapInfo>) {
    // ucause::write(user_trap_info.ucause);
    // utval::write(user_trap_info.utval);
    // uip::write(user_trap_info.uip);
    // uie::write(user_trap_info.uie);
}

pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    // mutable
    inner: Mutex<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    pub task_cx_ptr: usize,
    pub user_trap_info: Option<Arc<UserTrapInfo>>,
    pub task_status: TaskStatus,
    pub priority: isize,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
    pub mail_box: Arc<MailBox>,
}

impl TaskControlBlockInner {
    pub fn get_task_cx_ptr2(&self) -> *const usize {
        &self.task_cx_ptr as *const usize
    }
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }

    pub fn set_priority(&mut self, priority: isize) -> Result<isize, isize> {
        if priority < 2 {
            return Err(-1);
        }
        self.priority = priority;
        Ok(priority)
    }

    pub fn mmap(&mut self, start: usize, len: usize, port: usize) -> Result<isize, isize> {
        self.memory_set.mmap(start, len, port)
    }

    pub fn munmap(&mut self, start: usize, len: usize) -> Result<isize, isize> {
        self.memory_set.munmap(start, len)
    }

    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }

    pub fn is_mailbox_full(&self) -> bool {
        self.mail_box.is_full()
    }

    pub fn is_mailbox_empty(&self) -> bool {
        self.mail_box.is_empty()
    }
}

impl TaskControlBlock {
    pub fn acquire_inner_lock(&self) -> MutexGuard<TaskControlBlockInner> {
        self.inner.lock()
    }
    pub fn new(elf_data: &[u8]) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        // push a task context which goes to trap_return to the top of kernel stack
        let task_cx_ptr = kernel_stack.push_on_top(TaskContext::goto_trap_return());
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: Mutex::new(TaskControlBlockInner {
                trap_cx_ppn,
                base_size: user_sp,
                task_cx_ptr: task_cx_ptr as usize,
                user_trap_info: None,
                task_status: TaskStatus::Ready,
                memory_set,
                parent: None,
                children: Vec::new(),
                exit_code: 0,
                priority: 16,
                fd_table: vec![
                    // 0 -> stdin
                    Some(Arc::new(Stdin)),
                    // 1 -> stdout
                    Some(Arc::new(Stdout)),
                    // 2 -> stderr
                    Some(Arc::new(Stdout)),
                ],
                mail_box: Arc::new(MailBox::new()),
            }),
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.acquire_inner_lock().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }

    pub fn exec(&self, elf_data: &[u8]) {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        // **** hold current PCB lock
        let mut inner = self.acquire_inner_lock();
        // substitute memory_set
        inner.memory_set = memory_set;
        // update trap_cx ppn
        inner.trap_cx_ppn = trap_cx_ppn;
        // initialize trap_cx
        let trap_cx = inner.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize,
        );
        // **** release current PCB lock
    }
    pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {
        // ---- hold parent PCB lock
        let mut parent_inner = self.acquire_inner_lock();
        // copy user space(include trap context)
        let memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        // push a goto_trap_return task_cx on the top of kernel stack
        let task_cx_ptr = kernel_stack.push_on_top(TaskContext::goto_trap_return());
        // copy fd table
        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent_inner.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            kernel_stack,
            inner: Mutex::new(TaskControlBlockInner {
                trap_cx_ppn,
                base_size: parent_inner.base_size,
                task_cx_ptr: task_cx_ptr as usize,
                user_trap_info: None,
                task_status: TaskStatus::Ready,
                memory_set,
                parent: Some(Arc::downgrade(self)),
                children: Vec::new(),
                exit_code: 0,
                priority: 16,
                fd_table: new_fd_table,
                mail_box: Arc::new(MailBox::new()),
            }),
        });
        // add child
        parent_inner.children.push(task_control_block.clone());
        // modify kernel_sp in trap_cx
        // **** acquire child PCB lock
        let trap_cx = task_control_block.acquire_inner_lock().get_trap_cx();
        // **** release child PCB lock
        trap_cx.kernel_sp = kernel_stack_top;
        // return
        task_control_block
        // ---- release parent PCB lock
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }

    pub fn spawn(
        self: &Arc<TaskControlBlock>,
        file: *const u8,
    ) -> Result<Arc<TaskControlBlock>, isize> {
        let mut parent_inner = self.acquire_inner_lock();
        let parent_token = parent_inner.get_user_token();
        let f = translated_str(parent_token, file);
        debug!("SPAWN exec {}", &f);

        if let Some(elf_data) = get_app_data_by_name(f.as_str()) {
            let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
            let trap_cx_ppn = memory_set
                .translate(VirtAddr::from(TRAP_CONTEXT).into())
                .unwrap()
                .ppn();

            let pid_handle = pid_alloc();
            let kernel_stack = KernelStack::new(&pid_handle);
            let kernel_stack_top = kernel_stack.get_top();
            let task_cx_ptr = kernel_stack.push_on_top(TaskContext::goto_trap_return());

            let task_control_block = Arc::new(TaskControlBlock {
                pid: pid_handle,
                kernel_stack,
                inner: Mutex::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: user_sp,
                    task_cx_ptr: task_cx_ptr as usize,
                    user_trap_info: None,
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                    priority: 16,
                    fd_table: vec![
                        // 0 -> stdin
                        Some(Arc::new(Stdin)),
                        // 1 -> stdout
                        Some(Arc::new(Stdout)),
                        // 2 -> stderr
                        Some(Arc::new(Stdout)),
                    ],
                    mail_box: Arc::new(MailBox::new()),
                }),
            });

            parent_inner.children.push(task_control_block.clone());
            let trap_cx = task_control_block.acquire_inner_lock().get_trap_cx();
            *trap_cx = TrapContext::app_init_context(
                entry_point,
                user_sp,
                KERNEL_SPACE.lock().token(),
                kernel_stack_top,
                trap_handler as usize,
            );
            return Ok(task_control_block);
        }
        Err(-1)
    }

    pub fn create_socket(&self) -> Arc<Socket> {
        self.inner.lock().mail_box.create_socket()
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}
