#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::{sync::Arc, vec::Vec};
use bitflags::bitflags;
use core::{
    num::Wrapping,
    sync::atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicUsize, Ordering::Relaxed},
};
use embedded_hal::serial::{Read, Write};
use executor::spawn;
use heapless::spsc::Queue;
use lazy_static::*;
use nb::block;
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;
use riscv::register::uie;
use spin::Mutex;
use user_lib::{
    claim_ext_int, get_time, init_user_trap, read, set_ext_int_enable, set_timer, sleep,
    trace::{push_trace, SERIAL_CALL_ENTER, SERIAL_CALL_EXIT, U_TRAP_RETURN},
    trap::{get_context, hart_id, Plic},
    user_uart::*,
    write,
};

static UART_IRQN: AtomicU16 = AtomicU16::new(0);
static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
static IS_TIMEOUT: AtomicBool = AtomicBool::new(false);
static HAS_INTR: AtomicBool = AtomicBool::new(false);
static RX_SEED: AtomicU32 = AtomicU32::new(0);
static TX_SEED: AtomicU32 = AtomicU32::new(0);
static MODE: AtomicU32 = AtomicU32::new(0);

const TEST_TIME_US: isize = 1_000_000;
// const HALF_FIFO_DEPTH: usize = FIFO_DEPTH / 2;
const HALF_FIFO_DEPTH: usize = 2;

// const BAUD_RATE: usize = 9600;
// const BAUD_RATE: usize = 115_200;
// const BAUD_RATE: usize = 921_600;
// const BAUD_RATE: usize = 1_250_000;
const BAUD_RATE: usize = 6_250_000;
const MAX_SHIFT: isize = 2;

const SERIAL_POLL_READ: usize = 63;
const SERIAL_POLL_WRITE: usize = 64;
const SERIAL_INTR_READ: usize = 65;
const SERIAL_INTR_WRITE: usize = 66;

type Rng = Mutex<XorShiftRng>;
type Hasher = blake3::Hasher;

lazy_static! {
    static ref RX_RNG: Rng = Mutex::new(XorShiftRng::seed_from_u64(RX_SEED.load(Relaxed) as u64));
    static ref TX_RNG: Rng = Mutex::new(XorShiftRng::seed_from_u64(TX_SEED.load(Relaxed) as u64));
}

bitflags! {
    struct UartLoadConfig: u32 {
        const KERNEL_MODE = 0b1;
        const POLLING_MODE = 0b10;
        const INTR_MODE = 0b100;
        const UART3 = 0b1000;
        const UART4 = 0b10000;
        const ASYNC_MODE = 0b10_0000;
        const ALL_MODE = Self::ASYNC_MODE.bits | Self::KERNEL_MODE.bits | Self::POLLING_MODE.bits | Self::INTR_MODE.bits;
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
        // Some(UartLoadConfig::POLLING_MODE) => user_polling_test(),
        Some(UartLoadConfig::POLLING_MODE) => user_pingpong_test(),
        Some(UartLoadConfig::INTR_MODE) => user_intr_test(),
        Some(UartLoadConfig::ASYNC_MODE) => user_async_test(),
        _ => {
            println!("[uart load] Mode not supported!");
            (0, 0, 0)
        }
    };
    if irq_to_serial_id(UART_IRQN.load(Relaxed)) == 3 {
        sleep(100);
    }
    println!(
        "[uart {}] Test finished, {} bytes sent, {} bytes received, {} bytes error.",
        UART_IRQN.load(Relaxed),
        tx_count,
        rx_count,
        error_count
    );
    0
}

