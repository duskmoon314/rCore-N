#![no_std]
#![no_main]
#![feature(asm)]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::collections::VecDeque;
use alloc::sync::Arc;
use bitflags::bitflags;
use core::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering::Relaxed};
use embedded_hal::serial::{Read, Write};
use lazy_static::*;
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;
use riscv::register::uie;
use spin::Mutex;
use user_lib::{
    claim_ext_int, get_time, init_user_trap, read, set_ext_int_enable, set_timer, sleep,
    user_uart::*, write, yield_,
};

static UART_IRQN: AtomicU16 = AtomicU16::new(0);
static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
static IS_TIMEOUT: AtomicBool = AtomicBool::new(false);
static RX_SEED: AtomicU32 = AtomicU32::new(0);
static TX_SEED: AtomicU32 = AtomicU32::new(0);
static MODE: AtomicU32 = AtomicU32::new(0);

const TEST_TIME_US: isize = 1000_000;
const HALF_FIFO_DEPTH: usize = FIFO_DEPTH / 2;
const BAUD_RATE: usize = 1_152_000;

type Rng = Arc<Mutex<XorShiftRng>>;

lazy_static! {
    static ref RX_RNG: Rng = Arc::new(Mutex::new(XorShiftRng::seed_from_u64(
        RX_SEED.load(Relaxed) as u64
    )));
    static ref TX_RNG: Rng = Arc::new(Mutex::new(XorShiftRng::seed_from_u64(
        TX_SEED.load(Relaxed) as u64
    )));
}

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
    while !IS_INITIALIZED.load(Relaxed) {}

    let (rx_count, tx_count, error_count) = match UartLoadConfig::from_bits(MODE.load(Relaxed)) {
        Some(UartLoadConfig::KERNEL_MODE) => kernel_driver_test(),
        Some(UartLoadConfig::POLLING_MODE) => user_polling_test(),
        Some(UartLoadConfig::INTR_MODE) => user_intr_test(),
        _ => {
            println!("[uart load] Mode not supported!");
            (0, 0, 0)
        }
    };
    if irq_to_serial_id(UART_IRQN.load(Relaxed)) == 3 {
        sleep(100);
    }
    println!(
        "Test finished, {} bytes sent, {} bytes received, {} bytes error.",
        tx_count, rx_count, error_count
    );
    0
}

fn kernel_driver_test() -> (usize, usize, usize) {
    let (mut tx_rng, mut rx_rng) = (TX_RNG.lock(), RX_RNG.lock());
    let mut tx_count = 0;
    let mut rx_count = 0;
    let mut error_count: usize = 0;
    let mut next_tx = tx_rng.next_u32();
    let mut expect_rx = rx_rng.next_u32();
    let tx_fd = irq_to_serial_id(UART_IRQN.load(Relaxed)) + 1;
    let rx_fd = {
        if tx_fd == 3 {
            4
        } else {
            3
        }
    } as usize;
    println!(
        "[uart load] Kernel mode, tx fd: {}, rx_fd: {}",
        tx_fd, rx_fd
    );
    let mut tx_buf = [0u8; HALF_FIFO_DEPTH];
    let mut rx_buf = [0u8; 1];
    for _ in 0..1000 {
        read(rx_fd, &mut rx_buf);
    }
    let time_us = get_time() * 1000;

    set_timer(time_us + TEST_TIME_US);
    while !(IS_TIMEOUT.load(Relaxed)) {
        for i in 0..HALF_FIFO_DEPTH {
            tx_buf[i] = next_tx as u8;
            next_tx = tx_rng.next_u32();
        }
        write(tx_fd, &tx_buf);
        tx_count += HALF_FIFO_DEPTH;
        let mut rx_fifo_count = 0;
        while !(IS_TIMEOUT.load(Relaxed)) && rx_fifo_count < HALF_FIFO_DEPTH {
            read(rx_fd, &mut rx_buf);
            if rx_buf[0] != 0 {
                if rx_buf[0] != expect_rx as u8 {
                    error_count += 1;
                }
                rx_count += 1;
                expect_rx = rx_rng.next_u32();
                rx_fifo_count += 1;
            }
        }
    }
    (rx_count, tx_count, error_count)
}

fn user_polling_test() -> (usize, usize, usize) {
    let uart_irqn = UART_IRQN.load(Relaxed);
    let claim_res = claim_ext_int(uart_irqn as usize);
    let mut serial = PollingSerial::new(get_base_addr_from_irq(UART_IRQN.load(Relaxed)));
    serial.hardware_init(BAUD_RATE);
    println!("[uart load] Polling mode, claim result: {:#x}", claim_res);
    let mut tx_rng = TX_RNG.lock();
    let mut rx_rng = RX_RNG.lock();
    let mut error_count: usize = 0;
    let mut next_tx = tx_rng.next_u32();
    let mut expect_rx = rx_rng.next_u32();

    let time_us = get_time() * 1000;
    set_timer(time_us + TEST_TIME_US);

    while !(IS_TIMEOUT.load(Relaxed)) {
        for _ in 0..HALF_FIFO_DEPTH {
            serial.try_write(next_tx as u8).unwrap();
            next_tx = tx_rng.next_u32();
        }
        let mut rx_fifo_count = 0;
        while !(IS_TIMEOUT.load(Relaxed)) && rx_fifo_count < HALF_FIFO_DEPTH {
            if let Ok(rx_val) = serial.try_read() {
                if rx_val != expect_rx as u8 {
                    error_count += 1;
                }
                expect_rx = rx_rng.next_u32();
                rx_fifo_count += 1;
            }
        }
    }

    (serial.rx_count, serial.tx_count, error_count)
}

