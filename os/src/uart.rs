use crate::console_blog::{IN_BUFFER, OUT_BUFFER};
use alloc::sync::Arc;
use lazy_static::*;
use spin::Mutex;

#[cfg(feature = "board_qemu")]
use uart8250::{InterruptType, MmioUart8250};

#[cfg(feature = "board_lrv")]
use uart_xilinx::uart_16550::{InterruptType, MmioUartAxi16550};

#[cfg(feature = "board_qemu")]
lazy_static! {
    pub static ref UART: Arc<Mutex<MmioUart8250<'static>>> =
        Arc::new(Mutex::new(MmioUart8250::new(0x1000_0200)));
}

#[cfg(feature = "board_lrv")]
lazy_static! {
    pub static ref UART: Arc<Mutex<MmioUartAxi16550<'static>>> =
        Arc::new(Mutex::new(MmioUartAxi16550::new(0x6000_1000)));
}

#[cfg(feature = "board_lrv_uartlite")]
use uart_xilinx::MmioUartAxiLite;

#[cfg(feature = "board_lrv_uartlite")]
lazy_static! {
    pub static ref UART: Arc<Mutex<MmioUartAxiLite<'static>>> =
        Arc::new(Mutex::new(MmioUartAxiLite::new(0x6000_1000)));
}

#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
pub fn init() {
    let uart = UART.lock();
    uart.write_ier(0);
    let _ = uart.read_msr();
    let _ = uart.read_lsr();
    uart.init(100_000_000, 115200);
    // Rx FIFO trigger level=14, reset Rx & Tx FIFO, enable FIFO
    uart.write_fcr(0b11_000_11_1);
}

#[cfg(feature = "board_lrv_uartlite")]
pub fn init() {
    UART.lock().enable_interrupt();
}

#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
const FIFO_DEPTH: usize = 16;

#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
pub fn handle_interrupt() {
    let uart = UART.lock();
    if let Some(int_type) = uart.read_interrupt_type() {
        match int_type {
            InterruptType::ReceivedDataAvailable | InterruptType::Timeout => {
                trace!("Received data available");
                let mut stdin = IN_BUFFER.lock();
                while let Some(ch) = uart.read_byte() {
                    stdin.push_back(ch);
                }
            }
            InterruptType::TransmitterHoldingRegisterEmpty => {
                trace!("TransmitterHoldingRegisterEmpty");
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
                trace!(
                    "MSR: {:#x}, LSR: {:#x}, IER: {:#x}",
                    uart.read_msr(),
                    uart.read_lsr(),
                    uart.read_ier()
                );
            }
            _ => {
                warn!("[UART] {:?} not supported!", int_type);
            }
        }
    }
}

#[cfg(feature = "board_lrv_uartlite")]
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
