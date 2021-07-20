use rv_plic::PLIC;

use crate::task::find_task;
use crate::trap::{UserTrapRecord, USER_EXT_INT_MAP};
use crate::uart;

#[cfg(feature = "board_qemu")]
pub const PLIC_BASE: usize = 0xc00_0000;
#[cfg(feature = "board_qemu")]
pub const PLIC_PRIORITY_BIT: usize = 3;

pub type Plic = PLIC<PLIC_BASE, PLIC_PRIORITY_BIT>;

pub fn context(hartid: usize, mode: char) -> usize {
    const MODE_PER_HART: usize = 3;
    hartid * MODE_PER_HART
        + match mode {
            'M' => 0,
            'S' => 1,
            'U' => 2,
            _ => panic!("Wrong Mode"),
        }
}

pub fn handle_external_interrupt() {
    if let Some(irq) = Plic::claim(1) {
        let mut can_user_handle = false;
        if let Some(pid) = USER_EXT_INT_MAP.lock().get(&irq) {
            if let Some(tcb) = find_task(*pid) {
                let mut inner = tcb.acquire_inner_lock();
                if inner.is_user_trap_enabled() {
                    if let Some(trap_info) = &mut inner.user_trap_info {
                        unsafe {
                            trap_info.push_trap_record(UserTrapRecord {
                                cause: 8,
                                message: irq as usize,
                            })
                        }
                        can_user_handle = true;
                    }
                }
            }
        }
        if !can_user_handle {
            match irq {
                10 => {
                    uart::handle_interrupt();
                    debug!("PLIC: UART irq");
                }
                _ => {
                    debug!("PLIC: Not handle yet");
                }
            }
        }
        Plic::complete(1, irq)
    }
}
