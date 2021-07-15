// Based on https://github.com/sgmarz/osblog

use crate::uart;

// https://osblog.stephenmarz.com/ch5.html

pub const PLIC_BASE: usize = 0xc00_0000;
pub const PLIC_PRIORITY_BASE: usize = 0x00;
pub const PLIC_PENDING_BASE: usize = 0x1000;
pub const PLIC_ENABLE_BASE: usize = 0x2080;
pub const PLIC_ENABLE_STRIDE: usize = 0x80;
pub const PLIC_CONTEXT_BASE: usize = 0x20_1000;
pub const PLIC_CONTEXT_STRIDE: usize = 0x1000;

/// Enable a given interrupt id
pub fn enable_interrupt(id: u32) {
    let enables = (PLIC_BASE + PLIC_ENABLE_BASE) as *mut u32;
    unsafe {
        enables.write_volatile(enables.read_volatile() | 1 << id);
    }
}

/// Set a given interrupt priority to the given priority
/// The priority must be [0..7]
pub fn set_interrupt_priority(id: usize, priority: u8) {
    let priorities = (PLIC_BASE + PLIC_PRIORITY_BASE) as *mut u32;
    unsafe {
        priorities.add(id).write_volatile(priority as u32 & 7);
    }
}

/// Set the global threshold. The threshold can be a value [0..7].
/// The PLIC will mask any interrupts at or below the given threshold.
/// This means that a threshold of 7 will mask ALL interrupts and
/// a threshold of 0 will allow ALL interrupts.
pub fn set_interrupt_threshold(threshold: u8) {
    let thresholds = (PLIC_BASE + PLIC_CONTEXT_BASE) as *mut u32;
    unsafe {
        thresholds.write_volatile((threshold & 7) as u32);
    }
}

/// See if a given interrupt id is pending.
pub fn is_interrupt_pending(id: u32) -> bool {
    let pend = (PLIC_BASE + PLIC_PENDING_BASE) as *const u32;
    let actual_id = 1 << id;
    let pend_ids;
    unsafe {
        pend_ids = pend.read_volatile();
    }
    actual_id & pend_ids != 0
}

/// Get the next available interrupt. This is the "claim" process.
/// The plic will automatically sort by priority and hand us the
/// ID of the interrupt. For example, if the UART is interrupting
/// and it's next, we will get the value 10.
pub fn claim_interrupt() -> Option<u32> {
    let claim_reg = (PLIC_BASE + PLIC_CONTEXT_BASE + 4) as *const u32;
    let claim_no;
    // The claim register is filled with the highest-priority, enabled interrupt.
    unsafe {
        claim_no = claim_reg.read_volatile();
    }
    if claim_no == 0 {
        // The interrupt 0 is hardwired to 0, which tells us that there is no
        // interrupt to claim, hence we return None.
        None
    } else {
        // If we get here, we've gotten a non-0 interrupt.
        Some(claim_no)
    }
}

/// Complete a pending interrupt by id. The id should come
/// from the next() function above.
pub fn complete_interrupt(id: u32) {
    let complete_reg = (PLIC_BASE + PLIC_CONTEXT_BASE + 4) as *mut u32;
    unsafe {
        // We actually write a u32 into the entire complete_register.
        // This is the same register as the claim register, but it can
        // differentiate based on whether we're reading or writing.
        complete_reg.write_volatile(id);
    }
}

pub fn handle_external_interrupt() {
    if let Some(irq) = claim_interrupt() {
        match irq {
            10 => {
                uart::handle_interrupt();
                debug!("PLIC: UART irq");
            }
            _ => {
                debug!("PLIC: Not handle yet");
            }
        }
        complete_interrupt(irq);
    }
}
