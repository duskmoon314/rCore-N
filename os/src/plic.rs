use rv_plic::{Priority, PLIC};

use crate::trap::{push_trap_record, UserTrapRecord, USER_EXT_INT_MAP};
use crate::uart;

#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
pub const PLIC_BASE: usize = 0xc00_0000;
#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
pub const PLIC_PRIORITY_BIT: usize = 3;

pub type Plic = PLIC<{ PLIC_BASE }, { PLIC_PRIORITY_BIT }>;

pub fn get_context(hart_id: usize, mode: char) -> usize {
    const MODE_PER_HART: usize = 3;
    hart_id * MODE_PER_HART
        + match mode {
            'M' => 0,
            'S' => 1,
            'U' => 2,
            _ => panic!("Wrong Mode"),
        }
}

#[cfg(feature = "board_qemu")]
pub fn init() {
    Plic::set_priority(9, Priority::lowest());
    Plic::set_priority(12, Priority::lowest());
}

#[cfg(feature = "board_lrv")]
pub fn init() {
    Plic::set_priority(4, Priority::lowest());
    Plic::set_priority(5, Priority::lowest());
}

#[cfg(feature = "board_qemu")]
pub fn init_hart(hart_id: usize) {
    let context = get_context(hart_id, 'S');
    Plic::enable(context, 12);
    Plic::set_threshold(context, Priority::any());
}

#[cfg(feature = "board_lrv")]
pub fn init_hart(hart_id: usize) {
    let context = get_context(hart_id, 'S');
    Plic::enable(context, 4);
    Plic::disable(context, 5);
    Plic::set_threshold(context, Priority::any());
}

pub fn handle_external_interrupt(hart_id: usize) {
    let context = get_context(hart_id, 'S');
    while let Some(irq) = Plic::claim(context) {
        let mut can_user_handle = false;
        if let Some(pid) = USER_EXT_INT_MAP.lock().get(&irq) {
            debug!("[PLIC] irq {:?} mapped to pid {:?}", irq, pid);
            if push_trap_record(
                *pid,
                UserTrapRecord {
                    // User External Interrupt
                    cause: 8,
                    message: irq as usize,
                },
            )
            .is_ok()
            {
                can_user_handle = true;
            }
            Plic::complete(context, irq);
            Plic::disable(context, irq);
        }
        if !can_user_handle {
            match irq {
                #[cfg(feature = "board_qemu")]
                12 => {
                    uart::handle_interrupt();
                    trace!("[PLIC] irq {:?} handled by kenel, UART2", irq);
                }
                #[cfg(feature = "board_lrv")]
                4 => {
                    uart::handle_interrupt();
                    trace!("[PLIC] kenel handling uart");
                }
                _ => {
                    debug!("[PLIC]: irq {:?} not supported!", irq);
                }
            }
        }
        Plic::complete(context, irq);
    }
}
