use core::mem::size_of;

use crate::loader::get_app_data_by_name;
use crate::mm::{translate_writable_va, translated_refmut, translated_str};
use crate::task::{
    add_task, current_task, current_user_token, exit_current_and_run_next, mmap, munmap,
    set_current_priority, suspend_current_and_run_next,
};

use crate::timer::get_time;
use alloc::sync::Arc;
use alloc::vec::Vec;

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_set_priority(prio: isize) -> isize {
    match set_current_priority(prio) {
        Ok(prio) => prio,
        Err(err) => err,
    }
}

pub fn sys_get_time(time: usize, tz: usize) -> isize {
    let token = current_user_token();
    let mut pas: Vec<*mut usize> = Vec::new();
    match translate_writable_va(token, time) {
        Err(_) => return -1,
        Ok(pa) => pas.push(pa as *mut usize),
    }
    match translate_writable_va(token, time + size_of::<usize>()) {
        Err(_) => return -1,
        Ok(pa) => pas.push(pa as *mut usize),
    }
    get_time(pas, tz)
}

pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    mmap(start, len, port).unwrap_or(-1)
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    munmap(start, len).unwrap_or(-1)
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    debug!("Fork start");
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.acquire_inner_lock().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    debug!("new_task {:?}", new_pid);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    debug!("EXEC {}", &path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    // find a child process

    // ---- hold current PCB lock
    let mut inner = task.acquire_inner_lock();
    if inner
        .children
        .iter()
        .find(|p| pid == -1 || pid as usize == p.getpid())
        .is_none()
    {
        return -1;
        // ---- release current PCB lock
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily hold child PCB lock
        p.acquire_inner_lock().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB lock
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after removing from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily hold child lock
        let exit_code = child.acquire_inner_lock().exit_code;
        // ++++ release child PCB lock
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB lock automatically
}

pub fn sys_spawn(file: *const u8) -> isize {
    debug!("SPAWN start");
    let current_task = current_task().unwrap();
    match current_task.spawn(file) {
        Ok(new_task) => {
            let new_pid = new_task.pid.0;
            let trap_cx = new_task.acquire_inner_lock().get_trap_cx();
            trap_cx.x[10] = 0;
            add_task(new_task);
            debug!("new_task via spawn {:?}", new_pid);
            new_pid as isize
        }
        Err(_) => -1,
    }
}
