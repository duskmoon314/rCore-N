use crate::future::GetWakerFuture;
use crate::trace::{SERIAL_INTR_ENTER, SERIAL_INTR_EXIT};
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::future::Future;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::Relaxed;
use core::task::{Context, Poll, Waker};
use core::{convert::Infallible, pin::Pin, sync::atomic::AtomicBool};
use embedded_hal::serial::{Read, Write};
use heapless::spsc;
#[cfg(feature = "board_lrv")]
use lrv_pac::uart;
#[cfg(feature = "board_qemu")]
use qemu_pac::uart;
pub use serial_config::*;
use spin::Mutex;

pub const DEFAULT_TX_BUFFER_SIZE: usize = 1000;
pub const DEFAULT_RX_BUFFER_SIZE: usize = 1000;

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
        // Rx FIFO trigger level=8, reset Rx & Tx FIFO, enable FIFO
        hardware.write_fcr(0b10_000_11_1);
    }

    #[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
    pub fn interrupt_handler(&mut self) {
        // println!("[SERIAL] Interrupt!");

        use crate::trace::push_trace;
        let hardware = &self.hardware;
        while let Some(int_type) = hardware.read_interrupt_type() {
            let intr_id: usize = match int_type {
                InterruptType::ModemStatus => 0x0000,
                InterruptType::TransmitterHoldingRegisterEmpty => 0b0010,
                InterruptType::ReceivedDataAvailable => 0b0100,
                InterruptType::ReceiverLineStatus => 0b0110,
                InterruptType::Timeout => 0b1100,
                InterruptType::Reserved => 0b1000,
            };
            push_trace(SERIAL_INTR_ENTER + intr_id);
            self.intr_count += 1;
            match int_type {
                InterruptType::ReceivedDataAvailable | InterruptType::Timeout => {
                    // println!("[SERIAL] Received data available");
                    self.rx_intr_count += 1;
                    while let Some(ch) = hardware.read_byte() {
                        if self.rx_buffer.len() < DEFAULT_TX_BUFFER_SIZE {
                            self.rx_buffer.push_back(ch);
                            self.rx_count += 1;
                        } else {
                            // println!("[USER UART] Serial rx buffer overflow!");
                            hardware.disable_received_data_available_interrupt();
                            self.rx_intr_enabled = false;
                            break;
                        }
                    }
                }
                InterruptType::TransmitterHoldingRegisterEmpty => {
                    // println!("[SERIAL] Transmitter Holding Register Empty");
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
            push_trace(SERIAL_INTR_EXIT + intr_id);
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
            // println!("[USER SERIAL] Tx buffer overflow!");
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

    pub fn hardware_init(&mut self, baud_rate: usize) {
        let hardware = &mut self.hardware;
        hardware.write_ier(0);
        let _ = hardware.read_msr();
        let _ = hardware.read_lsr();
        hardware.init(100_000_000, baud_rate);
        hardware.write_ier(0);
        // Rx FIFO trigger level=4, reset Rx & Tx FIFO, enable FIFO
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
            self.rx_count += 1;
            Ok(ch)
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl Drop for PollingSerial {
    fn drop(&mut self) {
        let hardware = &mut self.hardware;
        hardware.write_ier(0);
        let _ = hardware.read_msr();
        let _ = hardware.read_lsr();
        // reset Rx & Tx FIFO, disable FIFO
        hardware.write_fcr(0b00_000_11_0);
    }
}

type RxProducer = spsc::Producer<'static, u8, DEFAULT_RX_BUFFER_SIZE>;
type RxConsumer = spsc::Consumer<'static, u8, DEFAULT_RX_BUFFER_SIZE>;
type TxProducer = spsc::Producer<'static, u8, DEFAULT_TX_BUFFER_SIZE>;
type TxConsumer = spsc::Consumer<'static, u8, DEFAULT_TX_BUFFER_SIZE>;

pub struct AsyncSerial {
    base_address: usize,
    rx_pro: Mutex<RxProducer>,
    rx_con: Mutex<RxConsumer>,
    tx_pro: Mutex<TxProducer>,
    tx_con: Mutex<TxConsumer>,
    pub rx_count: AtomicUsize,
    pub tx_count: AtomicUsize,
    pub intr_count: AtomicUsize,
    pub rx_intr_count: AtomicUsize,
    pub tx_intr_count: AtomicUsize,
    pub(super) rx_intr_enabled: AtomicBool,
    pub(super) tx_intr_enabled: AtomicBool,
    read_waker: Mutex<Option<Waker>>,
    write_waker: Mutex<Option<Waker>>,
}

impl AsyncSerial {
    pub fn new(
        base_address: usize,
        rx_pro: RxProducer,
        rx_con: RxConsumer,
        tx_pro: TxProducer,
        tx_con: TxConsumer,
    ) -> Self {
        AsyncSerial {
            base_address,
            rx_pro: Mutex::new(rx_pro),
            rx_con: Mutex::new(rx_con),
            tx_pro: Mutex::new(tx_pro),
            tx_con: Mutex::new(tx_con),
            rx_count: AtomicUsize::new(0),
            tx_count: AtomicUsize::new(0),
            intr_count: AtomicUsize::new(0),
            rx_intr_count: AtomicUsize::new(0),
            tx_intr_count: AtomicUsize::new(0),
            rx_intr_enabled: AtomicBool::new(false),
            tx_intr_enabled: AtomicBool::new(false),
            read_waker: Mutex::new(None),
            write_waker: Mutex::new(None),
        }
    }

    fn hardware(&self) -> &uart::RegisterBlock {
        unsafe { &*(self.base_address as *const _) }
    }

    fn set_divisor(&self, clock: usize, baud_rate: usize) {
        let block = self.hardware();
        let divisor = clock / (16 * baud_rate);
        block.lcr.write(|w| w.dlab().set_bit());
        #[cfg(feature = "board_lrv")]
        {
            block
                .dll()
                .write(|w| unsafe { w.bits((divisor & 0b1111_1111) as u32) });
            block
                .dlh()
                .write(|w| unsafe { w.bits(((divisor >> 8) & 0b1111_1111) as u32) });
        }
        #[cfg(feature = "board_qemu")]
        {
            block
                .dll()
                .write(|w| unsafe { w.bits((divisor & 0b1111_1111) as u8) });
            block
                .dlh()
                .write(|w| unsafe { w.bits(((divisor >> 8) & 0b1111_1111) as u8) });
        }

        block.lcr.write(|w| w.dlab().clear_bit());
    }

    pub(super) fn enable_rdai(&self) {
        self.hardware().ier().write(|w| w.erbfi().set_bit());
        self.rx_intr_enabled.store(true, Relaxed);
    }

    fn disable_rdai(&self) {
        self.hardware().ier().write(|w| w.erbfi().clear_bit());
        self.rx_intr_enabled.store(false, Relaxed);
    }

    pub(super) fn enable_threi(&self) {
        self.hardware().ier().write(|w| w.etbei().set_bit());
        self.tx_intr_enabled.store(true, Relaxed);
    }

    fn disable_threi(&self) {
        self.hardware().ier().write(|w| w.etbei().clear_bit());
        self.tx_intr_enabled.store(false, Relaxed);
    }

    fn try_recv(&self) -> Option<u8> {
        let block = self.hardware();
        if block.lsr.read().dr().bit_is_set() {
            Some(block.rbr().read().bits() as _)
        } else {
            None
        }
    }

    fn send(&self, ch: u8) {
        let block = self.hardware();
        block.thr().write(|w| w.thr().variant(ch));
    }

    pub(super) fn try_read(&self) -> Option<u8> {
        if let Some(mut rx_lock) = self.rx_con.try_lock() {
            rx_lock.dequeue()
        } else {
            None
        }
    }

    pub(super) fn try_write(&self, ch: u8) -> Result<(), u8> {
        if let Some(mut tx_lock) = self.tx_pro.try_lock() {
            tx_lock.enqueue(ch)
        } else {
            Err(ch)
        }
    }

    pub fn hardware_init(&self, baud_rate: usize) {
        let block = self.hardware();
        let _unused = block.msr.read().bits();
        let _unused = block.lsr.read().bits();
        block.ier().reset();
        // No modem control
        block.mcr.reset();
        // Enable DLAB and Set divisor
        self.set_divisor(100_000_000, baud_rate);
        // Disable DLAB and set word length 8 bits, no parity, 1 stop bit
        block.lcr.write(|w| w.dls().eight());
        // Enable FIFO
        block.fcr().write(|w| {
            w.fifoe()
                .set_bit()
                .rfifor()
                .set_bit()
                .xfifor()
                .set_bit()
                .rt()
                .half_full()
        });

        // Enable received_data_available_interrupt
        self.enable_rdai();
        // Enable transmitter_holding_register_empty_interrupt
        // self.enable_transmitter_holding_register_empty_interrupt();
    }

    #[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
    pub fn interrupt_handler(&self) {
        // println!("[SERIAL] Interrupt!");

        use uart::iir::IID_A;

        use crate::trace::push_trace;
        let block = self.hardware();
        while let Some(int_type) = block.iir().read().iid().variant() {
            if int_type == IID_A::NO_INTERRUPT_PENDING {
                break;
            }
            let intr_id: usize = int_type as u8 as _;
            push_trace(SERIAL_INTR_ENTER + intr_id);
            self.intr_count.fetch_add(1, Relaxed);
            match int_type {
                IID_A::RECEIVED_DATA_AVAILABLE | IID_A::CHARACTER_TIMEOUT => {
                    // println!("[SERIAL] Received data available");
                    self.rx_intr_count.fetch_add(1, Relaxed);
                    let mut rx_count = 0;
                    while let Some(ch) = self.try_recv() {
                        if let Ok(()) = self.rx_pro.lock().enqueue(ch) {
                            rx_count += 1;
                        } else {
                            // println!("[USER UART] Serial rx buffer overflow!");
                            self.disable_rdai();
                            break;
                        }
                    }
                    self.rx_count.fetch_add(rx_count, Relaxed);
                    if let Some(mut waker) = self.read_waker.try_lock() {
                        if waker.is_some() {
                            waker.take().unwrap().wake();
                        }
                    }
                }
                IID_A::THR_EMPTY => {
                    // println!("[SERIAL] Transmitter Holding Register Empty");
                    self.tx_intr_count.fetch_add(1, Relaxed);
                    let mut tx_count = 0;
                    for _ in 0..FIFO_DEPTH {
                        if let Some(ch) = self.tx_con.lock().dequeue() {
                            self.send(ch);
                            tx_count += 1;
                        } else {
                            self.disable_threi();
                            break;
                        }
                    }
                    self.tx_count.fetch_add(tx_count, Relaxed);
                    if let Some(mut waker) = self.write_waker.try_lock() {
                        if waker.is_some() {
                            waker.take().unwrap().wake();
                        }
                    }
                }
                IID_A::MODEM_STATUS => {
                    println!(
                        "[USER SERIAL] MSR: {:#x}, LSR: {:#x}, IER: {:#x}",
                        block.msr.read().bits(),
                        block.lsr.read().bits(),
                        block.ier().read().bits()
                    );
                }
                _ => {
                    println!("[USER SERIAL] {:?} not supported!", int_type);
                }
            }
            push_trace(SERIAL_INTR_EXIT + intr_id);
        }
    }

    async fn register_read(&self) {
        let raw_waker = GetWakerFuture.await;
        self.read_waker.lock().replace(raw_waker);
    }

    pub async fn read(self: Arc<Self>, buf: &mut [u8]) {
        let future = SerialReadFuture {
            buf,
            read_len: 0,
            driver: self.clone(),
        };
        self.register_read().await;
        future.await;
    }

    async fn register_write(&self) {
        let raw_waker = GetWakerFuture.await;
        self.write_waker.lock().replace(raw_waker);
    }

    pub async fn write(self: Arc<Self>, buf: &[u8]) {
        let future = SerialWriteFuture {
            buf,
            write_len: 0,
            driver: self.clone(),
        };
        self.register_write().await;
        future.await;
    }
}

impl Drop for AsyncSerial {
    fn drop(&mut self) {
        let block = self.hardware();
        block.ier().reset();
        let _unused = block.msr.read().bits();
        let _unused = block.lsr.read().bits();
        // reset Rx & Tx FIFO, disable FIFO
        block
            .fcr()
            .write(|w| w.fifoe().clear_bit().rt().one_character());
    }
}

struct SerialReadFuture<'a> {
    buf: &'a mut [u8],
    read_len: usize,
    driver: Arc<AsyncSerial>,
}

impl Future for SerialReadFuture<'_> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        while let Some(data) = self.driver.try_read() {
            if self.read_len < self.buf.len() {
                let len = self.read_len;
                self.buf[len] = data;
                self.read_len += 1;
            } else {
                return Poll::Ready(());
            }
        }

        if !self.driver.rx_intr_enabled.load(Relaxed) {
            self.driver.enable_rdai();
        }
        Poll::Pending
    }
}

struct SerialWriteFuture<'a> {
    buf: &'a [u8],
    write_len: usize,
    driver: Arc<AsyncSerial>,
}

impl Future for SerialWriteFuture<'_> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        while let Ok(()) = self.driver.try_write(self.buf[self.write_len]) {
            if self.write_len < self.buf.len() - 1 {
                self.write_len += 1;
            } else {
                return Poll::Ready(());
            }
        }

        if !self.driver.tx_intr_enabled.load(Relaxed) {
            self.driver.enable_threi();
        }
        Poll::Pending
    }
}
