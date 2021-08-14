#![no_std]
#![no_main]
#![feature(asm)]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::string::String;
use riscv::register::uie;
use user_console::pop_stdin;
use user_lib::{claim_ext_int, init_user_trap, set_ext_int_enable, yield_};

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

#[no_mangle]
pub fn main() -> i32 {
    println!("[uart ext] A user mode serial driver demo using UEI");
    let init_res = init_user_trap();
    let claim_res = claim_ext_int(uart::UART_IRQN as usize);
    uart::init();
    let en_res = set_ext_int_enable(uart::UART_IRQN as usize, 1);
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
        let c = pop_stdin();
        if c != 0 {
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
        // yield_();
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

#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
pub mod uart {
    use crate::user_console::{IN_BUFFER, OUT_BUFFER};
    use alloc::sync::Arc;
    use lazy_static::*;
    use spin::Mutex;
    #[cfg(feature = "board_qemu")]
    use uart8250::{InterruptType, MmioUart8250};
    #[cfg(feature = "board_qemu")]
    pub const UART_BASE_ADDRESS: usize = 0x1000_0100;
    #[cfg(feature = "board_qemu")]
    pub const UART_IRQN: u16 = 9;

    #[cfg(feature = "board_qemu")]
    lazy_static! {
        pub static ref UART: Arc<Mutex<MmioUart8250<'static>>> =
            Arc::new(Mutex::new(MmioUart8250::new(UART_BASE_ADDRESS)));
    }

    #[cfg(feature = "board_lrv")]
    use uart_xilinx::uart_16550::{InterruptType, MmioUartAxi16550};
    #[cfg(feature = "board_lrv")]
    pub const UART_BASE_ADDRESS: usize = 0x6000_2000;
    #[cfg(feature = "board_lrv")]
    pub const UART_IRQN: u16 = 5;

    #[cfg(feature = "board_lrv")]
    lazy_static! {
        pub static ref UART: Arc<Mutex<MmioUartAxi16550<'static>>> =
            Arc::new(Mutex::new(MmioUartAxi16550::new(UART_BASE_ADDRESS)));
    }

    pub fn init() {
        let uart = UART.lock();
        uart.write_ier(0);
        let _ = uart.read_msr();
        let _ = uart.read_lsr();
        uart.init(100_000_000, 115200);
        // Rx FIFO trigger level=14, reset Rx & Tx FIFO, enable FIFO
        uart.write_fcr(0b11_000_11_1);
    }

    const FIFO_DEPTH: usize = 16;

    pub fn handle_interrupt() {
        let uart = UART.lock();
        let int_type = uart.read_interrupt_type();
        match int_type {
            InterruptType::ReceivedDataAvailable | InterruptType::Timeout => {
                println!("Received data available");
                let mut stdin = IN_BUFFER.lock();
                while let Some(ch) = uart.read_byte() {
                    stdin.push_back(ch);
                }
            }
            InterruptType::TransmitterHoldingRegisterEmpty => {
                println!("Transmitter Holding Register Empty");
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
            InterruptType::ModemStatus => {
                let ms = uart.read_msr();
                println!("Modem Status: {:#x}", ms);
            }
            _ => {
                println!("[uart ext] {:?} not supported!", int_type);
            }
        }
    }
}

#[cfg(feature = "board_lrv_uartlite")]
mod uart {
    use crate::user_console::{IN_BUFFER, OUT_BUFFER};
    use alloc::sync::Arc;
    use lazy_static::*;
    use spin::Mutex;
    use uart_xilinx::MmioUartAxiLite;

    pub const UART_BASE_ADDRESS: usize = 0x6000_2000;
    pub const UART_IRQN: u16 = 5;

    lazy_static! {
        pub static ref UART: Arc<Mutex<MmioUartAxiLite<'static>>> =
            Arc::new(Mutex::new(MmioUartAxiLite::new(UART_BASE_ADDRESS)));
    }

    pub fn init() {
        UART.lock().enable_interrupt();
    }

    const FIFO_DEPTH: usize = 16;
    pub fn handle_interrupt() {
        use uart_xilinx::uart_lite::Status;
        let uart = UART.lock();
        let status = uart.status();
        if status.contains(Status::TX_FIFO_EMPTY) {
            let mut stdout = OUT_BUFFER.lock();
            while !uart.is_tx_fifo_full() {
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
}

mod user_console {
    // Based on https://github.com/sgmarz/osblog

    use super::uart::UART;
    use alloc::{collections::VecDeque, sync::Arc};
    use core::fmt::{self, Write};
    use lazy_static::*;
    use spin::Mutex;

    pub const DEFAULT_OUT_BUFFER_SIZE: usize = 1_000;
    pub const DEFAULT_IN_BUFFER_SIZE: usize = 1_000;

    lazy_static! {
        pub static ref IN_BUFFER: Arc<Mutex<VecDeque<u8>>> =
            Arc::new(Mutex::new(VecDeque::with_capacity(DEFAULT_IN_BUFFER_SIZE)));
        pub static ref OUT_BUFFER: Arc<Mutex<VecDeque<u8>>> =
            Arc::new(Mutex::new(VecDeque::with_capacity(DEFAULT_OUT_BUFFER_SIZE)));
    }

    #[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
    #[allow(dead_code)]
    pub fn push_stdout(c: u8) {
        let uart = UART.lock();
        if !uart.is_transmitter_holding_register_empty_interrupt_enabled() {
            uart.write_byte(c);
            uart.enable_transmitter_holding_register_empty_interrupt();
        } else {
            let mut out_buffer = OUT_BUFFER.lock();
            if out_buffer.len() < DEFAULT_OUT_BUFFER_SIZE {
                out_buffer.push_back(c);
            }
        }
    }

    #[cfg(feature = "board_lrv_uartlite")]
    #[allow(dead_code)]
    pub fn push_stdout(c: u8) {
        let uart = UART.lock();
        if uart.is_tx_fifo_empty() && OUT_BUFFER.lock().is_empty() {
            uart.write_byte(c);
        } else {
            let mut out_buffer = OUT_BUFFER.lock();
            if out_buffer.len() < DEFAULT_OUT_BUFFER_SIZE {
                out_buffer.push_back(c);
            }
        }
    }

    #[allow(dead_code)]
    pub fn pop_stdout() -> u8 {
        let mut out_buffer = OUT_BUFFER.lock();
        out_buffer.pop_front().unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn push_stdin(c: u8) {
        let mut in_buffer = IN_BUFFER.lock();
        if in_buffer.len() < DEFAULT_IN_BUFFER_SIZE {
            in_buffer.push_back(c);
        }
    }

    pub fn pop_stdin() -> u8 {
        let mut in_buffer = IN_BUFFER.lock();
        if let Some(ch) = in_buffer.pop_front() {
            ch
        } else {
            #[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
            {
                // Drain UART Rx FIFO
                let uart = UART.lock();
                while let Some(ch_read) = uart.read_byte() {
                    in_buffer.push_back(ch_read);
                }
            }
            in_buffer.pop_front().unwrap_or(0)
        }
    }

    struct UserStdout;

    impl Write for UserStdout {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            for c in s.chars() {
                push_stdout(c as u8);
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

    use crate::uart::{handle_interrupt, UART_IRQN};

    #[no_mangle]
    pub fn user_trap_handler(cx: &mut UserTrapContext) -> &mut UserTrapContext {
        let ucause = ucause::read();
        let utval = utval::read();
        match ucause.cause() {
            ucause::Trap::Interrupt(ucause::Interrupt::UserSoft) => {
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
                                println!("[uart ext] Received SIGTERM, exiting...");
                                user_lib::exit(15);
                            } else {
                                user_println!(
                                    "[uart ext] Received message 0x{:x} from pid {}",
                                    msg,
                                    pid
                                );
                            }
                        } else if ucause::Interrupt::from(cause) == ucause::Interrupt::UserExternal
                        {
                            let irq = trap_record.message as u16;
                            println!("[uart ext] Received UEI from kernel, irq: {}", irq);
                            if irq == UART_IRQN {
                                handle_interrupt();
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
                    if irq == UART_IRQN {
                        handle_interrupt();
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
