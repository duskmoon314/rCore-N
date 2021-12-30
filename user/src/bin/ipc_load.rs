#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use bitflags::bitflags;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering::Relaxed};
use heapless::mpmc::Q64;
use lazy_static::*;
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;
use riscv::register::uie;
use spin::Mutex;
use user_lib::{
    claim_ext_int, get_time, init_user_trap, mailread, mailwrite, send_msg, set_ext_int_enable,
    set_timer, sleep,
    trap::{get_context, hart_id, Plic},
};

static DST_PID: AtomicUsize = AtomicUsize::new(0);
static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
static IS_TIMEOUT: AtomicBool = AtomicBool::new(false);
static HAS_INTR: AtomicBool = AtomicBool::new(false);
static RX_SEED: AtomicU32 = AtomicU32::new(0);
static TX_SEED: AtomicU32 = AtomicU32::new(0);
static MODE: AtomicU32 = AtomicU32::new(0);

static MSG_QUEUE: Q64<u8> = Q64::new();

const TEST_TIME_US: isize = 1000_000;
const BUFFER_SIZE: usize = 8;
const MAX_SHIFT: isize = 10;

type Rng = Mutex<XorShiftRng>;
type Hasher = blake3::Hasher;

lazy_static! {
    static ref RX_RNG: Rng = Mutex::new(XorShiftRng::seed_from_u64(RX_SEED.load(Relaxed) as u64));
    static ref TX_RNG: Rng = Mutex::new(XorShiftRng::seed_from_u64(TX_SEED.load(Relaxed) as u64));
}

bitflags! {
    struct IpcLoadConfig: u32 {
        const MSG_MODE = 0b1;
        const MAIL_MODE = 0b10;
        const UINTC_MODE = 0b100;
        const ALL_MODE = Self::MSG_MODE.bits | Self::MAIL_MODE.bits | Self::UINTC_MODE.bits;
    }
}

#[no_mangle]
pub fn main() -> i32 {
    let init_res = init_user_trap();
    println!(
        "[ipc load] trap init result: {:#x}, now waiting for config init...",
        init_res
    );
    unsafe {
        uie::set_usoft();
        uie::set_utimer();
    }
    while !IS_INITIALIZED.load(Relaxed) {}

    let (rx_count, tx_count, error_count) = match IpcLoadConfig::from_bits(MODE.load(Relaxed)) {
        Some(IpcLoadConfig::MSG_MODE) => sendmsg_test(),
        Some(IpcLoadConfig::MAIL_MODE) => mailbox_test(),
        Some(IpcLoadConfig::UINTC_MODE) => uintc_test(),
        _ => {
            println!("[uart load] Mode not supported!");
            (0, 0, 0)
        }
    };
    if DST_PID.load(Relaxed) & 1 == 1 {
        sleep(100);
    }
    println!(
        "Test finished, {} bytes sent, {} bytes received, {} bytes error.",
        tx_count, rx_count, error_count
    );
    0
}

fn sendmsg_test() -> (usize, usize, usize) {
    let mut hasher = Hasher::new();
    let dst_pid = DST_PID.load(Relaxed);
    let mut tx_rng = TX_RNG.lock();
    let mut rx_rng = RX_RNG.lock();
    let mut error_count: usize = 0;
    let mut err_pos = -1;
    let mut next_tx = tx_rng.next_u32();
    let mut expect_rx = rx_rng.next_u32();
    let mut tx_count = 0;
    let mut rx_count = 0;

    let time_us = get_time() * 1000;
    set_timer(time_us + TEST_TIME_US);

    while !(IS_TIMEOUT.load(Relaxed)) {
        for _ in 0..BUFFER_SIZE {
            send_msg(dst_pid, next_tx as u8 as usize);
            // hasher.update(&[next_tx as u8]);
            next_tx = tx_rng.next_u32();
            tx_count += 1;
        }
        for _ in 0..BUFFER_SIZE {
            if let Some(rx_val) = MSG_QUEUE.dequeue() {
                let mut max_shift = MAX_SHIFT;
                if err_pos == -1 && rx_val != expect_rx as u8 {
                    err_pos = rx_count as isize;
                }
                while rx_val != expect_rx as u8 && max_shift > 0 {
                    error_count += 1;
                    expect_rx = rx_rng.next_u32();
                    max_shift -= 1;
                }
                // hasher.update(&[rx_val]);
                expect_rx = rx_rng.next_u32();
                rx_count += 1;
            }
        }
    }

    if dst_pid & 1 == 1 {
        sleep(500);
    }
    println!("[ipc load] err pos: {}", err_pos);
    (rx_count, tx_count, error_count)
}

