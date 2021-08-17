use super::TaskContext;
use super::{pid_alloc, KernelStack, PidHandle};
use crate::fs::{File, MailBox, Serial, Socket, Stdin, Stdout};
use crate::mm::{translate_writable_va, MemorySet, PhysAddr, PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::task::pid::add_task_2_map;
use crate::trap::{trap_handler, TrapContext, UserTrapInfo};
use crate::{
    config::{PAGE_SIZE, TRAP_CONTEXT, USER_TRAP_BUFFER},
    loader::get_app_data_by_name,
    mm::translated_str,
};
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};
use spin::{Mutex, MutexGuard};

#[derive(Debug)]
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
    pub user_trap_info: Option<UserTrapInfo>,
    pub task_status: TaskStatus,
    pub priority: isize,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
    pub mail_box: Arc<MailBox>,
}

impl Debug for TaskControlBlockInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "TCBInner: {{\r\n  trap cx addr: {:?} , base_size: {:#x} \r\n  task_cx_ptr: {:#x} , token: {:#x} \r\n}}",
            PhysAddr::from(self.trap_cx_ppn), self.base_size, self.task_cx_ptr, self.memory_set.token()
        ))
    }
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

    pub fn is_user_trap_enabled(&self) -> bool {
        self.get_trap_cx().sstatus.uie()
    }

    pub fn init_user_trap(&mut self) -> Result<isize, isize> {
        use riscv::register::sstatus;
        if self.user_trap_info.is_none() {
            // R | W
            if self.mmap(USER_TRAP_BUFFER, PAGE_SIZE, 0b11).is_ok() {
                let phys_addr =
                    translate_writable_va(self.get_user_token(), USER_TRAP_BUFFER).unwrap();
                self.user_trap_info = Some(UserTrapInfo {
                    user_trap_buffer_ppn: PhysPageNum::from(PhysAddr::from(phys_addr)),
                    user_trap_record_num: 0,
                    devices: Vec::new(),
                });
                unsafe {
                    sstatus::set_uie();
                }
                return Ok(USER_TRAP_BUFFER as isize);
            } else {
                warn!("[init user trap] mmap failed!");
            }
        } else {
            warn!("[init user trap] self user trap info is not None!");
        }
        Err(-1)
    }

    pub fn restore_user_trap_info(&mut self) {
        use riscv::register::{uip, uscratch};
        if self.is_user_trap_enabled() {
            if let Some(trap_info) = &mut self.user_trap_info {
                if trap_info.user_trap_record_num > 0 {
                    trace!("restore {} user trap", trap_info.user_trap_record_num);
                    uscratch::write(trap_info.user_trap_record_num as usize);
                    trap_info.user_trap_record_num = 0;
                    unsafe {
                        uip::set_usoft();
                    }
                }
            }
        }
    }
}

impl TaskControlBlock {
    pub fn acquire_inner_lock(&self) -> MutexGuard<TaskControlBlockInner> {
        self.inner.lock()
    }
    pub fn new(elf_data: &[u8]) -> Arc<TaskControlBlock> {
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
        trace!("new task cx ptr: {:#x?}", task_cx_ptr as usize);
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
                    // 3 -> serial 3
                    Some(Arc::new(Serial::<2>)),
                    // 4 -> serial 4
                    Some(Arc::new(Serial::<3>)),
                ],
                mail_box: Arc::new(MailBox::new()),
            }),
        });
        add_task_2_map(task_control_block.getpid(), task_control_block.clone());
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
        inner.user_trap_info = None;
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
        debug!("forked task cx ptr: {:#x?}", task_cx_ptr as usize);
        // copy fd table
        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent_inner.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        let mut user_trap_info: Option<UserTrapInfo> = None;
        if let Some(mut trap_info) = parent_inner.user_trap_info.clone() {
            debug!("[fork] copy parent trap info");
            trap_info.user_trap_buffer_ppn = memory_set
                .translate(VirtAddr::from(USER_TRAP_BUFFER).into())
                .unwrap()
                .ppn();
            user_trap_info = Some(trap_info);
        }
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            kernel_stack,
            inner: Mutex::new(TaskControlBlockInner {
                trap_cx_ppn,
                base_size: parent_inner.base_size,
                task_cx_ptr: task_cx_ptr as usize,
                user_trap_info,
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
        add_task_2_map(task_control_block.getpid(), task_control_block.clone());
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
        debug!("SPAWN exec {:?}", &f);

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
            trace!("spawned task cx ptr: {:#x?}", task_cx_ptr as usize);

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
                        // 3 -> serial 2
                        Some(Arc::new(Serial::<2>)),
                        // 4 -> serial 3
                        Some(Arc::new(Serial::<3>)),
                    ],
                    mail_box: Arc::new(MailBox::new()),
                }),
            });
            add_task_2_map(task_control_block.getpid(), task_control_block.clone());
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

impl PartialEq for TaskControlBlock {
    fn eq(&self, other: &Self) -> bool {
        self.pid == other.pid
    }
}

impl Eq for TaskControlBlock {}

impl PartialOrd for TaskControlBlock {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TaskControlBlock {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.pid.cmp(&other.pid)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}
