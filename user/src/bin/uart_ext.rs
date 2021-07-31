#![no_std]
#![no_main]
#![feature(asm)]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::{string::String, sync::Arc};
use lazy_static::*;
use riscv::register::uie;
use spin::Mutex;
use uart::UART1_BASE_ADDRESS;
use uart8250::MmioUart8250;
use user_lib::{claim_ext_int, init_user_trap};

lazy_static! {
    pub static ref LINE: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    pub static ref UART1: Arc<Mutex<MmioUart8250>> =
        Arc::new(Mutex::new(MmioUart8250::new(UART1_BASE_ADDRESS)));
}

#[no_mangle]
pub fn main() -> i32 {
    println!("[uart ext] A user mode serial driver demo using external interrupt");
    let init_res = init_user_trap();
    let claim_res = claim_ext_int(uart::UART1_IRQN as usize);
    println!(
        "[uart ext] init result: 0x{:x?}, claim result: 0x{:x?}",
        init_res as usize, claim_res
    );
    UART1.lock().init(11_059_200, 115200);
    println2!("Hello from UART1!");
    unsafe {
        uie::set_uext();
        uie::set_usoft();
        uie::set_utimer();
    }
    loop {}
}

#[macro_export]
macro_rules! print2 {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::uart::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println2 {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::uart::print(format_args!(concat!($fmt, "\r\n") $(, $($arg)+)?));
    }
}

mod uart {
    // Based on https://github.com/sgmarz/osblog
    pub const UART1_BASE_ADDRESS: usize = 0x10000100;
    pub const UART1_IRQN: u16 = 9;

    const LF: u8 = 0x0au8;
    const CR: u8 = 0x0du8;
    const DL: u8 = 0x7fu8;
    const BS: u8 = 0x08u8;

    use crate::UART1;
    use core::fmt::{self, Write};

    pub fn print(args: fmt::Arguments) {
        UART1.lock().write_fmt(args).unwrap();
    }

    pub fn handle_input() {
        // If we get here, the UART better have something! If not, what happened??
        let uart1 = UART1.lock();
        if let Some(c) = uart1.read_byte() {
            // If you recognize this code, it used to be in the lib.rs under kmain(). That
            // was because we needed to poll for UART data. Now that we have interrupts,
            // here it goes!
            drop(uart1);
            let mut line = crate::LINE.lock();
            match c {
                LF | CR => {
                    println2!("");
                    if *line == "exit" {
                        user_lib::exit(0);
                    }
                    println2!("{}", line);
                    line.clear();
                }
                BS | DL => {
                    if !line.is_empty() {
                        print2!("{}", BS as char);
                        print2!(" ");
                        print2!("{}", BS as char);
                        line.pop();
                    }
                }
                _ => {
                    print2!("{}", c as char);
                    line.push(c as char);
                }
            }
        }
    }
}
mod user_trap {
    use riscv::register::{ucause, uepc, uip, uscratch, utval};
    use user_lib::{UserTrapContext, UserTrapRecord};

    pub const PAGE_SIZE: usize = 0x1000;
    pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
    pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
    pub const USER_TRAP_BUFFER: usize = TRAP_CONTEXT - PAGE_SIZE;

    use rv_plic::PLIC;
    pub const PLIC_BASE: usize = 0xc00_0000;
    pub const PLIC_PRIORITY_BIT: usize = 3;
    pub type Plic = PLIC<PLIC_BASE, PLIC_PRIORITY_BIT>;

    use crate::uart::{handle_input, UART1_IRQN};

    #[no_mangle]
    pub fn user_trap_handler(cx: &mut UserTrapContext) -> &mut UserTrapContext {
        let ucause = ucause::read();
        let utval = utval::read();
        match ucause.cause() {
            ucause::Trap::Interrupt(ucause::Interrupt::UserSoft) => {
                println!("[uart ext] user soft interrupt");
                let trap_record_num = uscratch::read();
                let mut head_ptr = USER_TRAP_BUFFER as *const UserTrapRecord;
                for _ in 0..trap_record_num {
                    unsafe {
                        let trap_record = *head_ptr;
                        let cause = trap_record.cause;
                        if cause & 0xF == 0 {
                            // "real" soft interrupt
                            let pid = cause >> 4;
                            let msg = trap_record.message;
                            if msg == 15 {
                                println2!("[uart ext] Received SIGTERM, exiting...");
                                user_lib::exit(15);
                            } else {
                                println2!(
                                    "[uart ext] Received message 0x{:x} from pid {}",
                                    msg,
                                    pid
                                );
                            }
                        } else if ucause::Interrupt::from(cause) == ucause::Interrupt::UserExternal
                        {
                            if trap_record.message == UART1_IRQN as usize {
                                handle_input();
                            }
                        }
                        head_ptr = head_ptr.offset(1);
                    }
                }
                unsafe {
                    uip::clear_usoft();
                }
            }
            ucause::Trap::Interrupt(ucause::Interrupt::UserExternal) => {
                if let Some(irq) = Plic::claim(2) {
                    println!("[uart ext] user external interrupt, irq: {}", irq);
                    if irq == UART1_IRQN {
                        handle_input();
                    }
                    Plic::complete(2, irq);
                }
                // println!("[user trap] user external finished");
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
}
