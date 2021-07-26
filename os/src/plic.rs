use rv_plic::PLIC;

use crate::task::find_task;
use crate::trap::{push_trap_record, UserTrapRecord, USER_EXT_INT_MAP};
use crate::uart;

#[cfg(feature = "board_qemu")]
pub const PLIC_BASE: usize = 0xc00_0000;
#[cfg(feature = "board_qemu")]
pub const PLIC_PRIORITY_BIT: usize = 3;

pub type Plic = PLIC<PLIC_BASE, PLIC_PRIORITY_BIT>;

pub fn get_context(hartid: usize, mode: char) -> usize {
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
    if let Some(irq) = Plic::claim(get_context(0, 'S')) {
        debug!("[PLIC] IRQ: {:?}", irq);
        let mut can_user_handle = false;
        if let Some(pid) = USER_EXT_INT_MAP.lock().get(&irq) {
            debug!("[PLIC] irq mapped to pid {:?}", pid);
            if let Ok(_) = push_trap_record(
                *pid,
                UserTrapRecord {
                    cause: 8,
                    message: irq as usize,
                },
            ) {
                can_user_handle = true;
            }
        }
        if !can_user_handle {
            match irq {
                10 => {
                    uart::handle_interrupt();
                    debug!("[PLIC] kenel handling uart");
                }
                _ => {
                    warn!("[PLIC]: Not handle yet");
                }
            }
        }
        Plic::complete(get_context(0, 'S'), irq)
    }
}
