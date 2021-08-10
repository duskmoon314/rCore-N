use crate::console_blog::{IN_BUFFER, OUT_BUFFER};
use core::fmt::{self, Write};

use alloc::sync::Arc;
use lazy_static::*;
use spin::Mutex;

#[cfg(feature = "board_qemu")]
use uart8250::MmioUart8250;

#[cfg(feature = "board_qemu")]
lazy_static! {
    pub static ref UART: Arc<Mutex<MmioUart8250<'static>>> =
        Arc::new(Mutex::new(MmioUart8250::new(0x1000_0200)));
}

#[cfg(feature = "board_lrv")]
use uart_xilinx::MmioUartAxiLite;

#[cfg(feature = "board_lrv")]
lazy_static! {
    pub static ref UART: Arc<Mutex<MmioUartAxiLite<'static>>> =
        Arc::new(Mutex::new(MmioUartAxiLite::new(0x6000_1000)));
}

#[cfg(feature = "board_qemu")]
pub fn init() {
    let uart = UART.lock();
    uart.init(11_059_200, 115200);
    // Rx FIFO trigger level=14, reset Rx & Tx FIFO, enable FIFO
    uart.write_fcr(0b11_000_11_1);
}

#[cfg(feature = "board_lrv")]
pub fn init() {
    UART.lock().enable_interrupt();
}

#[allow(dead_code)]
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

#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
const FIFO_DEPTH: usize = 16;

#[cfg(feature = "board_qemu")]
pub fn handle_interrupt() {
    let uart = UART.lock();
    let int_id = uart.read_iir();
    // No interrupt is pending
    if int_id & 0b1 == 1 {
        return;
    }
    let int_id = (int_id >> 1) & 0b111;
    match int_id {
        // Received Data Available
        0b010 => {
            let mut stdin = IN_BUFFER.lock();
            while let Some(ch) = uart.read_byte() {
                stdin.push_back(ch);
            }
        }
        // Transmitter Holding Register Empty
        0b001 => {
            let mut stdout = OUT_BUFFER.lock();
            for _ in 0..FIFO_DEPTH {
                if let Some(ch) = stdout.pop_front() {
                    uart.write_byte(ch);
                } else {
                    uart.disable_transmitter_holding_register_empty_interrupt();
                    break;
                }
            }
        }
        _ => {}
    }
}

#[cfg(feature = "board_lrv")]
pub fn handle_interrupt() {
    use uart_xilinx::uart_lite::Status;
    let uart = UART.lock();
    let status = uart.status();
    if status.contains(Status::TX_FIFO_EMPTY) {
        let mut stdout = OUT_BUFFER.lock();
        for _ in 0..FIFO_DEPTH {
            if let Some(ch) = stdout.pop_front() {
                uart.write_byte(ch);
            } else {
                break;
            }
        }
    }
    if status.contains(Status::RX_FIFO_FULL) {
        let mut stdin = IN_BUFFER.lock();
        for _ in 0..FIFO_DEPTH {
            if let Some(ch) = uart.read_byte() {
                stdin.push_back(ch);
            } else {
                break;
            }
        }
    }
}
