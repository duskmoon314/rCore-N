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

impl UserTrapInfo {
    // caller of this function should check wheter user interrupt is enabled
    pub unsafe fn push_trap_record(&mut self, trap_record: UserTrapRecord) {
        debug!(
            "pushing trap record, cause: {:?}, message: {:?}",
            trap_record.cause, trap_record.message
        );
        if self.user_trap_record_num < MAX_USER_TRAP_NUM {
            let head_ptr: *mut UserTrapRecord =
                self.user_trap_buffer_ppn.get_mut::<UserTrapRecord>();
            let tail_ptr = head_ptr.offset(self.user_trap_record_num as isize);
            tail_ptr.write(trap_record);
            self.user_trap_record_num += 1;
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
    pub static ref USER_TIMER_INT_MAP: Arc<Mutex<BTreeMap<usize, usize>>> =
        Arc::new(Mutex::new(BTreeMap::new()));
}
