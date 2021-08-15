use alloc::collections::VecDeque;
use core::convert::Infallible;
use embedded_hal::serial::{Read, Write};

pub const DEFAULT_TX_BUFFER_SIZE: usize = 1_000;
pub const DEFAULT_RX_BUFFER_SIZE: usize = 1_000;

#[cfg(feature = "board_qemu")]
mod serial_config {
    pub use uart8250::{uart::LSR, InterruptType, MmioUart8250};
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
    pub use uart_xilinx::uart_16550::{uart::LSR, InterruptType, MmioUartAxi16550};
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

pub fn get_base_addr_from_irq(irq: u16) -> usize {
    SERIAL_BASE_ADDRESS + irq_to_serial_id(irq) * SERIAL_ADDRESS_STRIDE
}

pub use serial_config::*;

pub struct BufferedSerial {
    pub hardware: SerialHardware,
    pub rx_buffer: VecDeque<u8>,
    pub tx_buffer: VecDeque<u8>,
    pub rx_count: usize,
    pub tx_count: usize,
    pub intr_count: usize,
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
        }
    }

    pub fn hardware_init(&mut self) {
        let hardware = &mut self.hardware;
        hardware.write_ier(0);
        let _ = hardware.read_msr();
        let _ = hardware.read_lsr();
        hardware.init(100_000_000, 115200);
        // Rx FIFO trigger level=14, reset Rx & Tx FIFO, enable FIFO
        hardware.write_fcr(0b11_000_11_1);
    }

    #[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
    pub fn interrupt_handler(&mut self) {
        let hardware = &self.hardware;
        if let Some(int_type) = hardware.read_interrupt_type() {
            self.intr_count += 1;
            match int_type {
                InterruptType::ReceivedDataAvailable | InterruptType::Timeout => {
                    // println!("[SERIAL] Received data available");
                    while let Some(ch) = hardware.read_byte() {
                        self.rx_buffer.push_back(ch);
                        self.rx_count += 1;
                    }
                }
                InterruptType::TransmitterHoldingRegisterEmpty => {
                    // println!("[SERIAL] Transmitter Holding Register Empty");
                    for _ in 0..FIFO_DEPTH {
                        if let Some(ch) = self.tx_buffer.pop_front() {
                            hardware.write_byte(ch);
                            self.tx_count += 1;
                        } else {
                            hardware.disable_transmitter_holding_register_empty_interrupt();
                            break;
                        }
                    }
                }
                InterruptType::ModemStatus => {
                    println!(
                        "[USER SERIAL] MSR: {:#x}, LSR: {:#x}, IER: {:#x}",
                        hardware.read_msr(),
                        hardware.read_lsr(),
                        hardware.read_ier()
                    );
                }
                _ => {
                    println!("[USER SERIAL] {:?} not supported!", int_type);
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
        if !serial.is_transmitter_holding_register_empty_interrupt_enabled() {
            serial.write_byte(word);
            self.tx_count += 1;
            serial.enable_transmitter_holding_register_empty_interrupt();
        } else {
            if self.tx_buffer.len() < DEFAULT_TX_BUFFER_SIZE {
                self.tx_buffer.push_back(word);
            } else {
                return Err(nb::Error::WouldBlock);
            }
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
            #[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
            {
                // Drain UART Rx FIFO
                while let Some(ch_read) = self.hardware.read_byte() {
                    self.rx_buffer.push_back(ch_read);
                    self.rx_count += 1;
                }
            }
            self.rx_buffer.pop_front().ok_or(nb::Error::WouldBlock)
        }
    }
}

pub struct PollingSerial {
    pub hardware: SerialHardware,
    pub rx_count: usize,
    pub tx_count: usize,
    pub tx_fifo_count: usize,
}

impl PollingSerial {
    pub fn new(base_address: usize) -> Self {
        PollingSerial {
            hardware: SerialHardware::new(base_address),
            rx_count: 0,
            tx_count: 0,
            tx_fifo_count: 0,
        }
    }

    pub fn hardware_init(&mut self) {
        let hardware = &mut self.hardware;
        hardware.write_ier(0);
        let _ = hardware.read_msr();
        let _ = hardware.read_lsr();
        hardware.init(100_000_000, 115200);
        hardware.write_ier(0);
        // Rx FIFO trigger level=14, reset Rx & Tx FIFO, enable FIFO
        hardware.write_fcr(0b11_000_11_1);
    }

    pub fn interrupt_handler(&mut self) {}
}

impl Write<u8> for PollingSerial {
    type Error = Infallible;

    #[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
    fn try_write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        let serial = &mut self.hardware;
        while self.tx_fifo_count >= FIFO_DEPTH {
            if serial.lsr().contains(LSR::THRE) {
                self.tx_fifo_count = 0;
            }
        }
        serial.write_byte(word);
        self.tx_count += 1;
        self.tx_fifo_count += 1;
        Ok(())
    }

    fn try_flush(&mut self) -> nb::Result<(), Self::Error> {
        todo!()
    }
}

impl Read<u8> for PollingSerial {
    type Error = Infallible;

    #[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
    fn try_read(&mut self) -> nb::Result<u8, Self::Error> {
        if let Some(ch) = self.hardware.read_byte() {
            Ok(ch)
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}
