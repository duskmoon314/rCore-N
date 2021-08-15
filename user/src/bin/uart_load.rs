#![no_std]
#![no_main]
#![feature(asm)]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::sync::Arc;
use bitflags::bitflags;
use core::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering::SeqCst};
use embedded_hal::serial::{Read, Write};
use lazy_static::*;
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;
use riscv::register::uie;
use spin::Mutex;
use user_lib::{
    claim_ext_int, get_time, init_user_trap, set_ext_int_enable, set_timer, user_uart::*, yield_,
};

static UART_IRQN: AtomicU16 = AtomicU16::new(0);
static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
static IS_TIMEOUT: AtomicBool = AtomicBool::new(false);
static RX_SEED: AtomicU32 = AtomicU32::new(0);
static TX_SEED: AtomicU32 = AtomicU32::new(0);
static MODE: AtomicU32 = AtomicU32::new(0);

bitflags! {
    struct UartLoadConfig: u32 {
        const KERNEL_MODE = 0b1;
        const POLLING_MODE = 0b10;
        const INTR_MODE = 0b100;
        const UART3 = 0b1000;
        const UART4 = 0b10000;
        const ALL_MODE = Self::KERNEL_MODE.bits | Self::POLLING_MODE.bits | Self::INTR_MODE.bits;
    }
}

#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
lazy_static! {
    pub static ref INTR_SERIAL: Arc<Mutex<BufferedSerial>> = Arc::new(Mutex::new(
        BufferedSerial::new(get_base_addr_from_irq(UART_IRQN.load(SeqCst)))
    ));
}

#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
lazy_static! {
    pub static ref POLLING_SERIAL: Arc<Mutex<PollingSerial>> = Arc::new(Mutex::new(
        PollingSerial::new(get_base_addr_from_irq(UART_IRQN.load(SeqCst)))
    ));
}

#[no_mangle]
pub fn main() -> i32 {
    let init_res = init_user_trap();
    println!(
        "[uart load] trap init result: {:#x}, now waiting for config init...",
        init_res
    );
    unsafe {
        uie::set_usoft();
        uie::set_utimer();
    }
    while !IS_INITIALIZED.load(SeqCst) {}
    let time_us = get_time() * 1000;
    set_timer(time_us + 1000_000);

    let (rx_count, tx_count, error_count) = match UartLoadConfig::from_bits(MODE.load(SeqCst)) {
        Some(UartLoadConfig::KERNEL_MODE) => kernel_driver_test(),
        Some(UartLoadConfig::POLLING_MODE) => user_polling_test(),
        Some(UartLoadConfig::INTR_MODE) => user_intr_test(),
        _ => {
            println!("[uart load] Mode not supported!");
            (0, 0, 0)
        }
    };
    println!(
        "Test finished, {} bytes sent, {} bytes received, {} bytes error.",
        tx_count, rx_count, error_count
    );
    0
}

fn kernel_driver_test() -> (usize, usize, usize) {
    println!("[uart load] Kernel mode");
    while !(IS_TIMEOUT.load(SeqCst)) {}
    (0, 0, 0)
}

fn user_polling_test() -> (usize, usize, usize) {
    let uart_irqn = UART_IRQN.load(SeqCst);
    let claim_res = claim_ext_int(uart_irqn as usize);
    POLLING_SERIAL.lock().hardware_init();
    println!("[uart load] Polling mode, claim result: {:#x}", claim_res);
    let tx_seed = TX_SEED.load(SeqCst);
    let rx_seed = RX_SEED.load(SeqCst);
    let mut tx_rng = XorShiftRng::seed_from_u64(tx_seed as u64);
    let mut rx_rng = XorShiftRng::seed_from_u64(rx_seed as u64);
    let mut error_count: usize = 0;
    let mut next_tx = tx_rng.next_u32();
    let mut expect_rx = rx_rng.next_u32();
    let mut serial = POLLING_SERIAL.lock();
    while !(IS_TIMEOUT.load(SeqCst)) {
        let tx_res = serial.try_write(next_tx as u8);
        let rx_res = serial.try_read();
        if tx_res.is_ok() {
            next_tx = tx_rng.next_u32();
        }
        if rx_res.is_ok() {
            let rx_val = rx_res.unwrap();
            if rx_val != expect_rx as u8 {
                error_count += 1;
            }
            expect_rx = rx_rng.next_u32();
        }
    }
    (serial.rx_count, serial.tx_count, error_count)
}

