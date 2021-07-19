use rv_plic::PLIC;

use crate::uart;

#[cfg(feature = "board_qemu")]
pub const PLIC_BASE: usize = 0xc00_0000;
#[cfg(feature = "board_qemu")]
pub const PLIC_PRIORITY_BIT: usize = 3;

pub type plic = PLIC<PLIC_BASE, PLIC_PRIORITY_BIT>;

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
    // Assume only hart 0 S now
    if let Some(irq) = plic::claim(1) {
        match irq {
            10 => {
                uart::handle_interrupt();
                debug!("PLIC: UART irq");
            }
            _ => {
                debug!("PLIC: Not handle yet");
            }
        }
        plic::complete(1, irq)
    }
}
