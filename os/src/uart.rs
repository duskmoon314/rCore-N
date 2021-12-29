use alloc::collections::VecDeque;
use core::convert::Infallible;
use embedded_hal::serial::{Read, Write};
use lazy_static::*;
use spin::Mutex;

pub const DEFAULT_TX_BUFFER_SIZE: usize = 1_000;
pub const DEFAULT_RX_BUFFER_SIZE: usize = 1_000;

#[cfg(feature = "board_qemu")]
mod serial_config {
    pub use uart8250::{InterruptType, MmioUart8250};
    pub type SerialHardware = MmioUart8250<'static>;
    pub const FIFO_DEPTH: usize = 16;
    pub const SERIAL_NUM: usize = 4;
    pub const SERIAL_BASE_ADDRESS: usize = 0x1000_2000;
    pub const SERIAL_ADDRESS_STRIDE: usize = 0x1000;
    pub fn irq_to_serial_id(irq: u16) -> usize {
        match irq {
            12 => 0,
            13 => 1,
            14 => 2,
            15 => 3,
            _ => 0,
        }
    }
}

#[cfg(feature = "board_lrv")]
mod serial_config {
    pub use uart_xilinx::uart_16550::{InterruptType, MmioUartAxi16550};
    pub type SerialHardware = MmioUartAxi16550<'static>;
    pub const FIFO_DEPTH: usize = 16;
    pub const SERIAL_NUM: usize = 4;
    pub const SERIAL_BASE_ADDRESS: usize = 0x6000_1000;
    pub const SERIAL_ADDRESS_STRIDE: usize = 0x1000;
    pub fn irq_to_serial_id(irq: u16) -> usize {
        match irq {
            4 => 0,
            5 => 1,
            6 => 2,
            7 => 3,
            _ => 0,
        }
    }
}

pub use serial_config::*;

pub fn get_base_addr_from_irq(irq: u16) -> usize {
    SERIAL_BASE_ADDRESS + irq_to_serial_id(irq) * SERIAL_ADDRESS_STRIDE
}
pub struct BufferedSerial {
    pub hardware: SerialHardware,
    pub rx_buffer: VecDeque<u8>,
    pub tx_buffer: VecDeque<u8>,
    pub rx_count: usize,
    pub tx_count: usize,
    pub intr_count: usize,
    pub rx_intr_count: usize,
    pub tx_intr_count: usize,
    pub tx_fifo_count: usize,
    rx_intr_enabled: bool,
    tx_intr_enabled: bool,
}

impl BufferedSerial {
    pub fn new(base_address: usize) -> Self {
        BufferedSerial {
            hardware: SerialHardware::new(base_address),
            rx_buffer: VecDeque::with_capacity(DEFAULT_RX_BUFFER_SIZE),
            tx_buffer: VecDeque::with_capacity(DEFAULT_TX_BUFFER_SIZE),
            rx_count: 0,
            tx_count: 0,
            intr_count: 0,
            rx_intr_count: 0,
            tx_intr_count: 0,
            tx_fifo_count: 0,
            rx_intr_enabled: false,
            tx_intr_enabled: false,
        }
    }

    pub fn hardware_init(&mut self, baud_rate: usize) {
        let hardware = &mut self.hardware;
        hardware.write_ier(0);
        let _ = hardware.read_msr();
        let _ = hardware.read_lsr();
        hardware.write_mcr(0);
        hardware.init(100_000_000, baud_rate);
        hardware.enable_received_data_available_interrupt();
        self.rx_intr_enabled = true;
        // Rx FIFO trigger level=14, reset Rx & Tx FIFO, enable FIFO
        hardware.write_fcr(0b11_000_11_1);
    }

    #[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
    pub fn interrupt_handler(&mut self) {
        let hardware = &self.hardware;
        while let Some(int_type) = hardware.read_interrupt_type() {
            self.intr_count += 1;
            match int_type {
                InterruptType::ReceivedDataAvailable | InterruptType::Timeout => {
                    // trace!("Received data available");
                    self.rx_intr_count += 1;
                    while let Some(ch) = hardware.read_byte() {
                        if self.rx_buffer.len() < DEFAULT_TX_BUFFER_SIZE {
                            self.rx_buffer.push_back(ch);
                            self.rx_count += 1;
                        } else {
                            // warn!("Serial rx buffer overflow!");
                            hardware.disable_received_data_available_interrupt();
                            self.rx_intr_enabled = false;
                            break;
                        }
                    }
                }
                InterruptType::TransmitterHoldingRegisterEmpty => {
                    // trace!("TransmitterHoldingRegisterEmpty");
                    self.tx_intr_count += 1;
                    for _ in 0..FIFO_DEPTH {
                        if let Some(ch) = self.tx_buffer.pop_front() {
                            hardware.write_byte(ch);
                            self.tx_count += 1;
                        } else {
                            hardware.disable_transmitter_holding_register_empty_interrupt();
                            self.tx_intr_enabled = false;
                            break;
                        }
                    }
                }
                InterruptType::ModemStatus => {
                    debug!(
                        "MSR: {:#x}, LSR: {:#x}, IER: {:#x}",
                        hardware.read_msr(),
                        hardware.read_lsr(),
                        hardware.read_ier()
                    );
                }
                _ => {
                    warn!("[SERIAL] {:?} not supported!", int_type);
                }
            }
        }
    }
}