fn mailbox_test() -> (usize, usize, usize) {
    let mut tx_rng = TX_RNG.lock();
    let mut rx_rng = RX_RNG.lock();
    let mut tx_count = 0;
    let mut rx_count = 0;
    let mut error_count: usize = 0;
    let mut next_tx = tx_rng.next_u32();
    let mut expect_rx = rx_rng.next_u32();
    let dst_pid = DST_PID.load(Relaxed);
    let mut hasher = Hasher::new();

    let mut tx_buf = [0u8; BUFFER_SIZE];
    let mut rx_buf = [0u8; BUFFER_SIZE];
    while mailread(&mut rx_buf) > 0 {}
    let time_us = get_time() * 1000;
    set_timer(time_us + TEST_TIME_US);
    while !(IS_TIMEOUT.load(Relaxed)) {
        for i in 0..BUFFER_SIZE {
            tx_buf[i] = next_tx as u8;
            // hasher.update(&[next_tx as u8]);
            next_tx = tx_rng.next_u32();
            let tx_fifo_count = mailwrite(dst_pid, &tx_buf);
            if tx_fifo_count > 0 {
                tx_count += tx_fifo_count as usize;
            }
        }
        for i in 0..BUFFER_SIZE {
            let rx_fifo_count = mailread(&mut rx_buf);
            if rx_fifo_count > 0 {
                let rx_val = rx_buf[i];
                let mut max_shift = MAX_SHIFT;
                while rx_val != expect_rx as u8 && max_shift > 0 {
                    error_count += 1;
                    expect_rx = rx_rng.next_u32();
                    max_shift -= 1;
                }
                expect_rx = rx_rng.next_u32();
                // hasher.update(&[rx_val]);
                rx_count += rx_fifo_count as usize;
            }
        }
    }
    (rx_count, tx_count, error_count)
}

fn uintc_test() -> (usize, usize, usize) {
    unsafe {
        uie::clear_uext();
        uie::clear_usoft();
        uie::clear_utimer();
    }
    let mut hasher = Hasher::new();
    let uart_irqn = 0;
    let claim_res = claim_ext_int(uart_irqn as usize);
    let en_res = set_ext_int_enable(uart_irqn as usize, 1);
    println!(
        "[uart load] Interrupt mode, claim result: {:#x}, enable res: {:#x}",
        claim_res, en_res
    );
    let mut error_count: usize = 0;
    let mut err_pos = -1;
    let mut tx_rng = TX_RNG.lock();
    let mut rx_rng = RX_RNG.lock();
    let mut next_tx = tx_rng.next_u32();
    let mut expect_rx = rx_rng.next_u32();
    let time_us = get_time() * 1000;
    set_timer(time_us + TEST_TIME_US);

    unsafe {
        uie::set_uext();
        uie::set_usoft();
        uie::set_utimer();
    }

    while !(IS_TIMEOUT.load(Relaxed)) {
        for _ in 0..BUFFER_SIZE {}

        for _ in 0..BUFFER_SIZE {}

        if HAS_INTR.load(Relaxed) {
            HAS_INTR.store(false, Relaxed);
            Plic::complete(get_context(hart_id(), 'U'), uart_irqn);
        }
    }
    unsafe {
        uie::clear_uext();
        uie::clear_usoft();
        uie::clear_utimer();
    }

    if uart_irqn == 14 || uart_irqn == 6 {
        sleep(500);
    }
    (0, 0, error_count)
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
        let dst_pid = msg >> 8;
        if dst_pid > 0 {
            DST_PID.store(dst_pid, Relaxed);
            let config_bits = msg as u32 & IpcLoadConfig::ALL_MODE.bits();
            if let Some(config) = IpcLoadConfig::from_bits(config_bits) {
                let mode = config & IpcLoadConfig::ALL_MODE;
                MODE.store(mode.bits(), Relaxed);
                if dst_pid & 1 == 1 {
                    TX_SEED.store(20210821, Relaxed);
                    RX_SEED.store(1000000007, Relaxed);
                } else {
                    RX_SEED.store(20210821, Relaxed);
                    TX_SEED.store(1000000007, Relaxed);
                }
                IS_INITIALIZED.store(true, Relaxed);
            } else {
                println!("[uart load] Invalid config {:#x}!", msg);
            }
        } else {
            let _ = MSG_QUEUE.enqueue(msg as u8);
        }
    }

    #[no_mangle]
    pub fn ext_intr_handler(irq: u16, _is_from_kernel: bool) {
        // if _is_from_kernel {
        //     println!("[uart load] Received UEI from kernel, irq: {}", irq);
        // } else {
        //     println!("[uart load] user external interrupt, irq: {}", irq);
        // }
        if irq == 0 {
            HAS_INTR.store(true, Relaxed);
        } else {
            println!("[uart load] Unknown UEI!, irq: {}", irq);
        }
        // println!("[uart load] UEI fin");
    }

    #[no_mangle]
    pub fn timer_intr_handler(_time_us: usize) {
        IS_TIMEOUT.store(true, Relaxed);
    }
}