fn kernel_driver_test() -> (usize, usize, usize) {
    let mut tx_rng = TX_RNG.lock();
    let mut rx_rng = RX_RNG.lock();
    let mut tx_count = 0;
    let mut rx_count = 0;
    let mut error_count: usize = 0;
    let mut next_tx = tx_rng.next_u32();
    let mut expect_rx = rx_rng.next_u32();
    let tx_fd = irq_to_serial_id(UART_IRQN.load(Relaxed)) + 1;
    let rx_fd = tx_fd;
    let mut hasher = Hasher::new();

    // if tx_fd == 3 {
    //     println!(
    //         "[uart load] Kernel mode, tx fd: {}, rx_fd: {}, next_tx: {}, rx: {}",
    //         tx_fd, rx_fd, next_tx as u8, expect_rx as u8,
    //     );
    // }
    // let mut tx_buf = [0u8; HALF_FIFO_DEPTH * 5];
    // let mut rx_buf = [0u8; HALF_FIFO_DEPTH * 5];
    let mut tx_buf = [0u8; HALF_FIFO_DEPTH];
    let mut rx_buf = [0u8; HALF_FIFO_DEPTH];
    while read(rx_fd, &mut rx_buf) > 0 {}
    sleep(20);
    let time_us = get_time() * 1000;
    set_timer(time_us + TEST_TIME_US);
    while !(IS_TIMEOUT.load(Relaxed)) {
        // for i in 0..HALF_FIFO_DEPTH * 5 {
        for i in 0..HALF_FIFO_DEPTH {
            tx_buf[i] = next_tx as u8;
            // hasher.update(&[next_tx as u8]);
            next_tx = tx_rng.next_u32();
        }
        let tx_fifo_count = write(tx_fd, &tx_buf);
        if tx_fifo_count > 0 {
            tx_count += tx_fifo_count as usize;
        }

        let rx_fifo_count = read(rx_fd, &mut rx_buf);
        if rx_fifo_count > 0 {
            for rx_val in &rx_buf[0..rx_fifo_count as usize] {
                let mut max_shift = MAX_SHIFT;
                while *rx_val != expect_rx as u8 && max_shift > 0 {
                    error_count += 1;
                    expect_rx = rx_rng.next_u32();
                    max_shift -= 1;
                }
                // hasher.update(&[*rx_val]);
                expect_rx = rx_rng.next_u32();
            }
            rx_count += rx_fifo_count as usize;
        }
    }
    (rx_count, tx_count, error_count)
}

#[allow(unused)]
fn user_polling_test() -> (usize, usize, usize) {
    let mut hasher = Hasher::new();
    let uart_irqn = UART_IRQN.load(Relaxed);
    let claim_res = claim_ext_int(uart_irqn as usize);
    let mut serial = PollingSerial::new(get_base_addr_from_irq(UART_IRQN.load(Relaxed)));
    serial.hardware_init(BAUD_RATE);
    println!("[uart load] Polling mode, claim result: {:#x}", claim_res);
    let mut tx_rng = TX_RNG.lock();
    let mut rx_rng = RX_RNG.lock();
    let mut error_count: usize = 0;
    let mut err_pos = -1;
    let mut next_tx = tx_rng.next_u32();
    let mut expect_rx = rx_rng.next_u32();
    let mut empty_read = 0;

    let time_us = get_time() * 1000;
    set_timer(time_us + TEST_TIME_US);

    while !(IS_TIMEOUT.load(Relaxed)) {
        push_trace(SERIAL_CALL_ENTER + SERIAL_POLL_READ);
        for _ in 0..HALF_FIFO_DEPTH {
            serial.try_write(next_tx as u8).unwrap();
            // hasher.update(&[next_tx as u8]);
            next_tx = tx_rng.next_u32();
        }
        push_trace(SERIAL_CALL_EXIT + SERIAL_POLL_READ);

        push_trace(SERIAL_CALL_ENTER + SERIAL_POLL_WRITE);
        for _ in 0..HALF_FIFO_DEPTH {
            if let Ok(rx_val) = serial.try_read() {
                let mut max_shift = MAX_SHIFT;
                if err_pos == -1 && rx_val != expect_rx as u8 {
                    err_pos = serial.rx_count as isize;
                }
                while rx_val != expect_rx as u8 && max_shift > 0 {
                    error_count += 1;
                    expect_rx = rx_rng.next_u32();
                    max_shift -= 1;
                }
                // hasher.update(&[rx_val]);
                expect_rx = rx_rng.next_u32();
            } else {
                empty_read += 1;
            }
        }
        push_trace(SERIAL_CALL_EXIT + SERIAL_POLL_WRITE);
    }

    if uart_irqn == 14 || uart_irqn == 6 {
        sleep(500);
    }
    println!(
        "[uart load] err pos: {}, empty read: {}",
        err_pos, empty_read
    );
    (serial.rx_count, serial.tx_count, error_count)
}

