const MAX_USER_TRAP_NUM: usize = 128;

use crate::mm::PhysPageNum;
use crate::plic::Plic;
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use lazy_static::*;
use spin::Mutex;

#[derive(Clone)]
pub struct UserTrapInfo {
    pub user_trap_buffer_ppn: PhysPageNum,
    pub user_trap_record_num: usize,
    pub devices: Vec<u16>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UserTrapRecord {
    pub cause: usize,
    pub message: usize,
}

pub enum UserTrapError {
    TaskNotFound,
    TrapDisabled,
    TrapUninitialized,
    TrapBufferFull,
}

impl UserTrapInfo {
    // caller of this function should check wheter user interrupt is enabled
    pub unsafe fn push_trap_record(
        &mut self,
        trap_record: UserTrapRecord,
    ) -> Result<usize, UserTrapError> {
        if self.user_trap_record_num < MAX_USER_TRAP_NUM {
            let head_ptr: *mut UserTrapRecord =
                self.user_trap_buffer_ppn.get_mut::<UserTrapRecord>();
            let tail_ptr = head_ptr.offset(self.user_trap_record_num as isize);
            tail_ptr.write(trap_record);
            self.user_trap_record_num += 1;
            Ok(self.user_trap_record_num)
        } else {
            Err(UserTrapError::TrapBufferFull)
        }
    }

    pub fn enable_user_ext_int(&self) {
        for device_id in &self.devices {
            Plic::disable(1, *device_id);
            Plic::enable(2, *device_id);
        }
    }

    pub fn disable_user_ext_int(&self) {
        for device_id in &self.devices {
            Plic::enable(1, *device_id);
            Plic::disable(2, *device_id);
        }
    }

    pub fn remove_user_ext_int_map(&self) {
        let mut int_map = USER_EXT_INT_MAP.lock();
        for device_id in &self.devices {
            int_map.remove(device_id);
        }
    }
}

lazy_static! {
    pub static ref USER_EXT_INT_MAP: Arc<Mutex<BTreeMap<u16, usize>>> =
        Arc::new(Mutex::new(BTreeMap::new()));
}

pub fn push_trap_record(pid: usize, trap_record: UserTrapRecord) -> Result<usize, UserTrapError> {
    debug!(
        "[push trap record] pid: {}, cause: {}, message: {}",
        pid, trap_record.cause, trap_record.message
    );
    if let Some(tcb) = crate::task::find_task(pid) {
        let mut tcb_inner = tcb.acquire_inner_lock();
        if !tcb_inner.is_user_trap_enabled() {
            warn!("[push trap record] User trap disabled!");
            return Err(UserTrapError::TrapDisabled);
        }
        if let Some(trap_info) = &mut tcb_inner.user_trap_info {
            unsafe { trap_info.push_trap_record(trap_record) }
        } else {
            warn!("[push trap record] User trap uninitialized!");
            Err(UserTrapError::TrapUninitialized)
        }
    } else {
        warn!("[push trap record] Task Not Found!");
        Err(UserTrapError::TaskNotFound)
    }
}