impl Write<u8> for BufferedSerial {
    type Error = Infallible;

    #[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
    fn try_write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        let serial = &mut self.hardware;
        if self.tx_buffer.len() < DEFAULT_TX_BUFFER_SIZE {
            self.tx_buffer.push_back(word);
            if !self.tx_intr_enabled {
                serial.enable_transmitter_holding_register_empty_interrupt();
                self.tx_intr_enabled = true;
            }
        } else {
            // warn!("Serial tx buffer overflow!");
            return Err(nb::Error::WouldBlock);
        }
        Ok(())
    }

    fn try_flush(&mut self) -> nb::Result<(), Self::Error> {
        todo!()
    }
}

impl Read<u8> for BufferedSerial {
    type Error = Infallible;

    fn try_read(&mut self) -> nb::Result<u8, Self::Error> {
        if let Some(ch) = self.rx_buffer.pop_front() {
            Ok(ch)
        } else {
            let serial = &mut self.hardware;
            if !self.rx_intr_enabled {
                serial.enable_received_data_available_interrupt();
                self.rx_intr_enabled = true;
            }
            Err(nb::Error::WouldBlock)
        }
    }
}

impl Drop for BufferedSerial {
    fn drop(&mut self) {
        let hardware = &mut self.hardware;
        hardware.write_ier(0);
        let _ = hardware.read_msr();
        let _ = hardware.read_lsr();
        // reset Rx & Tx FIFO, disable FIFO
        hardware.write_fcr(0b00_000_11_0);
    }
}

#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
lazy_static! {
    pub static ref BUFFERED_SERIAL: [Mutex<BufferedSerial>; SERIAL_NUM] =
        array_init::array_init(|i| Mutex::new(BufferedSerial::new(
            SERIAL_BASE_ADDRESS + i * SERIAL_ADDRESS_STRIDE,
        )));
}

#[cfg(feature = "board_lrv_seriallite")]
use serial_xilinx::MmioSerialAxiLite;

#[cfg(feature = "board_lrv_seriallite")]
lazy_static! {
    pub static ref SERIAL: Arc<Mutex<MmioSerialAxiLite<'static>>> =
        Arc::new(Mutex::new(MmioSerialAxiLite::new(0x6000_1000)));
}

#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
pub fn init() {
    for serial_id in 0..2 {
        BUFFERED_SERIAL[serial_id].lock().hardware_init(115200);
    }
    for serial_id in 2..SERIAL_NUM {
        BUFFERED_SERIAL[serial_id].lock().hardware_init(6_250_000);
    }
}

#[cfg(feature = "board_lrv_seriallite")]
pub fn init() {
    SERIAL.lock().enable_interrupt();
}

pub fn handle_interrupt(irq: u16) {
    BUFFERED_SERIAL[irq_to_serial_id(irq)]
        .lock()
        .interrupt_handler();
}

#[cfg(feature = "board_lrv_seriallite")]
pub fn handle_interrupt() {
    todo!("Stdio Refactored!");
    use serial_xilinx::serial_lite::Status;
    let serial = SERIAL.lock();
    let status = serial.status();
    if status.contains(Status::TX_FIFO_EMPTY) {
        let mut stdout = OUT_BUFFER.lock();
        while !serial.is_tx_fifo_full() {
            if let Some(ch) = stdout.pop_front() {
                serial.write_byte(ch);
            } else {
                break;
            }
        }
    }
    if status.contains(Status::RX_FIFO_FULL) {
        let mut stdin = IN_BUFFER.lock();
        for _ in 0..FIFO_DEPTH {
            if let Some(ch) = serial.read_byte() {
                stdin.push_back(ch);
            } else {
                break;
            }
        }
    }
}

#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
pub fn serial_putchar(serial_id: usize, c: u8) -> nb::Result<(), Infallible> {
    BUFFERED_SERIAL[serial_id].lock().try_write(c)
}

#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
pub fn serial_getchar(serial_id: usize) -> nb::Result<u8, Infallible> {
    BUFFERED_SERIAL[serial_id].lock().try_read()
}
