#![no_std]
#![no_main]
#![feature(asm)]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::string::String;
use alloc::sync::Arc;
use lazy_static::*;
use riscv::register::uie;
use spin::Mutex;
use user_lib::{claim_ext_int, init_user_trap, set_ext_int_enable, user_uart::*, yield_};

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;
#[cfg(feature = "board_qemu")]
const UART_IRQN: u16 = 13;
#[cfg(feature = "board_lrv")]
const UART_IRQN: u16 = 5;

#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
lazy_static! {
    pub static ref SERIAL: Arc<Mutex<BufferedSerial>> = Arc::new(Mutex::new(BufferedSerial::new(
        get_base_addr_from_irq(UART_IRQN)
    )));
}

#[no_mangle]
pub fn main() -> i32 {
    println!("[uart ext] A user mode serial driver demo using UEI");
    let init_res = init_user_trap();
    let claim_res = claim_ext_int(UART_IRQN as usize);
    SERIAL.lock().hardware_init(115200);
    let en_res = set_ext_int_enable(UART_IRQN as usize, 1);
    println!(
        "[uart ext] init result: {:#x}, claim result: {:#x}, enable res: {:#x}",
        init_res as usize, claim_res, en_res
    );
    let mut line = String::new();
    user_println!("Hello from user UART!");
    loop {
        unsafe {
            uie::clear_uext();
            uie::clear_usoft();
            uie::clear_utimer();
        }
        loop {
            let c = user_console::stdio_getchar();
            if c == 0 {
                break;
            }
            match c {
                LF | CR => {
                    user_println!("");
                    if line == "exit" {
                        user_lib::exit(0);
                    }
                    user_println!("{}", line);
                    line.clear();
                }
                BS | DL => {
                    if !line.is_empty() {
                        user_print!("{}", BS as char);
                        user_print!(" ");
                        user_print!("{}", BS as char);
                        line.pop();
                    }
                }
                _ => {
                    user_print!("{}", c as char);
                    line.push(c as char);
                }
            }
        }
        unsafe {
            uie::set_uext();
            uie::set_usoft();
            uie::set_utimer();
        }
        // for _ in 0..1000 {}
        yield_();
    }
}

#[macro_export]
macro_rules! user_print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::user_console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! user_println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::user_console::print(format_args!(concat!($fmt, "\r\n") $(, $($arg)+)?));
    }
}

mod user_console {
    // Based on https://github.com/sgmarz/osblog
    use core::fmt::{self, Write};

    #[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
    pub fn stdio_putchar(c: u8) {
        use embedded_hal::serial::Write;
        let _ = crate::SERIAL.lock().try_write(c);
    }

    #[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
    pub fn stdio_getchar() -> u8 {
        use embedded_hal::serial::Read;
        crate::SERIAL.lock().try_read().unwrap_or(0)
    }
    struct UserStdout;

    impl Write for UserStdout {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            for c in s.chars() {
                stdio_putchar(c as u8);
            }
            Ok(())
        }
    }

    #[allow(dead_code)]
    pub fn print(args: fmt::Arguments) {
        UserStdout.write_fmt(args).unwrap();
    }
}

mod user_trap {
    #[no_mangle]
    pub fn soft_intr_handler(pid: usize, msg: usize) {
        if msg == 15 {
            println!("[uart ext] Received SIGTERM, exiting...");
            user_lib::exit(15);
        } else {
            user_println!("[uart ext] Received message 0x{:x} from pid {}", msg, pid);
        }
    }

    #[no_mangle]
    pub fn ext_intr_handler(irq: u16, is_from_kernel: bool) {
        if is_from_kernel {
            println!("[uart ext] Received UEI from kernel, irq: {}", irq);
        } else {
            println!("[uart ext] user external interrupt, irq: {}", irq);
        }
        if irq == crate::UART_IRQN {
            crate::SERIAL.lock().interrupt_handler();
        }
    }
}
