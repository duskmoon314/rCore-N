use crate::console_blog::push_stdin;
use core::fmt::{self, Write};

use alloc::sync::Arc;
use lazy_static::*;
use spin::Mutex;
use uart8250::{InterruptType, MmioUart8250};

lazy_static! {
    pub static ref UART: Arc<Mutex<MmioUart8250<'static>>> =
        Arc::new(Mutex::new(MmioUart8250::new(0x1000_0000)));
}

pub fn init() {
    let uart = UART.lock();
    uart.init(11_059_200, 115200);
    uart.write_fcr(0b1100_0001);
}

pub fn print(args: fmt::Arguments) {
    UART.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::uart::print(format_args!($fmt $(, $($arg)+)?));
    };
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::uart::print(format_args!(concat!($fmt, "\r\n") $(, $($arg)+)?));
    }
}

pub fn handle_interrupt() {
    // let uart = UART.lock();
    // match uart.read_interrupt_type() {
    //     InterruptType::ReceivedDataAvailable => {
    //         while let Some(c) = uart.read_byte() {
    //             push_stdin(c);
    //         }
    //     }
    //     InterruptType::TransmitterHoldingRegisterEmpty => {

    //     }
    //     _ => {}
    // }
    if let Some(c) = UART.lock().read_byte() {
        push_stdin(c);
        // match c {
        //     8 => {
        //         // This is a backspace, so we
        //         // essentially have to write a space and
        //         // backup again:
        //         print_uart!("{} {}", 8 as char, 8 as char);
        //     }
        //     10 | 13 => {
        //         // Newline or carriage-return
        //         println_uart!();
        //     }
        //     _ => {
        //         print_uart!("{}", c as char);
        //     }
        // }
    }
}
