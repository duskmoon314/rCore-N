use core::mem::size_of;

use crate::config::CPU_NUM;
use crate::loader::get_app_data_by_name;
use crate::mm;
use crate::plic::{get_context, Plic};
use crate::task::{
    add_task, current_task, current_user_token, exit_current_and_run_next, hart_id, mmap, munmap,
    set_current_priority, suspend_current_and_run_next, WAIT_LOCK,
};
use crate::trap::{push_trap_record, UserTrapRecord};

use crate::timer::get_time;
use alloc::sync::Arc;
use alloc::vec::Vec;

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    trace!("sys_yield");
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
    match mm::translate_writable_va(token, time) {
        Err(_) => return -1,
        Ok(pa) => pas.push(pa as *mut usize),
    }
    match mm::translate_writable_va(token, time + size_of::<usize>()) {
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
    debug!("new_task {:?} via fork", new_pid);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = mm::translated_str(token, path);
    debug!("EXEC {}", &path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        warn!("exec failed!");
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!("sys_waitpid {}", pid);
    let task = current_task().unwrap();
    // find a child process

    let _ = WAIT_LOCK.lock();
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
        *mm::translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
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

pub fn sys_init_user_trap() -> isize {
    debug!("init user trap!");
    match current_task()
        .unwrap()
        .acquire_inner_lock()
        .init_user_trap()
    {
        Ok(addr) => {
            debug!("init ok, addr:{:?}", addr);
            addr
        }
        Err(errno) => errno,
    }
}

pub fn sys_send_msg(pid: usize, msg: usize) -> isize {
    if push_trap_record(
        pid,
        UserTrapRecord {
            cause: pid << 4,
            message: msg,
        },
    )
    .is_ok()
    {
        0
    } else {
        -1
    }
}

pub fn sys_set_timer(time_us: usize) -> isize {
    let pid = current_task().unwrap().pid.0;
    use crate::config::CLOCK_FREQ;
    use crate::timer::{set_virtual_timer, USEC_PER_SEC};
    let time = time_us * CLOCK_FREQ / USEC_PER_SEC;
    set_virtual_timer(time, pid);
    0
}

pub fn sys_claim_ext_int(device_id: usize) -> isize {
    let device_id = device_id as u16;
    let current_task = current_task().unwrap();
    let mut inner = current_task.acquire_inner_lock();
    if !inner.is_user_trap_enabled() {
        return -1;
    }
    use crate::plic;
    use crate::trap::USER_EXT_INT_MAP;
    let user_trap_info = &mut inner.user_trap_info;
    match user_trap_info {
        Some(info) => {
            let mut map = USER_EXT_INT_MAP.lock();
            if !map.contains_key(&device_id) {
                let pid = current_task.getpid();
                debug!(
                    "[syscall claim] mapping device {} to pid {}",
                    device_id, pid
                );
                map.insert(device_id, pid);
                info.devices.push((device_id, false));
                for hart_id in 0..CPU_NUM {
                    let claim_addr = Plic::context_address(plic::get_context(hart_id, 'U'));
                    if inner
                        .memory_set
                        .mmio_map(claim_addr, claim_addr + crate::config::PAGE_SIZE, 0b11)
                        .is_err()
                    {
                        warn!("[syscall claim] map plic claim reg failed!");
                        return -6;
                    }
                }
            }
            use crate::uart;
            match device_id {
                #[cfg(feature = "board_qemu")]
                13 | 14 | 15 => {
                    let base_address = uart::get_base_addr_from_irq(device_id);
                    match inner.memory_set.mmio_map(
                        base_address,
                        base_address + uart::SERIAL_ADDRESS_STRIDE,
                        0x3,
                    ) {
                        Ok(_) => base_address as isize,
                        Err(_) => -2,
                    }
                }
                #[cfg(feature = "board_lrv")]
                5 | 6 | 7 => {
                    let base_address = uart::get_base_addr_from_irq(device_id);
                    match inner.memory_set.mmio_map(
                        base_address,
                        base_address + uart::SERIAL_ADDRESS_STRIDE,
                        0x3,
                    ) {
                        Ok(_) => base_address as isize,
                        Err(_) => -2,
                    }
                }
                _ => -4,
            }
        }
        None => {
            warn!("[syscall claim] user trap info is None!");
            -5
        }
    }
}

pub fn sys_set_ext_int_enable(device_id: usize, enable: usize) -> isize {
    debug!("[SET EXT INT] dev: {}, enable: {}", device_id, enable);
    let device_id = device_id as u16;
    let is_enable = enable > 0;
    let current_task = current_task().unwrap();
    let mut inner = current_task.acquire_inner_lock();
    if !inner.is_user_trap_enabled() {
        return -1;
    }
    use crate::trap::USER_EXT_INT_MAP;
    let user_trap_info = &mut inner.user_trap_info;
    match user_trap_info {
        Some(info) => {
            if let Some(pid) = USER_EXT_INT_MAP.lock().get(&device_id) {
                if *pid == current_task.getpid() {
                    for (dev_id, en) in &mut info.devices {
                        if *dev_id == device_id {
                            *en = is_enable;
                            if is_enable {
                                Plic::enable(get_context(hart_id(), 'U'), device_id);
                                for hart in 0..CPU_NUM {
                                    Plic::disable(get_context(hart, 'S'), device_id);
                                }
                            } else {
                                Plic::disable(get_context(hart_id(), 'U'), device_id);
                            }
                        }
                    }

                    return 0;
                } else {
                    warn!(
                        "[sys set ext] device {} not held by pid {}!",
                        device_id,
                        current_task.getpid()
                    );
                    return -1;
                }
            } else {
                warn!("[sys set ext] device not claimed!");
                return -2;
            }
        }
        None => {
            warn!("[syscall claim] user trap info is None!");
            -5
        }
    }
}