lazy_static! {
    static ref INTR_SERIAL: Mutex<BufferedSerial> = Mutex::new(BufferedSerial::new(
        get_base_addr_from_irq(UART_IRQN.load(Relaxed))
    ));
}

fn user_intr_test() -> (usize, usize, usize) {
    let uart_irqn = UART_IRQN.load(Relaxed);
    let claim_res = claim_ext_int(uart_irqn as usize);
    INTR_SERIAL.lock().hardware_init(BAUD_RATE);
    let en_res = set_ext_int_enable(uart_irqn as usize, 1);
    println!(
        "[uart load] Interrupt mode, claim result: {:#x}, enable res: {:#x}",
        claim_res, en_res
    );
    let mut error_count: usize = 0;
    // let mut tx_rng = TX_RNG.lock();
    // let mut rx_rng = RX_RNG.lock();
    // let mut next_tx = tx_rng.next_u32();
    // let mut expect_rx = rx_rng.next_u32();
    let mut next_tx = '0' as u8;
    let mut expect_rx = '0' as u8;
    let mut rx_buf = VecDeque::<u8>::with_capacity(10000);
    let time_us = get_time() * 1000;
    set_timer(time_us + TEST_TIME_US);

    while !(IS_TIMEOUT.load(Relaxed)) {
        unsafe {
            uie::clear_uext();
            uie::clear_usoft();
            uie::clear_utimer();
        }
        let mut serial = INTR_SERIAL.lock();
        for _ in 0..HALF_FIFO_DEPTH {
            let tx_res = serial.try_write(next_tx as u8);
            let rx_res = serial.try_read();
            if tx_res.is_ok() {
                // next_tx = tx_rng.next_u32();
                next_tx += 1;
                if next_tx > '9' as u8 {
                    next_tx = '0' as u8;
                }
            }
            if rx_res.is_ok() {
                let rx_val = rx_res.unwrap();
                if rx_val != expect_rx as u8 {
                    error_count += 1;
                }
                // expect_rx = rx_rng.next_u32();
                expect_rx += 1;
                if expect_rx > '9' as u8 {
                    expect_rx = '0' as u8;
                }
                rx_buf.push_back(rx_val);
            }
        }
        drop(serial);
        unsafe {
            uie::set_uext();
            uie::set_usoft();
            uie::set_utimer();
        }
        // for _ in 0..100 {}
        yield_();
    }
    unsafe {
        uie::clear_uext();
        uie::clear_usoft();
        uie::clear_utimer();
    }

    if uart_irqn == 7 {
        sleep(300);
        // println!("rx buf:");
        // for ch in rx_buf.iter() {
        //     print!("{}", *ch as char);
        // }
        // println!("");
    }
    let serial = INTR_SERIAL.lock();
    println!(
        "[uart load] Intr count: {}, Tx: {}, Rx: {}",
        serial.intr_count, serial.tx_intr_count, serial.rx_intr_count
    );
    (serial.rx_count, serial.tx_count, error_count)
}

mod user_trap {
    use super::*;
    #[no_mangle]
    pub fn soft_intr_handler(_pid: usize, msg: usize) {
        // if msg == 15 {
        //     println!("[uart load] Received SIGTERM, exiting...");
        //     user_lib::exit(15);
        // } else {
        //     println!("[uart load] Received message 0x{:x} from pid {}", msg, pid);
        // }
        if let Some(config) = UartLoadConfig::from_bits(msg as u32) {
            let mode = config & UartLoadConfig::ALL_MODE;
            MODE.store(mode.bits(), Relaxed);
            if config.contains(UartLoadConfig::UART3) {
                TX_SEED.store(20210821, Relaxed);
                RX_SEED.store(1000000007, Relaxed);
                #[cfg(feature = "board_qemu")]
                UART_IRQN.store(14, Relaxed);
                #[cfg(feature = "board_lrv")]
                UART_IRQN.store(6, Relaxed);
            } else if config.contains(UartLoadConfig::UART4) {
                RX_SEED.store(20210821, Relaxed);
                TX_SEED.store(1000000007, Relaxed);
                #[cfg(feature = "board_qemu")]
                UART_IRQN.store(15, Relaxed);
                #[cfg(feature = "board_lrv")]
                UART_IRQN.store(7, Relaxed);
            } else {
                println!("[uart load] UART config invalid!");
            }
            IS_INITIALIZED.store(true, Relaxed);
        } else {
            println!("[uart load] Invalid config {:#x}!", msg);
        }
    }

    #[no_mangle]
    pub fn ext_intr_handler(irq: u16, _is_from_kernel: bool) {
        // if _is_from_kernel {
        //     println!("[uart load] Received UEI from kernel, irq: {}", irq);
        // } else {
        //     println!("[uart load] user external interrupt, irq: {}", irq);
        // }
        if irq == UART_IRQN.load(Relaxed) {
            INTR_SERIAL.lock().interrupt_handler();
        }
    }

    #[no_mangle]
    pub fn timer_intr_handler(_time_us: usize) {
        IS_TIMEOUT.store(true, Relaxed);
    }
}