#[allow(unused)]
fn user_pingpong_test() -> (usize, usize, usize) {
    let uart_irqn = UART_IRQN.load(Relaxed);
    let claim_res = claim_ext_int(uart_irqn as usize);
    let mut serial = PollingSerial::new(get_base_addr_from_irq(uart_irqn));
    serial.hardware_init(BAUD_RATE);
    println!("[uart load] Polling mode, claim result: {:#x}", claim_res);
    let mut error_count: usize = 0;
    let mut err_pos = -1;
    let (mut next_tx, mut next_rx) = (Wrapping(0u8), Wrapping(0u8));
    let mut block_cnt = 0;
    let mut batch_rx_cnt = 0;
    const BATCH_SIZE: u8 = 16;

    let time_us = get_time() * 1_000;
    set_timer(time_us + TEST_TIME_US);

    while !(IS_TIMEOUT.load(Relaxed)) {
        push_trace(SERIAL_CALL_ENTER + SERIAL_INTR_WRITE);
        for _ in 0..BATCH_SIZE {
            if let Ok(()) = serial.try_write(next_tx.0) {
                next_tx += 1;
            } else {
                block_cnt += 1;
            }
        }
        push_trace(SERIAL_CALL_EXIT + SERIAL_POLL_WRITE);

        push_trace(SERIAL_CALL_ENTER + SERIAL_POLL_READ);
        while let Ok(rx_val) = serial.try_read() {
            let expect = next_rx.0;
            if rx_val != expect {
                err_pos = serial.rx_count as isize;
                println!(
                    "[uart {}] error at {}, expect: {:x}, err_val: {:x}",
                    uart_irqn, err_pos, expect, rx_val
                );
                error_count += 1;
            } else {
                next_rx += 1;
            }
            if error_count > 3 {
                break;
            }
        }
        push_trace(SERIAL_CALL_EXIT + SERIAL_INTR_READ);

        if error_count > 3 {
            break;
        }

        // while !(IS_TIMEOUT.load(Relaxed)) {
        //     serial.error_handler();
        //     serial.interrupt_handler();
        //     if let Ok(rx_val) = serial.try_read() {
        //         let expect = (next_tx + Wrapping(batch_read_count as _)).0;
        //         if rx_val != expect {
        //             err_pos = serial.rx_count as isize;
        //             println!(
        //                 "[uart load] error at {}, expect: {:x}, err_val: {:x}",
        //                 err_pos, expect, rx_val
        //             );
        //             error_count += 1;
        //         } else {
        //             batch_read_count += 1;
        //         }
        //     }
        //     if batch_read_count == BATCH_SIZE as _ {
        //         next_tx += BATCH_SIZE;
        //         break;
        //     }
        // }
    }

    if uart_irqn == 14 || uart_irqn == 6 {
        sleep(500);
    }
    println!("[uart load] block cnt: {}, err pos: {}", block_cnt, err_pos);
    (serial.rx_count, serial.tx_count, error_count)
}

#[allow(unused)]
fn user_flow_control_test() -> (usize, usize, usize) {
    let uart_irqn = UART_IRQN.load(Relaxed);
    let claim_res = claim_ext_int(uart_irqn as usize);
    let mut serial = PollingSerial::new(get_base_addr_from_irq(UART_IRQN.load(Relaxed)));
    serial.hardware_init(BAUD_RATE);
    println!("[uart load] Polling mode, claim result: {:#x}", claim_res);
    let mut error_count: usize = 0;

    let time_us = get_time() * 1_000;
    set_timer(time_us + TEST_TIME_US);

    const BATCH_SIZE: u8 = 20;
    const TX_WORD: u8 = 0x75;

    if uart_irqn & 1 == 0 {
        // Tx
        serial.rts(false);

        while !serial.cts() {}
        println!("[tx] cts set!");
        for idx in 0..BATCH_SIZE {
            println!("[tx] tx {}", idx);
            block!(serial.try_write(idx));
        }
        serial.rts(false);
    } else {
        // Rx
        // println!("[fc rx] rts set!");
        serial.rts(true);
        while !serial.iid_rda() {}
        serial.rts(false);
        println!("[rx] rts clear!");
        let mut rx_cnt = 0;
        while rx_cnt < BATCH_SIZE {
            if let Ok(ch) = serial.try_read() {
                rx_cnt += 1;
                println!("[rx] rx {}", ch);
            }
        }
        serial.rts(false);
    }

    println!("--- role change! ---");

    if uart_irqn & 1 == 1 {
        // Tx
        serial.rts(false);
        while !serial.cts() {}
        // println!("[fc tx] cts set!");
        for idx in 0..BATCH_SIZE {
            // println!("[fc tx] sending {} char", idx);
            block!(serial.try_write(idx));
        }
        serial.rts(false);
    } else {
        // Rx
        println!("[rx] rts set!");
        serial.rts(true);
        while !serial.iid_rda() {}
        serial.rts(false);
        println!("[rx] rts clear!");

        let mut rx_cnt = 0;
        while rx_cnt < BATCH_SIZE {
            if let Ok(ch) = serial.try_read() {
                rx_cnt += 1;
                println!("[rx] rx {}", ch);
            }
        }
    }

    if uart_irqn == 14 || uart_irqn == 6 {
        sleep(500);
    }
    println!("uart: {}", uart_irqn);
    (serial.rx_count, serial.tx_count, error_count)
}

