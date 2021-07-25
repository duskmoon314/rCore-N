#![no_std]
#![no_main]
#![feature(asm)]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::{string::String, sync::Arc};
use lazy_static::*;
use spin::Mutex;
use uart::{Uart, UART1_BASE_ADDRSS};
use user_lib::{claim_ext_int, init_user_trap};

lazy_static! {
    pub static ref LINE: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    pub static ref UART1: Arc<Mutex<Uart>> = Arc::new(Mutex::new(Uart::new(UART1_BASE_ADDRSS)));
}

#[no_mangle]
pub fn main() -> i32 {
    println!("user mode serial demo using external interrupt");
    let init_res = init_user_trap();
    let claim_res = claim_ext_int(uart::UART1_IRQN as usize);
    println!(
        ">>> init result: {:?}, claim result: {:?}",
        init_res, claim_res
    );
    UART1.lock().init();
    println2!("Hello from UART1!");
    loop {}
}

#[macro_export]
macro_rules! print2 {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::uart::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println2 {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::uart::print(format_args!(concat!($fmt, "\r\n") $(, $($arg)+)?));
    }
}

mod uart {
    // Based on https://github.com/sgmarz/osblog
    pub const UART1_BASE_ADDRSS: usize = 0x10000100;
    pub const UART1_IRQN: u16 = 9;

    const LF: u8 = 0x0au8;
    const CR: u8 = 0x0du8;
    const DL: u8 = 0x7fu8;
    const BS: u8 = 0x08u8;

    use crate::UART1;
    use core::{
        convert::TryInto,
        fmt::{self, Error, Write},
    };

    pub struct Uart {
        base_address: usize,
    }

    pub fn print(args: fmt::Arguments) {
        UART1.lock().write_fmt(args).unwrap();
    }

    impl Write for Uart {
        fn write_str(&mut self, out: &str) -> Result<(), Error> {
            for c in out.bytes() {
                self.put(c);
            }
            Ok(())
        }
    }

    impl Uart {
        pub fn new(base_address: usize) -> Self {
            Uart { base_address }
        }

        pub fn init(&mut self) {
            let ptr = self.base_address as *mut u8;
            unsafe {
                // First, set the word length, which
                // are bits 0 and 1 of the line control register (LCR)
                // which is at base_address + 3
                // We can easily write the value 3 here or 0b11, but I'm
                // extending it so that it is clear we're setting two
                // individual fields
                //             Word 0     Word 1
                //             ~~~~~~     ~~~~~~
                let lcr: u8 = (1 << 0) | (1 << 1);
                ptr.add(3).write_volatile(lcr);

                // Now, enable the FIFO, which is bit index 0 of the
                // FIFO control register (FCR at offset 2).
                // Again, we can just write 1 here, but when we use left
                // shift, it's easier to see that we're trying to write
                // bit index #0.
                ptr.add(2).write_volatile(1 << 0);

                // Enable receiver buffer interrupts, which is at bit
                // index 0 of the interrupt enable register (IER at
                // offset 1).
                ptr.add(1).write_volatile(1 << 0);

                // If we cared about the divisor, the code below would
                // set the divisor from a global clock rate of 22.729
                // MHz (22,729,000 cycles per second) to a signaling
                // rate of 2400 (BAUD). We usually have much faster
                // signalling rates nowadays, but this demonstrates what
                // the divisor actually does. The formula given in the
                // NS16500A specification for calculating the divisor
                // is:
                // divisor = ceil( (clock_hz) / (baud_sps x 16) )
                // So, we substitute our values and get:
                // divisor = ceil( 22_729_000 / (2400 x 16) )
                // divisor = ceil( 22_729_000 / 38_400 )
                // divisor = ceil( 591.901 ) = 592

                // The divisor register is two bytes (16 bits), so we
                // need to split the value 592 into two bytes.
                // Typically, we would calculate this based on measuring
                // the clock rate, but again, for our purposes [qemu],
                // this doesn't really do anything.
                let divisor: u16 = 592;
                let divisor_least: u8 = (divisor & 0xff).try_into().unwrap();
                let divisor_most: u8 = (divisor >> 8).try_into().unwrap();

                // Notice that the divisor register DLL (divisor latch
                // least) and DLM (divisor latch most) have the same
                // base address as the receiver/transmitter and the
                // interrupt enable register. To change what the base
                // address points to, we open the "divisor latch" by
                // writing 1 into the Divisor Latch Access Bit (DLAB),
                // which is bit index 7 of the Line Control Register
                // (LCR) which is at base_address + 3.
                ptr.add(3).write_volatile(lcr | 1 << 7);

                // Now, base addresses 0 and 1 point to DLL and DLM,
                // respectively. Put the lower 8 bits of the divisor
                // into DLL
                ptr.add(0).write_volatile(divisor_least);
                ptr.add(1).write_volatile(divisor_most);

                // Now that we've written the divisor, we never have to
                // touch this again. In hardware, this will divide the
                // global clock (22.729 MHz) into one suitable for 2,400
                // signals per second. So, to once again get access to
                // the RBR/THR/IER registers, we need to close the DLAB
                // bit by clearing it to 0.
                ptr.add(3).write_volatile(lcr);
            }
        }

        pub fn put(&mut self, c: u8) {
            let ptr = self.base_address as *mut u8;
            unsafe {
                ptr.add(0).write_volatile(c);
            }
        }

        pub fn get(&mut self) -> Option<u8> {
            let ptr = self.base_address as *mut u8;
            unsafe {
                if ptr.add(5).read_volatile() & 1 == 0 {
                    // The DR bit is 0, meaning no data
                    None
                } else {
                    // The DR bit is 1, meaning data!
                    Some(ptr.add(0).read_volatile())
                }
            }
        }
    }

    pub fn handle_input() {
        // If we get here, the UART better have something! If not, what happened??
        let mut uart1 = UART1.lock();
        if let Some(c) = uart1.get() {
            // If you recognize this code, it used to be in the lib.rs under kmain(). That
            // was because we needed to poll for UART data. Now that we have interrupts,
            // here it goes!
            drop(uart1);
            let mut line = crate::LINE.lock();
            match c {
                LF | CR => {
                    println2!("");
                    if *line == "exit" {
                        user_lib::exit(0);
                    }
                    println2!("{}", line);
                    line.clear();
                }
                BS | DL => {
                    if !line.is_empty() {
                        print2!("{}", BS as char);
                        print2!(" ");
                        print2!("{}", BS as char);
                        line.pop();
                    }
                }
                _ => {
                    print2!("{}", c as char);
                    line.push(c as char);
                }
            }
        }
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

    use crate::uart::{handle_input, UART1_IRQN};

    #[no_mangle]
    pub fn user_trap_handler(cx: &mut UserTrapContext) -> &mut UserTrapContext {
        let ucause = ucause::read();
        let utval = utval::read();
        match ucause.cause() {
            ucause::Trap::Interrupt(ucause::Interrupt::UserSoft) => {
                println!("[user mode trap] user soft");
                let trap_record_num = uscratch::read();
                let mut head_ptr = USER_TRAP_BUFFER as *const UserTrapRecord;
                for _ in 0..trap_record_num {
                    unsafe {
                        let trap_record = *head_ptr;
                        if ucause::Interrupt::from(trap_record.cause)
                            == ucause::Interrupt::UserExternal
                        {
                            if trap_record.message == UART1_IRQN as usize {
                                handle_input();
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
                    println!("[user trap] user external, irq: {}", irq);
                    if irq == UART1_IRQN {
                        handle_input();
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