fn user_intr_test() -> (usize, usize, usize) {
    let uart_irqn = UART_IRQN.load(SeqCst);
    let claim_res = claim_ext_int(uart_irqn as usize);
    INTR_SERIAL.lock().hardware_init();
    let en_res = set_ext_int_enable(uart_irqn as usize, 1);
    println!(
        "[uart load] Interrupt mode, claim result: {:#x}, enable res: {:#x}",
        claim_res, en_res
    );
    let tx_seed = TX_SEED.load(SeqCst);
    let rx_seed = RX_SEED.load(SeqCst);
    let mut tx_rng = XorShiftRng::seed_from_u64(tx_seed as u64);
    let mut rx_rng = XorShiftRng::seed_from_u64(rx_seed as u64);
    let mut error_count: usize = 0;
    let mut next_tx = tx_rng.next_u32();
    let mut expect_rx = rx_rng.next_u32();
    while !(IS_TIMEOUT.load(SeqCst)) {
        unsafe {
            uie::clear_uext();
            uie::clear_usoft();
            uie::clear_utimer();
        }
        let mut serial = INTR_SERIAL.lock();
        loop {
            let tx_res = serial.try_write(next_tx as u8);
            let rx_res = serial.try_read();
            if tx_res.is_ok() {
                next_tx = tx_rng.next_u32();
            }
            if rx_res.is_ok() {
                let rx_val = rx_res.unwrap();
                if rx_val != expect_rx as u8 {
                    error_count += 1;
                }
                expect_rx = rx_rng.next_u32();
            }
            if tx_res.is_err() && rx_res.is_err() {
                break;
            }
        }
        drop(serial);
        unsafe {
            uie::set_uext();
            uie::set_usoft();
            uie::set_utimer();
        }
        yield_();
    }
    let serial = INTR_SERIAL.lock();
    (serial.rx_count, serial.tx_count, error_count)
}

mod user_trap {
    use super::*;
    #[no_mangle]
    pub fn soft_intr_handler(pid: usize, msg: usize) {
        if msg == 15 {
            println!("[uart load] Received SIGTERM, exiting...");
            user_lib::exit(15);
        } else {
            println!("[uart load] Received message 0x{:x} from pid {}", msg, pid);
        }
        if let Some(config) = UartLoadConfig::from_bits(msg as u32) {
            let mode = config & UartLoadConfig::ALL_MODE;
            MODE.store(mode.bits(), SeqCst);
            if config.contains(UartLoadConfig::UART3) {
                TX_SEED.store(20210821, SeqCst);
                RX_SEED.store(1000000007, SeqCst);
                #[cfg(feature = "board_qemu")]
                UART_IRQN.store(14, SeqCst);
                #[cfg(feature = "board_lrv")]
                UART_IRQN.store(5, SeqCst);
            } else if config.contains(UartLoadConfig::UART4) {
                RX_SEED.store(20210821, SeqCst);
                TX_SEED.store(1000000007, SeqCst);
                #[cfg(feature = "board_qemu")]
                UART_IRQN.store(15, SeqCst);
                #[cfg(feature = "board_lrv")]
                UART_IRQN.store(6, SeqCst);
            } else {
                println!("[uart load] UART config invalid!");
            }
            IS_INITIALIZED.store(true, SeqCst);
        } else {
            println!("[uart load] Invalid config {:#x}!", msg);
        }
    }

    #[no_mangle]
    pub fn ext_intr_handler(irq: u16, is_from_kernel: bool) {
        // if is_from_kernel {
        //     println!("[uart load] Received UEI from kernel, irq: {}", irq);
        // } else {
        //     println!("[uart load] user external interrupt, irq: {}", irq);
        // }
        let uart_irqn = UART_IRQN.load(SeqCst);
        if irq == uart_irqn {
            INTR_SERIAL.lock().interrupt_handler();
        }
    }

    #[no_mangle]
    pub fn timer_intr_handler(time_us: usize) {
        IS_TIMEOUT.store(true, SeqCst);
    }
}