#[allow(unused)]
fn user_full_load_test() -> (usize, usize, usize) {
    let uart_irqn = UART_IRQN.load(Relaxed);
    let claim_res = claim_ext_int(uart_irqn as usize);
    let mut serial = PollingSerial::new(get_base_addr_from_irq(UART_IRQN.load(Relaxed)));
    serial.hardware_init(BAUD_RATE);
    println!("[uart load] Polling mode, claim result: {:#x}", claim_res);
    let mut error_count: usize = 0;

    const BATCH_SIZE: u8 = 16;
    const TX_WORD: u8 = 0x75;
    const ACK_WORD: u8 = 0x65;

    let time_us = get_time() * 1_000;
    set_timer(time_us + TEST_TIME_US);

    let mut rx_cnt = 0;

    while !(IS_TIMEOUT.load(Relaxed)) {
        push_trace(SERIAL_CALL_ENTER + SERIAL_POLL_READ);

        if uart_irqn & 1 == 0 {
            // Tx
            for _ in 0..BATCH_SIZE {
                serial.try_write(TX_WORD).unwrap();
            }
            if let Ok(ch) = serial.try_read() {
                assert!(ch == ACK_WORD);
            }
            serial.error_handler();
        } else {
            // Rx
            // while let Some(bulk_buf) = serial.try_bulk_recv() {
            //     for ch in bulk_buf {
            //         assert!(ch == TX_WORD);
            //     }
            //     rx_cnt += BULK_BUF_SIZE;
            //     if serial.error_handler() {
            //         error_count += 1;
            //     }
            // }
            while let Ok(ch) = serial.try_read() {
                assert!(ch == TX_WORD);
                rx_cnt += 1;
                if serial.error_handler() {
                    error_count += 1;
                }
            }
            if rx_cnt >= 16 {
                serial.try_write(ACK_WORD).unwrap();
                rx_cnt = 0;
            }
        }
        push_trace(SERIAL_CALL_EXIT + SERIAL_POLL_READ);

        push_trace(SERIAL_CALL_ENTER + SERIAL_POLL_WRITE);
    }

    if uart_irqn == 14 || uart_irqn == 6 {
        sleep(500);
    }
    (serial.rx_count, serial.tx_count, error_count)
}

#[allow(unused)]
fn user_short_buf_test() -> (usize, usize, usize) {
    let uart_irqn = UART_IRQN.load(Relaxed);
    let claim_res = claim_ext_int(uart_irqn as usize);
    let mut serial = PollingSerial::new(get_base_addr_from_irq(UART_IRQN.load(Relaxed)));
    serial.hardware_init(BAUD_RATE);
    println!("[uart load] Polling mode, claim result: {:#x}", claim_res);
    let mut error_count: usize = 0;

    const BATCH_SIZE: u8 = 16;

    let time_us = get_time() * 1_000;
    set_timer(time_us + TEST_TIME_US);

    let mut buf = Vec::new();
    if uart_irqn & 1 == 0 {
        for i in 0..16 {
            buf.push(i);
        }
    }

    while !(IS_TIMEOUT.load(Relaxed)) {
        while let Some(ch) = buf.pop() {
            serial.try_write(ch).unwrap();
            serial.error_handler();
        }
        // push_trace(SERIAL_CALL_EXIT + SERIAL_POLL_READ);

        // push_trace(SERIAL_CALL_ENTER + SERIAL_POLL_WRITE);
        while let Ok(rx_val) = serial.try_read() {
            buf.push(rx_val);
            serial.error_handler();
        }
    }

    if uart_irqn == 14 || uart_irqn == 6 {
        sleep(500);
    }
    (serial.rx_count, serial.tx_count, error_count)
}

