use crate::console_blog::push_stdin;
use core::fmt::{self, Write};

use alloc::sync::Arc;
use lazy_static::*;
use spin::Mutex;

#[cfg(feature = "board_qemu")]
use uart8250::MmioUart8250;

#[cfg(feature = "board_qemu")]
lazy_static! {
    pub static ref UART: Arc<Mutex<MmioUart8250>> =
        Arc::new(Mutex::new(MmioUart8250::new(0x1000_0000)));
}

#[cfg(feature = "board_lrv")]
use uart_xilinx::MmioUartAxiLite;

#[cfg(feature = "board_lrv")]
lazy_static! {
    pub static ref UART: Arc<Mutex<MmioUartAxiLite<'static>>> =
        Arc::new(Mutex::new(MmioUartAxiLite::new(0x6000_0000)));
}

pub fn init() {
    #[cfg(feature = "board_qemu")]
    {
        let uart = UART.lock();
        uart.init(11_059_200, 115200);
    }
}

pub fn print_uart(args: fmt::Arguments) {
    UART.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print_uart {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::uart::print_uart(format_args!($fmt $(, $($arg)+)?));
    };
}

#[macro_export]
macro_rules! println_uart {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::uart::print_uart(format_args!(concat!($fmt, "\r\n") $(, $($arg)+)?));
    }
}

pub fn handle_interrupt() {
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
