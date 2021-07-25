use riscv::register::{ucause, uepc, uip, ustatus::Ustatus, utval};

#[repr(C)]
pub struct UserTrapContext {
    pub x: [usize; 32],
    pub ustatus: Ustatus,
    pub uepc: usize,
    pub utvec: usize,
    pub uie: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UserTrapRecord {
    pub cause: usize,
    pub message: usize,
}

global_asm!(include_str!("trap.asm"));

#[linkage = "weak"]
#[no_mangle]
pub fn user_trap_handler(cx: &mut UserTrapContext) -> &mut UserTrapContext {
    let ucause = ucause::read();
    let utval = utval::read();
    match ucause.cause() {
        ucause::Trap::Interrupt(ucause::Interrupt::UserSoft) => {
            println!("[user mode trap] user soft");
            unsafe {
                uip::clear_usoft();
            }
        }
        _ => {
            println!(
                "Unsupported trap {:?}, utval = {:#x}, uepc = {:#x}!",
                ucause.cause(),
                utval,
                uepc::read()
            );
        }
    }
    cx
}