fn user_intr_test() -> (usize, usize, usize) {
    unsafe {
        uie::clear_uext();
        uie::clear_usoft();
        uie::clear_utimer();
    }
    let mut hasher = Hasher::new();
    let uart_irqn = UART_IRQN.load(Relaxed);
    let claim_res = claim_ext_int(uart_irqn as usize);
    let mut serial = BufferedSerial::new(get_base_addr_from_irq(uart_irqn));
    serial.hardware_init(BAUD_RATE);
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
        push_trace(SERIAL_CALL_ENTER + SERIAL_INTR_READ);
        for _ in 0..HALF_FIFO_DEPTH {
            if let Ok(()) = serial.try_write(next_tx as u8) {
                // hasher.update(&[next_tx as u8]);
                next_tx = tx_rng.next_u32();
            }
        }
        push_trace(SERIAL_CALL_EXIT + SERIAL_INTR_READ);

        push_trace(SERIAL_CALL_ENTER + SERIAL_INTR_WRITE);
        // for _ in 0..HALF_FIFO_DEPTH {
        while !(IS_TIMEOUT.load(Relaxed)) {
            if let Ok(rx_val) = serial.try_read() {
                let mut max_shift = MAX_SHIFT;
                if err_pos == -1 && rx_val != expect_rx as u8 {
                    err_pos = serial.rx_count as isize;
                }
                while rx_val != expect_rx as u8 && max_shift > 0 {
                    println!(
                        "[uart {}] error at {}, expect: {:x}, err_val: {:x}",
                        uart_irqn, err_pos, expect_rx, rx_val
                    );
                    error_count += 1;
                    max_shift -= 1;
                    expect_rx = rx_rng.next_u32();
                }
                if error_count > 3 {
                    break;
                }
                // hasher.update(&[rx_val]);
                expect_rx = rx_rng.next_u32();
            } else {
                break;
            }
        }
        push_trace(SERIAL_CALL_EXIT + SERIAL_INTR_WRITE);

        if error_count > 3 {
            break;
        }

        if HAS_INTR.load(Relaxed) {
            serial.interrupt_handler();
            push_trace(U_TRAP_RETURN | 8 | 128);
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
    println!(
        "[uart load] Intr count: {}, Tx: {}, Rx: {}, err pos: {}",
        serial.intr_count, serial.tx_intr_count, serial.rx_intr_count, err_pos,
    );
    (serial.rx_count, serial.tx_count, error_count)
}

static ERROR_COUNT: AtomicUsize = AtomicUsize::new(0);

async fn read_task(serial: Arc<AsyncSerial>) {
    let mut error_count = 0;
    let mut rx_buf = [0; HALF_FIFO_DEPTH];
    serial.read(&mut rx_buf).await;
    let mut rx_rng = RX_RNG.lock();
    let mut expect_rx = rx_rng.next_u32();

    for rx_val in rx_buf {
        let mut max_shift = MAX_SHIFT;
        while rx_val != expect_rx as u8 && max_shift > 0 {
            error_count += 1;
            max_shift -= 1;
            expect_rx = rx_rng.next_u32();
        }
        // hasher.update(&[rx_val]);
        expect_rx = rx_rng.next_u32();
    }
    ERROR_COUNT.fetch_add(error_count, Relaxed);
}

async fn write_task(serial: Arc<AsyncSerial>) {
    let mut tx_rng = TX_RNG.lock();
    let tx_buf: [u8; HALF_FIFO_DEPTH] = array_init::array_init(|_| tx_rng.next_u32() as _);
    serial.write(&tx_buf).await;
}

fn user_async_test() -> (usize, usize, usize) {
    unsafe {
        uie::clear_uext();
        uie::clear_usoft();
        uie::clear_utimer();
    }
    let mut hasher = Hasher::new();
    let uart_irqn = UART_IRQN.load(Relaxed);
    let claim_res = claim_ext_int(uart_irqn as usize);
    type RxBuffer = Queue<u8, DEFAULT_RX_BUFFER_SIZE>;
    type TxBuffer = Queue<u8, DEFAULT_TX_BUFFER_SIZE>;
    static mut DRIVER_RX_BUFFER: RxBuffer = RxBuffer::new();
    static mut DRIVER_TX_BUFFER: TxBuffer = TxBuffer::new();
    let (rx_pro, rx_con) = unsafe { DRIVER_RX_BUFFER.split() };
    let (tx_pro, tx_con) = unsafe { DRIVER_TX_BUFFER.split() };

    let serial = Arc::new(AsyncSerial::new(
        get_base_addr_from_irq(uart_irqn),
        rx_pro,
        rx_con,
        tx_pro,
        tx_con,
    ));
    serial.hardware_init(BAUD_RATE);
    let en_res = set_ext_int_enable(uart_irqn as usize, 1);
    println!(
        "[uart load] Async mode, claim result: {:#x}, enable res: {:#x}",
        claim_res, en_res
    );
    let mut err_pos = -1;

    let (reader, writer) = (executor::Executor::default(), executor::Executor::default());

    let time_us = get_time() * 1000;
    set_timer(time_us + TEST_TIME_US);
    unsafe {
        uie::set_uext();
        uie::set_usoft();
        uie::set_utimer();
    }

    while !(IS_TIMEOUT.load(Relaxed)) {
        push_trace(SERIAL_CALL_ENTER + SERIAL_INTR_READ);
        if !reader.run_until_idle() {
            // println!("[uart load] spawn read tasks");
            reader.spawn(read_task(serial.clone()));
        }
        push_trace(SERIAL_CALL_EXIT + SERIAL_INTR_READ);

        push_trace(SERIAL_CALL_ENTER + SERIAL_INTR_WRITE);
        if !writer.run_until_idle() {
            // println!("[uart load] spawn write tasks");
            writer.spawn(write_task(serial.clone()));
        }
        push_trace(SERIAL_CALL_EXIT + SERIAL_INTR_WRITE);

        if HAS_INTR.load(Relaxed) {
            serial.interrupt_handler();
            push_trace(U_TRAP_RETURN | 8 | 128);
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
    println!(
        "[uart load] Intr count: {}, Tx: {}, Rx: {}, err pos: {}",
        serial.intr_count.load(Relaxed),
        serial.tx_intr_count.load(Relaxed),
        serial.rx_intr_count.load(Relaxed),
        err_pos,
    );
    (
        serial.rx_count.load(Relaxed),
        serial.tx_count.load(Relaxed),
        ERROR_COUNT.load(Relaxed),
    )
}

mod user_trap {
    use riscv::register::ucause;
    use user_lib::trace::{push_trace, U_EXT_HANDLER, U_TRAP_HANDLER, U_TRAP_RETURN};

    use super::*;
    #[no_mangle]
    pub fn soft_intr_handler(_pid: usize, msg: usize) {
        // if msg == 15 {
        //     println!("[uart load] Received SIGTERM, exiting...");
        //     user_lib::exit(15);
        // } else {
        //     println!("[uart load] Received message 0x{:x} from pid {}", msg, pid);
        // }
        // push_trace(U_TRAP_HANDLER | 0 | 128);
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
        // push_trace(U_TRAP_RETURN | 0 | 128);
    }

    #[no_mangle]
    pub fn ext_intr_handler(irq: u16, _is_from_kernel: bool) {
        // if _is_from_kernel {
        //     println!("[uart load] Received UEI from kernel, irq: {}", irq);
        // } else {
        //     println!("[uart load] user external interrupt, irq: {}", irq);
        // }
        // push_trace(U_EXT_HANDLER);
        if irq == UART_IRQN.load(Relaxed) {
            if !HAS_INTR.load(Relaxed) {
                push_trace(U_TRAP_HANDLER | 8 | 128);
                HAS_INTR.store(true, Relaxed);
            }
        } else {
            println!("[uart load] Unknown UEI!, irq: {}", irq);
        }
        // println!("[uart load] UEI fin");
    }

    #[no_mangle]
    pub fn timer_intr_handler(_time_us: usize) {
        // push_trace(U_TRAP_HANDLER | 4 | 128);
        IS_TIMEOUT.store(true, Relaxed);
        // push_trace(U_TRAP_RETURN | 4 | 128);
    }
}
